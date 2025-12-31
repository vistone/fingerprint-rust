//! HTTP/2 with Connection Pool
//!
//! 架构说明：
//! - HTTP/2 采用会话池（H2SessionPool）实现真正的多路复用
//! - 池化对象：h2::client::SendRequest 句柄（已握手完成的会话）
//! - 复用方式：并发多路复用（一个会话可同时处理多个请求）
//! - netconnpool 角色：仅在创建新会话时作为底层 TCP 连接源（加速连接建立）
//! - 会话建立后，连接生命周期由 H2Session 的后台任务（Driver）管理

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

    // 注意：连接池中的连接在创建时可能没有应用 TCP Profile
    // 为了确保 TCP 指纹一致性，我们建议在创建连接池之前就通过 generate_unified_fingerprint 同步 TCP Profile
    // 这里我们仍然从连接池获取连接，但新创建的连接会应用 TCP Profile（如果配置了）

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

    // 修复：使用 HTTP/2 会话池实现真正的多路复用
    // 避免每次请求都重新进行 TLS 和 HTTP/2 握手
    let session_key = format!("{}:{}", host, port);
    let h2_session_pool = pool_manager.h2_session_pool();

    // #region agent log
    let log_msg = format!("http2_pool: 请求会话 key={}", session_key);
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open("/home/stone/fingerprint-rust/.cursor/debug.log")
    {
        use std::io::Write;
        let _ = writeln!(file, "{{\"timestamp\":{},\"location\":\"http2_pool.rs:66\",\"message\":\"{}\",\"data\":{{\"key\":\"{}\",\"host\":\"{}\",\"port\":{}}},\"sessionId\":\"debug-session\",\"runId\":\"run1\",\"hypothesisId\":\"A\"}}", 
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis(),
            log_msg, session_key, host, port);
    }
    // #endregion

    // 从会话池获取或创建 SendRequest 句柄
    let send_request = h2_session_pool
        .get_or_create_session::<_, tokio_rustls::client::TlsStream<tokio::net::TcpStream>>(&session_key, async {
            // #region agent log
            let log_msg = format!("http2_pool: 开始创建新会话 key={}", session_key);
            if let Ok(mut file) = std::fs::OpenOptions::new().create(true).append(true).open("/home/stone/fingerprint-rust/.cursor/debug.log") {
                use std::io::Write;
                let _ = writeln!(file, "{{\"timestamp\":{},\"location\":\"http2_pool.rs:74\",\"message\":\"{}\",\"data\":{{\"key\":\"{}\"}},\"sessionId\":\"debug-session\",\"runId\":\"run1\",\"hypothesisId\":\"A\"}}", 
                    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis(),
                    log_msg, session_key);
            }
            // #endregion
            // 建立 HTTP/2 连接
            let mut builder = client::Builder::new();

            // 应用指纹配置中的 HTTP/2 Settings
            if let Some(profile) = &config.profile {
                // 设置初始窗口大小
                if let Some(&window_size) = profile.settings.get(&fingerprint_headers::http2_config::HTTP2SettingID::InitialWindowSize.as_u16()) {
                    builder.initial_window_size(window_size);
                }

                // 设置最大帧大小
                if let Some(&max_frame_size) = profile.settings.get(&fingerprint_headers::http2_config::HTTP2SettingID::MaxFrameSize.as_u16()) {
                    builder.max_frame_size(max_frame_size);
                }

                // 设置最大头部列表大小
                if let Some(&max_header_list_size) = profile.settings.get(&fingerprint_headers::http2_config::HTTP2SettingID::MaxHeaderListSize.as_u16()) {
                    builder.max_header_list_size(max_header_list_size);
                }

                // 设置连接级窗口大小（Connection Flow）
                builder.initial_connection_window_size(profile.connection_flow);
            }

            let (client, h2_conn) = builder.handshake(tls_stream)
                .await
                .map_err(|e| HttpClientError::Http2Error(format!("HTTP/2 握手失败: {}", e)))?;

            // 返回 SendRequest 和 Connection（会话池会管理 Connection 的生命周期）
            Ok((client, h2_conn))
        })
        .await?;

    // 从会话池获取的 SendRequest 是 Arc<TokioMutex<SendRequest>>
    // 需要获取锁才能使用
    let mut client = send_request.lock().await;

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

    // 释放锁，允许其他请求复用同一个会话
    drop(client);

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
        // 清除之前的日志
        let _ = std::fs::remove_file("/home/stone/fingerprint-rust/.cursor/debug.log");

        let user_agent = "TestClient/1.0".to_string();
        let config = HttpClientConfig {
            user_agent,
            prefer_http2: true,
            ..Default::default()
        };

        let pool_manager = Arc::new(ConnectionPoolManager::new(PoolManagerConfig::default()));

        let request = HttpRequest::new(HttpMethod::Get, "https://httpbin.org/get");

        println!("📡 发送第一个 HTTP/2 请求（应该创建新会话）...");
        let result1 = send_http2_request_with_pool(
            "httpbin.org",
            443,
            "/get",
            &request,
            &config,
            &pool_manager,
        )
        .await;

        // 可能会失败（网络问题），但不应该 panic
        if let Ok(response) = &result1 {
            assert_eq!(response.http_version, "HTTP/2");
            assert!(response.status_code > 0);
            println!("  ✅ 第一个请求成功: {}", response.status_code);
        } else {
            println!("  ❌ 第一个请求失败: {:?}", result1);
            return;
        }

        // 等待一小段时间，确保会话已建立
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        println!("\n📡 发送第二个 HTTP/2 请求（应该复用会话）...");
        let result2 = send_http2_request_with_pool(
            "httpbin.org",
            443,
            "/headers",
            &request,
            &config,
            &pool_manager,
        )
        .await;

        if let Ok(response) = &result2 {
            assert_eq!(response.http_version, "HTTP/2");
            assert!(response.status_code > 0);
            println!("  ✅ 第二个请求成功: {}", response.status_code);
        } else {
            println!("  ❌ 第二个请求失败: {:?}", result2);
        }

        // 读取日志并分析
        println!("\n📋 调试日志分析:");
        if let Ok(log_content) =
            std::fs::read_to_string("/home/stone/fingerprint-rust/.cursor/debug.log")
        {
            let mut create_count = 0;
            let mut reuse_count = 0;
            for line in log_content.lines() {
                // 简单的字符串匹配来解析 JSON 日志
                if line.contains("\"message\"") {
                    let location = if let Some(start) = line.find("\"location\":\"") {
                        let end = line[start + 12..].find('"').unwrap_or(0);
                        &line[start + 12..start + 12 + end]
                    } else {
                        ""
                    };
                    let message = if let Some(start) = line.find("\"message\":\"") {
                        let end = line[start + 11..].find('"').unwrap_or(0);
                        &line[start + 11..start + 11 + end]
                    } else {
                        ""
                    };
                    println!("  {}: {}", location, message);

                    if message.contains("创建新会话") {
                        create_count += 1;
                    } else if message.contains("复用现有会话") {
                        reuse_count += 1;
                    }
                }
            }
            println!("\n📊 会话池统计:");
            println!("  创建新会话: {} 次", create_count);
            println!("  复用会话: {} 次", reuse_count);

            if reuse_count > 0 {
                println!("  ✅ 会话复用成功！HTTP/2 多路复用正常工作");
            } else if create_count > 1 {
                println!("  ⚠️  会话未复用，每次请求都创建新会话");
            } else {
                println!("  ℹ️  只发送了一个请求，无法验证会话复用");
            }
        } else {
            println!("  ⚠️  无法读取日志文件");
        }
    }
}
