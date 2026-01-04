//! HTTP/2 implement
//!
//! use h2 crate implementcomplete HTTP/2 support
//! application fingerprint-rust HTTP/2 Settings

use super::{HttpClientConfig, HttpClientError, HttpRequest, HttpResponse, Result};

#[cfg(feature = "http2")]
use h2::client;

// Fix: useglobalsingleton Runtime avoidfrequentCreate
#[cfg(feature = "http2")]
use once_cell::sync::Lazy;

#[cfg(feature = "http2")]
static RUNTIME: Lazy<tokio::runtime::Runtime> =
    Lazy::new(|| tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime"));

/// send HTTP/2 request
#[cfg(feature = "http2")]
pub fn send_http2_request(
    host: &str,
    port: u16,
    path: &str,
    request: &HttpRequest,
    config: &HttpClientConfig,
) -> Result<HttpResponse> {
    // Fix: useglobalsingleton Runtime, avoideach timerequest都Create a newrun when
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

    // 1. establish TCP connection (application TCP Profile)
    let addr = format!("{}:{}", host, port);
    let socket_addrs = addr
        .to_socket_addrs()
        .map_err(|e| HttpClientError::InvalidUrl(format!("DNS Parsefailure: {}", e)))?
        .next()
        .ok_or_else(|| HttpClientError::InvalidUrl("unable toParseaddress".to_string()))?;

    // application TCP Profile ( if configuration了)
    let tcp = if let Some(profile) = &config.profile {
        if let Some(ref tcp_profile) = profile.tcp_profile {
            super::tcp_fingerprint::connect_tcp_with_profile(socket_addrs, Some(tcp_profile))
                .await
                .map_err(|e| {
                    HttpClientError::ConnectionFailed(format!("TCP Connection failed: {}", e))
                })?
        } else {
            TcpStream::connect(socket_addrs).await.map_err(|e| {
                HttpClientError::ConnectionFailed(format!("TCP Connection failed: {}", e))
            })?
        }
    } else {
        TcpStream::connect(socket_addrs).await.map_err(|e| {
            HttpClientError::ConnectionFailed(format!("TCP Connection failed: {}", e))
        })?
    };

    // 2. TLS handshake
    let tls_stream = perform_tls_handshake(tcp, host, config).await?;

    // 3. HTTP/2 handshake (application Settings configuration)
    let mut builder = client::Builder::new();

    // applicationfingerprintconfiguration in HTTP/2 Settings
    if let Some(profile) = &config.profile {
        // settingsinitialbeginningwindowsize
        if let Some(&window_size) = profile
            .settings
            .get(&fingerprint_headers::http2_config::HTTP2SettingID::InitialWindowSize.as_u16())
        {
            builder.initial_window_size(window_size);
        }

        // settingsmaximumframesize
        if let Some(&max_frame_size) = profile
            .settings
            .get(&fingerprint_headers::http2_config::HTTP2SettingID::MaxFrameSize.as_u16())
        {
            builder.max_frame_size(max_frame_size);
        }

        // settingsmaximumheaderlistsize
        if let Some(&max_header_list_size) = profile
            .settings
            .get(&fingerprint_headers::http2_config::HTTP2SettingID::MaxHeaderListSize.as_u16())
        {
            builder.max_header_list_size(max_header_list_size);
        }

        // settingsconnectionlevelwindowsize (Connection Flow)
        builder.initial_connection_window_size(profile.connection_flow);
    }

    let (mut client, h2_conn) = builder.handshake(tls_stream).await.map_err(|e| {
        HttpClientError::ConnectionFailed(format!("HTTP/2 handshakefailure: {}", e))
    })?;

    // in backdriver HTTP/2 connection
    tokio::spawn(async move {
        if let Err(e) = h2_conn.await {
            eprintln!("warning: HTTP/2 connectionerror: {}", e);
        }
    });

    // 4. Buildrequest
    let uri = format!("https://{}{}", host, path);
    let mut http_request = http::Request::builder()
        .method(request.method.as_str())
        .uri(uri)
        .version(http::Version::HTTP_2);

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

    // Add headers
    // Note: do notmanualAdd host header, h2 willautomatic from URI Extract
    http_request = http_request.header("user-agent", &config.user_agent);

    for (key, value) in &request_with_cookies.headers {
        // skip host header ( if userpassed in)
        if key.to_lowercase() != "host" {
            http_request = http_request.header(key, value);
        }
    }

    // Fix: Buildrequest (h2 need Request<()>, thenthrough SendStream send body)
    let http_request = http_request
        .body(())
        .map_err(|e| HttpClientError::InvalidResponse(format!("Buildrequestfailure: {}", e)))?;

    // 6. sendrequest (Get SendStream for send body)
    // Fix: end_of_stream must as false, otherwisestreamwillimmediatelyclose, unable tosend body
    let has_body = request_with_cookies.body.is_some()
        && !request_with_cookies.body.as_ref().unwrap().is_empty();
    let (response_future, mut send_stream) = client
        .send_request(http_request, false) // Fix: 改 as false，only in send完 body back才endstream
        .map_err(|e| HttpClientError::ConnectionFailed(format!("sendrequestfailure: {}", e)))?;

    // Fix: through SendStream sendrequest体 ( if exists)
    if let Some(body) = &request_with_cookies.body {
        if !body.is_empty() {
            // send body countdata, end_of_stream = true representthis isfinallycountdata
            send_stream
                .send_data(bytes::Bytes::from(body.clone()), true)
                .map_err(|e| {
                    HttpClientError::ConnectionFailed(format!("Failed to send request body: {}", e))
                })?;
        } else {
            // empty body, sendemptycountdata并endstream
            send_stream
                .send_data(bytes::Bytes::new(), true)
                .map_err(|e| {
                    HttpClientError::ConnectionFailed(format!("Failed to send request body: {}", e))
                })?;
        }
    } else if !has_body {
        // no body, sendemptycountdata并endstream
        send_stream
            .send_data(bytes::Bytes::new(), true)
            .map_err(|e| {
                HttpClientError::ConnectionFailed(format!("Failed to send request body: {}", e))
            })?;
    }

    // 7. receiveresponse
    let response = response_future
        .await
        .map_err(|e| HttpClientError::InvalidResponse(format!("receiveresponsefailure: {}", e)))?;

    let status_code = response.status().as_u16();
    let headers = response.headers().clone();

    // securityFix: Check HTTP/2 responseheadersize, prevent Header compressionbombattack
    // h2 crate 0.4 default MAX_HEADER_LIST_SIZE usuallylarger, weAdd额outsideCheck
    const MAX_HTTP2_HEADER_SIZE: usize = 64 * 1024; // 64KB (RFC 7540 suggestminimumvalue)
    let total_header_size: usize = headers
        .iter()
        .map(|(k, v)| k.as_str().len() + v.len())
        .sum();
    if total_header_size > MAX_HTTP2_HEADER_SIZE {
        return Err(HttpClientError::InvalidResponse(format!(
            "HTTP/2 responseheadertoo large (>{} bytes)",
            MAX_HTTP2_HEADER_SIZE
        )));
    }

    // receive body
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

    let elapsed = start.elapsed().as_millis() as u64;

    // 8. Buildresponse
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
        .map_err(|_| HttpClientError::TlsError("Invalid server name".to_string()))?;

    connector
        .connect(server_name, tcp)
        .await
        .map_err(|e| HttpClientError::TlsError(format!("TLS handshakefailure: {}", e)))
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
        "HTTP/2 supportnotenabled，请use --features http2 compile".to_string(),
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
                // Google maywillredirect or one whoreturn 200
                println!("Status: {}", response.status_code);
                println!("Version: {}", response.http_version);
            }
            Err(e) => {
                println!("⚠️ HTTP/2 Request failed: {}", e);
            }
        }
    }
}
