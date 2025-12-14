//! HTTP/3 实现
//!
//! 使用 quinn + h3 实现完整的 HTTP/3 支持
//! HTTP/3 基于 QUIC 协议

use super::{HttpClientConfig, HttpClientError, HttpRequest, HttpResponse, Result};

#[cfg(feature = "http3")]
use quinn::{ClientConfig, Endpoint};

#[cfg(feature = "http3")]
use tokio::runtime::Runtime;

/// 发送 HTTP/3 请求
#[cfg(feature = "http3")]
pub fn send_http3_request(
    host: &str,
    port: u16,
    path: &str,
    request: &HttpRequest,
    config: &HttpClientConfig,
) -> Result<HttpResponse> {
    // 创建 Tokio 运行时
    let rt = Runtime::new()
        .map_err(|e| HttpClientError::ConnectionFailed(format!("创建运行时失败: {}", e)))?;

    rt.block_on(async { send_http3_request_async(host, port, path, request, config).await })
}

#[cfg(feature = "http3")]
async fn send_http3_request_async(
    host: &str,
    port: u16,
    path: &str,
    request: &HttpRequest,
    config: &HttpClientConfig,
) -> Result<HttpResponse> {
    use std::net::{IpAddr, Ipv4Addr, SocketAddr, ToSocketAddrs};
    use std::sync::Arc;
    use std::time::Instant;

    let start = Instant::now();

    // 1. 配置 QUIC 客户端
    let mut roots = rustls::RootCertStore::empty();
    roots.add_trust_anchors(webpki_roots::TLS_SERVER_ROOTS.iter().map(|ta| {
        rustls::OwnedTrustAnchor::from_subject_spki_name_constraints(
            ta.subject,
            ta.spki,
            ta.name_constraints,
        )
    }));

    let mut tls_config = rustls::ClientConfig::builder()
        .with_safe_defaults()
        .with_root_certificates(roots)
        .with_no_client_auth();

    // 设置 ALPN 协议为 h3
    tls_config.alpn_protocols = vec![b"h3".to_vec()];

    let client_config = ClientConfig::new(Arc::new(tls_config));

    // 2. 创建 QUIC endpoint
    let mut endpoint = Endpoint::client(SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 0))
        .map_err(|e| HttpClientError::ConnectionFailed(format!("创建 endpoint 失败: {}", e)))?;

    endpoint.set_default_client_config(client_config);

    // 3. DNS 解析
    let addr_str = format!("{}:{}", host, port);
    let remote_addr = addr_str
        .to_socket_addrs()
        .map_err(|e| HttpClientError::InvalidUrl(format!("DNS 解析失败: {}", e)))?
        .next()
        .ok_or_else(|| HttpClientError::InvalidUrl("无法解析地址".to_string()))?;

    // 4. 连接到服务器
    let connection = endpoint
        .connect(remote_addr, host)
        .map_err(|e| HttpClientError::ConnectionFailed(format!("QUIC 连接失败: {}", e)))?
        .await
        .map_err(|e| HttpClientError::ConnectionFailed(format!("QUIC 握手失败: {}", e)))?;

    // 5. 建立 HTTP/3 连接
    let (driver, mut send_request) = h3::client::new(h3_quinn::Connection::new(connection))
        .await
        .map_err(|e| HttpClientError::ConnectionFailed(format!("HTTP/3 连接失败: {}", e)))?;

    // 在后台驱动连接 (注意：旧版本的 h3 可能不需要主动 await driver)
    tokio::spawn(async move {
        // 对于某些版本的 h3，driver 不需要显式驱动
        // 保留这个结构用于未来版本
        drop(driver);
    });

    // 6. 构建请求
    let uri = format!("https://{}{}", host, path);
    let mut http_request = http::Request::builder()
        .method(request.method.as_str())
        .uri(uri)
        .version(http::Version::HTTP_3);

    // 添加 headers
    http_request = http_request.header("host", host);
    http_request = http_request.header("user-agent", &config.user_agent);

    for (key, value) in &request.headers {
        http_request = http_request.header(key, value);
    }

    let http_request = http_request
        .body(())
        .map_err(|e| HttpClientError::InvalidResponse(format!("构建请求失败: {}", e)))?;

    // 7. 发送请求
    let mut stream = send_request
        .send_request(http_request)
        .await
        .map_err(|e| HttpClientError::ConnectionFailed(format!("发送请求失败: {}", e)))?;

    stream
        .finish()
        .await
        .map_err(|e| HttpClientError::ConnectionFailed(format!("结束请求失败: {}", e)))?;

    // 8. 接收响应
    let response = stream
        .recv_response()
        .await
        .map_err(|e| HttpClientError::InvalidResponse(format!("接收响应失败: {}", e)))?;

    let status_code = response.status().as_u16();
    let headers = response.headers().clone();

    // 接收 body
    use bytes::Buf;
    let mut body_data = Vec::new();
    while let Some(mut chunk) = stream.recv_data().await.map_err(|e| {
        HttpClientError::Io(std::io::Error::other(
            format!("读取 body 失败: {}", e),
        ))
    })? {
        // 使用 Buf trait 读取数据
        let chunk_len = chunk.remaining();
        let mut chunk_bytes = vec![0u8; chunk_len];
        chunk.copy_to_slice(&mut chunk_bytes);
        body_data.extend_from_slice(&chunk_bytes);
    }

    let elapsed = start.elapsed().as_millis() as u64;

    // 9. 构建响应
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
        http_version: "HTTP/3".to_string(),
        response_time_ms: elapsed,
    })
}

#[cfg(not(feature = "http3"))]
pub fn send_http3_request(
    _host: &str,
    _port: u16,
    _path: &str,
    _request: &HttpRequest,
    _config: &HttpClientConfig,
) -> Result<HttpResponse> {
    Err(HttpClientError::InvalidResponse(
        "HTTP/3 支持未启用，请使用 --features http3 编译".to_string(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(feature = "http3")]
    #[ignore]
    fn test_http3_request() {
        let request = HttpRequest::new(
            crate::http_client::request::HttpMethod::Get,
            "https://quic.aiortc.org:443/",
        );

        let config = HttpClientConfig::default();

        let result = send_http3_request("quic.aiortc.org", 443, "/", &request, &config);
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.http_version, "HTTP/3");
        assert!(response.is_success());
    }
}
