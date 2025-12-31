//! HTTP/2 实现
//!
//! 使用 h2 crate 实现完整的 HTTP/2 支持
//! 应用 fingerprint-rust 的 HTTP/2 Settings

use super::{HttpClientConfig, HttpClientError, HttpRequest, HttpResponse, Result};

#[cfg(feature = "http2")]
use h2::client;

// 修复：使用全局单例 Runtime 避免频繁创建
#[cfg(feature = "http2")]
use once_cell::sync::Lazy;

#[cfg(feature = "http2")]
static RUNTIME: Lazy<tokio::runtime::Runtime> =
    Lazy::new(|| tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime"));

/// 发送 HTTP/2 请求
#[cfg(feature = "http2")]
pub fn send_http2_request(
    host: &str,
    port: u16,
    path: &str,
    request: &HttpRequest,
    config: &HttpClientConfig,
) -> Result<HttpResponse> {
    // 修复：使用全局单例 Runtime，避免每次请求都创建新的运行时
    RUNTIME.block_on(async { send_http2_request_async(host, port, path, request, config).await })
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

    // 1. 建立 TCP 连接（应用 TCP Profile）
    let addr = format!("{}:{}", host, port);
    let socket_addrs = addr
        .to_socket_addrs()
        .map_err(|e| HttpClientError::InvalidUrl(format!("DNS 解析失败: {}", e)))?
        .next()
        .ok_or_else(|| HttpClientError::InvalidUrl("无法解析地址".to_string()))?;

    // 应用 TCP Profile（如果配置了）
    let tcp = if let Some(profile) = &config.profile {
        if let Some(ref tcp_profile) = profile.tcp_profile {
            super::tcp_fingerprint::connect_tcp_with_profile(socket_addrs, Some(tcp_profile))
                .await
                .map_err(|e| HttpClientError::ConnectionFailed(format!("TCP 连接失败: {}", e)))?
        } else {
            TcpStream::connect(socket_addrs)
                .await
                .map_err(|e| HttpClientError::ConnectionFailed(format!("TCP 连接失败: {}", e)))?
        }
    } else {
        TcpStream::connect(socket_addrs)
            .await
            .map_err(|e| HttpClientError::ConnectionFailed(format!("TCP 连接失败: {}", e)))?
    };

    // 2. TLS 握手
    let tls_stream = perform_tls_handshake(tcp, host, config).await?;

    // 3. HTTP/2 握手（应用 Settings 配置）
    let mut builder = client::Builder::new();

    // 应用指纹配置中的 HTTP/2 Settings
    if let Some(profile) = &config.profile {
        // 设置初始窗口大小
        if let Some(&window_size) = profile
            .settings
            .get(&fingerprint_headers::http2_config::HTTP2SettingID::InitialWindowSize.as_u16())
        {
            builder.initial_window_size(window_size);
        }

        // 设置最大帧大小
        if let Some(&max_frame_size) = profile
            .settings
            .get(&fingerprint_headers::http2_config::HTTP2SettingID::MaxFrameSize.as_u16())
        {
            builder.max_frame_size(max_frame_size);
        }

        // 设置最大头部列表大小
        if let Some(&max_header_list_size) = profile
            .settings
            .get(&fingerprint_headers::http2_config::HTTP2SettingID::MaxHeaderListSize.as_u16())
        {
            builder.max_header_list_size(max_header_list_size);
        }

        // 设置连接级窗口大小（Connection Flow）
        builder.initial_connection_window_size(profile.connection_flow);
    }

    let (mut client, h2_conn) = builder
        .handshake(tls_stream)
        .await
        .map_err(|e| HttpClientError::ConnectionFailed(format!("HTTP/2 握手失败: {}", e)))?;

    // 在后台驱动 HTTP/2 连接
    tokio::spawn(async move {
        if let Err(e) = h2_conn.await {
            eprintln!("警告: HTTP/2 连接错误: {}", e);
        }
    });

    // 4. 构建请求
    let uri = format!("https://{}{}", host, path);
    let mut http_request = http::Request::builder()
        .method(request.method.as_str())
        .uri(uri)
        .version(http::Version::HTTP_2);

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
    // 注意：不要手动添加 host header，h2 会自动从 URI 提取
    http_request = http_request.header("user-agent", &config.user_agent);

    for (key, value) in &request_with_cookies.headers {
        // 跳过 host header（如果用户传入了）
        if key.to_lowercase() != "host" {
            http_request = http_request.header(key, value);
        }
    }

    // 修复：构建请求（h2 需要 Request<()>，然后通过 SendStream 发送 body）
    let http_request = http_request
        .body(())
        .map_err(|e| HttpClientError::InvalidResponse(format!("构建请求失败: {}", e)))?;

    // 6. 发送请求（获取 SendStream 用于发送 body）
    // 修复：end_of_stream 必须为 false，否则流会立即关闭，无法发送 body
    let has_body = request_with_cookies.body.is_some()
        && !request_with_cookies.body.as_ref().unwrap().is_empty();
    let (response_future, mut send_stream) = client
        .send_request(http_request, false) // 修复：改为 false，只有在发送完 body 后才结束流
        .map_err(|e| HttpClientError::ConnectionFailed(format!("发送请求失败: {}", e)))?;

    // 修复：通过 SendStream 发送请求体（如果存在）
    if let Some(body) = &request_with_cookies.body {
        if !body.is_empty() {
            // 发送 body 数据，end_of_stream = true 表示这是最后的数据
            send_stream
                .send_data(bytes::Bytes::from(body.clone()), true)
                .map_err(|e| HttpClientError::ConnectionFailed(format!("发送请求体失败: {}", e)))?;
        } else {
            // 空 body，发送空数据并结束流
            send_stream
                .send_data(bytes::Bytes::new(), true)
                .map_err(|e| HttpClientError::ConnectionFailed(format!("发送请求体失败: {}", e)))?;
        }
    } else if !has_body {
        // 没有 body，发送空数据并结束流
        send_stream
            .send_data(bytes::Bytes::new(), true)
            .map_err(|e| HttpClientError::ConnectionFailed(format!("发送请求体失败: {}", e)))?;
    }

    // 7. 接收响应
    let response = response_future
        .await
        .map_err(|e| HttpClientError::InvalidResponse(format!("接收响应失败: {}", e)))?;

    let status_code = response.status().as_u16();
    let headers = response.headers().clone();

    // 安全修复：检查 HTTP/2 响应头大小，防止 Header 压缩炸弹攻击
    // h2 crate 0.4 的默认 MAX_HEADER_LIST_SIZE 通常较大，我们添加额外的检查
    const MAX_HTTP2_HEADER_SIZE: usize = 64 * 1024; // 64KB (RFC 7540 建议的最小值)
    let total_header_size: usize = headers
        .iter()
        .map(|(k, v)| k.as_str().len() + v.len())
        .sum();
    if total_header_size > MAX_HTTP2_HEADER_SIZE {
        return Err(HttpClientError::InvalidResponse(format!(
            "HTTP/2 响应头过大（>{} bytes）",
            MAX_HTTP2_HEADER_SIZE
        )));
    }

    // 接收 body
    let mut body_stream = response.into_body();
    let mut body_data = Vec::new();

    // 安全限制：防止 HTTP/2 响应体过大导致内存耗尽
    const MAX_HTTP2_BODY_SIZE: usize = 100 * 1024 * 1024; // 100MB

    while let Some(chunk) = body_stream.data().await {
        let chunk = chunk.map_err(|e| {
            HttpClientError::Io(std::io::Error::other(format!("读取 body 失败: {}", e)))
        })?;

        // 安全检查：防止响应体过大
        if body_data.len().saturating_add(chunk.len()) > MAX_HTTP2_BODY_SIZE {
            return Err(HttpClientError::InvalidResponse(format!(
                "HTTP/2 响应体过大（>{} bytes）",
                MAX_HTTP2_BODY_SIZE
            )));
        }

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
    config: &HttpClientConfig,
) -> Result<tokio_rustls::client::TlsStream<tokio::net::TcpStream>> {
    use std::sync::Arc;
    use tokio_rustls::rustls::ServerName;
    use tokio_rustls::TlsConnector;

    let tls_config = super::rustls_utils::build_client_config(
        config.verify_tls,
        vec![b"h2".to_vec(), b"http/1.1".to_vec()],
        config.profile.as_ref(),
    );

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
    #[test]
    #[cfg(feature = "http2")]
    #[ignore]
    fn test_http2_request() {
        use super::*;
        let request = HttpRequest::new(
            crate::http_client::request::HttpMethod::Get,
            "https://www.google.com/",
        );

        let config = HttpClientConfig::default();

        match send_http2_request("www.google.com", 443, "/", &request, &config) {
            Ok(response) => {
                // Google 可能会重定向或者返回 200
                println!("Status: {}", response.status_code);
                println!("Version: {}", response.http_version);
            }
            Err(e) => {
                println!("⚠️  HTTP/2 Request failed: {}", e);
            }
        }
    }
}
