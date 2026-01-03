//! HTTP/3 with Connection Pool
//!
//! architectureexplain：
//! - HTTP/3 adoptsessionpool (H3SessionPool)implement QUIC sessionreuse
//! - pool化pair象：h3::client::SendRequest handle (alreadyhandshakecomplete QUIC session)
//! - reusemethod：concurrentmultiplereuse (an QUIC connectioncan when processmultiple Stream)
//! - QUIC Features：protocolthis身includingconnectionmigrate and statusmanage，no need netconnpool
//! - sessionestablishback，connectionlifecycleby H3Session backbackground task (Driver)manage

#[cfg(all(feature = "connection-pool", feature = "http3"))]
use super::pool::ConnectionPoolManager;
use super::{HttpClientConfig, HttpClientError, HttpRequest, HttpResponse, Result};
#[cfg(all(feature = "connection-pool", feature = "http3"))]
use std::sync::Arc;
#[cfg(all(feature = "connection-pool", feature = "http3"))]
use std::time::Duration;

/// useconnection poolsend HTTP/3 request
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

 // Fix: use H3SessionPool implementtrue multiplexreuse
 let session_pool = pool_manager.h3_session_pool();
 let key = format!("{}:{}", host, port);

 // Get or Createsession
 let send_request_mutex = session_pool
.get_or_create_session(&key, async {
 // Parsetargetaddress
 use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, ToSocketAddrs};
 let addr = format!("{}:{}", host, port);
 let mut addrs: Vec<SocketAddr> = addr
.to_socket_addrs()
.map_err(|e| HttpClientError::ConnectionFailed(format!("DNS Parsefailure: {}", e)))?
.collect();
 if addrs.is_empty() {
 return Err(HttpClientError::ConnectionFailed(
 "DNS Parse无result".to_string(),
 ));
 }
 addrs.sort_by_key(|a| matches!(a.ip(), IpAddr::V6(_))); // IPv4 priority
 let remote_addr = addrs[0];

 // Create QUIC clientconfiguration
 let tls_config = super::rustls_utils::build_client_config(
 config.verify_tls,
 vec![b"h3".to_vec()],
 config.profile.as_ref(),
 );

 let mut client_config = quinn::ClientConfig::new(std::sync::Arc::new(tls_config));

 // optimizetransferconfiguration以improveperformance
 let mut transport_config = quinn::TransportConfig::default();
 transport_config.initial_rtt(Duration::from_millis(100));
 transport_config.max_idle_timeout(Some(
 quinn::IdleTimeout::try_from(std::time::Duration::from_secs(60))
.map_err(|e| HttpClientError::Http3Error(format!("configurationtimeoutfailure: {}", e)))?,
 ));
 transport_config.keep_alive_interval(Some(Duration::from_secs(10)));

 // increasereceivewindow以improvethroughput
 transport_config.stream_receive_window((1024 * 1024u32).into()); // 1MB
 transport_config.receive_window((10 * 1024 * 1024u32).into()); // 10MB

 // allowmoreconcurrentstream
 transport_config.max_concurrent_bidi_streams(100u32.into());
 transport_config.max_concurrent_uni_streams(100u32.into());

 client_config.transport_config(std::sync::Arc::new(transport_config));

 // Create QUIC endpoint
 let bind_addr = match remote_addr.ip() {
 IpAddr::V4(_) => SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 0),
 IpAddr::V6(_) => SocketAddr::new(IpAddr::V6(Ipv6Addr::UNSPECIFIED), 0),
 };
 let mut endpoint = quinn::Endpoint::client(bind_addr)
.map_err(|e| HttpClientError::Http3Error(format!("Create endpoint failure: {}", e)))?;
 endpoint.set_default_client_config(client_config);

 // connection to server
 let connecting = endpoint
.connect(remote_addr, host)
.map_err(|e| HttpClientError::Http3Error(format!("Connection failed: {}", e)))?;

 let connection = connecting
.await
.map_err(|e| HttpClientError::Http3Error(format!("establishConnection failed: {}", e)))?;

 // establish HTTP/3 connection
 let quinn_conn = h3_quinn::Connection::new(connection);

 let (driver, send_request) = h3::client::new(quinn_conn)
.await
.map_err(|e| HttpClientError::Http3Error(format!("HTTP/3 handshakefailure: {}", e)))?;

 Ok((driver, send_request))
 })
.await?;

 // Getand exclude othersproperty地use SendRequest
 let mut send_request = send_request_mutex.lock().await;

 // Build HTTP/3 request
 let uri: http::Uri = format!("https://{}:{}{}", host, port, path)
.parse()
.map_err(|e| HttpClientError::InvalidRequest(format!("invalid URI: {}", e)))?;

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
 // do notmanualAdd host header，h3 willautomatic from URI Extract
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

 let http3_request = request_with_cookies
.headers
.iter()
 // skip host header
.filter(|(k, _)| k.to_lowercase() != "host")
.fold(http3_request, |builder, (k, v)| builder.header(k, v));

 // Fix: Buildrequest (h3 need Request<()>，thenthrough stream send body)
 let http3_request = http3_request
.body(())
.map_err(|e| HttpClientError::InvalidRequest(format!("Buildrequestfailure: {}", e)))?;

 // sendrequest
 let mut stream = send_request
.send_request(http3_request)
.await
.map_err(|e| HttpClientError::Http3Error(format!("sendrequestfailure: {}", e)))?;

 // Fix: through stream sendrequest体 ( if exists)
 if let Some(body) = &request.body {
 if !body.is_empty() {
 stream
.send_data(bytes::Bytes::from(body.clone()))
.await
.map_err(|e| HttpClientError::Http3Error(format!("Failed to send request body: {}", e)))?;
 }
 }

 stream
.finish()
.await
.map_err(|e| HttpClientError::Http3Error(format!("completerequestfailure: {}", e)))?;

 // receiveresponse
 let response = stream
.recv_response()
.await
.map_err(|e| HttpClientError::Http3Error(format!("receiveresponsefailure: {}", e)))?;

 // readresponse体
 let mut body_data = Vec::new();

 // securitylimit：prevent HTTP/3 responsebody too largecauseinsidememory exhausted
 const MAX_HTTP3_BODY_SIZE: usize = 100 * 1024 * 1024; // 100MB

 while let Some(mut chunk) = stream
.recv_data()
.await
.map_err(|e| HttpClientError::Io(std::io::Error::other(format!("read body failure: {}", e))))?
 {
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

 // Parseresponse
 let status_code = response.status().as_u16();

 // securityFix: Check HTTP/3 responseheadersize，prevent QPACK compressionbombattack
 const MAX_HTTP3_HEADER_SIZE: usize = 64 * 1024; // 64KB (RFC 9114 suggestminimumvalue)
 let total_header_size: usize = response
.headers()
.iter()
.map(|(k, v)| k.as_str().len() + v.len())
.sum();
 if total_header_size > MAX_HTTP3_HEADER_SIZE {
 return Err(HttpClientError::InvalidResponse(format!(
 "HTTP/3 responseheadertoo large (>{} bytes)",
 MAX_HTTP3_HEADER_SIZE
 )));
 }

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
 response_time_ms: 0, // TODO: Add计 when 
 })
}

#[cfg(test)]
#[cfg(all(feature = "connection-pool", feature = "http3"))]
mod tests {
 use super::*;
 use crate::http_client::pool::PoolManagerConfig;
 use crate::http_client::request::HttpMethod;

 #[tokio::test]
 #[ignore] // neednetworkconnection and HTTP/3 support
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

 // maywillfailure (networkissue or server不support HTTP/3)，but不should panic
 if let Ok(response) = result {
 assert_eq!(response.http_version, "HTTP/3");
 }
 }
}
