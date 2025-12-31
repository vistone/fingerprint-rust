//! HTTP/3 实现
//!
//! 使用 quinn + h3 实现完整的 HTTP/3 支持
//! HTTP/3 基于 QUIC 协议

use super::{HttpClientConfig, HttpClientError, HttpRequest, HttpResponse, Result};

#[cfg(feature = "http3")]
use quinn::{ClientConfig, Endpoint, TransportConfig};

// 修复：使用全局单例 Runtime 避免频繁创建
#[cfg(feature = "http3")]
use once_cell::sync::Lazy;

#[cfg(feature = "http3")]
static RUNTIME: Lazy<tokio::runtime::Runtime> =
    Lazy::new(|| tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime"));

/// 发送 HTTP/3 请求
#[cfg(feature = "http3")]
pub fn send_http3_request(
    host: &str,
    port: u16,
    path: &str,
    request: &HttpRequest,
    config: &HttpClientConfig,
) -> Result<HttpResponse> {
    // 修复：使用全局单例 Runtime，避免每次请求都创建新的运行时
    RUNTIME.block_on(async { send_http3_request_async(host, port, path, request, config).await })
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

    // 优化传输配置以提升性能和连接迁移能力
    let mut transport = TransportConfig::default();

    // 连接迁移 (Connection Migration) 优化
    // QUIC 允许在 IP 切换时保持连接，这对移动端模拟至关重要
    transport.initial_rtt(Duration::from_millis(100));
    transport.max_idle_timeout(Some(
        Duration::from_secs(60)
            .try_into()
            .map_err(|e| HttpClientError::ConnectionFailed(format!("配置超时失败: {}", e)))?,
    ));
    // 增加保活频率以辅助连接迁移识别
    transport.keep_alive_interval(Some(Duration::from_secs(20)));

    // 允许对端迁移（默认已开启，此处显式说明其重要性）
    // transport.allow_peer_migration(true);

    // 模拟 Chrome 的流控制窗口 (Chrome 通常使用较大的窗口以提升吞吐)
    transport.stream_receive_window((6 * 1024 * 1024u32).into()); // 6MB (Chrome 风格)
    transport.receive_window((15 * 1024 * 1024u32).into()); // 15MB (Chrome 风格)

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

    // 4. 连接到服务器 (Happy Eyeballs 简化版：循环尝试所有解析到的地址)
    let mut connection_result = Err(HttpClientError::ConnectionFailed("无可用地址".to_string()));

    for remote_addr in addrs {
        // 创建 QUIC endpoint（按 remote 的地址族选择绑定）
        let bind_addr = match remote_addr.ip() {
            IpAddr::V4(_) => SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 0),
            IpAddr::V6(_) => SocketAddr::new(IpAddr::V6(Ipv6Addr::UNSPECIFIED), 0),
        };

        let endpoint = match Endpoint::client(bind_addr) {
            Ok(mut ep) => {
                ep.set_default_client_config(client_config.clone());
                ep
            }
            Err(_) => continue,
        };

        match endpoint.connect(remote_addr, host) {
            Ok(connecting) => {
                match connecting.await {
                    Ok(conn) => {
                        // 5. 建立 HTTP/3 连接
                        match h3::client::new(h3_quinn::Connection::new(conn)).await {
                            Ok((driver, send_request)) => {
                                connection_result = Ok((driver, send_request));
                                break;
                            }
                            Err(e) => {
                                connection_result = Err(HttpClientError::ConnectionFailed(
                                    format!("HTTP/3 握手失败: {}", e),
                                ));
                            }
                        }
                    }
                    Err(e) => {
                        connection_result = Err(HttpClientError::ConnectionFailed(format!(
                            "QUIC 握手失败: {}",
                            e
                        )));
                    }
                }
            }
            Err(e) => {
                connection_result = Err(HttpClientError::ConnectionFailed(format!(
                    "QUIC 连接发起失败: {}",
                    e
                )));
            }
        }
    }

    let (driver, mut send_request) = connection_result?;

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

    // 添加 headers
    // 注意：不要手动添加 host header，h3 会自动从 URI 提取
    http_request = http_request.header("user-agent", &config.user_agent);

    for (key, value) in &request_with_cookies.headers {
        // 跳过 host header（如果用户传入了）
        if key.to_lowercase() != "host" {
            http_request = http_request.header(key, value);
        }
    }

    // 修复：构建请求（h3 需要 Request<()>，然后通过 stream 发送 body）
    let http_request = http_request
        .body(())
        .map_err(|e| HttpClientError::InvalidResponse(format!("构建请求失败: {}", e)))?;

    // 7. 发送请求
    let mut stream = send_request
        .send_request(http_request)
        .await
        .map_err(|e| HttpClientError::ConnectionFailed(format!("发送请求失败: {}", e)))?;

    // 修复：通过 stream 发送请求体（如果存在）
    if let Some(body) = &request.body {
        if !body.is_empty() {
            stream
                .send_data(bytes::Bytes::from(body.clone()))
                .await
                .map_err(|e| HttpClientError::ConnectionFailed(format!("发送请求体失败: {}", e)))?;
        }
    }

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

    // 安全修复：检查 HTTP/3 响应头大小，防止 QPACK 压缩炸弹攻击
    // h3 crate 0.0.4 的默认限制通常较大，我们添加额外的检查
    const MAX_HTTP3_HEADER_SIZE: usize = 64 * 1024; // 64KB (RFC 9114 建议的最小值)
    let total_header_size: usize = headers
        .iter()
        .map(|(k, v)| k.as_str().len() + v.len())
        .sum();
    if total_header_size > MAX_HTTP3_HEADER_SIZE {
        return Err(HttpClientError::InvalidResponse(format!(
            "HTTP/3 响应头过大（>{} bytes）",
            MAX_HTTP3_HEADER_SIZE
        )));
    }

    // 接收 body
    use bytes::Buf;
    let mut body_data = Vec::new();

    // 安全限制：防止 HTTP/3 响应体过大导致内存耗尽
    const MAX_HTTP3_BODY_SIZE: usize = 100 * 1024 * 1024; // 100MB

    while let Some(mut chunk) = stream
        .recv_data()
        .await
        .map_err(|e| HttpClientError::Io(std::io::Error::other(format!("读取 body 失败: {}", e))))?
    {
        // 使用 Buf trait 读取数据
        let chunk_len = chunk.remaining();

        // 安全检查：防止响应体过大
        if body_data.len().saturating_add(chunk_len) > MAX_HTTP3_BODY_SIZE {
            return Err(HttpClientError::InvalidResponse(format!(
                "HTTP/3 响应体过大（>{} bytes）",
                MAX_HTTP3_BODY_SIZE
            )));
        }

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
