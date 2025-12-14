//! HTTP/1.1 with Connection Pool
//!
//! 使用 netconnpool 管理 TCP 连接复用

#[cfg(feature = "connection-pool")]
use super::pool::ConnectionPoolManager;
use super::{HttpClientConfig, HttpClientError, HttpRequest, HttpResponse, Result};
#[cfg(feature = "connection-pool")]
use std::io::{Read, Write};
#[cfg(feature = "connection-pool")]
use std::sync::Arc;

/// 使用连接池发送 HTTP/1.1 请求
#[cfg(feature = "connection-pool")]
pub fn send_http1_request_with_pool(
    host: &str,
    port: u16,
    path: &str,
    request: &HttpRequest,
    _config: &HttpClientConfig,
    pool_manager: &Arc<ConnectionPoolManager>,
) -> Result<HttpResponse> {
    // 从连接池获取连接
    let pool = pool_manager.get_pool(host, port)?;

    // 获取 TCP 连接
    let conn = pool
        .GetTCP()
        .map_err(|e| HttpClientError::ConnectionFailed(format!("从连接池获取连接失败: {:?}", e)))?;

    // 从 Connection 中提取 TcpStream
    let tcp_stream = conn
        .GetTcpConn()
        .ok_or_else(|| HttpClientError::ConnectionFailed("期望 TCP 连接但得到 UDP".to_string()))?;

    // 克隆 TcpStream 以便我们可以使用它
    let mut stream = tcp_stream.try_clone().map_err(HttpClientError::Io)?;

    // 构建 HTTP 请求
    let http_request = request.build_http1_request(host, path);

    // 发送请求
    stream
        .write_all(http_request.as_bytes())
        .map_err(HttpClientError::Io)?;

    // 读取响应
    let mut buffer = Vec::new();
    let mut temp_buffer = [0u8; 8192];

    loop {
        match stream.read(&mut temp_buffer) {
            Ok(0) => break, // 连接关闭
            Ok(n) => {
                buffer.extend_from_slice(&temp_buffer[..n]);
                // 检查是否读取完整
                if buffer.ends_with(b"\r\n\r\n") || buffer.ends_with(b"\n\n") {
                    break;
                }
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => continue,
            Err(e) => return Err(HttpClientError::Io(e)),
        }
    }

    // 连接会自动归还到连接池（通过 Drop）

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
