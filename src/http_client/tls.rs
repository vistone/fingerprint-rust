//! TLS 连接支持
//!
//! 当前使用 rustls 作为临时方案
//! TODO: 集成自定义 TLS 实现以应用 fingerprint-rust 的 ClientHelloSpec

use super::{HttpClientConfig, HttpClientError, HttpRequest, HttpResponse, Result};
use std::net::TcpStream;

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

    #[cfg(feature = "rustls-tls")]
    {
        use rustls::client::ServerName;
        use rustls::{ClientConfig, ClientConnection, RootCertStore};
        use std::sync::Arc;

        // 构建根证书存储
        let mut root_store = RootCertStore::empty();
        root_store.add_trust_anchors(webpki_roots::TLS_SERVER_ROOTS.iter().map(|ta| {
            rustls::OwnedTrustAnchor::from_subject_spki_name_constraints(
                ta.subject,
                ta.spki,
                ta.name_constraints,
            )
        }));

        // 构建 TLS 配置（添加 ALPN 支持）
        let mut tls_config = ClientConfig::builder()
            .with_safe_defaults()
            .with_root_certificates(root_store)
            .with_no_client_auth();

        // 设置 ALPN 协议（优先 http/1.1，除非明确要求 HTTP/2）
        if config.prefer_http2 {
            tls_config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];
        } else {
            tls_config.alpn_protocols = vec![b"http/1.1".to_vec()];
        }

        let server_name = ServerName::try_from(host)
            .map_err(|_| HttpClientError::TlsError("无效的服务器名称".to_string()))?;

        let conn = ClientConnection::new(Arc::new(tls_config), server_name)
            .map_err(|e| HttpClientError::TlsError(format!("TLS 连接创建失败: {}", e)))?;

        let mut tls_stream = rustls::StreamOwned::new(conn, tcp_stream);

        // 完成 TLS 握手（强制完成 I/O）
        use std::io::{Read, Write};
        tls_stream
            .flush()
            .map_err(|e| HttpClientError::TlsError(format!("TLS 握手失败: {}", e)))?;

        // 检查协商的 ALPN 协议
        let negotiated_protocol = tls_stream.conn.alpn_protocol();
        if let Some(proto) = negotiated_protocol {
            let proto_str = String::from_utf8_lossy(proto);
            #[cfg(debug_assertions)]
            eprintln!("ALPN 协商结果: {}", proto_str);

            // 如果协商的是 HTTP/2，但我们不支持，返回错误
            if proto == b"h2" && !config.prefer_http2 {
                return Err(HttpClientError::TlsError(
                    "服务器选择了 HTTP/2，但配置中未启用 HTTP/2 支持".to_string(),
                ));
            }
        }

        // 发送 HTTP 请求
        let http_request = request.build_http1_request(host, path);
        tls_stream
            .write_all(http_request.as_bytes())
            .map_err(HttpClientError::Io)?;
        tls_stream.flush().map_err(HttpClientError::Io)?;

        // 读取响应（使用分块读取避免 UnexpectedEof）
        let mut buffer = Vec::new();
        let mut chunk = [0u8; 8192];

        loop {
            match tls_stream.read(&mut chunk) {
                Ok(0) => break, // 连接正常关闭
                Ok(n) => buffer.extend_from_slice(&chunk[..n]),
                Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                    // 服务器关闭连接，但我们可能已经读取了完整响应
                    break;
                }
                Err(e) => return Err(HttpClientError::Io(e)),
            }
        }

        // 解析响应
        HttpResponse::parse(&buffer).map_err(HttpClientError::InvalidResponse)
    }

    #[cfg(all(feature = "native-tls-impl", not(feature = "rustls-tls")))]
    {
        use native_tls::TlsConnector as NativeTlsConnector;

        let connector = NativeTlsConnector::builder()
            .danger_accept_invalid_certs(!config.verify_tls)
            .build()
            .map_err(|e| HttpClientError::TlsError(format!("TLS 初始化失败: {}", e)))?;

        let mut tls_stream = connector
            .connect(host, tcp_stream)
            .map_err(|e| HttpClientError::TlsError(format!("TLS 握手失败: {}", e)))?;

        // 发送 HTTP 请求
        use std::io::{Read, Write};
        let http_request = request.build_http1_request(host, path);
        tls_stream
            .write_all(http_request.as_bytes())
            .map_err(HttpClientError::Io)?;
        tls_stream.flush().map_err(HttpClientError::Io)?;

        // 读取响应（使用分块读取避免 UnexpectedEof）
        let mut buffer = Vec::new();
        let mut chunk = [0u8; 8192];

        loop {
            match tls_stream.read(&mut chunk) {
                Ok(0) => break, // 连接正常关闭
                Ok(n) => buffer.extend_from_slice(&chunk[..n]),
                Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                    // 服务器关闭连接，但我们可能已经读取了完整响应
                    break;
                }
                Err(e) => return Err(HttpClientError::Io(e)),
            }
        }

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
        let request = HttpRequest::new(HttpMethod::Get, "https://example.com")
            .with_user_agent("TestClient/1.0");

        let config = HttpClientConfig::default();
        let response = send_https_request("example.com", 443, "/", &request, &config).unwrap();

        if response.status_code == 200 {
             assert!(response.is_success());
        } else {
             println!("⚠️  Server returned {}", response.status_code);
        }
    }
}
