//! HTTP/1.1 实现
//!
//! 使用 netconnpool 管理 TCP 连接，发送 HTTP/1.1 请求

use super::{HttpClientConfig, HttpClientError, HttpRequest, HttpResponse, Result};
use std::io::{Read, Write};
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
        .map_err(|e| HttpClientError::Io(e))?;
    stream
        .set_write_timeout(Some(config.write_timeout))
        .map_err(|e| HttpClientError::Io(e))?;

    // 构建并发送 HTTP/1.1 请求
    let http_request = request.build_http1_request(host, path);
    stream
        .write_all(http_request.as_bytes())
        .map_err(|e| HttpClientError::Io(e))?;
    stream.flush().map_err(|e| HttpClientError::Io(e))?;

    // 读取响应
    let mut buffer = Vec::new();
    stream
        .read_to_end(&mut buffer)
        .map_err(|e| HttpClientError::Io(e))?;

    // 解析响应
    HttpResponse::parse(&buffer).map_err(|e| HttpClientError::InvalidResponse(e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::HTTPHeaders;

    #[test]
    #[ignore] // 需要网络连接
    fn test_send_http1_request() {
        let request = HttpRequest::new(
            crate::http_client::request::HttpMethod::Get,
            "http://httpbin.org/get",
        )
        .with_user_agent("TestClient/1.0");

        let config = HttpClientConfig::default();
        let response = send_http1_request("httpbin.org", 80, "/get", &request, &config).unwrap();

        assert_eq!(response.status_code, 200);
        assert!(response.is_success());
    }
}
