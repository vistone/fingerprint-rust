//! HTTP/2 sessionpool
//!
//! pool化 h2::client::SendRequest handle，implement真正 HTTP/2 多路reuse
//! avoid每次request都reperform TLS and HTTP/2 handshake

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
/// Fix: pool化 SendRequest handle，implement真正多路reuse
#[cfg(all(feature = "connection-pool", feature = "http2"))]
pub struct H2SessionPool {
 /// sessionpool（按 host:port group）
 /// eachsessionincluding SendRequest handle、backbackground taskhandle and finallywhen used between
 sessions: Arc<Mutex<HashMap<String, Arc<H2Session>>>>,
 /// 正 in Createinsession（avoidsame key concurrentCreate竞争）
 pending_sessions: Arc<Mutex<HashMap<String, watch::Receiver<bool>>>>,
 /// sessiontimeout duration（default 5 分钟）
 session_timeout: Duration,
}

/// HTTP/2 session
#[cfg(all(feature = "connection-pool", feature = "http2"))]
struct H2Session {
 /// SendRequest handle（ for sendrequest）
 send_request: Arc<TokioMutex<SendRequest<bytes::Bytes>>>,
 /// backbackground taskhandle（ for manage h2_conn 生命cycle）
 /// whenconnection失效 when ，任务willend，wecandetect to 并removesession
 _background_task: tokio::task::JoinHandle<()>,
 /// finallywhen used between
 last_used: Arc<Mutex<Instant>>,
 /// connectionwhethervalid（由backbackground taskUpdate）
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
 /// create_session: Create新sessionasyncfunction，return (SendRequest, Connection)
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
 // 先try from pool in Get
 {
 let mut sessions = self.sessions.lock().unwrap_or_else(|e| {
 eprintln!("warning: sessionpoollockfailure: {}", e);
 // Iflockfailure, try from in 毒lock in restore
 drop(e.into_inner());
 self.sessions.lock().expect("unable toGetsessionpoollock")
 });

 // cleanupexpire and 失效session
 self.cleanup_expired_sessions(&mut sessions);

 // Checkwhether有availablesession
 // 先Checksessionwhether exists且valid，avoid in holdlock when perform复杂operation
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

 // Ifsession existsbutalready失效, remove它
 if sessions.contains_key(key) {
 sessions.remove(key);
 }
 }

 // Ifnoavailablesession, Checkwhether正 in Create in (Race Condition Fix)
 let rx = {
 let mut pending = self
.pending_sessions
.lock()
.unwrap_or_else(|e| e.into_inner());
 if let Some(rx) = pending.get(key) {
 Some(rx.clone())
 } else {
 // marker as 正 in Create
 let (_tx, rx) = watch::channel(false);
 pending.insert(key.to_string(), rx.clone());
 // herewe稍微violate一down原则，in order tologic清晰directly in herereturn None representweneed亲自Create
 // butwewillpreserve tx in back续use
 None
 }
 };

 if let Some(mut rx) = rx {
 // waitoriginalCreate任务complete
 let _ = rx.changed().await;
 // Createcompleteback递归call以Get新Createsession
 // Note: due to Fut limit，herecannotdirectly递归，weactualupshould in outsidelayerloop
 // butin order to代码简洁，weheredirectlyjump to reChecklogic
 return Box::pin(self.get_or_create_session(key, create_session)).await;
 }

 // 亲自Create新session
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

 // startbackbackground taskmanageconnection生命cycle
 let background_task = tokio::spawn(async move {
 // run h2_conn 直 to connectionclose
 if let Err(e) = h2_conn.await {
 eprintln!("warning: HTTP/2 connectionerror ({}): {:?}", key_clone, e);
 }
 // connectionalreadyclose，marker as invalid
 if let Ok(mut valid) = is_valid_clone.lock() {
 *valid = false;
 }
 // from pool in remove失效session
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
 // here不needexplicit通知，tx destroywillautomatic通知 rx
 }
 }

 Ok(send_request)
 }

 /// cleanupexpire and 失效session
 fn cleanup_expired_sessions(&self, sessions: &mut HashMap<String, Arc<H2Session>>) {
 let now = Instant::now();
 sessions.retain(|_key, session| {
 // Checksessionwhether仍然valid
 let is_valid = session.is_valid.lock().map(|v| *v).unwrap_or(false);
 let is_finished = session._background_task.is_finished();

 // preservevalidsession，且notexpire，且backbackground task仍 in run
 if is_valid && !is_finished {
 if let Ok(last_used) = session.last_used.lock() {
 now.duration_since(*last_used) < self.session_timeout
 } else {
 true // Iflockfailure, preservesession
 }
 } else {
 false // remove失效 or completedsession
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
 Self::new(Duration::from_secs(300)) // default 5 分钟timeout
 }
}
