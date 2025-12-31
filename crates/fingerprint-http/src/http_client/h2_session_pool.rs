//! HTTP/2 会话池
//!
//! 池化 h2::client::SendRequest 句柄，实现真正的 HTTP/2 多路复用
//! 避免每次请求都重新进行 TLS 和 HTTP/2 握手

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

/// HTTP/2 会话池管理器
/// 修复：池化 SendRequest 句柄，实现真正的多路复用
#[cfg(all(feature = "connection-pool", feature = "http2"))]
pub struct H2SessionPool {
    /// 会话池（按 host:port 分组）
    /// 每个会话包含 SendRequest 句柄、后台任务句柄和最后使用时间
    sessions: Arc<Mutex<HashMap<String, Arc<H2Session>>>>,
    /// 正在创建中的会话（避免相同 key 的并发创建竞争）
    pending_sessions: Arc<Mutex<HashMap<String, watch::Receiver<bool>>>>,
    /// 会话超时时间（默认 5 分钟）
    session_timeout: Duration,
}

/// HTTP/2 会话
#[cfg(all(feature = "connection-pool", feature = "http2"))]
struct H2Session {
    /// SendRequest 句柄（用于发送请求）
    send_request: Arc<TokioMutex<SendRequest<bytes::Bytes>>>,
    /// 后台任务句柄（用于管理 h2_conn 生命周期）
    /// 当连接失效时，任务会结束，我们可以检测到并移除会话
    _background_task: tokio::task::JoinHandle<()>,
    /// 最后使用时间
    last_used: Arc<Mutex<Instant>>,
    /// 连接是否有效（由后台任务更新）
    is_valid: Arc<Mutex<bool>>,
}

#[cfg(all(feature = "connection-pool", feature = "http2"))]
impl H2SessionPool {
    /// 创建新的会话池
    pub fn new(session_timeout: Duration) -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
            pending_sessions: Arc::new(Mutex::new(HashMap::new())),
            session_timeout,
        }
    }

    /// 获取或创建 HTTP/2 会话
    /// 返回 SendRequest 句柄的克隆
    /// create_session: 创建新会话的异步函数，返回 (SendRequest, Connection)
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
        // 先尝试从池中获取
        {
            let mut sessions = self.sessions.lock().unwrap_or_else(|e| {
                eprintln!("警告: 会话池锁失败: {}", e);
                // 如果锁失败，尝试从中毒的锁中恢复
                drop(e.into_inner());
                self.sessions.lock().expect("无法获取会话池锁")
            });

            // 清理过期和失效的会话
            self.cleanup_expired_sessions(&mut sessions);

            // 检查是否有可用的会话
            // 先检查会话是否存在且有效，避免在持有锁时进行复杂操作
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

            // 如果会话存在但已失效，移除它
            if sessions.contains_key(key) {
                sessions.remove(key);
            }
        }

        // 如果没有可用会话，检查是否正在创建中 (Race Condition Fix)
        let rx = {
            let mut pending = self
                .pending_sessions
                .lock()
                .unwrap_or_else(|e| e.into_inner());
            if let Some(rx) = pending.get(key) {
                Some(rx.clone())
            } else {
                // 标记为正在创建
                let (_tx, rx) = watch::channel(false);
                pending.insert(key.to_string(), rx.clone());
                // 这里我们稍微违反一下原则，为了逻辑清晰直接在这里返回 None 表示我们需要亲自创建
                // 但我们会保留 tx 在后续使用
                None
            }
        };

        if let Some(mut rx) = rx {
            // 等待原有创建任务完成
            let _ = rx.changed().await;
            // 创建完成后递归调用以获取新创建的会话
            // 注意：由于 Fut 的限制，这里不能直接递归，我们实际上应该在外层循环
            // 但为了代码简洁，我们这里直接跳转到重新检查逻辑
            return Box::pin(self.get_or_create_session(key, create_session)).await;
        }

        // 亲自创建新会话
        let (send_request_h2, h2_conn) = create_session.await.inspect_err(|_e| {
            // 创建失败也需要从 pending 中移除
            if let Ok(mut pending) = self.pending_sessions.lock() {
                pending.remove(key);
            }
        })?;
        let send_request = Arc::new(TokioMutex::new(send_request_h2));
        let is_valid = Arc::new(Mutex::new(true));
        let is_valid_clone = is_valid.clone();
        let key_clone = key.to_string();
        let sessions_clone = self.sessions.clone();

        // 启动后台任务管理连接生命周期
        let background_task = tokio::spawn(async move {
            // 运行 h2_conn 直到连接关闭
            if let Err(e) = h2_conn.await {
                eprintln!("警告: HTTP/2 连接错误 ({}): {:?}", key_clone, e);
            }
            // 连接已关闭，标记为无效
            if let Ok(mut valid) = is_valid_clone.lock() {
                *valid = false;
            }
            // 从池中移除失效的会话
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

        // 写入池中并清理 pending 状态
        {
            if let Ok(mut sessions) = self.sessions.lock() {
                sessions.insert(key.to_string(), session);
            }
            if let Ok(mut pending) = self.pending_sessions.lock() {
                pending.remove(key);
                // 这里不需要显式通知，tx 销毁会自动通知 rx
            }
        }

        Ok(send_request)
    }

    /// 清理过期和失效的会话
    fn cleanup_expired_sessions(&self, sessions: &mut HashMap<String, Arc<H2Session>>) {
        let now = Instant::now();
        sessions.retain(|_key, session| {
            // 检查会话是否仍然有效
            let is_valid = session.is_valid.lock().map(|v| *v).unwrap_or(false);
            let is_finished = session._background_task.is_finished();

            // 保留有效的会话，且未过期，且后台任务仍在运行
            if is_valid && !is_finished {
                if let Ok(last_used) = session.last_used.lock() {
                    now.duration_since(*last_used) < self.session_timeout
                } else {
                    true // 如果锁失败，保留会话
                }
            } else {
                false // 移除失效或已完成的会话
            }
        });
    }

    /// 移除指定会话
    pub fn remove_session(&self, key: &str) {
        if let Ok(mut sessions) = self.sessions.lock() {
            sessions.remove(key);
        }
    }

    /// 清理所有会话
    pub fn clear(&self) {
        if let Ok(mut sessions) = self.sessions.lock() {
            sessions.clear();
        }
    }
}

#[cfg(all(feature = "connection-pool", feature = "http2"))]
impl Default for H2SessionPool {
    fn default() -> Self {
        Self::new(Duration::from_secs(300)) // 默认 5 分钟超时
    }
}
