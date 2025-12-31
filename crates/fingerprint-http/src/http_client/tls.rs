//! TLS 连接支持
//!
//! 使用官方 rustls 作为底层 TLS 实现
//! 通过 ClientHelloCustomizer 应用浏览器指纹（Chrome、Firefox、Safari 等）
//! 模拟市场成熟浏览器的 TLS 指纹，不自定义自己的指纹

use super::{HttpClientConfig, HttpClientError, HttpRequest, HttpResponse, Result};
use std::io::Write;
use std::net::TcpStream;
#[allow(unused_imports)]
use std::sync::Arc;

/// TLS 连接器
///
/// 使用官方 rustls，通过 ClientHelloCustomizer 应用浏览器指纹
pub struct TlsConnector {
    // rustls 配置通过 HttpClientConfig 传递
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
/// 使用官方 rustls 作为底层 TLS 实现
/// 如果配置了 ClientProfile，会通过 ClientHelloCustomizer 应用浏览器指纹
/// 模拟市场成熟浏览器的 TLS 指纹（Chrome、Firefox、Safari 等）
pub fn send_https_request(
    host: &str,
    port: u16,
    path: &str,
    request: &HttpRequest,
    config: &HttpClientConfig,
) -> Result<HttpResponse> {
    // 使用 rustls，如果配置了 profile，会自动通过 ClientHelloCustomizer 应用浏览器指纹

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

    // 使用官方 rustls，通过 ClientHelloCustomizer 应用浏览器指纹

    #[cfg(feature = "rustls-tls")]
    {
        use rustls::client::ServerName;
        use std::sync::Arc;

        // 构建 TLS 配置（尊重 verify_tls）
        let tls_config = super::rustls_utils::build_client_config(
            config.verify_tls,
            Vec::new(),
            config.profile.as_ref(),
        );

        let server_name = ServerName::try_from(host)
            .map_err(|_| HttpClientError::TlsError("无效的服务器名称".to_string()))?;

        let conn = rustls::ClientConnection::new(Arc::new(tls_config), server_name)
            .map_err(|e| HttpClientError::TlsError(format!("TLS 连接创建失败: {}", e)))?;

        let mut tls_stream = rustls::StreamOwned::new(conn, tcp_stream);

        // 修复：添加 Cookie 到请求（如果存在）
        let mut request_with_cookies = request.clone();
        if let Some(cookie_store) = &config.cookie_store {
            super::request::add_cookies_to_request(
                &mut request_with_cookies,
                cookie_store,
                host,
                path,
                true, // HTTPS 是安全连接
            );
        }

        // 发送 HTTP 请求
        let header_order = config.profile.as_ref().map(|p| p.header_order.as_slice());
        let http_request = request_with_cookies.build_http1_request_bytes(host, path, header_order);
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

    #[cfg(not(feature = "rustls-tls"))]
    {
        Err(HttpClientError::TlsError(
            "需要启用 rustls-tls 特性".to_string(),
        ))
    }
}

/// 使用连接池发送 HTTPS（HTTP/1.1 over TLS）请求
///
/// 说明：
/// - 这是“连接池 + TLS”的同步实现（面向 `kh.google.com` 这类 https 站点）
/// - 目前只用于回归测试与 `HttpClient` 的 https+pool 路径
#[cfg(feature = "connection-pool")]
pub fn send_https_request_with_pool(
    host: &str,
    port: u16,
    path: &str,
    request: &HttpRequest,
    config: &HttpClientConfig,
    pool_manager: &std::sync::Arc<super::pool::ConnectionPoolManager>,
) -> Result<HttpResponse> {
    use std::io::Write;

    let pool = pool_manager.get_pool(host, port)?;
    let conn = pool
        .get_tcp()
        .map_err(|e| HttpClientError::ConnectionFailed(format!("从连接池获取连接失败: {:?}", e)))?;

    // PooledConnection 实现了 Deref<Target = Connection>，可以直接使用 Connection 的方法
    let tcp_stream = conn
        .tcp_conn()
        .ok_or_else(|| HttpClientError::ConnectionFailed("期望 TCP 连接但得到 UDP".to_string()))?;

    // 保持 conn 生命周期覆盖整个请求；同时用 clone 得到可用的 std::net::TcpStream
    let tcp_stream = tcp_stream.try_clone().map_err(HttpClientError::Io)?;

    tcp_stream
        .set_read_timeout(Some(config.read_timeout))
        .map_err(HttpClientError::Io)?;
    tcp_stream
        .set_write_timeout(Some(config.write_timeout))
        .map_err(HttpClientError::Io)?;

    // rustls 路径与 send_https_request 保持一致
    #[cfg(feature = "rustls-tls")]
    {
        use rustls::client::ServerName;
        use std::sync::Arc;

        let tls_config = super::rustls_utils::build_client_config(
            config.verify_tls,
            Vec::new(),
            config.profile.as_ref(),
        );
        let server_name = ServerName::try_from(host)
            .map_err(|_| HttpClientError::TlsError("无效的服务器名称".to_string()))?;
        let conn_tls = rustls::ClientConnection::new(Arc::new(tls_config), server_name)
            .map_err(|e| HttpClientError::TlsError(format!("TLS 连接创建失败: {}", e)))?;

        let mut tls_stream = rustls::StreamOwned::new(conn_tls, tcp_stream);

        // 修复：添加 Cookie 到请求（如果存在）
        let mut request_with_cookies = request.clone();
        if let Some(cookie_store) = &config.cookie_store {
            super::request::add_cookies_to_request(
                &mut request_with_cookies,
                cookie_store,
                host,
                path,
                true, // HTTPS 是安全连接
            );
        }

        let header_order = config.profile.as_ref().map(|p| p.header_order.as_slice());
        let http_request = request_with_cookies.build_http1_request_bytes(host, path, header_order);
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

    #[cfg(not(feature = "rustls-tls"))]
    {
        let _ = conn; // keep for symmetry
        Err(HttpClientError::TlsError(
            "需要启用 rustls-tls 特性".to_string(),
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
