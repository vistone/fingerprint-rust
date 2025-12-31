//! HTTP/3 会话池
//!
//! 池化 h3::client::SendRequest 句柄，实现真正的 HTTP/3 多路复用
//! 避免每次请求都重新进行 QUIC 握手和 HTTP/3 连接建立

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

/// HTTP/3 会话池管理器
#[cfg(all(feature = "connection-pool", feature = "http3"))]
pub struct H3SessionPool {
    /// 会话池（按 host:port 分组）
    sessions: Arc<Mutex<HashMap<String, Arc<H3Session>>>>,
    /// 正在创建中的会话（避免竞争）
    pending_sessions: Arc<Mutex<HashMap<String, watch::Receiver<bool>>>>,
    /// 会话超时时间（默认 5 分钟）
    session_timeout: Duration,
}

/// HTTP/3 会话
#[cfg(all(feature = "connection-pool", feature = "http3"))]
struct H3Session {
    /// SendRequest 句柄（用于发送请求）
    send_request: Arc<TokioMutex<SendRequest<h3_quinn::OpenStreams, bytes::Bytes>>>,
    /// 后台任务句柄（用于管理 h3 连接驱动）
    _background_task: tokio::task::JoinHandle<()>,
    /// 最后使用时间
    last_used: Arc<Mutex<Instant>>,
    /// 连接是否有效
    is_valid: Arc<Mutex<bool>>,
}

#[cfg(all(feature = "connection-pool", feature = "http3"))]
impl H3SessionPool {
    /// 创建新的会话池
    pub fn new(session_timeout: Duration) -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
            pending_sessions: Arc::new(Mutex::new(HashMap::new())),
            session_timeout,
        }
    }

    /// 获取或创建 HTTP/3 会话
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
        // 尝试从池中获取
        {
            let mut sessions = self.sessions.lock().unwrap_or_else(|e| {
                eprintln!("警告: H3 会话池锁失败: {}", e);
                drop(e.into_inner());
                self.sessions.lock().expect("无法获取 H3 会话池锁")
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
                // 更新最后使用时间
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

        // 检查是否正在创建中 (Race Condition Fix)
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

        // 创建新会话
        let result = create_session.await;

        // 无论成功失败，都从 pending 中移除
        if let Ok(mut pending) = self.pending_sessions.lock() {
            pending.remove(key);
        }

        let (mut driver, send_request_h3) = result?;
        let send_request = Arc::new(TokioMutex::new(send_request_h3));
        let is_valid = Arc::new(Mutex::new(true));
        let is_valid_clone = is_valid.clone();
        let key_clone = key.to_string();
        let sessions_clone = self.sessions.clone();

        // 启动后台任务管理连接生命周期
        let background_task = tokio::spawn(async move {
            // 驱动 h3 连接
            let _ = std::future::poll_fn(|cx| driver.poll_close(cx)).await;

            // 标记为无效
            if let Ok(mut valid) = is_valid_clone.lock() {
                *valid = false;
            }
            // 从池中移除
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

        // 添加到池中
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
