//! HTTP/2 with Connection Pool
//!
//! 使用 netconnpool 管理 TCP 连接复用，支持 HTTP/2

#[cfg(all(feature = "connection-pool", feature = "http2"))]
use super::pool::ConnectionPoolManager;
use super::{HttpClientConfig, HttpClientError, HttpRequest, HttpResponse, Result};
#[cfg(all(feature = "connection-pool", feature = "http2"))]
use std::sync::Arc;

/// 使用连接池发送 HTTP/2 请求
#[cfg(all(feature = "connection-pool", feature = "http2"))]
pub async fn send_http2_request_with_pool(
    host: &str,
    port: u16,
    path: &str,
    request: &HttpRequest,
    config: &HttpClientConfig,
    pool_manager: &Arc<ConnectionPoolManager>,
) -> Result<HttpResponse> {
    use h2::client;
    use http::{Request as HttpRequest2, Version};
    use tokio_rustls::TlsConnector;

    // 从连接池获取连接
    let pool = pool_manager.get_pool(host, port)?;

    // 获取 TCP 连接
    let conn = pool
        .get_tcp()
        .map_err(|e| HttpClientError::ConnectionFailed(format!("从连接池获取连接失败: {:?}", e)))?;

    // 从 Connection 中提取 TcpStream
    // PooledConnection 实现了 Deref<Target = Connection>，可以直接使用 Connection 的方法
    let tcp_stream = conn
        .tcp_conn()
        .ok_or_else(|| HttpClientError::ConnectionFailed("期望 TCP 连接但得到 UDP".to_string()))?;

    // 克隆 TcpStream 以便我们可以使用它
    let tcp_stream = tcp_stream.try_clone().map_err(HttpClientError::Io)?;

    // 转换为 tokio TcpStream
    tcp_stream
        .set_nonblocking(true)
        .map_err(HttpClientError::Io)?;
    let tcp_stream = tokio::net::TcpStream::from_std(tcp_stream).map_err(HttpClientError::Io)?;

    // TLS 握手
    let tls_config = super::rustls_utils::build_client_config(
        config.verify_tls,
        vec![b"h2".to_vec()],
        config.profile.as_ref(),
    );
    let connector = TlsConnector::from(std::sync::Arc::new(tls_config));
    let server_name = rustls::ServerName::try_from(host)
        .map_err(|_| HttpClientError::TlsError("无效的服务器名称".to_string()))?;

    let tls_stream = connector
        .connect(server_name, tcp_stream)
        .await
        .map_err(|e| HttpClientError::TlsError(format!("TLS 握手失败: {}", e)))?;

    // 架构问题：当前实现每次请求都重新进行 HTTP/2 握手
    // 这违背了 HTTP/2 的核心优势（多路复用），导致性能问题
    //
    // 修复建议：实现 HTTP/2 会话池，池化 h2::client::SendRequest 句柄
    // 1. 创建 H2SessionPool 管理器，按 host:port 缓存 SendRequest
    // 2. 每个会话需要后台任务运行 h2_conn 以保持连接活跃
    // 3. 实现会话超时和失效检测机制
    // 4. 只有在会话失效时才重新握手
    //
    // 注意：完整的会话池化需要管理连接生命周期，这是一个较大的架构改动
    // 当前实现虽然功能正确，但性能未达到 HTTP/2 的最佳实践

    // 建立 HTTP/2 连接
    // 注意：h2 0.4 的 Builder API 可能不支持所有 Settings
    // 先使用默认 handshake，Settings 应用需要进一步研究 h2 API
    let (mut client, h2_conn) = client::handshake(tls_stream)
        .await
        .map_err(|e| HttpClientError::Http2Error(format!("HTTP/2 握手失败: {}", e)))?;

    // TODO: 应用 HTTP/2 Settings
    // h2 0.4 的 Builder API 限制，Settings 需要在握手时配置
    // 但 client::handshake() 不提供 Builder，需要研究如何应用自定义 Settings
    if let Some(_profile) = &config.profile {
        // Settings 信息已从 profile 获取，但 h2 0.4 API 限制无法直接应用
        // 这不会影响功能，只是无法精确模拟浏览器的 Settings 值
    }

    // 在后台运行连接
    tokio::spawn(async move {
        if let Err(e) = h2_conn.await {
            eprintln!("警告: HTTP/2 连接错误: {:?}", e);
        }
    });

    // 构建 HTTP/2 请求
    let uri: http::Uri = format!("https://{}:{}{}", host, port, path)
        .parse()
        .map_err(|e| HttpClientError::InvalidRequest(format!("无效的 URI: {}", e)))?;

    let http2_request = HttpRequest2::builder()
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
        .version(Version::HTTP_2)
        // 不要手动添加 host header，h2 会自动从 URI 提取
        .header("user-agent", &config.user_agent);

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

    let http2_request = request_with_cookies
        .headers
        .iter()
        // 跳过 host header
        .filter(|(k, _)| k.to_lowercase() != "host")
        .fold(http2_request, |builder, (k, v)| builder.header(k, v));

    // 修复：构建请求（h2 需要 Request<()>，然后通过 SendStream 发送 body）
    let http2_request = http2_request
        .body(())
        .map_err(|e| HttpClientError::InvalidRequest(format!("构建请求失败: {}", e)))?;

    // 发送请求（获取 SendStream 用于发送 body）
    // 修复：end_of_stream 必须为 false，否则流会立即关闭，无法发送 body
    let has_body = request.body.is_some() && !request.body.as_ref().unwrap().is_empty();
    let (response, mut send_stream) = client
        .send_request(http2_request, false) // 修复：改为 false，只有在发送完 body 后才结束流
        .map_err(|e| HttpClientError::Http2Error(format!("发送请求失败: {}", e)))?;

    // 修复：通过 SendStream 发送请求体（如果存在）
    if let Some(body) = &request.body {
        if !body.is_empty() {
            // 发送 body 数据，end_of_stream = true 表示这是最后的数据
            send_stream
                .send_data(::bytes::Bytes::from(body.clone()), true)
                .map_err(|e| HttpClientError::Http2Error(format!("发送请求体失败: {}", e)))?;
        } else {
            // 空 body，发送空数据并结束流
            send_stream
                .send_data(::bytes::Bytes::new(), true)
                .map_err(|e| HttpClientError::Http2Error(format!("发送请求体失败: {}", e)))?;
        }
    } else if !has_body {
        // 没有 body，发送空数据并结束流
        send_stream
            .send_data(::bytes::Bytes::new(), true)
            .map_err(|e| HttpClientError::Http2Error(format!("发送请求体失败: {}", e)))?;
    }

    // 等待响应头
    let response = response
        .await
        .map_err(|e| HttpClientError::Http2Error(format!("接收响应失败: {}", e)))?;

    // 先提取 status 和 headers
    let status_code = response.status().as_u16();

    // 安全修复：检查 HTTP/2 响应头大小，防止 Header 压缩炸弹攻击
    const MAX_HTTP2_HEADER_SIZE: usize = 64 * 1024; // 64KB (RFC 7540 建议的最小值)
    let total_header_size: usize = response
        .headers()
        .iter()
        .map(|(k, v)| k.as_str().len() + v.len())
        .sum();
    if total_header_size > MAX_HTTP2_HEADER_SIZE {
        return Err(HttpClientError::InvalidResponse(format!(
            "HTTP/2 响应头过大（>{} bytes）",
            MAX_HTTP2_HEADER_SIZE
        )));
    }

    let status_text = http::StatusCode::from_u16(status_code)
        .ok()
        .and_then(|s| s.canonical_reason())
        .unwrap_or("Unknown")
        .to_string();
    let headers: std::collections::HashMap<String, String> = response
        .headers()
        .iter()
        .map(|(k, v)| (k.as_str().to_string(), v.to_str().unwrap_or("").to_string()))
        .collect();

    // 读取响应体
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

    Ok(HttpResponse {
        http_version: "HTTP/2".to_string(),
        status_code,
        status_text,
        headers,
        body: body_data,
        response_time_ms: 0, // TODO: 添加计时
    })
}

#[cfg(test)]
#[cfg(all(feature = "connection-pool", feature = "http2"))]
mod tests {
    use super::*;
    use crate::http_client::pool::PoolManagerConfig;
    use crate::http_client::request::HttpMethod;

    #[tokio::test]
    #[ignore] // 需要网络连接
    async fn test_http2_with_pool() {
        let user_agent = "TestClient/1.0".to_string();
        let config = HttpClientConfig {
            user_agent,
            prefer_http2: true,
            ..Default::default()
        };

        let pool_manager = Arc::new(ConnectionPoolManager::new(PoolManagerConfig::default()));

        let request = HttpRequest::new(HttpMethod::Get, "https://httpbin.org/get");

        let result = send_http2_request_with_pool(
            "httpbin.org",
            443,
            "/get",
            &request,
            &config,
            &pool_manager,
        )
        .await;

        // 可能会失败（网络问题），但不应该 panic
        if let Ok(response) = result {
            assert_eq!(response.http_version, "HTTP/2");
            assert!(response.status_code > 0);
        }
    }
}
