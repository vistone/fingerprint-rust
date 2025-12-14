//! TLS 连接支持
//!
//! 当前使用 rustls 作为临时方案
//! TODO: 集成自定义 TLS 实现以应用 fingerprint-rust 的 ClientHelloSpec

use super::{HttpClientConfig, HttpClientError, HttpRequest, HttpResponse, Result};
use std::io::Write;
use std::net::TcpStream;
#[allow(unused_imports)]
use std::sync::Arc;

/// TLS 连接器
///
/// 当前使用 native-tls，将来可替换为自定义 TLS 实现
pub struct TlsConnector {
    // TODO: 添加自定义 TLS 配置
}

impl TlsConnector {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for TlsConnector {
    fn default() -> Self {
        Self::new()
    }
}

/// 发送 HTTPS 请求
///
/// ⚠️ 警告：当前使用 native-tls，TLS 指纹不可自定义
/// TODO: 实现自定义 TLS ClientHello
pub fn send_https_request(
    host: &str,
    port: u16,
    path: &str,
    request: &HttpRequest,
    config: &HttpClientConfig,
) -> Result<HttpResponse> {
    // 建立 TCP 连接
    let addr = format!("{}:{}", host, port);
    let tcp_stream = TcpStream::connect(&addr)
        .map_err(|e| HttpClientError::ConnectionFailed(format!("连接失败 {}: {}", addr, e)))?;

    // 设置超时
    tcp_stream
        .set_read_timeout(Some(config.read_timeout))
        .map_err(HttpClientError::Io)?;
    tcp_stream
        .set_write_timeout(Some(config.write_timeout))
        .map_err(HttpClientError::Io)?;

    // ⚠️ 临时方案：使用 rustls (默认) 或 native-tls
    // TODO: 这里应该使用自定义 TLS 实现，应用 ClientHelloSpec

    // 同时启用两套 TLS 实现时，默认优先使用 rustls（纯 Rust，便于跨平台与一致性）
    #[cfg(all(feature = "rustls-tls", feature = "native-tls-impl"))]
    {
        use rustls::client::ServerName;
        use std::sync::Arc;

        let tls_config = super::rustls_utils::build_client_config(config.verify_tls, Vec::new());
        let server_name = ServerName::try_from(host)
            .map_err(|_| HttpClientError::TlsError("无效的服务器名称".to_string()))?;

        let conn = rustls::ClientConnection::new(Arc::new(tls_config), server_name)
            .map_err(|e| HttpClientError::TlsError(format!("TLS 连接创建失败: {}", e)))?;

        let mut tls_stream = rustls::StreamOwned::new(conn, tcp_stream);

        let http_request = request.build_http1_request_bytes(host, path);
        tls_stream
            .write_all(&http_request)
            .map_err(HttpClientError::Io)?;
        tls_stream.flush().map_err(HttpClientError::Io)?;

        let buffer = super::io::read_http1_response_bytes(
            &mut tls_stream,
            super::io::DEFAULT_MAX_RESPONSE_BYTES,
        )
        .map_err(HttpClientError::Io)?;

        HttpResponse::parse(&buffer).map_err(HttpClientError::InvalidResponse)
    }

    #[cfg(all(feature = "rustls-tls", not(feature = "native-tls-impl")))]
    {
        use rustls::client::ServerName;
        use std::sync::Arc;

        // 构建 TLS 配置（尊重 verify_tls）
        let tls_config = super::rustls_utils::build_client_config(config.verify_tls, Vec::new());

        let server_name = ServerName::try_from(host)
            .map_err(|_| HttpClientError::TlsError("无效的服务器名称".to_string()))?;

        let conn = rustls::ClientConnection::new(Arc::new(tls_config), server_name)
            .map_err(|e| HttpClientError::TlsError(format!("TLS 连接创建失败: {}", e)))?;

        let mut tls_stream = rustls::StreamOwned::new(conn, tcp_stream);

        // 发送 HTTP 请求
        let http_request = request.build_http1_request_bytes(host, path);
        tls_stream
            .write_all(&http_request)
            .map_err(HttpClientError::Io)?;
        tls_stream.flush().map_err(HttpClientError::Io)?;

        // 读取响应
        let buffer = super::io::read_http1_response_bytes(
            &mut tls_stream,
            super::io::DEFAULT_MAX_RESPONSE_BYTES,
        )
        .map_err(HttpClientError::Io)?;

        // 解析响应
        HttpResponse::parse(&buffer).map_err(HttpClientError::InvalidResponse)
    }

    #[cfg(all(feature = "native-tls-impl", not(feature = "rustls-tls")))]
    {
        use native_tls::TlsConnector as NativeTlsConnector;
        use std::io::Read;

        let connector = NativeTlsConnector::builder()
            .danger_accept_invalid_certs(!config.verify_tls)
            .build()
            .map_err(|e| HttpClientError::TlsError(format!("TLS 初始化失败: {}", e)))?;

        let mut tls_stream = connector
            .connect(host, tcp_stream)
            .map_err(|e| HttpClientError::TlsError(format!("TLS 握手失败: {}", e)))?;

        // 发送 HTTP 请求
        let http_request = request.build_http1_request_bytes(host, path);
        tls_stream
            .write_all(&http_request)
            .map_err(HttpClientError::Io)?;
        tls_stream.flush().map_err(HttpClientError::Io)?;

        // 读取响应
        let buffer = super::io::read_http1_response_bytes(
            &mut tls_stream,
            super::io::DEFAULT_MAX_RESPONSE_BYTES,
        )
        .map_err(HttpClientError::Io)?;

        // 解析响应
        HttpResponse::parse(&buffer).map_err(HttpClientError::InvalidResponse)
    }

    #[cfg(not(any(feature = "rustls-tls", feature = "native-tls-impl")))]
    {
        Err(HttpClientError::TlsError(
            "需要启用 rustls-tls 或 native-tls-impl 特性".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::http_client::request::HttpMethod;

    #[test]
    #[ignore] // 需要网络连接
    fn test_send_https_request() {
        let request = HttpRequest::new(HttpMethod::Get, "https://httpbin.org/get")
            .with_user_agent("TestClient/1.0");

        let config = HttpClientConfig::default();
        let response = send_https_request("httpbin.org", 443, "/get", &request, &config).unwrap();

        // 外部服务可能会短暂返回 429/503 等；这里主要验证“能建立 TLS + 能解析响应”。
        assert!(response.status_code > 0);
    }
}
