//! HTTP/1.1 实现
//!
//! 使用 netconnpool 管理 TCP 连接，发送 HTTP/1.1 请求

use super::{HttpClientConfig, HttpClientError, HttpRequest, HttpResponse, Result};
use std::io::Write;
use std::net::TcpStream;

/// 发送 HTTP/1.1 请求
pub fn send_http1_request(
    host: &str,
    port: u16,
    path: &str,
    request: &HttpRequest,
    config: &HttpClientConfig,
) -> Result<HttpResponse> {
    // 连接服务器
    let addr = format!("{}:{}", host, port);
    let mut stream = TcpStream::connect(&addr)
        .map_err(|e| HttpClientError::ConnectionFailed(format!("连接失败 {}: {}", addr, e)))?;

    // 设置超时
    stream
        .set_read_timeout(Some(config.read_timeout))
        .map_err(HttpClientError::Io)?;
    stream
        .set_write_timeout(Some(config.write_timeout))
        .map_err(HttpClientError::Io)?;

    // 构建并发送 HTTP/1.1 请求
    let http_request = request.build_http1_request_bytes(host, path);
    stream
        .write_all(&http_request)
        .map_err(HttpClientError::Io)?;
    stream.flush().map_err(HttpClientError::Io)?;

    // 读取响应
    let mut stream = stream;
    let buffer = super::io::read_http1_response_bytes(
        &mut stream,
        super::io::DEFAULT_MAX_RESPONSE_BYTES,
    )
    .map_err(HttpClientError::Io)?;

    // 解析响应
    HttpResponse::parse(&buffer).map_err(HttpClientError::InvalidResponse)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore] // 需要网络连接
    fn test_send_http1_request() {
        let request = HttpRequest::new(
            crate::http_client::request::HttpMethod::Get,
            "http://example.com",
        )
        .with_user_agent("TestClient/1.0");

        let config = HttpClientConfig::default();
        let response = send_http1_request("example.com", 80, "/", &request, &config).unwrap();

        if response.status_code == 200 {
            assert!(response.is_success());
        } else {
            println!("⚠️  Server returned {}", response.status_code);
        }
    }
}
