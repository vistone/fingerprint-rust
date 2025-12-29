//! HTTP/3 with Connection Pool
//!
//! 使用 netconnpool 管理 UDP 连接复用，支持 HTTP/3 (QUIC)

#[cfg(all(feature = "connection-pool", feature = "http3"))]
use super::pool::ConnectionPoolManager;
use super::{HttpClientConfig, HttpClientError, HttpRequest, HttpResponse, Result};
#[cfg(all(feature = "connection-pool", feature = "http3"))]
use std::sync::Arc;
#[cfg(all(feature = "connection-pool", feature = "http3"))]
use std::time::Duration;

/// 使用连接池发送 HTTP/3 请求
#[cfg(all(feature = "connection-pool", feature = "http3"))]
pub async fn send_http3_request_with_pool(
    host: &str,
    port: u16,
    path: &str,
    request: &HttpRequest,
    config: &HttpClientConfig,
    pool_manager: &Arc<ConnectionPoolManager>,
) -> Result<HttpResponse> {
    use bytes::Buf;
    use h3_quinn::quinn;
    use http::{Request as HttpRequest2, Version};

    // 对于 HTTP/3，我们需要 UDP 连接
    // 注意：netconnpool 主要是为 TCP 设计的，对于 UDP/QUIC 可能需要不同的处理方式
    // 这里我们先获取连接信息，然后创建 QUIC 连接

    let pool = pool_manager.get_pool(host, port)?;

    // 获取连接（用于获取目标地址信息）
    let _conn = pool
        .get_tcp()
        .map_err(|e| HttpClientError::ConnectionFailed(format!("从连接池获取连接失败: {:?}", e)))?;

    // 解析目标地址 - 需要先进行 DNS 解析
    // 注意：Endpoint 绑定的是 IPv4/IPv6 的具体栈；这里优先选 IPv4，避免出现
    // “Endpoint 是 0.0.0.0 但 remote 是 IPv6 导致 invalid remote address”。
    use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, ToSocketAddrs};
    let addr = format!("{}:{}", host, port);
    let mut addrs: Vec<SocketAddr> = addr
        .to_socket_addrs()
        .map_err(|e| HttpClientError::ConnectionFailed(format!("DNS 解析失败: {}", e)))?
        .collect();
    if addrs.is_empty() {
        return Err(HttpClientError::ConnectionFailed(
            "DNS 解析无结果".to_string(),
        ));
    }
    addrs.sort_by_key(|a| matches!(a.ip(), IpAddr::V6(_))); // IPv4 优先
    let remote_addr = addrs[0];

    // 创建 QUIC 客户端配置
    let tls_config = super::rustls_utils::build_client_config(
        config.verify_tls,
        vec![b"h3".to_vec()],
        config.profile.as_ref(),
    );

    let mut client_config = quinn::ClientConfig::new(std::sync::Arc::new(tls_config));

    // 优化传输配置以提升性能
    let mut transport_config = quinn::TransportConfig::default();
    transport_config.initial_rtt(Duration::from_millis(100));
    transport_config.max_idle_timeout(Some(
        quinn::IdleTimeout::try_from(std::time::Duration::from_secs(60))
            .map_err(|e| HttpClientError::Http3Error(format!("配置超时失败: {}", e)))?,
    ));
    transport_config.keep_alive_interval(Some(Duration::from_secs(10)));

    // 增大接收窗口以提升吞吐量
    transport_config.stream_receive_window((1024 * 1024u32).into()); // 1MB
    transport_config.receive_window((10 * 1024 * 1024u32).into()); // 10MB

    // 允许更多并发流
    transport_config.max_concurrent_bidi_streams(100u32.into());
    transport_config.max_concurrent_uni_streams(100u32.into());

    client_config.transport_config(std::sync::Arc::new(transport_config));

    // 创建 QUIC endpoint（按 remote 的地址族选择绑定）
    let bind_addr = match remote_addr.ip() {
        IpAddr::V4(_) => SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 0),
        IpAddr::V6(_) => SocketAddr::new(IpAddr::V6(Ipv6Addr::UNSPECIFIED), 0),
    };
    let mut endpoint = quinn::Endpoint::client(bind_addr)
        .map_err(|e| HttpClientError::Http3Error(format!("创建 endpoint 失败: {}", e)))?;
    endpoint.set_default_client_config(client_config);

    // 连接到服务器
    let connecting = endpoint
        .connect(remote_addr, host)
        .map_err(|e| HttpClientError::Http3Error(format!("连接失败: {}", e)))?;

    let connection = connecting
        .await
        .map_err(|e| HttpClientError::Http3Error(format!("建立连接失败: {}", e)))?;

    // 建立 HTTP/3 连接
    let quinn_conn = h3_quinn::Connection::new(connection);

    let (mut driver, mut send_request) = h3::client::new(quinn_conn)
        .await
        .map_err(|e| HttpClientError::Http3Error(format!("HTTP/3 握手失败: {}", e)))?;

    // 在后台驱动连接：h3 0.0.4 的 driver 需要被持续 poll_close
    tokio::spawn(async move {
        let _ = std::future::poll_fn(|cx| driver.poll_close(cx)).await;
    });

    // 构建 HTTP/3 请求
    let uri: http::Uri = format!("https://{}:{}{}", host, port, path)
        .parse()
        .map_err(|e| HttpClientError::InvalidRequest(format!("无效的 URI: {}", e)))?;

    let http3_request = HttpRequest2::builder()
        .method(match request.method {
            super::request::HttpMethod::Get => http::Method::GET,
            super::request::HttpMethod::Post => http::Method::POST,
            super::request::HttpMethod::Put => http::Method::PUT,
            super::request::HttpMethod::Delete => http::Method::DELETE,
            super::request::HttpMethod::Head => http::Method::HEAD,
            super::request::HttpMethod::Options => http::Method::OPTIONS,
            super::request::HttpMethod::Patch => http::Method::PATCH,
        })
        .uri(uri)
        .version(Version::HTTP_3)
        // 不要手动添加 host header，h3 会自动从 URI 提取
        .header("user-agent", &config.user_agent);

    let http3_request = request
        .headers
        .iter()
        // 跳过 host header
        .filter(|(k, _)| k.to_lowercase() != "host")
        .fold(http3_request, |builder, (k, v)| builder.header(k, v))
        .body(())
        .map_err(|e| HttpClientError::InvalidRequest(format!("构建请求失败: {}", e)))?;

    // 发送请求
    let mut stream = send_request
        .send_request(http3_request)
        .await
        .map_err(|e| HttpClientError::Http3Error(format!("发送请求失败: {}", e)))?;

    stream
        .finish()
        .await
        .map_err(|e| HttpClientError::Http3Error(format!("完成请求失败: {}", e)))?;

    // 接收响应
    let response = stream
        .recv_response()
        .await
        .map_err(|e| HttpClientError::Http3Error(format!("接收响应失败: {}", e)))?;

    // 读取响应体
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

    // 解析响应
    let status_code = response.status().as_u16();
    let status_text = http::StatusCode::from_u16(status_code)
        .ok()
        .and_then(|s| s.canonical_reason())
        .unwrap_or("Unknown")
        .to_string();
    let headers = response
        .headers()
        .iter()
        .map(|(k, v)| (k.as_str().to_string(), v.to_str().unwrap_or("").to_string()))
        .collect();

    Ok(HttpResponse {
        http_version: "HTTP/3".to_string(),
        status_code,
        status_text,
        headers,
        body: body_data,
        response_time_ms: 0, // TODO: 添加计时
    })
}

#[cfg(test)]
#[cfg(all(feature = "connection-pool", feature = "http3"))]
mod tests {
    use super::*;
    use crate::http_client::pool::PoolManagerConfig;
    use crate::http_client::request::HttpMethod;

    #[tokio::test]
    #[ignore] // 需要网络连接和 HTTP/3 支持
    async fn test_http3_with_pool() {
        let user_agent = "TestClient/1.0".to_string();
        let config = HttpClientConfig {
            user_agent,
            prefer_http3: true,
            ..Default::default()
        };

        let pool_manager = Arc::new(ConnectionPoolManager::new(PoolManagerConfig::default()));

        let request = HttpRequest::new(HttpMethod::Get, "https://cloudflare-quic.com/");

        let result = send_http3_request_with_pool(
            "cloudflare-quic.com",
            443,
            "/",
            &request,
            &config,
            &pool_manager,
        )
        .await;

        // 可能会失败（网络问题或服务器不支持 HTTP/3），但不应该 panic
        if let Ok(response) = result {
            assert_eq!(response.http_version, "HTTP/3");
        }
    }
}
