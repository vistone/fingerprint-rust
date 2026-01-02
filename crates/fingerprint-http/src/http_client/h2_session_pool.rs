//! HTTP/2 sessionpool
//!
//! pool化 h2::client::SendRequest 句柄，implement真正 HTTP/2 多路复用
//! 避免每次request都重新进行 TLS  and HTTP/2 handshake

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

/// HTTP/2 sessionpool管理器
/// Fix: pool化 SendRequest 句柄，implement真正的多路复用
#[cfg(all(feature = "connection-pool", feature = "http2"))]
pub struct H2SessionPool {
    /// sessionpool（按 host:port 分组）
    /// eachsessionincluding SendRequest 句柄、back台任务句柄 and finallywhen used 间
    sessions: Arc<Mutex<HashMap<String, Arc<H2Session>>>>,
    /// 正 in Create中的session（避免相同 key 的并发Create竞争）
    pending_sessions: Arc<Mutex<HashMap<String, watch::Receiver<bool>>>>,
    /// sessiontimeout duration（default 5 分钟）
    session_timeout: Duration,
}

/// HTTP/2 session
#[cfg(all(feature = "connection-pool", feature = "http2"))]
struct H2Session {
    /// SendRequest 句柄（ for sendrequest）
    send_request: Arc<TokioMutex<SendRequest<bytes::Bytes>>>,
    /// back台任务句柄（ for 管理 h2_conn 生命周期）
    /// whenconnection失效 when ，任务willend，我们can检测 to 并removesession
    _background_task: tokio::task::JoinHandle<()>,
    /// finallywhen used 间
    last_used: Arc<Mutex<Instant>>,
    /// connectionwhethervalid（由back台任务Update）
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
    /// return SendRequest 句柄的克隆
    /// create_session: Create新session的asyncfunction，return (SendRequest, Connection)
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
        // 先try from pool中Get
        {
            let mut sessions = self.sessions.lock().unwrap_or_else(|e| {
                eprintln!("warning: sessionpool锁failure: {}", e);
                // If锁failure, try from 中毒的锁中恢复
                drop(e.into_inner());
                self.sessions.lock().expect("unable toGetsessionpool锁")
            });

            // 清理过期 and 失效的session
            self.cleanup_expired_sessions(&mut sessions);

            // Checkwhether有available的session
            // 先Checksessionwhether exists且valid，避免 in 持有锁 when 进行复杂操作
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
                // Updatefinallywhen used 间
                if let Some(session) = sessions.get(key) {
                    if let Ok(mut last_used) = session.last_used.lock() {
                        *last_used = Instant::now();
                    }
                }
                return Ok(send_request);
            }

            // Ifsession existsbutalready失效, remove它
            if sessions.contains_key(key) {
                sessions.remove(key);
            }
        }

        // If没有availablesession, Checkwhether正 in Create中 (Race Condition Fix)
        let rx = {
            let mut pending = self
                .pending_sessions
                .lock()
                .unwrap_or_else(|e| e.into_inner());
            if let Some(rx) = pending.get(key) {
                Some(rx.clone())
            } else {
                // 标记为正 in Create
                let (_tx, rx) = watch::channel(false);
                pending.insert(key.to_string(), rx.clone());
                // 这里我们稍微违反一down原则，为了逻辑清晰直接 in 这里return None 表示我们need亲自Create
                // but我们will保留 tx  in back续use
                None
            }
        };

        if let Some(mut rx) = rx {
            // wait原有Create任务complete
            let _ = rx.changed().await;
            // Createcompleteback递归call以Get新Create的session
            // Note: 由于 Fut 的limit，这里不能直接递归，我们实际upshould in outsidelayer循环
            // but为了代码简洁，我们这里直接跳转 to 重新Check逻辑
            return Box::pin(self.get_or_create_session(key, create_session)).await;
        }

        // 亲自Create新session
        let (send_request_h2, h2_conn) = create_session.await.inspect_err(|_e| {
            // Createfailurealsoneed from  pending 中remove
            if let Ok(mut pending) = self.pending_sessions.lock() {
                pending.remove(key);
            }
        })?;
        let send_request = Arc::new(TokioMutex::new(send_request_h2));
        let is_valid = Arc::new(Mutex::new(true));
        let is_valid_clone = is_valid.clone();
        let key_clone = key.to_string();
        let sessions_clone = self.sessions.clone();

        // startback台任务管理connection生命周期
        let background_task = tokio::spawn(async move {
            // run h2_conn 直 to connectionclose
            if let Err(e) = h2_conn.await {
                eprintln!("warning: HTTP/2 connectionerror ({}): {:?}", key_clone, e);
            }
            // connectionalreadyclose，标记为invalid
            if let Ok(mut valid) = is_valid_clone.lock() {
                *valid = false;
            }
            //  from pool中remove失效的session
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

        // writepool中并清理 pending status
        {
            if let Ok(mut sessions) = self.sessions.lock() {
                sessions.insert(key.to_string(), session);
            }
            if let Ok(mut pending) = self.pending_sessions.lock() {
                pending.remove(key);
                // 这里不need显式通知，tx 销毁willautomatic通知 rx
            }
        }

        Ok(send_request)
    }

    /// 清理过期 and 失效的session
    fn cleanup_expired_sessions(&self, sessions: &mut HashMap<String, Arc<H2Session>>) {
        let now = Instant::now();
        sessions.retain(|_key, session| {
            // Checksessionwhether仍然valid
            let is_valid = session.is_valid.lock().map(|v| *v).unwrap_or(false);
            let is_finished = session._background_task.is_finished();

            // 保留valid的session，且not过期，且back台任务仍 in run
            if is_valid && !is_finished {
                if let Ok(last_used) = session.last_used.lock() {
                    now.duration_since(*last_used) < self.session_timeout
                } else {
                    true // If锁failure, 保留session
                }
            } else {
                false // remove失效 or completed的session
            }
        });
    }

    /// removespecifiedsession
    pub fn remove_session(&self, key: &str) {
        if let Ok(mut sessions) = self.sessions.lock() {
            sessions.remove(key);
        }
    }

    /// 清理allsession
    pub fn clear(&self) {
        if let Ok(mut sessions) = self.sessions.lock() {
            sessions.clear();
        }
    }
}

#[cfg(all(feature = "connection-pool", feature = "http2"))]
impl Default for H2SessionPool {
    fn default() -> Self {
        Self::new(Duration::from_secs(300)) // default 5 分钟timeout
    }
}
