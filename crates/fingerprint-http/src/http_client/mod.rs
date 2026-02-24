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
pub mod dns_helper;
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
pub use dns_helper::DNSHelper;
pub use pool::{ConnectionPoolManager, PoolManagerConfig, PoolStats};
pub use proxy::{ProxyConfig, ProxyType};
pub use reporter::{ReportFormat, ReportSection, ValidationReport};
pub use request::{HttpMethod, HttpRequest};
pub use response::HttpResponse;
pub use tls::TlsConnector;

use fingerprint_headers::headers::HTTPHeaders;
use fingerprint_profiles::BrowserProfile;
use std::io as std_io;
use std::time::Duration;

// Fix: use global singleton Runtime avoid frequent Create (for HTTP/2 and HTTP/3 connection pool scenario)
// Note: only in connection-pool enabled when need, because only connection pool scenario needs sync wrap async code
#[cfg(all(feature = "connection-pool", any(feature = "http2", feature = "http3")))]
use once_cell::sync::Lazy;

#[cfg(all(feature = "connection-pool", any(feature = "http2", feature = "http3")))]
static SHARED_RUNTIME: Lazy<Result<tokio::runtime::Runtime>> = Lazy::new(|| {
    tokio::runtime::Runtime::new().map_err(|e| {
        HttpClientError::ConnectionFailed(format!("Failed to create Tokio runtime: {}", e))
    })
});

#[cfg(all(feature = "connection-pool", any(feature = "http2", feature = "http3")))]
fn get_shared_runtime() -> Result<&'static tokio::runtime::Runtime> {
    SHARED_RUNTIME.as_ref().map_err(|e| match e {
        HttpClientError::ConnectionFailed(msg) => HttpClientError::ConnectionFailed(msg.clone()),
        _ => HttpClientError::ConnectionFailed("Runtime initialization failed".to_string()),
    })
}

/// HTTP client error
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

/// HTTP client configuration
#[derive(Debug)]
pub struct HttpClientConfig {
    /// User agent
    pub user_agent: String,
    /// HTTP Headers
    pub headers: HTTPHeaders,
    /// Browser configuration
    pub profile: Option<BrowserProfile>,
    /// Connection timeout
    pub connect_timeout: Duration,
    /// Read timeout
    pub read_timeout: Duration,
    /// Write timeout
    pub write_timeout: Duration,
    /// Maximum redirect times count
    pub max_redirects: usize,
    /// Whether validate TLS certificate
    pub verify_tls: bool,
    /// Priority use HTTP/2
    pub prefer_http2: bool,
    /// Priority use HTTP/3
    pub prefer_http3: bool,
    /// Cookie store (optional)
    pub cookie_store: Option<Arc<CookieStore>>,
    /// DNS helper (optional, for DNS cache and pre-parse)
    pub dns_helper: Option<Arc<DNSHelper>>,
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
            prefer_http2: true,  // Default priority use HTTP/2
            prefer_http3: false, // HTTP/3 default close (need special configuration)
            cookie_store: None,
            dns_helper: None, // DNS helper default close (optional functionality)
        }
    }
}

/// HTTP client
///
/// Use netconnpool manage connection, application fingerprint-rust configuration
pub struct HttpClient {
    config: HttpClientConfig,
    /// Connection pool manager (optional)
    #[allow(clippy::arc_with_non_send_sync)]
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

    /// Create bring connection pool HTTP client
    #[allow(clippy::arc_with_non_send_sync)]
    pub fn with_pool(config: HttpClientConfig, pool_config: PoolManagerConfig) -> Self {
        Self {
            config,
            pool_manager: Some(Arc::new(ConnectionPoolManager::new(pool_config))),
        }
    }

    /// Use browser configuration create client
    pub fn with_profile(profile: BrowserProfile, headers: HTTPHeaders, user_agent: String) -> Self {
        let config = HttpClientConfig {
            profile: Some(profile),
            headers,
            user_agent,
            ..Default::default()
        };
        Self::new(config)
    }

    /// Get connection pool statistics info
    pub fn pool_stats(&self) -> Option<Vec<PoolStats>> {
        self.pool_manager.as_ref().map(|pm| pm.get_stats())
    }

    /// Cleanup empty idle connection
    pub fn cleanup_idle_connections(&self) {
        if let Some(pm) = &self.pool_manager {
            pm.cleanup_idle();
        }
    }

    /// Send GET request
    pub fn get(&self, url: &str) -> Result<HttpResponse> {
        let request = HttpRequest::new(HttpMethod::Get, url)
            .with_user_agent(&self.config.user_agent)
            .with_headers(&self.config.headers);
        self.send_request(&request)
    }

    /// Send POST request
    pub fn post(&self, url: &str, body: &[u8]) -> Result<HttpResponse> {
        let request = HttpRequest::new(HttpMethod::Post, url)
            .with_user_agent(&self.config.user_agent)
            .with_headers(&self.config.headers)
            .with_body(body.to_vec());
        self.send_request(&request)
    }

    /// Send custom request (support redirect)
    pub fn send_request(&self, request: &HttpRequest) -> Result<HttpResponse> {
        use std::time::Instant;
        let request_start = Instant::now();
        self.send_request_with_redirects(request, 0, request_start)
    }

    /// Send request and process redirect
    fn send_request_with_redirects(
        &self,
        request: &HttpRequest,
        redirect_count: usize,
        request_start: std::time::Instant,
    ) -> Result<HttpResponse> {
        self.send_request_with_redirects_internal(
            request,
            redirect_count,
            &mut std::collections::HashSet::new(),
            request_start,
        )
    }

    /// Inside redirect process (bring loop detect and cumulative timeout protection)
    fn send_request_with_redirects_internal(
        &self,
        request: &HttpRequest,
        redirect_count: usize,
        visited_urls: &mut std::collections::HashSet<String>,
        request_start: std::time::Instant,
    ) -> Result<HttpResponse> {
        // Check cumulative timeout (5 minutes maximum for entire request including redirects)
        if request_start.elapsed() > Duration::from_secs(300) {
            return Err(HttpClientError::Timeout);
        }

        // Check redirect times count
        if redirect_count >= self.config.max_redirects {
            return Err(HttpClientError::InvalidResponse(format!(
                "Redirect count exceed limit: {}",
                self.config.max_redirects
            )));
        }

        // Check redirect loop
        if visited_urls.contains(&request.url) {
            return Err(HttpClientError::InvalidResponse(format!(
                "Detect redirect loop: {}",
                request.url
            )));
        }
        visited_urls.insert(request.url.clone());

        // Parse URL
        let (scheme, host, port, path) = self.parse_url(&request.url)?;

        // Based on protocol select process method
        let response = match scheme.as_str() {
            "http" => self.send_http_request(&host, port, &path, request)?,
            "https" => self.send_https_request(&host, port, &path, request)?,
            _ => {
                return Err(HttpClientError::InvalidUrl(format!(
                    "Not support protocol: {}",
                    scheme
                )));
            }
        };

        // Process redirect
        if (300..400).contains(&response.status_code) {
            if let Some(location) = response.headers.get("location") {
                // Build new URL (may is absolute path or relative path)
                let redirect_url =
                    if location.starts_with("http://") || location.starts_with("https://") {
                        location.clone()
                    } else if location.starts_with("//") {
                        format!("{}:{}", scheme, location)
                    } else if location.starts_with('/') {
                        format!("{}://{}:{}{}", scheme, host, port, location)
                    } else {
                        // Relative path
                        // Fix: correct process path concatenate, avoid double slash
                        let base_path = if path.ends_with('/') {
                            &path
                        } else {
                            path.rsplit_once('/').map(|(p, _)| p).unwrap_or("/")
                        };
                        // Ensure base_path ending with / ending, location not / header
                        let location = location.trim_start_matches('/');
                        if base_path == "/" {
                            format!("{}://{}:{}/{}", scheme, host, port, location)
                        } else {
                            format!("{}://{}:{}{}/{}", scheme, host, port, base_path, location)
                        }
                    };

                // Fix: Based on HTTP status code correct process redirect method (RFC 7231)
                let redirect_method = match response.status_code {
                    301..=303 => {
                        // 301, 302, 303: POST should change as GET, and remove request body
                        HttpMethod::Get
                    }
                    307 | 308 => {
                        // 307, 308: keep original HTTP method (POST still is POST)
                        request.method
                    }
                    _ => {
                        // Other 3xx status code keep original method
                        request.method
                    }
                };

                // Fix: process Set-Cookie (if redirect response has Cookie)
                if let Some(cookie_store) = &self.config.cookie_store {
                    if let Some(set_cookie) = response.headers.get("set-cookie") {
                        // Parse and add Cookie
                        if let Some(cookie) =
                            super::cookie::Cookie::parse_set_cookie(set_cookie, host.clone())
                        {
                            cookie_store.add_cookie(cookie);
                        }
                    }
                }

                // Parse new URL domain and path (for Cookie field filter)
                let (new_scheme, new_host, _new_port, new_path) = self.parse_url(&redirect_url)?;

                // Fix: rebuild request, only including suitable for new domain Cookie
                let mut final_redirect_request = HttpRequest::new(redirect_method, &redirect_url);

                // Copy non Cookie headers, and add Referer
                for (key, value) in &request.headers {
                    if key.to_lowercase() != "cookie" {
                        final_redirect_request = final_redirect_request.with_header(key, value);
                    }
                }
                // Fix: add Referer header (simulate browser behavior)
                final_redirect_request =
                    final_redirect_request.with_header("Referer", &request.url);

                // Add suitable for new domain Cookie
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

                // If keep POST/PUT/PATCH, preserve request body; if change as GET, remove request body (RFC 7231 require)
                if redirect_method != HttpMethod::Get {
                    if let Some(body) = &request.body {
                        final_redirect_request = final_redirect_request.with_body(body.clone());
                    }
                }

                // Recursive process redirect (pass visited_urls end with detect loop)
                return self.send_request_with_redirects_internal(
                    &final_redirect_request,
                    redirect_count + 1,
                    visited_urls,
                    request_start,
                );
            }
        }

        Ok(response)
    }

    /// Parse URL
    /// Fix: support IPv6 address and correct process query/fragment
    fn parse_url(&self, url: &str) -> Result<(String, String, u16, String)> {
        let url = url.trim();

        // Extract scheme
        let (scheme, rest) = if let Some(stripped) = url.strip_prefix("https://") {
            ("https", stripped)
        } else if let Some(stripped) = url.strip_prefix("http://") {
            ("http", stripped)
        } else {
            return Err(HttpClientError::InvalidUrl("Missing protocol".to_string()));
        };

        // Remove fragment (#后面部分)
        let rest = if let Some(frag_pos) = rest.find('#') {
            &rest[..frag_pos]
        } else {
            rest
        };

        // Separate query parameter (?后面部分) and path
        let (host_port, path_with_query) = if let Some(pos) = rest.find('/') {
            (&rest[..pos], &rest[pos..])
        } else {
            (rest, "/")
        };

        // Extract path (remove query parameter, but preserve in path in send)
        // Note: query parameter should preserve in path in, because server need them
        let path = path_with_query.to_string();

        // Parse host and port
        // Fix: support IPv6 address format [2001:db8::1]:8080
        let (host, port) = if host_port.starts_with('[') {
            // IPv6 address format
            if let Some(close_bracket) = host_port.find(']') {
                let host = host_port[1..close_bracket].to_string();
                if let Some(colon_pos) = host_port[close_bracket + 1..].find(':') {
                    let port_str = &host_port[close_bracket + 2 + colon_pos..];
                    let port = port_str
                        .parse::<u16>()
                        .map_err(|_| HttpClientError::InvalidUrl("Invalid port".to_string()))?;
                    (host, port)
                } else {
                    let default_port = if scheme == "https" { 443 } else { 80 };
                    (host, default_port)
                }
            } else {
                return Err(HttpClientError::InvalidUrl(
                    "IPv6 address format error".to_string(),
                ));
            }
        } else {
            // IPv4 address or domain
            if let Some(pos) = host_port.find(':') {
                let host = host_port[..pos].to_string();
                let port = host_port[pos + 1..]
                    .parse::<u16>()
                    .map_err(|_| HttpClientError::InvalidUrl("Invalid port".to_string()))?;
                (host, port)
            } else {
                let default_port = if scheme == "https" { 443 } else { 80 };
                (host_port.to_string(), default_port)
            }
        };

        Ok((scheme.to_string(), host, port, path))
    }

    /// Send HTTP request
    fn send_http_request(
        &self,
        host: &str,
        port: u16,
        path: &str,
        request: &HttpRequest,
    ) -> Result<HttpResponse> {
        // If has connection pool, use connection pool
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
        // Otherwise use ordinary connection
        http1::send_http1_request(host, port, path, request, &self.config)
    }

    /// Send HTTPS request (support HTTP/1.1, HTTP/2, HTTP/3)
    fn send_https_request(
        &self,
        host: &str,
        port: u16,
        path: &str,
        request: &HttpRequest,
    ) -> Result<HttpResponse> {
        // If has connection pool, priority use connection pool (HTTPS: HTTP/3 > HTTP/2 > HTTP/1.1)
        #[cfg(feature = "connection-pool")]
        if let Some(pool_manager) = &self.pool_manager {
            // HTTP/3 with pool (async -> sync wrap)
            #[cfg(feature = "http3")]
            if self.config.prefer_http3 {
                // Fix: use global singleton Runtime
                let runtime = get_shared_runtime()?;
                return runtime.block_on(async {
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

            // HTTP/2 with pool (async -> sync wrap)
            #[cfg(feature = "http2")]
            if self.config.prefer_http2 {
                // Fix: use global singleton Runtime
                // Note: here not do "automatic downgrade", because pool scenario we more hope by user preference go specified protocol
                // (test also will strict validate version)
                let runtime = get_shared_runtime()?;
                return runtime.block_on(async {
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

        // Priority: HTTP/3 > HTTP/2 > HTTP/1.1

        // Try HTTP/3
        #[cfg(feature = "http3")]
        {
            if self.config.prefer_http3 {
                // If opened HTTP/3, we try it.
                // If failure, we may want to downgrade, but HTTP/3 to TCP is different transfer layer,
                // Usually if user explicit require HTTP/3, failure then should report error.
                // But here in order to stable property, if is because protocol error, we can downgrade.
                // Temporary when keep simple: directly return.
                match http3::send_http3_request(host, port, path, request, &self.config) {
                    Ok(resp) => return Ok(resp),
                    Err(e) => {
                        // If only only is preference, can try downgrade
                        // If is Connection failed, may is network issue, also may is server not support
                        eprintln!("Warning: HTTP/3 failure, try downgrade: {}", e);
                    }
                }
            }
        }

        // Try HTTP/2
        #[cfg(feature = "http2")]
        {
            if self.config.prefer_http2 {
                match http2::send_http2_request(host, port, path, request, &self.config) {
                    Ok(resp) => return Ok(resp),
                    Err(_e) => {
                        // Record error but continue try HTTP/1.1
                        // In actual production should use log system
                        // eprintln!("HTTP/2 try failure: {}, back to HTTP/1.1", e);
                    }
                }
            }
        }

        // Back to HTTP/1.1 + TLS
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
