//! HTTP/2 实现
//! 
//! TODO: 实现 HTTP/2 支持
//! 当前版本暂不支持 HTTP/2

use super::{HttpClientConfig, HttpClientError, HttpRequest, HttpResponse, Result};

/// 发送 HTTP/2 请求
/// 
/// TODO: 实现完整的 HTTP/2 支持
pub fn send_http2_request(
    _host: &str,
    _port: u16,
    _path: &str,
    _request: &HttpRequest,
    _config: &HttpClientConfig,
) -> Result<HttpResponse> {
    Err(HttpClientError::InvalidResponse(
        "HTTP/2 支持尚未实现".to_string(),
    ))
}
