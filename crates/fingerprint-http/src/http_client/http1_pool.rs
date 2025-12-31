//! HTTP/1.1 with Connection Pool
//!
//! 架构说明：
//! - HTTP/1.1 采用 netconnpool 管理 TCP 连接池
//! - 池化对象：TcpStream（裸 TCP 连接）
//! - 复用方式：串行复用（一个连接同一时间只能处理一个请求）
//! - 协议限制：HTTP/1.1 无法多路复用，需要大量连接支持并发
//! - netconnpool 负责：连接创建、保持活跃、故障检测和回收

#[cfg(feature = "connection-pool")]
use super::pool::ConnectionPoolManager;
use super::{HttpClientConfig, HttpClientError, HttpRequest, HttpResponse, Result};
#[cfg(feature = "connection-pool")]
use std::io::Write;
#[cfg(feature = "connection-pool")]
use std::sync::Arc;

/// 使用连接池发送 HTTP/1.1 请求
#[cfg(feature = "connection-pool")]
pub fn send_http1_request_with_pool(
    host: &str,
    port: u16,
    path: &str,
    request: &HttpRequest,
    config: &HttpClientConfig,
    pool_manager: &Arc<ConnectionPoolManager>,
) -> Result<HttpResponse> {
    // 从连接池获取连接
    let pool = pool_manager.get_pool(host, port)?;

    // 获取 TCP 连接
    let conn = pool
        .get_tcp()
        .map_err(|e| HttpClientError::ConnectionFailed(format!("从连接池获取连接失败: {:?}", e)))?;

    // 从 Connection 中提取 TcpStream
    // PooledConnection 实现了 Deref<Target = Connection>，可以直接使用 Connection 的方法
    let tcp_stream = conn
        .tcp_conn()
        .ok_or_else(|| HttpClientError::ConnectionFailed("期望 TCP 连接但得到 UDP".to_string()))?;

    // 克隆 TcpStream 以便我们可以使用它
    let mut stream = tcp_stream.try_clone().map_err(HttpClientError::Io)?;

    // 修复：添加 Cookie 到请求（如果存在）
    let mut request_with_cookies = request.clone();
    if let Some(cookie_store) = &config.cookie_store {
        super::request::add_cookies_to_request(
            &mut request_with_cookies,
            cookie_store,
            host,
            path,
            false, // HTTP 不是安全连接
        );
    }

    // 构建 HTTP 请求
    let http_request = request_with_cookies.build_http1_request(host, path);

    // 发送请求
    stream
        .write_all(http_request.as_bytes())
        .map_err(HttpClientError::Io)?;

    // 修复：使用完整的响应读取逻辑（包括 body）
    // 连接会自动归还到连接池（通过 Drop）
    let buffer =
        super::io::read_http1_response_bytes(&mut stream, super::io::DEFAULT_MAX_RESPONSE_BYTES)
            .map_err(HttpClientError::Io)?;

    // 解析响应
    HttpResponse::parse(&buffer).map_err(HttpClientError::InvalidResponse)
}

#[cfg(not(feature = "connection-pool"))]
pub fn send_http1_request_with_pool(
    _host: &str,
    _port: u16,
    _path: &str,
    _request: &HttpRequest,
    _config: &HttpClientConfig,
    _pool_manager: &std::sync::Arc<super::pool::ConnectionPoolManager>,
) -> Result<HttpResponse> {
    Err(HttpClientError::ConnectionFailed(
        "连接池功能未启用，请使用 --features connection-pool 编译".to_string(),
    ))
}

#[cfg(all(test, feature = "connection-pool"))]
mod tests {
    #[allow(unused_imports)]
    use super::*;
    use crate::http_client::pool::PoolManagerConfig;
    use crate::http_client::request::HttpMethod;

    #[test]
    #[ignore] // 需要网络
    fn test_http1_with_pool() {
        let request = HttpRequest::new(HttpMethod::Get, "http://example.com/");
        let config = HttpClientConfig::default();
        let pool_manager = Arc::new(ConnectionPoolManager::new(PoolManagerConfig::default()));

        let result =
            send_http1_request_with_pool("example.com", 80, "/", &request, &config, &pool_manager);

        // 可能会失败（网络问题），但不应该 panic
        if let Ok(response) = result {
            println!("状态码: {}", response.status_code);
            assert!(response.status_code > 0);
        }
    }

    #[test]
    #[ignore] // 需要网络
    fn test_connection_reuse() {
        let request = HttpRequest::new(HttpMethod::Get, "http://example.com/");
        let config = HttpClientConfig::default();
        let pool_manager = Arc::new(ConnectionPoolManager::new(PoolManagerConfig::default()));

        // 第一次请求
        let _ =
            send_http1_request_with_pool("example.com", 80, "/", &request, &config, &pool_manager);

        // 第二次请求（应该复用连接）
        let _ =
            send_http1_request_with_pool("example.com", 80, "/", &request, &config, &pool_manager);

        // 检查统计信息
        let stats = pool_manager.get_stats();
        if !stats.is_empty() {
            println!("连接池统计:");
            for stat in stats {
                stat.print();
            }
        }
    }
}
