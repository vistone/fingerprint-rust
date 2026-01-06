//! HTTP client module
//!
//! Complete HTTP client implementation combining netconnpool + fingerprint-rust
//!
//! Features:
//! - Use netconnpool for connection management
//! - Apply fingerprint-rust configurations
//! - Support HTTP/1.1 and HTTP/2
//! - TLS layer designed to be replaceable

pub mod cookie;
#[cfg(all(feature = "connection-pool", feature = "http2"))]
mod h2_session_pool;
#[cfg(all(feature = "connection-pool", feature = "http3"))]
mod h3_session_pool;
pub mod http1;
pub mod http1_pool;
pub mod http2;
#[cfg(all(feature = "connection-pool", feature = "http2"))]
pub mod http2_pool;
pub mod http3;
#[cfg(all(feature = "connection-pool", feature = "http3"))]
pub mod http3_pool;
pub mod io;
pub mod pool;
pub mod proxy;
pub mod reporter;
pub mod request;
pub mod response;
#[cfg(feature = "rustls-client-hello-customizer")]
mod rustls_client_hello_customizer;
#[cfg(any(feature = "rustls-tls", feature = "http2", feature = "http3"))]
mod rustls_utils;
pub mod tcp_fingerprint;
pub mod tls;

pub use cookie::{Cookie, CookieStore, SameSite};
pub use pool::{ConnectionPoolManager, PoolManagerConfig, PoolStats};
pub use proxy::{ProxyConfig, ProxyType};
pub use reporter::{ReportFormat, ReportSection, ValidationReport};
pub use request::{HttpMethod, HttpRequest};
pub use response::HttpResponse;
pub use tls::TlsConnector;

use fingerprint_headers::headers::HTTPHeaders;
use fingerprint_profiles::profiles::ClientProfile;
use std::io as std_io;
use std::time::Duration;

// Fix: useglobalsingleton Runtime avoidfrequentCreate ( for HTTP/2 and HTTP/3 connection poolscenario)
// Note: only in connection-pool enabled when 才need, becauseonlyconnection poolscenario才needsyncwrapasynccode
#[cfg(all(feature = "connection-pool", any(feature = "http2", feature = "http3")))]
use once_cell::sync::Lazy;

#[cfg(all(feature = "connection-pool", any(feature = "http2", feature = "http3")))]
static SHARED_RUNTIME: Lazy<tokio::runtime::Runtime> =
    Lazy::new(|| tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime"));

/// HTTP clienterror
#[derive(Debug)]
pub enum HttpClientError {
    Io(std_io::Error),
    InvalidUrl(String),
    InvalidResponse(String),
    TlsError(String),
    ConnectionFailed(String),
    Timeout,
    #[cfg(feature = "http2")]
    Http2Error(String),
    #[cfg(feature = "http3")]
    Http3Error(String),
    InvalidRequest(String),
}

impl std::fmt::Display for HttpClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HttpClientError::Io(e) => write!(f, "IO error: {}", e),
            HttpClientError::InvalidUrl(s) => write!(f, "Invalid URL: {}", s),
            HttpClientError::InvalidResponse(s) => write!(f, "Invalid response: {}", s),
            HttpClientError::TlsError(s) => write!(f, "TLS error: {}", s),
            HttpClientError::ConnectionFailed(s) => write!(f, "Connection failed: {}", s),
            HttpClientError::Timeout => write!(f, "Request timeout"),
            #[cfg(feature = "http2")]
            HttpClientError::Http2Error(s) => write!(f, "HTTP/2 error: {}", s),
            #[cfg(feature = "http3")]
            HttpClientError::Http3Error(s) => write!(f, "HTTP/3 error: {}", s),
            HttpClientError::InvalidRequest(s) => write!(f, "Invalid request: {}", s),
        }
    }
}

impl std::error::Error for HttpClientError {}

impl From<std_io::Error> for HttpClientError {
    fn from(err: std_io::Error) -> Self {
        HttpClientError::Io(err)
    }
}

pub type Result<T> = std::result::Result<T, HttpClientError>;

/// HTTP clientconfiguration
#[derive(Debug, Clone)]
pub struct HttpClientConfig {
    /// userproxy
    pub user_agent: String,
    /// HTTP Headers
    pub headers: HTTPHeaders,
    /// browserconfiguration
    pub profile: Option<ClientProfile>,
    /// connectiontimeout
    pub connect_timeout: Duration,
    /// readtimeout
    pub read_timeout: Duration,
    /// writetimeout
    pub write_timeout: Duration,
    /// maximumredirecttimecount
    pub max_redirects: usize,
    /// whetherValidate TLS certificate
    pub verify_tls: bool,
    /// priorityuse HTTP/2
    pub prefer_http2: bool,
    /// priorityuse HTTP/3
    pub prefer_http3: bool,
    /// Cookie store (optional)
    pub cookie_store: Option<Arc<CookieStore>>,
}

impl Default for HttpClientConfig {
    fn default() -> Self {
        Self {
            user_agent: "Mozilla/5.0".to_string(),
            headers: HTTPHeaders::default(),
            profile: None,
            connect_timeout: Duration::from_secs(30),
            read_timeout: Duration::from_secs(30),
            write_timeout: Duration::from_secs(30),
            max_redirects: 10,
            verify_tls: true,
            prefer_http2: true,  // defaultpriorityuse HTTP/2
            prefer_http3: false, // HTTP/3 defaultclose (needspecialconfiguration)
            cookie_store: None,
        }
    }
}

/// HTTP client
///
/// use netconnpool manageconnection, application fingerprint-rust configuration
pub struct HttpClient {
    config: HttpClientConfig,
    /// connection poolmanageer (optional)
    pool_manager: Option<Arc<ConnectionPoolManager>>,
}

use std::sync::Arc;

impl HttpClient {
    /// Create a new HTTP client
    pub fn new(config: HttpClientConfig) -> Self {
        Self {
            config,
            pool_manager: None,
        }
    }

    /// Createbringconnection pool HTTP client
    pub fn with_pool(config: HttpClientConfig, pool_config: PoolManagerConfig) -> Self {
        Self {
            config,
            pool_manager: Some(Arc::new(ConnectionPoolManager::new(pool_config))),
        }
    }

    /// usebrowserconfigurationCreateclient
    pub fn with_profile(profile: ClientProfile, headers: HTTPHeaders, user_agent: String) -> Self {
        let config = HttpClientConfig {
            profile: Some(profile),
            headers,
            user_agent,
            ..Default::default()
        };
        Self::new(config)
    }

    /// Getconnection poolstatisticsinfo
    pub fn pool_stats(&self) -> Option<Vec<PoolStats>> {
        self.pool_manager.as_ref().map(|pm| pm.get_stats())
    }

    /// cleanupempty闲connection
    pub fn cleanup_idle_connections(&self) {
        if let Some(pm) = &self.pool_manager {
            pm.cleanup_idle();
        }
    }

    /// send GET request
    pub fn get(&self, url: &str) -> Result<HttpResponse> {
        let request = HttpRequest::new(HttpMethod::Get, url)
            .with_user_agent(&self.config.user_agent)
            .with_headers(&self.config.headers);
        self.send_request(&request)
    }

    /// send POST request
    pub fn post(&self, url: &str, body: &[u8]) -> Result<HttpResponse> {
        let request = HttpRequest::new(HttpMethod::Post, url)
            .with_user_agent(&self.config.user_agent)
            .with_headers(&self.config.headers)
            .with_body(body.to_vec());
        self.send_request(&request)
    }

    /// sendcustomrequest (supportredirect)
    pub fn send_request(&self, request: &HttpRequest) -> Result<HttpResponse> {
        self.send_request_with_redirects(request, 0)
    }

    /// sendrequest并processredirect
    fn send_request_with_redirects(
        &self,
        request: &HttpRequest,
        redirect_count: usize,
    ) -> Result<HttpResponse> {
        self.send_request_with_redirects_internal(
            request,
            redirect_count,
            &mut std::collections::HashSet::new(),
        )
    }

    /// inside部redirectprocess (bringloopdetect)
    fn send_request_with_redirects_internal(
        &self,
        request: &HttpRequest,
        redirect_count: usize,
        visited_urls: &mut std::collections::HashSet<String>,
    ) -> Result<HttpResponse> {
        // Checkredirecttimecount
        if redirect_count >= self.config.max_redirects {
            return Err(HttpClientError::InvalidResponse(format!(
                "redirect次countexceedlimit: {}",
                self.config.max_redirects
            )));
        }

        // Checkredirectloop
        if visited_urls.contains(&request.url) {
            return Err(HttpClientError::InvalidResponse(format!(
                "detect to redirectloop: {}",
                request.url
            )));
        }
        visited_urls.insert(request.url.clone());

        // Parse URL
        let (scheme, host, port, path) = self.parse_url(&request.url)?;

        // Based onprotocolselectprocessmethod
        let response = match scheme.as_str() {
            "http" => self.send_http_request(&host, port, &path, request)?,
            "https" => self.send_https_request(&host, port, &path, request)?,
            _ => {
                return Err(HttpClientError::InvalidUrl(format!(
                    "不supportprotocol: {}",
                    scheme
                )));
            }
        };

        // processredirect
        if (300..400).contains(&response.status_code) {
            if let Some(location) = response.headers.get("location") {
                // Buildnew URL (may is mutualpairpath or 绝pairpath)
                let redirect_url =
                    if location.starts_with("http://") || location.starts_with("https://") {
                        location.clone()
                    } else if location.starts_with("//") {
                        format!("{}:{}", scheme, location)
                    } else if location.starts_with('/') {
                        format!("{}://{}:{}{}", scheme, host, port, location)
                    } else {
                        // mutualpairpath
                        // Fix: correctprocesspathconcatenate, avoiddouble slash
                        let base_path = if path.ends_with('/') {
                            &path
                        } else {
                            path.rsplit_once('/').map(|(p, _)| p).unwrap_or("/")
                        };
                        // ensure base_path 以 / ending, location not / openheader
                        let location = location.trim_start_matches('/');
                        if base_path == "/" {
                            format!("{}://{}:{}/{}", scheme, host, port, location)
                        } else {
                            format!("{}://{}:{}{}/{}", scheme, host, port, base_path, location)
                        }
                    };

                // Fix: Based on HTTP status codecorrectprocessredirectmethod (RFC 7231)
                let redirect_method = match response.status_code {
                    301..=303 => {
                        // 301, 302, 303: POST shouldchange as GET, 并removerequest体
                        HttpMethod::Get
                    }
                    307 | 308 => {
                        // 307, 308: keeporiginal HTTP method (POST still is POST)
                        request.method
                    }
                    _ => {
                        // other 3xx status codekeeporiginalmethod
                        request.method
                    }
                };

                // Fix: process Set-Cookie ( if redirectresponse in 有 Cookie)
                if let Some(cookie_store) = &self.config.cookie_store {
                    if let Some(set_cookie) = response.headers.get("set-cookie") {
                        // Parse并Add Cookie
                        if let Some(cookie) =
                            super::cookie::Cookie::parse_set_cookie(set_cookie, host.clone())
                        {
                            cookie_store.add_cookie(cookie);
                        }
                    }
                }

                // Parsenew URL domain and path ( for Cookie fieldfilter)
                let (new_scheme, new_host, _new_port, new_path) = self.parse_url(&redirect_url)?;

                // Fix: reBuildrequest, onlyincludingsuitable for newdomain Cookie
                let mut final_redirect_request = HttpRequest::new(redirect_method, &redirect_url);

                // copynon Cookie headers, 并Add Referer
                for (key, value) in &request.headers {
                    if key.to_lowercase() != "cookie" {
                        final_redirect_request = final_redirect_request.with_header(key, value);
                    }
                }
                // Fix: Add Referer header (simulatebrowserbehavior)
                final_redirect_request =
                    final_redirect_request.with_header("Referer", &request.url);

                // Addsuitable for newdomain Cookie
                if let Some(cookie_store) = &self.config.cookie_store {
                    if let Some(cookie_header) = cookie_store.generate_cookie_header(
                        &new_host,
                        &new_path,
                        new_scheme == "https",
                    ) {
                        final_redirect_request =
                            final_redirect_request.with_header("Cookie", &cookie_header);
                    }
                }

                // Ifkeep POST/PUT/PATCH, preserverequest体； if change as GET, removerequest体 (RFC 7231 require)
                if redirect_method != HttpMethod::Get {
                    if let Some(body) = &request.body {
                        final_redirect_request = final_redirect_request.with_body(body.clone());
                    }
                }

                // recursiveprocessredirect (pass visited_urls 以detectloop)
                return self.send_request_with_redirects_internal(
                    &final_redirect_request,
                    redirect_count + 1,
                    visited_urls,
                );
            }
        }

        Ok(response)
    }

    /// Parse URL
    /// Fix: support IPv6 address and correctprocess query/fragment
    fn parse_url(&self, url: &str) -> Result<(String, String, u16, String)> {
        let url = url.trim();

        // Extract scheme
        let (scheme, rest) = if let Some(stripped) = url.strip_prefix("https://") {
            ("https", stripped)
        } else if let Some(stripped) = url.strip_prefix("http://") {
            ("http", stripped)
        } else {
            return Err(HttpClientError::InvalidUrl("missingprotocol".to_string()));
        };

        // remove fragment (# back面partial)
        let rest = if let Some(frag_pos) = rest.find('#') {
            &rest[..frag_pos]
        } else {
            rest
        };

        // separate query parameter (? back面partial) and path
        let (host_port, path_with_query) = if let Some(pos) = rest.find('/') {
            (&rest[..pos], &rest[pos..])
        } else {
            (rest, "/")
        };

        // Extract path (remove query parameter, butpreserve in path in send)
        // Note: query parametershouldpreserve in path in , becauseserverneedthem
        let path = path_with_query.to_string();

        // Parse host and port
        // Fix: support IPv6 addressformat [2001:db8::1]:8080
        let (host, port) = if host_port.starts_with('[') {
            // IPv6 addressformat
            if let Some(close_bracket) = host_port.find(']') {
                let host = host_port[1..close_bracket].to_string();
                if let Some(colon_pos) = host_port[close_bracket + 1..].find(':') {
                    let port_str = &host_port[close_bracket + 2 + colon_pos..];
                    let port = port_str
                        .parse::<u16>()
                        .map_err(|_| HttpClientError::InvalidUrl("invalidport".to_string()))?;
                    (host, port)
                } else {
                    let default_port = if scheme == "https" { 443 } else { 80 };
                    (host, default_port)
                }
            } else {
                return Err(HttpClientError::InvalidUrl(
                    "IPv6 addressformaterror".to_string(),
                ));
            }
        } else {
            // IPv4 address or domain
            if let Some(pos) = host_port.find(':') {
                let host = host_port[..pos].to_string();
                let port = host_port[pos + 1..]
                    .parse::<u16>()
                    .map_err(|_| HttpClientError::InvalidUrl("invalidport".to_string()))?;
                (host, port)
            } else {
                let default_port = if scheme == "https" { 443 } else { 80 };
                (host_port.to_string(), default_port)
            }
        };

        Ok((scheme.to_string(), host, port, path))
    }

    /// send HTTP request
    fn send_http_request(
        &self,
        host: &str,
        port: u16,
        path: &str,
        request: &HttpRequest,
    ) -> Result<HttpResponse> {
        // If有connection pool, useconnection pool
        #[cfg(feature = "connection-pool")]
        {
            if let Some(pool_manager) = &self.pool_manager {
                return http1_pool::send_http1_request_with_pool(
                    host,
                    port,
                    path,
                    request,
                    &self.config,
                    pool_manager,
                );
            }
        }
        // otherwiseuseordinaryconnection
        http1::send_http1_request(host, port, path, request, &self.config)
    }

    /// send HTTPS request (support HTTP/1.1, HTTP/2, HTTP/3)
    fn send_https_request(
        &self,
        host: &str,
        port: u16,
        path: &str,
        request: &HttpRequest,
    ) -> Result<HttpResponse> {
        // If有connection pool, priorityuseconnection pool (HTTPS：HTTP/3 > HTTP/2 > HTTP/1.1)
        #[cfg(feature = "connection-pool")]
        if let Some(pool_manager) = &self.pool_manager {
            // HTTP/3 with pool (async -> syncwrap)
            #[cfg(feature = "http3")]
            if self.config.prefer_http3 {
                // Fix: useglobalsingleton Runtime
                return SHARED_RUNTIME.block_on(async {
                    http3_pool::send_http3_request_with_pool(
                        host,
                        port,
                        path,
                        request,
                        &self.config,
                        pool_manager,
                    )
                    .await
                });
            }

            // HTTP/2 with pool (async -> syncwrap)
            #[cfg(feature = "http2")]
            if self.config.prefer_http2 {
                // Fix: useglobalsingleton Runtime
                // Note: herenot do"automatic降level", because pool scenariowemore希望by userpreference走specifiedprotocol
                // (test里alsowillstrictValidateversion)
                return SHARED_RUNTIME.block_on(async {
                    http2_pool::send_http2_request_with_pool(
                        host,
                        port,
                        path,
                        request,
                        &self.config,
                        pool_manager,
                    )
                    .await
                });
            }

            // HTTP/1.1 over TLS with pool
            return tls::send_https_request_with_pool(
                host,
                port,
                path,
                request,
                &self.config,
                pool_manager,
            );
        }

        // priority：HTTP/3 > HTTP/2 > HTTP/1.1

        // try HTTP/3
        #[cfg(feature = "http3")]
        {
            if self.config.prefer_http3 {
                // Ifopen了 HTTP/3, wetry它.
                // Iffailure, wemaywant to reducelevel, but HTTP/3 to TCP is differenttransferlayer,
                // usually if userexplicitrequire HTTP/3, failurethenshouldreport error.
                // butherein order tostable健property,  if is becauseprotocolerror, wecan降level.
                // temporary when keepsimple：directlyreturn.
                match http3::send_http3_request(host, port, path, request, &self.config) {
                    Ok(resp) => return Ok(resp),
                    Err(e) => {
                        // Ifonlyonly is preference, cantry降level
                        // If is Connection failed, may is networkissue, alsomay is server不support
                        eprintln!("warning: HTTP/3 failure，try降level: {}", e);
                    }
                }
            }
        }

        // try HTTP/2
        #[cfg(feature = "http2")]
        {
            if self.config.prefer_http2 {
                match http2::send_http2_request(host, port, path, request, &self.config) {
                    Ok(resp) => return Ok(resp),
                    Err(_e) => {
                        // recorderrorbutcontinuetry HTTP/1.1
                        // in actualproduction in shoulduselogsystem
                        // eprintln!("HTTP/2 tryfailure: {}, back to HTTP/1.1", e);
                    }
                }
            }
        }

        // back to HTTP/1.1 + TLS
        tls::send_https_request(host, port, path, request, &self.config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_url() {
        let client = HttpClient::new(HttpClientConfig::default());

        let (scheme, host, port, path) = client.parse_url("https://example.com/path").unwrap();
        assert_eq!(scheme, "https");
        assert_eq!(host, "example.com");
        assert_eq!(port, 443);
        assert_eq!(path, "/path");

        let (scheme, host, port, path) = client.parse_url("http://example.com:8080/api").unwrap();
        assert_eq!(scheme, "http");
        assert_eq!(host, "example.com");
        assert_eq!(port, 8080);
        assert_eq!(path, "/api");
    }
}
