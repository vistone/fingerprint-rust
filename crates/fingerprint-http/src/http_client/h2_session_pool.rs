//! HTTP/2 sessionpool
//!
// ! pool化 h2::client::SendRequest handle, implementtrue HTTP/2 multiplereuse
// ! avoideach timerequest都reperform TLS and HTTP/2 handshake

#[cfg(all(feature = "connection-pool", feature = "http2"))]
use super::Result;
#[cfg(all(feature = "connection-pool", feature = "http2"))]
use std::collections::HashMap;
#[cfg(all(feature = "connection-pool", feature = "http2"))]
use std::sync::{Arc, Mutex};
#[cfg(all(feature = "connection-pool", feature = "http2"))]
use std::time::{Duration, Instant};
#[cfg(all(feature = "connection-pool", feature = "http2"))]
use tokio::sync::watch;
#[cfg(all(feature = "connection-pool", feature = "http2"))]
use tokio::sync::Mutex as TokioMutex;

#[cfg(all(feature = "connection-pool", feature = "http2"))]
use h2::client::SendRequest;

/// HTTP/2 sessionpoolmanageer
// / Fix: pool化 SendRequest handle, implementtrue multiplexreuse
#[cfg(all(feature = "connection-pool", feature = "http2"))]
pub struct H2SessionPool {
    /// sessionpool ( by  host:port group)
    /// eachsessionincluding SendRequest handle, backbackground taskhandle and finallywhen used between
    sessions: Arc<Mutex<HashMap<String, Arc<H2Session>>>>,
    /// positive in Createinsession (avoidsame key concurrentCreatecompetition)
    pending_sessions: Arc<Mutex<HashMap<String, watch::Receiver<bool>>>>,
    /// sessiontimeout duration (default 5 minutes)
    session_timeout: Duration,
}

/// HTTP/2 session
#[cfg(all(feature = "connection-pool", feature = "http2"))]
struct H2Session {
    /// SendRequest handle ( for sendrequest)
    send_request: Arc<TokioMutex<SendRequest<bytes::Bytes>>>,
    /// backbackground taskhandle ( for manage h2_conn lifecycle)
    // / whenconnectioninvalid when , taskwillend, wecandetect to 并removesession
    _background_task: tokio::task::JoinHandle<()>,
    /// finallywhen used between
    last_used: Arc<Mutex<Instant>>,
    /// connectionwhethervalid (bybackbackground taskUpdate)
    is_valid: Arc<Mutex<bool>>,
}

#[cfg(all(feature = "connection-pool", feature = "http2"))]
impl H2SessionPool {
    /// Create a newsessionpool
    pub fn new(session_timeout: Duration) -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
            pending_sessions: Arc::new(Mutex::new(HashMap::new())),
            session_timeout,
        }
    }

    /// Get or Create HTTP/2 session
    /// return SendRequest handleclone
    /// create_session: Createnewsessionasyncfunction, return (SendRequest, Connection)
    pub async fn get_or_create_session<Fut, IO>(
        &self,
        key: &str,
        create_session: Fut,
    ) -> Result<Arc<TokioMutex<SendRequest<bytes::Bytes>>>>
    where
        Fut: std::future::Future<
            Output = Result<(SendRequest<bytes::Bytes>, h2::client::Connection<IO>)>,
        >,
        IO: tokio::io::AsyncRead + tokio::io::AsyncWrite + Send + Unpin + 'static,
    {
        // Try to get from pool first
        {
            // Use unwrap_or_else to handle poisoned mutex gracefully
            // If the mutex is poisoned, we recover the guard from the poison error
            let mut sessions = self.sessions.lock().unwrap_or_else(|poisoned| {
                eprintln!("warning: session pool lock was poisoned, recovering...");
                poisoned.into_inner()
            });

            // Cleanup expired and invalid sessions
            self.cleanup_expired_sessions(&mut sessions);

            // Check if there's an available session
            // First check if session exists and is valid, avoid complex operations while holding lock
            let session_valid = sessions.get(key).and_then(|session| {
                let is_valid = session.is_valid.lock().ok().map(|v| *v).unwrap_or(false);
                let is_finished = session._background_task.is_finished();
                if is_valid && !is_finished {
                    Some(session.send_request.clone())
                } else {
                    None
                }
            });

            if let Some(send_request) = session_valid {
                // Updatefinallywhen used between
                if let Some(session) = sessions.get(key) {
                    if let Ok(mut last_used) = session.last_used.lock() {
                        *last_used = Instant::now();
                    }
                }
                return Ok(send_request);
            }

            // Ifsession existsbutalreadyinvalid, remove它
            if sessions.contains_key(key) {
                sessions.remove(key);
            }
        }

        // If no available session, check if one is being created (Race Condition Fix)
        let rx = {
            // Use unwrap_or_else to handle poisoned mutex gracefully
            let mut pending = self
                .pending_sessions
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            if let Some(rx) = pending.get(key) {
                Some(rx.clone())
            } else {
                // marker as positive in Create
                let (_tx, rx) = watch::channel(false);
                pending.insert(key.to_string(), rx.clone());
                // herewe稍microviolate一downprinciple, in order tologiccleardirectly in herereturn None representweneedpersonallyCreate
                // butwewillpreserve tx in backcontinueuse
                None
            }
        };

        if let Some(mut rx) = rx {
            // waitoriginalCreatetaskcomplete
            let _ = rx.changed().await;
            // Createcompletebackrecursivecallending withGetnewCreatesession
            // Note: due to Fut limit, herecannotdirectlyrecursive, weactualupshould in outsidelayerloop
            // butin order tocodeconcise, weheredirectlyjump to reChecklogic
            return Box::pin(self.get_or_create_session(key, create_session)).await;
        }

        // personallyCreatenewsession
        let (send_request_h2, h2_conn) = create_session.await.inspect_err(|_e| {
            // Createfailurealsoneed from pending in remove
            if let Ok(mut pending) = self.pending_sessions.lock() {
                pending.remove(key);
            }
        })?;
        let send_request = Arc::new(TokioMutex::new(send_request_h2));
        let is_valid = Arc::new(Mutex::new(true));
        let is_valid_clone = is_valid.clone();
        let key_clone = key.to_string();
        let sessions_clone = self.sessions.clone();

        // startbackbackground taskmanageconnectionlifecycle
        let background_task = tokio::spawn(async move {
            // run h2_conn direct to connectionclose
            if let Err(e) = h2_conn.await {
                eprintln!("warning: HTTP/2 connectionerror ({}): {:?}", key_clone, e);
            }
            // connectionalreadyclose, marker as invalid
            if let Ok(mut valid) = is_valid_clone.lock() {
                *valid = false;
            }
            // from pool in removeinvalidsession
            if let Ok(mut sessions) = sessions_clone.lock() {
                sessions.remove(&key_clone);
            }
        });

        let session = Arc::new(H2Session {
            send_request: send_request.clone(),
            _background_task: background_task,
            last_used: Arc::new(Mutex::new(Instant::now())),
            is_valid,
        });

        // writepool in 并cleanup pending status
        {
            if let Ok(mut sessions) = self.sessions.lock() {
                sessions.insert(key.to_string(), session);
            }
            if let Ok(mut pending) = self.pending_sessions.lock() {
                pending.remove(key);
                // here不needexplicitnotification, tx destroywillautomaticnotification rx
            }
        }

        Ok(send_request)
    }

    /// cleanupexpire and invalidsession
    fn cleanup_expired_sessions(&self, sessions: &mut HashMap<String, Arc<H2Session>>) {
        let now = Instant::now();
        sessions.retain(|_key, session| {
            // Checksessionwhetherstillvalid
            let is_valid = session.is_valid.lock().map(|v| *v).unwrap_or(false);
            let is_finished = session._background_task.is_finished();

            // preservevalidsession, and notexpire, and backbackground task仍 in run
            if is_valid && !is_finished {
                if let Ok(last_used) = session.last_used.lock() {
                    now.duration_since(*last_used) < self.session_timeout
                } else {
                    true // Iflockfailure, preservesession
                }
            } else {
                false // removeinvalid or completedsession
            }
        });
    }

    /// removespecifiedsession
    pub fn remove_session(&self, key: &str) {
        if let Ok(mut sessions) = self.sessions.lock() {
            sessions.remove(key);
        }
    }

    /// cleanupallsession
    pub fn clear(&self) {
        if let Ok(mut sessions) = self.sessions.lock() {
            sessions.clear();
        }
    }
}

#[cfg(all(feature = "connection-pool", feature = "http2"))]
impl Default for H2SessionPool {
    fn default() -> Self {
        Self::new(Duration::from_secs(300)) // default 5 minutestimeout
    }
}
