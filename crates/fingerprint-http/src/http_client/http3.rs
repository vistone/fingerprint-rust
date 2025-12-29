//! HTTP/3 实现
//!
//! 使用 quinn + h3 实现完整的 HTTP/3 支持
//! HTTP/3 基于 QUIC 协议

use super::{HttpClientConfig, HttpClientError, HttpRequest, HttpResponse, Result};

#[cfg(feature = "http3")]
use quinn::{ClientConfig, Endpoint, TransportConfig};

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
    use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, ToSocketAddrs};
    use std::sync::Arc;
    use std::time::{Duration, Instant};

    let start = Instant::now();

    // 1. 配置 QUIC 客户端
    let tls_config = super::rustls_utils::build_client_config(
        config.verify_tls,
        vec![b"h3".to_vec()],
        config.profile.as_ref(),
    );

    let mut client_config = ClientConfig::new(Arc::new(tls_config));

    // 优化传输配置以提升性能
    let mut transport = TransportConfig::default();
    transport.initial_rtt(Duration::from_millis(100));
    transport.max_idle_timeout(Some(
        Duration::from_secs(60)
            .try_into()
            .map_err(|e| HttpClientError::ConnectionFailed(format!("配置超时失败: {}", e)))?,
    ));
    transport.keep_alive_interval(Some(Duration::from_secs(10)));

    // 增大接收窗口以提升吞吐量
    transport.stream_receive_window((1024 * 1024u32).into()); // 1MB
    transport.receive_window((10 * 1024 * 1024u32).into()); // 10MB

    // 允许更多并发流
    transport.max_concurrent_bidi_streams(100u32.into());
    transport.max_concurrent_uni_streams(100u32.into());

    client_config.transport_config(Arc::new(transport));

    // 2. DNS 解析（优先 IPv4，避免 IPv4 endpoint 连接 IPv6 remote 导致 invalid remote address）
    let addr_str = format!("{}:{}", host, port);
    let mut addrs: Vec<SocketAddr> = addr_str
        .to_socket_addrs()
        .map_err(|e| HttpClientError::InvalidUrl(format!("DNS 解析失败: {}", e)))?
        .collect();
    if addrs.is_empty() {
        return Err(HttpClientError::InvalidUrl("无法解析地址".to_string()));
    }
    addrs.sort_by_key(|a| matches!(a.ip(), IpAddr::V6(_))); // IPv4 优先
    let remote_addr = addrs[0];

    // 3. 创建 QUIC endpoint（按 remote 的地址族选择绑定）
    let bind_addr = match remote_addr.ip() {
        IpAddr::V4(_) => SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 0),
        IpAddr::V6(_) => SocketAddr::new(IpAddr::V6(Ipv6Addr::UNSPECIFIED), 0),
    };
    let mut endpoint = Endpoint::client(bind_addr)
        .map_err(|e| HttpClientError::ConnectionFailed(format!("创建 endpoint 失败: {}", e)))?;

    endpoint.set_default_client_config(client_config);

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

    // 在后台驱动连接：h3 0.0.4 的 driver 需要被持续 poll_close
    tokio::spawn(async move {
        let mut driver = driver;
        let _ = std::future::poll_fn(|cx| driver.poll_close(cx)).await;
    });

    // 6. 构建请求
    let uri = format!("https://{}{}", host, path);
    let mut http_request = http::Request::builder()
        .method(request.method.as_str())
        .uri(uri)
        .version(http::Version::HTTP_3);

    // 添加 headers
    // 注意：不要手动添加 host header，h3 会自动从 URI 提取
    http_request = http_request.header("user-agent", &config.user_agent);

    for (key, value) in &request.headers {
        // 跳过 host header（如果用户传入了）
        if key.to_lowercase() != "host" {
            http_request = http_request.header(key, value);
        }
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
    while let Some(mut chunk) = stream
        .recv_data()
        .await
        .map_err(|e| HttpClientError::Io(std::io::Error::other(format!("读取 body 失败: {}", e))))?
    {
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

#[cfg(all(test, feature = "http3"))]
mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[test]
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
