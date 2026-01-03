//! TLS connectionsupport
//!
//! useofficial rustls asbottomlayer TLS implement
//! through ClientHelloCustomizer applicationbrowserfingerprint (Chrome, Firefox, Safari etc.)
//! simulatemarket maturebrowser TLS fingerprint, 不customselffingerprint

use super::{HttpClientConfig, HttpClientError, HttpRequest, HttpResponse, Result};
use std::io::Write;
use std::net::TcpStream;
#[allow(unused_imports)]
use std::sync::Arc;

/// TLS connectioner
///
/// useofficial rustls, through ClientHelloCustomizer applicationbrowserfingerprint
pub struct TlsConnector {
 // rustls configurationthrough HttpClientConfig pass
}

impl TlsConnector {
 pub fn new() -> Self {
 Self {}
 }
}

impl Default for TlsConnector {
 fn default() -> Self {
 Self::new()
 }
}

/// send HTTPS request
///
/// useofficial rustls asbottomlayer TLS implement
/// Ifconfiguration了 ClientProfile, willthrough ClientHelloCustomizer applicationbrowserfingerprint
/// simulatemarket maturebrowser TLS fingerprint (Chrome, Firefox, Safari etc.)
pub fn send_https_request(
 host: &str,
 port: u16,
 path: &str,
 request: &HttpRequest,
 config: &HttpClientConfig,
) -> Result<HttpResponse> {
 // use rustls,  if configuration了 profile, willautomaticthrough ClientHelloCustomizer applicationbrowserfingerprint

 // establish TCP connection
 let addr = format!("{}:{}", host, port);
 let tcp_stream = TcpStream::connect(&addr)
.map_err(|e| HttpClientError::ConnectionFailed(format!("Connection failed {}: {}", addr, e)))?;

 // settingstimeout
 tcp_stream
.set_read_timeout(Some(config.read_timeout))
.map_err(HttpClientError::Io)?;
 tcp_stream
.set_write_timeout(Some(config.write_timeout))
.map_err(HttpClientError::Io)?;

 // useofficial rustls, through ClientHelloCustomizer applicationbrowserfingerprint

 #[cfg(feature = "rustls-tls")]
 {
 use rustls::client::ServerName;
 use std::sync::Arc;

 // Build TLS configuration (尊重 verify_tls)
 let tls_config = super::rustls_utils::build_client_config(
 config.verify_tls,
 Vec::new(),
 config.profile.as_ref(),
 );

 let server_name = ServerName::try_from(host)
.map_err(|_| HttpClientError::TlsError("Invalid server name".to_string()))?;

 let conn = rustls::ClientConnection::new(Arc::new(tls_config), server_name)
.map_err(|e| HttpClientError::TlsError(format!("TLS connectionCreatefailure: {}", e)))?;

 let mut tls_stream = rustls::StreamOwned::new(conn, tcp_stream);

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

 // send HTTP request
 let header_order = config.profile.as_ref().map(|p| p.header_order.as_slice());
 let http_request = request_with_cookies.build_http1_request_bytes(host, path, header_order);
 tls_stream
.write_all(&http_request)
.map_err(HttpClientError::Io)?;
 tls_stream.flush().map_err(HttpClientError::Io)?;

 // readresponse
 let buffer = super::io::read_http1_response_bytes(
 &mut tls_stream,
 super::io::DEFAULT_MAX_RESPONSE_BYTES,
 )
.map_err(HttpClientError::Io)?;

 // Parseresponse
 HttpResponse::parse(&buffer).map_err(HttpClientError::InvalidResponse)
 }

 #[cfg(not(feature = "rustls-tls"))]
 {
 Err(HttpClientError::TlsError(
 "needenabled rustls-tls Features".to_string(),
 ))
 }
}

/// useconnection poolsend HTTPS (HTTP/1.1 over TLS)request
///
/// explain：
/// - this is“connection pool + TLS”syncimplement (面toward `kh.google.com` thisclass https 站point)
/// - itemfrontonly for 回归test and `HttpClient` https+pool path
#[cfg(feature = "connection-pool")]
pub fn send_https_request_with_pool(
 host: &str,
 port: u16,
 path: &str,
 request: &HttpRequest,
 config: &HttpClientConfig,
 pool_manager: &std::sync::Arc<super::pool::ConnectionPoolManager>,
) -> Result<HttpResponse> {
 use std::io::Write;

 let pool = pool_manager.get_pool(host, port)?;
 let conn = pool
.get_tcp()
.map_err(|e| HttpClientError::ConnectionFailed(format!("Failed to get connection from pool: {:?}", e)))?;

 // PooledConnection implement了 Deref<Target = Connection>, candirectlyuse Connection method
 let tcp_stream = conn
.tcp_conn()
.ok_or_else(|| HttpClientError::ConnectionFailed("Expected TCP connection but got UDP".to_string()))?;

 // keep conn lifecyclecoverwholerequest；same when 用 clone get to available std::net::TcpStream
 let tcp_stream = tcp_stream.try_clone().map_err(HttpClientError::Io)?;

 tcp_stream
.set_read_timeout(Some(config.read_timeout))
.map_err(HttpClientError::Io)?;
 tcp_stream
.set_write_timeout(Some(config.write_timeout))
.map_err(HttpClientError::Io)?;

 // rustls path and send_https_request keepconsistent
 #[cfg(feature = "rustls-tls")]
 {
 use rustls::client::ServerName;
 use std::sync::Arc;

 let tls_config = super::rustls_utils::build_client_config(
 config.verify_tls,
 Vec::new(),
 config.profile.as_ref(),
 );
 let server_name = ServerName::try_from(host)
.map_err(|_| HttpClientError::TlsError("Invalid server name".to_string()))?;
 let conn_tls = rustls::ClientConnection::new(Arc::new(tls_config), server_name)
.map_err(|e| HttpClientError::TlsError(format!("TLS connectionCreatefailure: {}", e)))?;

 let mut tls_stream = rustls::StreamOwned::new(conn_tls, tcp_stream);

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

 let header_order = config.profile.as_ref().map(|p| p.header_order.as_slice());
 let http_request = request_with_cookies.build_http1_request_bytes(host, path, header_order);
 tls_stream
.write_all(&http_request)
.map_err(HttpClientError::Io)?;
 tls_stream.flush().map_err(HttpClientError::Io)?;

 let buffer = super::io::read_http1_response_bytes(
 &mut tls_stream,
 super::io::DEFAULT_MAX_RESPONSE_BYTES,
 )
.map_err(HttpClientError::Io)?;

 HttpResponse::parse(&buffer).map_err(HttpClientError::InvalidResponse)
 }

 #[cfg(not(feature = "rustls-tls"))]
 {
 let _ = conn; // keep for symmetry
 Err(HttpClientError::TlsError(
 "needenabled rustls-tls Features".to_string(),
 ))
 }
}

#[cfg(test)]
mod tests {
 use super::*;
 use crate::http_client::request::HttpMethod;

 #[test]
 #[ignore] // neednetworkconnection
 fn test_send_https_request() {
 let request = HttpRequest::new(HttpMethod::Get, "https://httpbin.org/get")
.with_user_agent("TestClient/1.0");

 let config = HttpClientConfig::default();
 let response = send_https_request("httpbin.org", 443, "/get", &request, &config).unwrap();

 // outside部servicemaywillshorttemporaryreturn 429/503 etc.；heremainValidate“canestablish TLS + canParseresponse”. 
 assert!(response.status_code > 0);
 }
}
