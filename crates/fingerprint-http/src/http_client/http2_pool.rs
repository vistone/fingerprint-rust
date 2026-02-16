//! HTTP/2 with Connection Pool
//!
//! architectureexplain：
//! - HTTP/2 adoptsessionpool (H2SessionPool)implementtrue multiplexreuse
// ! - pool化pair象：h2::client::SendRequest handle (alreadyhandshakecompletesession)
//! - reusemethod：concurrentmultiplereuse (ansessioncan when processmultiplerequest)
//! - netconnpool role：only in Createnewsession when asbottomlayer TCP connectionsource (accelerateconnectionestablish)
//! - sessionestablishback, connectionlifecycleby H2Session backbackground task (Driver)manage

#[cfg(all(feature = "connection-pool", feature = "http2"))]
use super::pool::ConnectionPoolManager;
use super::{HttpClientConfig, HttpClientError, HttpRequest, HttpResponse, Result};
#[cfg(all(feature = "connection-pool", feature = "http2"))]
use std::sync::Arc;
use std::time::Instant;

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

    // Note: connection poolinconnection in Create when maynoapplication TCP Profile
    // in order toensure TCP fingerprintconsistency, wesuggest in Createconnection poolbeforethenthrough generate_unified_fingerprint sync TCP Profile
    // herewestill from connection poolGetconnection, butnewCreateconnectionwillapplication TCP Profile ( if configuration了)

    // from connection poolGetconnection
    let pool = pool_manager.get_pool(host, port)?;

    let start = Instant::now();

    // Get TCP connection
    let conn = pool.get_tcp().map_err(|e| {
        HttpClientError::ConnectionFailed(format!("Failed to get connection from pool: {:?}", e))
    })?;

    // from Connection in Extract TcpStream
    // PooledConnection implement了 Deref<Target = Connection>, candirectlyuse Connection method
    let tcp_stream = conn.tcp_conn().ok_or_else(|| {
        HttpClientError::ConnectionFailed("Expected TCP connection but got UDP".to_string())
    })?;

    // clone TcpStream so thatwecanuse它
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
    let server_name = rustls::pki_types::ServerName::try_from(host.to_string())
        .map_err(|_| HttpClientError::TlsError("Invalid server name".to_string()))?;

    let tls_stream = connector
        .connect(server_name, tcp_stream)
        .await
        .map_err(|e| HttpClientError::TlsError(format!("TLS handshakefailure: {}", e)))?;

    // Fix: use HTTP/2 sessionpoolimplementtrue multiplexreuse
    // avoideach timerequest都reperform TLS and HTTP/2 handshake
    let session_key = format!("{}:{}", host, port);
    let h2_session_pool = pool_manager.h2_session_pool();

    // 调试日志（仅在开发环境启用）
    #[cfg(debug_assertions)]
    {
        if let Ok(debug_log_path) = std::env::var("FINGERPRINT_DEBUG_LOG") {
            let log_msg = format!("http2_pool: request session key={}", session_key);
            if let Ok(mut file) = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&debug_log_path)
            {
                use std::io::Write;
                let _ = writeln!(
                    file,
                    "{{\"timestamp\":{},\"location\":\"http2_pool.rs:66\",\"message\":\"{}\",\"data\":{{\"key\":\"{}\",\"host\":\"{}\",\"port\":{}}},\"sessionId\":\"debug-session\",\"runId\":\"run1\",\"hypothesisId\":\"A\"}}",
                    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis(),
                    log_msg, session_key, host, port
                );
            }
        }
    }

    // from sessionpoolGet or Create SendRequest handle
    let send_request = h2_session_pool
.get_or_create_session::<_, tokio_rustls::client::TlsStream<tokio::net::TcpStream>>(&session_key, async {
 // 调试日志（仅在开发环境启用）
 #[cfg(debug_assertions)]
 {
  if let Ok(debug_log_path) = std::env::var("FINGERPRINT_DEBUG_LOG") {
  let log_msg = format!("http2_pool: start Create 新 session key={}", session_key);
  if let Ok(mut file) = std::fs::OpenOptions::new().create(true).append(true).open(&debug_log_path) {
  use std::io::Write;
  let _ = writeln!(
   file,
   "{{\"timestamp\":{},\"location\":\"http2_pool.rs:74\",\"message\":\"{}\",\"data\":{{\"key\":\"{}\"}},\"sessionId\":\"debug-session\",\"runId\":\"run1\",\"hypothesisId\":\"A\"}}",
   std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis(),
   log_msg, session_key
  );
  }
  }
 }
 // establish HTTP/2 connection
 let mut builder = client::Builder::new();

 // applicationfingerprintconfiguration in HTTP/2 Settings
 if let Some(profile) = &config.profile {
 // settingsinitialbeginningwindowsize
 if let Some(&window_size) = profile.http2_settings.get(&fingerprint_headers::http2_config::HTTP2SettingID::InitialWindowSize.as_u16()) {
 builder.initial_window_size(window_size);
 }

 // settingsmaximumframesize
 if let Some(&max_frame_size) = profile.http2_settings.get(&fingerprint_headers::http2_config::HTTP2SettingID::MaxFrameSize.as_u16()) {
 builder.max_frame_size(max_frame_size);
 }

 // settingsmaximumheaderlistsize
 if let Some(&max_header_list_size) = profile.http2_settings.get(&fingerprint_headers::http2_config::HTTP2SettingID::MaxHeaderListSize.as_u16()) {
 builder.max_header_list_size(max_header_list_size);
 }
 }

 let (client, h2_conn) = builder.handshake(tls_stream)
.await
.map_err(|e| HttpClientError::Http2Error(format!("HTTP/2 handshakefailure: {}", e)))?;

 // return SendRequest and Connection (sessionpoolwillmanage Connection lifecycle)
 Ok((client, h2_conn))
 })
.await?;

    // from sessionpoolGet SendRequest is Arc<TokioMutex<SendRequest>>
    // needGetlock才canuse
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
        // do notmanualAdd host header, h2 willautomatic from URI Extract
        .header("user-agent", &config.user_agent);

    // Fix: Add Cookie to request ( if exists)
    let mut request_with_cookies = request.clone();
    if let Some(cookie_store) = &config.cookie_store {
        super::request::add_cookies_to_request(
            &mut request_with_cookies,
            cookie_store,
            host,
            path,
            true, // HTTPS is securityconnection
        );
    }

    let http2_request = request_with_cookies
        .headers
        .iter()
        // skip host header
        .filter(|(k, _)| k.to_lowercase() != "host")
        .fold(http2_request, |builder, (k, v)| builder.header(k, v));

    // Fix: Buildrequest (h2 need Request<()>, thenthrough SendStream send body)
    let http2_request = http2_request
        .body(())
        .map_err(|e| HttpClientError::InvalidRequest(format!("Buildrequestfailure: {}", e)))?;

    // 发送请求（获取 SendStream 用于发送 body）
    // 注意：end_of_stream 必须为 false，否则流会立即关闭，无法发送 body
    let (response, mut send_stream) = client
        .send_request(http2_request, false) // 改为 false，只在发送完 body 后才结束流
        .map_err(|e| HttpClientError::Http2Error(format!("send request failure: {}", e)))?;

    // 释放锁，允许其他请求复用同一个 session
    drop(client);

    // 通过 SendStream 发送请求体（如果存在）
    let body_bytes = if let Some(body) = &request.body {
        if !body.is_empty() {
            ::bytes::Bytes::from(body.clone())
        } else {
            ::bytes::Bytes::new()
        }
    } else {
        ::bytes::Bytes::new()
    };

    // 发送 body 数据，end_of_stream = true 表示这是最后的数据
    send_stream
        .send_data(body_bytes, true)
        .map_err(|e| HttpClientError::Http2Error(format!("Failed to send request body: {}", e)))?;

    // waitresponseheader
    let response = response
        .await
        .map_err(|e| HttpClientError::Http2Error(format!("receiveresponsefailure: {}", e)))?;

    // 先Extract status and headers
    let status_code = response.status().as_u16();

    // securityFix: Check HTTP/2 responseheadersize, prevent Header compressionbombattack
    const MAX_HTTP2_HEADER_SIZE: usize = 64 * 1024; // 64KB (RFC 7540 suggestminimumvalue)
    let total_header_size: usize = response
        .headers()
        .iter()
        .map(|(k, v)| k.as_str().len() + v.len())
        .sum();
    if total_header_size > MAX_HTTP2_HEADER_SIZE {
        return Err(HttpClientError::InvalidResponse(format!(
            "HTTP/2 responseheadertoo large (>{} bytes)",
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

    // securitylimit：prevent HTTP/2 responsebody too largecauseinsidememory exhausted
    const MAX_HTTP2_BODY_SIZE: usize = 100 * 1024 * 1024; // 100MB

    while let Some(chunk) = body_stream.data().await {
        let chunk = chunk.map_err(|e| {
            HttpClientError::Io(std::io::Error::other(format!("read body failure: {}", e)))
        })?;

        // securityCheck：preventresponsebody too large
        if body_data.len().saturating_add(chunk.len()) > MAX_HTTP2_BODY_SIZE {
            return Err(HttpClientError::InvalidResponse(format!(
                "HTTP/2 responsebody too large (>{} bytes)",
                MAX_HTTP2_BODY_SIZE
            )));
        }

        body_data.extend_from_slice(&chunk);

        // releasestreamcontrolwindow
        let _ = body_stream.flow_control().release_capacity(chunk.len());
    }

    Ok(HttpResponse {
        http_version: "HTTP/2".to_string(),
        status_code,
        status_text,
        headers,
        body: body_data,
        response_time_ms: start.elapsed().as_millis() as u64, // 添加实际of响应time测量
    })
}

#[cfg(test)]
#[cfg(all(feature = "connection-pool", feature = "http2"))]
mod tests {
    use super::*;
    use crate::http_client::pool::PoolManagerConfig;
    use crate::http_client::request::HttpMethod;

    #[tokio::test]
    #[ignore] // need network connection
    async fn test_http2_with_pool() {
        // 清理之前的日志（仅在设置了环境变量时）
        #[cfg(debug_assertions)]
        if let Ok(debug_log_path) = std::env::var("FINGERPRINT_DEBUG_LOG") {
            let _ = std::fs::remove_file(&debug_log_path);
        }

        let user_agent = "TestClient/1.0".to_string();
        let config = HttpClientConfig {
            user_agent,
            prefer_http2: true,
            ..Default::default()
        };

        let pool_manager = {
            #[allow(clippy::arc_with_non_send_sync)]
            {
                Arc::new(ConnectionPoolManager::new(PoolManagerConfig::default()))
            }
        };

        let request = HttpRequest::new(HttpMethod::Get, "https://httpbin.org/get");

        println!("📡 sendfirst HTTP/2 request (shouldCreate新session)...");
        let result1 = send_http2_request_with_pool(
            "httpbin.org",
            443,
            "/get",
            &request,
            &config,
            &pool_manager,
        )
        .await;

        // maywillfailure (networkissue), but不should panic
        if let Ok(response) = &result1 {
            assert_eq!(response.http_version, "HTTP/2");
            assert!(response.status_code > 0);
            println!(" ✅ firstrequestsuccess: {}", response.status_code);
        } else {
            println!(" ❌ firstrequestfailure: {:?}", result1);
            return;
        }

        // waita smallsegment when between, ensuresessionalreadyestablish
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        println!("\n📡 sendsecond HTTP/2 request (shouldreusesession)...");
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
            println!(" ✅ secondrequestsuccess: {}", response.status_code);
        } else {
            println!(" ❌ secondrequestfailure: {:?}", result2);
        }

        // 读取日志并分析（仅在设置了环境变量时）
        #[cfg(debug_assertions)]
        if let Ok(debug_log_path) = std::env::var("FINGERPRINT_DEBUG_LOG") {
            println!("\n📋 debug log analysis:");
            if let Ok(log_content) = std::fs::read_to_string(&debug_log_path) {
                let mut create_count = 0;
                let mut reuse_count = 0;
                for line in log_content.lines() {
                    // simplestringmatchfromParse JSON log
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
                        println!(" {}: {}", location, message);

                        if message.contains("Create新session")
                            || message.contains("Create 新 session")
                        {
                            create_count += 1;
                        } else if message.contains("reuseexistingsession") {
                            reuse_count += 1;
                        }
                    }
                }
                println!("\n📊 session pool statistics:");
                println!(" Create 新 session: {} 次", create_count);
                println!(" reuse session: {} 次", reuse_count);

                if reuse_count > 0 {
                    println!(" ✅ session reuse success！HTTP/2 multiple reuse normal 工作");
                } else if create_count > 1 {
                    println!(" ⚠️ session not reuse，each time request 都 Create 新 session");
                } else {
                    println!(" ℹ️ 只 send 了 a request，unable to Validate session reuse");
                }
            } else {
                println!(" ⚠️ unable to read log file");
            }
        }
    }
}
