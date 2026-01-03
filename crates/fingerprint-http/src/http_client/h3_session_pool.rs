//! HTTP/3 sessionpool
//!
//! pool化 h3::client::SendRequest handle，implementtrue HTTP/3 multiplereuse
//! avoideach timerequest都reperform QUIC handshake and HTTP/3 connectionestablish

#[cfg(all(feature = "connection-pool", feature = "http3"))]
use super::Result;
#[cfg(all(feature = "connection-pool", feature = "http3"))]
use std::collections::HashMap;
#[cfg(all(feature = "connection-pool", feature = "http3"))]
use std::sync::{Arc, Mutex};
#[cfg(all(feature = "connection-pool", feature = "http3"))]
use std::time::{Duration, Instant};
#[cfg(all(feature = "connection-pool", feature = "http3"))]
use tokio::sync::watch;
#[cfg(all(feature = "connection-pool", feature = "http3"))]
use tokio::sync::Mutex as TokioMutex;

#[cfg(all(feature = "connection-pool", feature = "http3"))]
use h3::client::SendRequest;

/// HTTP/3 sessionpoolmanageer
#[cfg(all(feature = "connection-pool", feature = "http3"))]
pub struct H3SessionPool {
 /// sessionpool ( by  host:port group)
 sessions: Arc<Mutex<HashMap<String, Arc<H3Session>>>>,
 /// 正 in Createinsession (avoidcompetition)
 pending_sessions: Arc<Mutex<HashMap<String, watch::Receiver<bool>>>>,
 /// sessiontimeout duration (default 5 minutes)
 session_timeout: Duration,
}

/// HTTP/3 session
#[cfg(all(feature = "connection-pool", feature = "http3"))]
struct H3Session {
 /// SendRequest handle ( for sendrequest)
 send_request: Arc<TokioMutex<SendRequest<h3_quinn::OpenStreams, bytes::Bytes>>>,
 /// backbackground taskhandle ( for manage h3 connectiondriver)
 _background_task: tokio::task::JoinHandle<()>,
 /// finallywhen used between
 last_used: Arc<Mutex<Instant>>,
 /// connectionwhethervalid
 is_valid: Arc<Mutex<bool>>,
}

#[cfg(all(feature = "connection-pool", feature = "http3"))]
impl H3SessionPool {
 /// Create a newsessionpool
 pub fn new(session_timeout: Duration) -> Self {
 Self {
 sessions: Arc::new(Mutex::new(HashMap::new())),
 pending_sessions: Arc::new(Mutex::new(HashMap::new())),
 session_timeout,
 }
 }

 /// Get or Create HTTP/3 session
 pub async fn get_or_create_session<Fut>(
 &self,
 key: &str,
 create_session: Fut,
 ) -> Result<Arc<TokioMutex<SendRequest<h3_quinn::OpenStreams, bytes::Bytes>>>>
 where
 Fut: std::future::Future<
 Output = Result<(
 h3::client::Connection<h3_quinn::Connection, bytes::Bytes>,
 SendRequest<h3_quinn::OpenStreams, bytes::Bytes>,
 )>,
 >,
 {
 // try from pool in Get
 {
 let mut sessions = self.sessions.lock().unwrap_or_else(|e| {
 eprintln!("warning: H3 sessionpoollockfailure: {}", e);
 drop(e.into_inner());
 self.sessions.lock().expect("unable toGet H3 sessionpoollock")
 });

 self.cleanup_expired_sessions(&mut sessions);

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

 if sessions.contains_key(key) {
 sessions.remove(key);
 }
 }

 // Checkwhether正 in Create in (Race Condition Fix)
 let rx = {
 let mut pending = self
.pending_sessions
.lock()
.unwrap_or_else(|e| e.into_inner());
 if let Some(rx) = pending.get(key) {
 Some(rx.clone())
 } else {
 let (_tx, rx) = watch::channel(false);
 pending.insert(key.to_string(), rx.clone());
 None
 }
 };

 if let Some(mut rx) = rx {
 let _ = rx.changed().await;
 return Box::pin(self.get_or_create_session(key, create_session)).await;
 }

 // Createnewsession
 let result = create_session.await;

 // none论successfailure，都 from pending in remove
 if let Ok(mut pending) = self.pending_sessions.lock() {
 pending.remove(key);
 }

 let (mut driver, send_request_h3) = result?;
 let send_request = Arc::new(TokioMutex::new(send_request_h3));
 let is_valid = Arc::new(Mutex::new(true));
 let is_valid_clone = is_valid.clone();
 let key_clone = key.to_string();
 let sessions_clone = self.sessions.clone();

 // startbackbackground taskmanageconnectionlifecycle
 let background_task = tokio::spawn(async move {
 // driver h3 connection
 let _ = std::future::poll_fn(|cx| driver.poll_close(cx)).await;

 // marker as invalid
 if let Ok(mut valid) = is_valid_clone.lock() {
 *valid = false;
 }
 // from pool in remove
 if let Ok(mut sessions) = sessions_clone.lock() {
 sessions.remove(&key_clone);
 }
 });

 let session = Arc::new(H3Session {
 send_request: send_request.clone(),
 _background_task: background_task,
 last_used: Arc::new(Mutex::new(Instant::now())),
 is_valid,
 });

 // Add to pool in 
 {
 if let Ok(mut sessions) = self.sessions.lock() {
 sessions.insert(key.to_string(), session);
 }
 }

 Ok(send_request)
 }

 fn cleanup_expired_sessions(&self, sessions: &mut HashMap<String, Arc<H3Session>>) {
 let now = Instant::now();
 sessions.retain(|_key, session| {
 let is_valid = session.is_valid.lock().map(|v| *v).unwrap_or(false);
 let is_finished = session._background_task.is_finished();

 if is_valid && !is_finished {
 if let Ok(last_used) = session.last_used.lock() {
 now.duration_since(*last_used) < self.session_timeout
 } else {
 true
 }
 } else {
 false
 }
 });
 }

 pub fn remove_session(&self, key: &str) {
 if let Ok(mut sessions) = self.sessions.lock() {
 sessions.remove(key);
 }
 }

 pub fn clear(&self) {
 if let Ok(mut sessions) = self.sessions.lock() {
 sessions.clear();
 }
 }
}

#[cfg(all(feature = "connection-pool", feature = "http3"))]
impl Default for H3SessionPool {
 fn default() -> Self {
 Self::new(Duration::from_secs(300))
 }
}
