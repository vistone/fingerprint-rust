//! HTTP/3 implement
//!
//! use quinn + h3 implementcomplete HTTP/3 support
//! HTTP/3 based on QUIC protocol

use super::{HttpClientConfig, HttpClientError, HttpRequest, HttpResponse, Result};

#[cfg(feature = "http3")]
use quinn::{ClientConfig, Endpoint, TransportConfig};

// Fix: useglobalsingleton Runtime avoidfrequentCreate
#[cfg(feature = "http3")]
use once_cell::sync::Lazy;

#[cfg(feature = "http3")]
static RUNTIME: Lazy<tokio::runtime::Runtime> =
    Lazy::new(|| tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime"));

/// send HTTP/3 request
#[cfg(feature = "http3")]
pub fn send_http3_request(
    host: &str,
    port: u16,
    path: &str,
    request: &HttpRequest,
    config: &HttpClientConfig,
) -> Result<HttpResponse> {
    // Fix: useglobalsingleton Runtime, avoideach timerequest都Create a newrun when
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

    // 1. configuration QUIC client
    let tls_config = super::rustls_utils::build_client_config(
        config.verify_tls,
        vec![b"h3".to_vec()],
        config.profile.as_ref(),
    );

    let mut client_config = ClientConfig::new(Arc::new(
        quinn::crypto::rustls::QuicClientConfig::try_from(tls_config).map_err(|e| {
            HttpClientError::TlsError(format!("Failed to create QUIC config: {}", e))
        })?,
    ));

    // optimizetransferconfigurationending withimproveperformance and connectionmigratecapability
    let mut transport = TransportConfig::default();

    // connectionmigrate (Connection Migration) optimize
    // QUIC allow in IP toggle when keepconnection, thispairmobilesimulate至closeimportant
    transport.initial_rtt(Duration::from_millis(100));
    transport.max_idle_timeout(Some(Duration::from_secs(60).try_into().map_err(|e| {
        HttpClientError::ConnectionFailed(format!("configurationtimeoutfailure: {}", e))
    })?));
    // increasekeep-alivefrequencyending withauxiliaryconnectionmigrateidentify
    transport.keep_alive_interval(Some(Duration::from_secs(20)));

    // allowpairendmigrate (defaultalreadyopen, hereexplicitexplain其importantproperty)
    // transport.allow_peer_migration(true);

    // simulate Chrome streamcontrolwindow (Chrome usuallyuselargerwindowending withimprove吞吐)
    transport.stream_receive_window((6 * 1024 * 1024u32).into()); // 6MB (Chrome style)
    transport.receive_window((15 * 1024 * 1024u32).into()); // 15MB (Chrome style)

    // allowmoreconcurrentstream
    transport.max_concurrent_bidi_streams(100u32.into());
    transport.max_concurrent_uni_streams(100u32.into());

    client_config.transport_config(Arc::new(transport));

    // 2. DNS Parse (priority IPv4, avoid IPv4 endpoint connection IPv6 remote cause invalid remote address)
    let addr_str = format!("{}:{}", host, port);
    let mut addrs: Vec<SocketAddr> = addr_str
        .to_socket_addrs()
        .map_err(|e| HttpClientError::InvalidUrl(format!("DNS Parsefailure: {}", e)))?
        .collect();
    if addrs.is_empty() {
        return Err(HttpClientError::InvalidUrl(
            "unable toParseaddress".to_string(),
        ));
    }
    addrs.sort_by_key(|a| matches!(a.ip(), IpAddr::V6(_))); // IPv4 priority

    // 4. connection to server (Happy Eyeballs simplify版：looptryallParse to address)
    let mut connection_result = Err(HttpClientError::ConnectionFailed(
        "无availableaddress".to_string(),
    ));

    for remote_addr in addrs {
        // Create QUIC endpoint ( by remote address族selectbind)
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
                        // 5. establish HTTP/3 connection
                        match h3::client::new(h3_quinn::Connection::new(conn)).await {
                            Ok((driver, send_request)) => {
                                connection_result = Ok((driver, send_request));
                                break;
                            }
                            Err(e) => {
                                connection_result = Err(HttpClientError::ConnectionFailed(
                                    format!("HTTP/3 handshakefailure: {}", e),
                                ));
                            }
                        }
                    }
                    Err(e) => {
                        connection_result = Err(HttpClientError::ConnectionFailed(format!(
                            "QUIC handshakefailure: {}",
                            e
                        )));
                    }
                }
            }
            Err(e) => {
                connection_result = Err(HttpClientError::ConnectionFailed(format!(
                    "QUIC connection发起failure: {}",
                    e
                )));
            }
        }
    }

    let (driver, mut send_request) = connection_result?;

    // in backdriverconnection：h3 0.0.4 driver need被continuous poll_close
    tokio::spawn(async move {
        let mut driver = driver;
        let _ = std::future::poll_fn(|cx| driver.poll_close(cx)).await;
    });

    // 6. Buildrequest
    let uri = format!("https://{}{}", host, path);
    let mut http_request = http::Request::builder()
        .method(request.method.as_str())
        .uri(uri)
        .version(http::Version::HTTP_3);

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
    // Note: do notmanualAdd host header, h3 willautomatic from URI Extract
    http_request = http_request.header("user-agent", &config.user_agent);

    for (key, value) in &request_with_cookies.headers {
        // skip host header ( if userpassed in)
        if key.to_lowercase() != "host" {
            http_request = http_request.header(key, value);
        }
    }

    // Fix: Buildrequest (h3 need Request<()>, thenthrough stream send body)
    let http_request = http_request
        .body(())
        .map_err(|e| HttpClientError::InvalidResponse(format!("Buildrequestfailure: {}", e)))?;

    // 7. sendrequest
    let mut stream = send_request
        .send_request(http_request)
        .await
        .map_err(|e| HttpClientError::ConnectionFailed(format!("sendrequestfailure: {}", e)))?;

    // Fix: through stream sendrequest体 ( if exists)
    if let Some(body) = &request.body {
        if !body.is_empty() {
            stream
                .send_data(bytes::Bytes::from(body.clone()))
                .await
                .map_err(|e| {
                    HttpClientError::ConnectionFailed(format!("Failed to send request body: {}", e))
                })?;
        }
    }

    stream
        .finish()
        .await
        .map_err(|e| HttpClientError::ConnectionFailed(format!("endrequestfailure: {}", e)))?;

    // 8. receiveresponse
    let response = stream
        .recv_response()
        .await
        .map_err(|e| HttpClientError::InvalidResponse(format!("receiveresponsefailure: {}", e)))?;

    let status_code = response.status().as_u16();
    let headers = response.headers().clone();

    // securityFix: Check HTTP/3 responseheadersize, prevent QPACK compressionbombattack
    // h3 crate 0.0.4 defaultlimitusuallylarger, weAdd额outsideCheck
    const MAX_HTTP3_HEADER_SIZE: usize = 64 * 1024; // 64KB (RFC 9114 suggestminimumvalue)
    let total_header_size: usize = headers
        .iter()
        .map(|(k, v)| k.as_str().len() + v.len())
        .sum();
    if total_header_size > MAX_HTTP3_HEADER_SIZE {
        return Err(HttpClientError::InvalidResponse(format!(
            "HTTP/3 responseheadertoo large (>{} bytes)",
            MAX_HTTP3_HEADER_SIZE
        )));
    }

    // receive body
    use bytes::Buf;
    let mut body_data = Vec::new();

    // securitylimit：prevent HTTP/3 responsebody too largecauseinsidememory exhausted
    const MAX_HTTP3_BODY_SIZE: usize = 100 * 1024 * 1024; // 100MB

    while let Some(mut chunk) = stream.recv_data().await.map_err(|e| {
        HttpClientError::Io(std::io::Error::other(format!("read body failure: {}", e)))
    })? {
        // use Buf trait readcountdata
        let chunk_len = chunk.remaining();

        // securityCheck：preventresponsebody too large
        if body_data.len().saturating_add(chunk_len) > MAX_HTTP3_BODY_SIZE {
            return Err(HttpClientError::InvalidResponse(format!(
                "HTTP/3 responsebody too large (>{} bytes)",
                MAX_HTTP3_BODY_SIZE
            )));
        }

        let mut chunk_bytes = vec![0u8; chunk_len];
        chunk.copy_to_slice(&mut chunk_bytes);
        body_data.extend_from_slice(&chunk_bytes);
    }

    let elapsed = start.elapsed().as_millis() as u64;

    // 9. Buildresponse
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
        "HTTP/3 supportnotenabled，请use --features http3 compile".to_string(),
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
