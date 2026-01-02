//! HTTP/2 with Connection Pool
//!
//! 架构说明：
//! - HTTP/2 采用sessionpool（H2SessionPool）implement真正的多路复用
//! - pool化pair象：h2::client::SendRequest 句柄（alreadyhandshakecomplete的session）
//! - 复用方式：并发多路复用（ansession可同 when processmultiplerequest）
//! - netconnpool 角色：only in Create新session when 作为bottomlayer TCP connectionsource（加速connection建立）
//! - session建立back，connection生命周期由 H2Session 的back台任务（Driver）管理

#[cfg(all(feature = "connection-pool", feature = "http2"))]
use super::pool::ConnectionPoolManager;
use super::{HttpClientConfig, HttpClientError, HttpRequest, HttpResponse, Result};
#[cfg(all(feature = "connection-pool", feature = "http2"))]
use std::sync::Arc;

/// useconnection poolsend HTTP/2 request
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

    // Note: connection pool中的connection in Create when may没有application TCP Profile
    // 为了确保 TCP fingerprint一致性，我们建议 in Createconnection poolbefore就through generate_unified_fingerprint sync TCP Profile
    // 这里我们仍然 from connection poolGetconnection，but新Create的connectionwillapplication TCP Profile（ if configuration了）

    //  from connection poolGetconnection
    let pool = pool_manager.get_pool(host, port)?;

    // Get TCP connection
    let conn = pool
        .get_tcp()
        .map_err(|e| HttpClientError::ConnectionFailed(format!("Failed to get connection from pool: {:?}", e)))?;

    //  from  Connection 中Extract TcpStream
    // PooledConnection implement了 Deref<Target = Connection>，can直接use Connection 的method
    let tcp_stream = conn
        .tcp_conn()
        .ok_or_else(|| HttpClientError::ConnectionFailed("Expected TCP connection but got UDP".to_string()))?;

    // 克隆 TcpStream 以便我们canuse它
    let tcp_stream = tcp_stream.try_clone().map_err(HttpClientError::Io)?;

    // convert to tokio TcpStream
    tcp_stream
        .set_nonblocking(true)
        .map_err(HttpClientError::Io)?;
    let tcp_stream = tokio::net::TcpStream::from_std(tcp_stream).map_err(HttpClientError::Io)?;

    // TLS handshake
    let tls_config = super::rustls_utils::build_client_config(
        config.verify_tls,
        vec![b"h2".to_vec()],
        config.profile.as_ref(),
    );
    let connector = TlsConnector::from(std::sync::Arc::new(tls_config));
    let server_name = rustls::ServerName::try_from(host)
        .map_err(|_| HttpClientError::TlsError("Invalid server name".to_string()))?;

    let tls_stream = connector
        .connect(server_name, tcp_stream)
        .await
        .map_err(|e| HttpClientError::TlsError(format!("TLS handshakefailure: {}", e)))?;

    // Fix: use HTTP/2 sessionpoolimplement真正的多路复用
    // 避免每次request都重新进行 TLS  and HTTP/2 handshake
    let session_key = format!("{}:{}", host, port);
    let h2_session_pool = pool_manager.h2_session_pool();

    // #region agent log
    let log_msg = format!("http2_pool: requestsession key={}", session_key);
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

    //  from sessionpoolGet or Create SendRequest 句柄
    let send_request = h2_session_pool
        .get_or_create_session::<_, tokio_rustls::client::TlsStream<tokio::net::TcpStream>>(&session_key, async {
            // #region agent log
            let log_msg = format!("http2_pool: startCreate新session key={}", session_key);
            if let Ok(mut file) = std::fs::OpenOptions::new().create(true).append(true).open("/home/stone/fingerprint-rust/.cursor/debug.log") {
                use std::io::Write;
                let _ = writeln!(file, "{{\"timestamp\":{},\"location\":\"http2_pool.rs:74\",\"message\":\"{}\",\"data\":{{\"key\":\"{}\"}},\"sessionId\":\"debug-session\",\"runId\":\"run1\",\"hypothesisId\":\"A\"}}", 
                    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis(),
                    log_msg, session_key);
            }
            // #endregion
            // 建立 HTTP/2 connection
            let mut builder = client::Builder::new();

            // applicationfingerprintconfiguration中 HTTP/2 Settings
            if let Some(profile) = &config.profile {
                // settingsinitialbeginningwindowsize
                if let Some(&window_size) = profile.settings.get(&fingerprint_headers::http2_config::HTTP2SettingID::InitialWindowSize.as_u16()) {
                    builder.initial_window_size(window_size);
                }

                // settingsmaximumframesize
                if let Some(&max_frame_size) = profile.settings.get(&fingerprint_headers::http2_config::HTTP2SettingID::MaxFrameSize.as_u16()) {
                    builder.max_frame_size(max_frame_size);
                }

                // settingsmaximumheaderlistsize
                if let Some(&max_header_list_size) = profile.settings.get(&fingerprint_headers::http2_config::HTTP2SettingID::MaxHeaderListSize.as_u16()) {
                    builder.max_header_list_size(max_header_list_size);
                }

                // settingsconnectionlevelwindowsize（Connection Flow）
                builder.initial_connection_window_size(profile.connection_flow);
            }

            let (client, h2_conn) = builder.handshake(tls_stream)
                .await
                .map_err(|e| HttpClientError::Http2Error(format!("HTTP/2 handshakefailure: {}", e)))?;

            // return SendRequest  and Connection（sessionpoolwill管理 Connection 的生命周期）
            Ok((client, h2_conn))
        })
        .await?;

    //  from sessionpoolGet SendRequest 是 Arc<TokioMutex<SendRequest>>
    // needGet锁才能use
    let mut client = send_request.lock().await;

    // Build HTTP/2 request
    let uri: http::Uri = format!("https://{}:{}{}", host, port, path)
        .parse()
        .map_err(|e| HttpClientError::InvalidRequest(format!("invalid URI: {}", e)))?;

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
        // 不要manualAdd host header，h2 willautomatic from  URI Extract
        .header("user-agent", &config.user_agent);

    // Fix: Add Cookie  to request（ if  exists）
    let mut request_with_cookies = request.clone();
    if let Some(cookie_store) = &config.cookie_store {
        super::request::add_cookies_to_request(
            &mut request_with_cookies,
            cookie_store,
            host,
            path,
            true, // HTTPS 是securityconnection
        );
    }

    let http2_request = request_with_cookies
        .headers
        .iter()
        // skip host header
        .filter(|(k, _)| k.to_lowercase() != "host")
        .fold(http2_request, |builder, (k, v)| builder.header(k, v));

    // Fix: Buildrequest（h2 need Request<()>，thenthrough SendStream send body）
    let http2_request = http2_request
        .body(())
        .map_err(|e| HttpClientError::InvalidRequest(format!("Buildrequestfailure: {}", e)))?;

    // sendrequest（Get SendStream  for send body）
    // Fix: end_of_stream must为 false，otherwisestreamwill立即close，unable tosend body
    let has_body = request.body.is_some() && !request.body.as_ref().unwrap().is_empty();
    let (response, mut send_stream) = client
        .send_request(http2_request, false) // Fix: 改为 false，只有 in send完 body back才endstream
        .map_err(|e| HttpClientError::Http2Error(format!("sendrequestfailure: {}", e)))?;

    // 释放锁，允许其他request复用同ansession
    drop(client);

    // Fix: through SendStream sendrequest体（ if  exists）
    if let Some(body) = &request.body {
        if !body.is_empty() {
            // send body count据，end_of_stream = true 表示这是finally的count据
            send_stream
                .send_data(::bytes::Bytes::from(body.clone()), true)
                .map_err(|e| HttpClientError::Http2Error(format!("Failed to send request body: {}", e)))?;
        } else {
            // empty body，sendemptycount据并endstream
            send_stream
                .send_data(::bytes::Bytes::new(), true)
                .map_err(|e| HttpClientError::Http2Error(format!("Failed to send request body: {}", e)))?;
        }
    } else if !has_body {
        // 没有 body，sendemptycount据并endstream
        send_stream
            .send_data(::bytes::Bytes::new(), true)
            .map_err(|e| HttpClientError::Http2Error(format!("Failed to send request body: {}", e)))?;
    }

    // waitresponseheader
    let response = response
        .await
        .map_err(|e| HttpClientError::Http2Error(format!("receiveresponsefailure: {}", e)))?;

    // 先Extract status  and headers
    let status_code = response.status().as_u16();

    // securityFix: Check HTTP/2 responseheadersize，防止 Header compression炸弹攻击
    const MAX_HTTP2_HEADER_SIZE: usize = 64 * 1024; // 64KB (RFC 7540 建议的minimumvalue)
    let total_header_size: usize = response
        .headers()
        .iter()
        .map(|(k, v)| k.as_str().len() + v.len())
        .sum();
    if total_header_size > MAX_HTTP2_HEADER_SIZE {
        return Err(HttpClientError::InvalidResponse(format!(
            "HTTP/2 responseheader过大（>{} bytes）",
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

    // readresponse体
    let mut body_stream = response.into_body();
    let mut body_data = Vec::new();

    // securitylimit：防止 HTTP/2 response体过大导致inside存耗尽
    const MAX_HTTP2_BODY_SIZE: usize = 100 * 1024 * 1024; // 100MB

    while let Some(chunk) = body_stream.data().await {
        let chunk = chunk.map_err(|e| {
            HttpClientError::Io(std::io::Error::other(format!("read body failure: {}", e)))
        })?;

        // securityCheck：防止response体过大
        if body_data.len().saturating_add(chunk.len()) > MAX_HTTP2_BODY_SIZE {
            return Err(HttpClientError::InvalidResponse(format!(
                "HTTP/2 response体过大（>{} bytes）",
                MAX_HTTP2_BODY_SIZE
            )));
        }

        body_data.extend_from_slice(&chunk);

        // 释放stream控制window
        let _ = body_stream.flow_control().release_capacity(chunk.len());
    }

    Ok(HttpResponse {
        http_version: "HTTP/2".to_string(),
        status_code,
        status_text,
        headers,
        body: body_data,
        response_time_ms: 0, // TODO: Add计 when 
    })
}

#[cfg(test)]
#[cfg(all(feature = "connection-pool", feature = "http2"))]
mod tests {
    use super::*;
    use crate::http_client::pool::PoolManagerConfig;
    use crate::http_client::request::HttpMethod;

    #[tokio::test]
    #[ignore] // neednetworkconnection
    async fn test_http2_with_pool() {
        // clearbefore的日志
        let _ = std::fs::remove_file("/home/stone/fingerprint-rust/.cursor/debug.log");

        let user_agent = "TestClient/1.0".to_string();
        let config = HttpClientConfig {
            user_agent,
            prefer_http2: true,
            ..Default::default()
        };

        let pool_manager = Arc::new(ConnectionPoolManager::new(PoolManagerConfig::default()));

        let request = HttpRequest::new(HttpMethod::Get, "https://httpbin.org/get");

        println!("📡 sendfirst HTTP/2 request（shouldCreate新session）...");
        let result1 = send_http2_request_with_pool(
            "httpbin.org",
            443,
            "/get",
            &request,
            &config,
            &pool_manager,
        )
        .await;

        // maywillfailure（network问题），but不should panic
        if let Ok(response) = &result1 {
            assert_eq!(response.http_version, "HTTP/2");
            assert!(response.status_code > 0);
            println!("  ✅ firstrequestsuccess: {}", response.status_code);
        } else {
            println!("  ❌ firstrequestfailure: {:?}", result1);
            return;
        }

        // wait一小段 when 间，确保sessionalready建立
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        println!("\n📡 send第二个 HTTP/2 request（should复用session）...");
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
            println!("  ✅ 第二个requestsuccess: {}", response.status_code);
        } else {
            println!("  ❌ 第二个requestfailure: {:?}", result2);
        }

        // read日志并analysis
        println!("\n📋 debug日志analysis:");
        if let Ok(log_content) =
            std::fs::read_to_string("/home/stone/fingerprint-rust/.cursor/debug.log")
        {
            let mut create_count = 0;
            let mut reuse_count = 0;
            for line in log_content.lines() {
                // 简单的stringmatch来Parse JSON 日志
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

                    if message.contains("Create新session") {
                        create_count += 1;
                    } else if message.contains("复用现有session") {
                        reuse_count += 1;
                    }
                }
            }
            println!("\n📊 sessionpoolstatistics:");
            println!("  Create新session: {} 次", create_count);
            println!("  复用session: {} 次", reuse_count);

            if reuse_count > 0 {
                println!("  ✅ session复用success！HTTP/2 多路复用正常工作");
            } else if create_count > 1 {
                println!("  ⚠️  sessionnot复用，每次request都Create新session");
            } else {
                println!("  ℹ️  只send了anrequest，unable toValidatesession复用");
            }
        } else {
            println!("  ⚠️  unable toread日志file");
        }
    }
}
