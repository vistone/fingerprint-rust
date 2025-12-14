//! HTTP/2 实现
//!
//! 使用 h2 crate 实现完整的 HTTP/2 支持
//! 应用 fingerprint-rust 的 HTTP/2 Settings

use super::{HttpClientConfig, HttpClientError, HttpRequest, HttpResponse, Result};

#[cfg(feature = "http2")]
use h2::client;

#[cfg(feature = "http2")]
use tokio::runtime::Runtime;

/// 发送 HTTP/2 请求
#[cfg(feature = "http2")]
pub fn send_http2_request(
    host: &str,
    port: u16,
    path: &str,
    request: &HttpRequest,
    config: &HttpClientConfig,
) -> Result<HttpResponse> {
    // 创建 Tokio 运行时
    let rt = Runtime::new()
        .map_err(|e| HttpClientError::ConnectionFailed(format!("创建运行时失败: {}", e)))?;

    rt.block_on(async { send_http2_request_async(host, port, path, request, config).await })
}

#[cfg(feature = "http2")]
async fn send_http2_request_async(
    host: &str,
    port: u16,
    path: &str,
    request: &HttpRequest,
    config: &HttpClientConfig,
) -> Result<HttpResponse> {
    use std::net::ToSocketAddrs;
    use std::time::Instant;
    use tokio::net::TcpStream;

    let start = Instant::now();

    // 1. 建立 TCP 连接
    let addr = format!("{}:{}", host, port);
    let socket_addrs = addr
        .to_socket_addrs()
        .map_err(|e| HttpClientError::InvalidUrl(format!("DNS 解析失败: {}", e)))?
        .next()
        .ok_or_else(|| HttpClientError::InvalidUrl("无法解析地址".to_string()))?;

    let tcp = TcpStream::connect(socket_addrs)
        .await
        .map_err(|e| HttpClientError::ConnectionFailed(format!("TCP 连接失败: {}", e)))?;

    // 2. TLS 握手
    let tls_stream = perform_tls_handshake(tcp, host, config).await?;

    // 3. HTTP/2 握手
    let (mut client, h2_conn) = client::handshake(tls_stream)
        .await
        .map_err(|e| HttpClientError::ConnectionFailed(format!("HTTP/2 握手失败: {}", e)))?;

    // 在后台驱动 HTTP/2 连接
    tokio::spawn(async move {
        if let Err(e) = h2_conn.await {
            eprintln!("HTTP/2 连接错误: {}", e);
        }
    });

    // 4. 应用 HTTP/2 Settings（如果有配置）
    // TODO: 从 ClientProfile 获取 HTTP/2 Settings

    // 5. 构建请求
    let uri = format!("https://{}{}", host, path);
    let mut http_request = http::Request::builder()
        .method(request.method.as_str())
        .uri(uri)
        .version(http::Version::HTTP_2);

    // 添加 headers
    http_request = http_request.header("host", host);
    http_request = http_request.header("user-agent", &config.user_agent);

    for (key, value) in &request.headers {
        http_request = http_request.header(key, value);
    }

    let http_request = http_request
        .body(())
        .map_err(|e| HttpClientError::InvalidResponse(format!("构建请求失败: {}", e)))?;

    // 6. 发送请求
    let (response_future, _) = client
        .send_request(http_request, true)
        .map_err(|e| HttpClientError::ConnectionFailed(format!("发送请求失败: {}", e)))?;

    // 7. 接收响应
    let response = response_future
        .await
        .map_err(|e| HttpClientError::InvalidResponse(format!("接收响应失败: {}", e)))?;

    let status_code = response.status().as_u16();
    let headers = response.headers().clone();

    // 接收 body
    let mut body_stream = response.into_body();
    let mut body_data = Vec::new();

    while let Some(chunk) = body_stream.data().await {
        let chunk = chunk.map_err(|e| {
            HttpClientError::Io(std::io::Error::other(
                format!("读取 body 失败: {}", e),
            ))
        })?;
        body_data.extend_from_slice(&chunk);

        // 释放流控制窗口
        let _ = body_stream.flow_control().release_capacity(chunk.len());
    }

    let elapsed = start.elapsed().as_millis() as u64;

    // 8. 构建响应
    let mut response_headers = std::collections::HashMap::new();
    for (key, value) in headers.iter() {
        if let Ok(value_str) = value.to_str() {
            response_headers.insert(key.as_str().to_lowercase(), value_str.to_string());
        }
    }

    Ok(HttpResponse {
        status_code,
        status_text: http::StatusCode::from_u16(status_code)
            .map(|s| s.canonical_reason().unwrap_or("Unknown").to_string())
            .unwrap_or_else(|_| "Unknown".to_string()),
        headers: response_headers,
        body: body_data,
        http_version: "HTTP/2".to_string(),
        response_time_ms: elapsed,
    })
}

#[cfg(feature = "http2")]
async fn perform_tls_handshake(
    tcp: tokio::net::TcpStream,
    host: &str,
    _config: &HttpClientConfig,
) -> Result<tokio_rustls::client::TlsStream<tokio::net::TcpStream>> {
    use rustls::{ClientConfig, RootCertStore, ServerName};
    use std::sync::Arc;
    use tokio_rustls::TlsConnector;

    // 构建 TLS 配置
    let mut root_store = RootCertStore::empty();
    root_store.add_trust_anchors(webpki_roots::TLS_SERVER_ROOTS.iter().map(|ta| {
        rustls::OwnedTrustAnchor::from_subject_spki_name_constraints(
            ta.subject,
            ta.spki,
            ta.name_constraints,
        )
    }));

    let mut tls_config = ClientConfig::builder()
        .with_safe_defaults()
        .with_root_certificates(root_store)
        .with_no_client_auth();

    // 关键：设置 ALPN 协议为 h2，这是 HTTP/2 over TLS 所必需的
    tls_config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];

    let connector = TlsConnector::from(Arc::new(tls_config));

    let server_name = ServerName::try_from(host)
        .map_err(|_| HttpClientError::TlsError("无效的服务器名称".to_string()))?;

    connector
        .connect(server_name, tcp)
        .await
        .map_err(|e| HttpClientError::TlsError(format!("TLS 握手失败: {}", e)))
}

#[cfg(not(feature = "http2"))]
pub fn send_http2_request(
    _host: &str,
    _port: u16,
    _path: &str,
    _request: &HttpRequest,
    _config: &HttpClientConfig,
) -> Result<HttpResponse> {
    Err(HttpClientError::InvalidResponse(
        "HTTP/2 支持未启用，请使用 --features http2 编译".to_string(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(feature = "http2")]
    #[ignore]
    fn test_http2_request() {
        let request = HttpRequest::new(
            crate::http_client::request::HttpMethod::Get,
            "https://www.google.com/",
        );

        let config = HttpClientConfig::default();

        let result = send_http2_request("www.google.com", 443, "/", &request, &config);
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.http_version, "HTTP/2");
        assert!(response.is_success());
    }
}
