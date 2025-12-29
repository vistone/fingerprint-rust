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
use tokio::sync::Mutex as TokioMutex;

#[cfg(all(feature = "connection-pool", feature = "http2"))]
use h2::client::SendRequest;

/// HTTP/2 会话池管理器
/// 修复：池化 SendRequest 句柄，实现真正的多路复用
/// 注意：当前未使用，为未来的架构改进预留
#[cfg(all(feature = "connection-pool", feature = "http2"))]
#[allow(dead_code)]
pub struct H2SessionPool {
    /// 会话池（按 host:port 分组）
    /// 每个会话包含 SendRequest 句柄和最后使用时间
    sessions: Arc<Mutex<HashMap<String, Arc<H2Session>>>>,
    /// 会话超时时间（默认 5 分钟）
    session_timeout: Duration,
}

/// HTTP/2 会话
/// 注意：当前未使用，为未来的架构改进预留
#[cfg(all(feature = "connection-pool", feature = "http2"))]
#[allow(dead_code)]
struct H2Session {
    /// SendRequest 句柄（用于发送请求）
    send_request: Arc<TokioMutex<SendRequest<bytes::Bytes>>>,
    /// 最后使用时间
    last_used: Arc<Mutex<Instant>>,
}

#[cfg(all(feature = "connection-pool", feature = "http2"))]
impl H2SessionPool {
    /// 创建新的会话池
    /// 注意：当前未使用，为未来的架构改进预留
    #[allow(dead_code)]
    pub fn new(session_timeout: Duration) -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
            session_timeout,
        }
    }

    /// 获取或创建 HTTP/2 会话
    /// 返回 SendRequest 句柄的克隆
    /// 注意：当前未使用，为未来的架构改进预留
    #[allow(dead_code)]
    pub async fn get_or_create_session(
        &self,
        key: &str,
        create_session: impl std::future::Future<Output = Result<SendRequest<bytes::Bytes>>>,
    ) -> Result<Arc<TokioMutex<SendRequest<bytes::Bytes>>>> {
        // 先尝试从池中获取
        {
            let mut sessions = self.sessions.lock().unwrap_or_else(|e| {
                eprintln!("警告: 会话池锁失败: {}", e);
                // 如果锁失败，尝试从中毒的锁中恢复
                drop(e.into_inner());
                self.sessions.lock().expect("无法获取会话池锁")
            });

            // 清理过期会话
            self.cleanup_expired_sessions(&mut sessions);

            // 检查是否有可用的会话
            if let Some(session) = sessions.get(key) {
                // 更新最后使用时间
                if let Ok(mut last_used) = session.last_used.lock() {
                    *last_used = Instant::now();
                }
                return Ok(session.send_request.clone());
            }
        }

        // 创建新会话
        let send_request = create_session.await?;
        let send_request = Arc::new(TokioMutex::new(send_request));

        let session = Arc::new(H2Session {
            send_request: send_request.clone(),
            last_used: Arc::new(Mutex::new(Instant::now())),
        });

        // 添加到池中
        {
            let mut sessions = self.sessions.lock().unwrap_or_else(|e| {
                eprintln!("警告: 会话池锁失败: {}", e);
                // 如果锁失败，尝试从中毒的锁中恢复
                drop(e.into_inner());
                self.sessions.lock().expect("无法获取会话池锁")
            });
            sessions.insert(key.to_string(), session);
        }

        Ok(send_request)
    }

    /// 清理过期会话
    /// 注意：当前未使用，为未来的架构改进预留
    #[allow(dead_code)]
    fn cleanup_expired_sessions(&self, sessions: &mut HashMap<String, Arc<H2Session>>) {
        let now = Instant::now();
        sessions.retain(|_key, session| {
            if let Ok(last_used) = session.last_used.lock() {
                now.duration_since(*last_used) < self.session_timeout
            } else {
                true // 如果锁失败，保留会话
            }
        });
    }

    /// 移除指定会话
    /// 注意：当前未使用，为未来的架构改进预留
    #[allow(dead_code)]
    pub fn remove_session(&self, key: &str) {
        if let Ok(mut sessions) = self.sessions.lock() {
            sessions.remove(key);
        }
    }

    /// 清理所有会话
    /// 注意：当前未使用，为未来的架构改进预留
    #[allow(dead_code)]
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
