//! HTTP 客户端模块
//!
//! 结合 netconnpool + fingerprint-rust 实现完整的 HTTP 客户端
//!
//! 特性：
//! - 使用 netconnpool 管理连接
//! - 应用 fingerprint-rust 的配置
//! - 支持 HTTP/1.1 和 HTTP/2
//! - TLS 层设计为可替换

pub mod cookie;
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

// 修复：使用全局单例 Runtime 避免频繁创建（用于 HTTP/2 和 HTTP/3 连接池场景）
#[cfg(any(feature = "http2", feature = "http3"))]
use once_cell::sync::Lazy;

#[cfg(any(feature = "http2", feature = "http3"))]
static SHARED_RUNTIME: Lazy<tokio::runtime::Runtime> =
    Lazy::new(|| tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime"));

/// HTTP 客户端错误
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
            HttpClientError::Io(e) => write!(f, "IO 错误: {}", e),
            HttpClientError::InvalidUrl(s) => write!(f, "无效的 URL: {}", s),
            HttpClientError::InvalidResponse(s) => write!(f, "无效的响应: {}", s),
            HttpClientError::TlsError(s) => write!(f, "TLS 错误: {}", s),
            HttpClientError::ConnectionFailed(s) => write!(f, "连接失败: {}", s),
            HttpClientError::Timeout => write!(f, "请求超时"),
            #[cfg(feature = "http2")]
            HttpClientError::Http2Error(s) => write!(f, "HTTP/2 错误: {}", s),
            #[cfg(feature = "http3")]
            HttpClientError::Http3Error(s) => write!(f, "HTTP/3 错误: {}", s),
            HttpClientError::InvalidRequest(s) => write!(f, "无效的请求: {}", s),
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

/// HTTP 客户端配置
#[derive(Debug, Clone)]
pub struct HttpClientConfig {
    /// 用户代理
    pub user_agent: String,
    /// HTTP Headers
    pub headers: HTTPHeaders,
    /// 浏览器配置
    pub profile: Option<ClientProfile>,
    /// 连接超时
    pub connect_timeout: Duration,
    /// 读取超时
    pub read_timeout: Duration,
    /// 写入超时
    pub write_timeout: Duration,
    /// 最大重定向次数
    pub max_redirects: usize,
    /// 是否验证 TLS 证书
    pub verify_tls: bool,
    /// 优先使用 HTTP/2
    pub prefer_http2: bool,
    /// 优先使用 HTTP/3
    pub prefer_http3: bool,
    /// Cookie 存储（可选）
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
            prefer_http2: true,  // 默认优先使用 HTTP/2
            prefer_http3: false, // HTTP/3 默认关闭（需要特殊配置）
            cookie_store: None,
        }
    }
}

/// HTTP 客户端
///
/// 使用 netconnpool 管理连接，应用 fingerprint-rust 的配置
pub struct HttpClient {
    config: HttpClientConfig,
    /// 连接池管理器（可选）
    pool_manager: Option<Arc<ConnectionPoolManager>>,
}

use std::sync::Arc;

impl HttpClient {
    /// 创建新的 HTTP 客户端
    pub fn new(config: HttpClientConfig) -> Self {
        Self {
            config,
            pool_manager: None,
        }
    }

    /// 创建带连接池的 HTTP 客户端
    pub fn with_pool(config: HttpClientConfig, pool_config: PoolManagerConfig) -> Self {
        Self {
            config,
            pool_manager: Some(Arc::new(ConnectionPoolManager::new(pool_config))),
        }
    }

    /// 使用浏览器配置创建客户端
    pub fn with_profile(profile: ClientProfile, headers: HTTPHeaders, user_agent: String) -> Self {
        let config = HttpClientConfig {
            profile: Some(profile),
            headers,
            user_agent,
            ..Default::default()
        };
        Self::new(config)
    }

    /// 获取连接池统计信息
    pub fn pool_stats(&self) -> Option<Vec<PoolStats>> {
        self.pool_manager.as_ref().map(|pm| pm.get_stats())
    }

    /// 清理空闲连接
    pub fn cleanup_idle_connections(&self) {
        if let Some(pm) = &self.pool_manager {
            pm.cleanup_idle();
        }
    }

    /// 发送 GET 请求
    pub fn get(&self, url: &str) -> Result<HttpResponse> {
        let request = HttpRequest::new(HttpMethod::Get, url)
            .with_user_agent(&self.config.user_agent)
            .with_headers(&self.config.headers);
        self.send_request(&request)
    }

    /// 发送 POST 请求
    pub fn post(&self, url: &str, body: &[u8]) -> Result<HttpResponse> {
        let request = HttpRequest::new(HttpMethod::Post, url)
            .with_user_agent(&self.config.user_agent)
            .with_headers(&self.config.headers)
            .with_body(body.to_vec());
        self.send_request(&request)
    }

    /// 发送自定义请求（支持重定向）
    pub fn send_request(&self, request: &HttpRequest) -> Result<HttpResponse> {
        self.send_request_with_redirects(request, 0)
    }

    /// 发送请求并处理重定向
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

    /// 内部重定向处理（带循环检测）
    fn send_request_with_redirects_internal(
        &self,
        request: &HttpRequest,
        redirect_count: usize,
        visited_urls: &mut std::collections::HashSet<String>,
    ) -> Result<HttpResponse> {
        // 检查重定向次数
        if redirect_count >= self.config.max_redirects {
            return Err(HttpClientError::InvalidResponse(format!(
                "重定向次数超过限制: {}",
                self.config.max_redirects
            )));
        }

        // 检查重定向循环
        if visited_urls.contains(&request.url) {
            return Err(HttpClientError::InvalidResponse(format!(
                "检测到重定向循环: {}",
                request.url
            )));
        }
        visited_urls.insert(request.url.clone());

        // 解析 URL
        let (scheme, host, port, path) = self.parse_url(&request.url)?;

        // 根据协议选择处理方式
        let response = match scheme.as_str() {
            "http" => self.send_http_request(&host, port, &path, request)?,
            "https" => self.send_https_request(&host, port, &path, request)?,
            _ => {
                return Err(HttpClientError::InvalidUrl(format!(
                    "不支持的协议: {}",
                    scheme
                )));
            }
        };

        // 处理重定向
        if (300..400).contains(&response.status_code) {
            if let Some(location) = response.headers.get("location") {
                // 构建新的 URL（可能是相对路径或绝对路径）
                let redirect_url =
                    if location.starts_with("http://") || location.starts_with("https://") {
                        location.clone()
                    } else if location.starts_with("//") {
                        format!("{}:{}", scheme, location)
                    } else if location.starts_with('/') {
                        format!("{}://{}:{}{}", scheme, host, port, location)
                    } else {
                        // 相对路径
                        let base_path = if path.ends_with('/') {
                            &path
                        } else {
                            path.rsplit_once('/').map(|(p, _)| p).unwrap_or("/")
                        };
                        format!("{}://{}:{}{}{}", scheme, host, port, base_path, location)
                    };

                // 修复：根据 HTTP 状态码正确处理重定向方法（RFC 7231）
                let redirect_method = match response.status_code {
                    301..=303 => {
                        // 301, 302, 303: POST 应该改为 GET，并移除请求体
                        HttpMethod::Get
                    }
                    307 | 308 => {
                        // 307, 308: 保持原 HTTP 方法（POST 仍然是 POST）
                        request.method
                    }
                    _ => {
                        // 其他 3xx 状态码保持原方法
                        request.method
                    }
                };

                // 修复：处理 Set-Cookie（如果重定向响应中有 Cookie）
                if let Some(cookie_store) = &self.config.cookie_store {
                    if let Some(set_cookie) = response.headers.get("set-cookie") {
                        // 解析并添加 Cookie
                        if let Some(cookie) =
                            super::cookie::Cookie::parse_set_cookie(set_cookie, host.clone())
                        {
                            cookie_store.add_cookie(cookie);
                        }
                    }
                }

                // 解析新 URL 的域名和路径（用于 Cookie 域过滤）
                let (new_scheme, new_host, _new_port, new_path) = self.parse_url(&redirect_url)?;

                // 修复：重新构建请求，只包含适用于新域名的 Cookie
                let mut final_redirect_request = HttpRequest::new(redirect_method, &redirect_url);

                // 复制非 Cookie 的 headers，并添加 Referer
                for (key, value) in &request.headers {
                    if key.to_lowercase() != "cookie" {
                        final_redirect_request = final_redirect_request.with_header(key, value);
                    }
                }
                // 修复：添加 Referer 头（模拟浏览器行为）
                final_redirect_request =
                    final_redirect_request.with_header("Referer", &request.url);

                // 添加适用于新域名的 Cookie
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

                // 如果保持 POST/PUT/PATCH，保留请求体；如果改为 GET，移除请求体（RFC 7231 要求）
                if redirect_method != HttpMethod::Get {
                    if let Some(body) = &request.body {
                        final_redirect_request = final_redirect_request.with_body(body.clone());
                    }
                }

                // 递归处理重定向（传递 visited_urls 以检测循环）
                return self.send_request_with_redirects_internal(
                    &final_redirect_request,
                    redirect_count + 1,
                    visited_urls,
                );
            }
        }

        Ok(response)
    }

    /// 解析 URL
    /// 修复：支持 IPv6 地址和正确处理 query/fragment
    fn parse_url(&self, url: &str) -> Result<(String, String, u16, String)> {
        let url = url.trim();

        // 提取 scheme
        let (scheme, rest) = if let Some(stripped) = url.strip_prefix("https://") {
            ("https", stripped)
        } else if let Some(stripped) = url.strip_prefix("http://") {
            ("http", stripped)
        } else {
            return Err(HttpClientError::InvalidUrl("缺少协议".to_string()));
        };

        // 移除 fragment（# 后面的部分）
        let rest = if let Some(frag_pos) = rest.find('#') {
            &rest[..frag_pos]
        } else {
            rest
        };

        // 分离 query 参数（? 后面的部分）和 path
        let (host_port, path_with_query) = if let Some(pos) = rest.find('/') {
            (&rest[..pos], &rest[pos..])
        } else {
            (rest, "/")
        };

        // 提取 path（移除 query 参数，但保留在 path 中发送）
        // 注意：query 参数应该保留在 path 中，因为服务器需要它们
        let path = path_with_query.to_string();

        // 解析 host 和 port
        // 修复：支持 IPv6 地址格式 [2001:db8::1]:8080
        let (host, port) = if host_port.starts_with('[') {
            // IPv6 地址格式
            if let Some(close_bracket) = host_port.find(']') {
                let host = host_port[1..close_bracket].to_string();
                if let Some(colon_pos) = host_port[close_bracket + 1..].find(':') {
                    let port_str = &host_port[close_bracket + 2 + colon_pos..];
                    let port = port_str
                        .parse::<u16>()
                        .map_err(|_| HttpClientError::InvalidUrl("无效的端口".to_string()))?;
                    (host, port)
                } else {
                    let default_port = if scheme == "https" { 443 } else { 80 };
                    (host, default_port)
                }
            } else {
                return Err(HttpClientError::InvalidUrl("IPv6 地址格式错误".to_string()));
            }
        } else {
            // IPv4 地址或域名
            if let Some(pos) = host_port.find(':') {
                let host = host_port[..pos].to_string();
                let port = host_port[pos + 1..]
                    .parse::<u16>()
                    .map_err(|_| HttpClientError::InvalidUrl("无效的端口".to_string()))?;
                (host, port)
            } else {
                let default_port = if scheme == "https" { 443 } else { 80 };
                (host_port.to_string(), default_port)
            }
        };

        Ok((scheme.to_string(), host, port, path))
    }

    /// 发送 HTTP 请求
    fn send_http_request(
        &self,
        host: &str,
        port: u16,
        path: &str,
        request: &HttpRequest,
    ) -> Result<HttpResponse> {
        // 如果有连接池，使用连接池
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
        // 否则使用普通连接
        http1::send_http1_request(host, port, path, request, &self.config)
    }

    /// 发送 HTTPS 请求（支持 HTTP/1.1、HTTP/2、HTTP/3）
    fn send_https_request(
        &self,
        host: &str,
        port: u16,
        path: &str,
        request: &HttpRequest,
    ) -> Result<HttpResponse> {
        // 如果有连接池，优先使用连接池（HTTPS：HTTP/3 > HTTP/2 > HTTP/1.1）
        #[cfg(feature = "connection-pool")]
        if let Some(pool_manager) = &self.pool_manager {
            // HTTP/3 with pool（异步 -> 同步包装）
            #[cfg(feature = "http3")]
            if self.config.prefer_http3 {
                // 修复：使用全局单例 Runtime
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

            // HTTP/2 with pool（异步 -> 同步包装）
            #[cfg(feature = "http2")]
            if self.config.prefer_http2 {
                // 修复：使用全局单例 Runtime
                // 注意：这里不做"自动降级"，因为 pool 场景我们更希望按用户偏好走指定协议
                //（测试里也会严格验证版本）
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

        // 优先级：HTTP/3 > HTTP/2 > HTTP/1.1

        // 尝试 HTTP/3
        #[cfg(feature = "http3")]
        {
            if self.config.prefer_http3 {
                // 如果开启了 HTTP/3，我们尝试它。
                // 如果失败，我们可能希望降级，但 HTTP/3 到 TCP 是不同的传输层，
                // 通常如果用户明确要求 HTTP/3，失败就应该报错。
                // 但这里为了稳健性，如果是因为协议错误，我们可以降级。
                // 暂时保持简单：直接返回。
                match http3::send_http3_request(host, port, path, request, &self.config) {
                    Ok(resp) => return Ok(resp),
                    Err(e) => {
                        // 如果仅仅是偏好，可以尝试降级
                        // 如果是连接失败，可能是网络问题，也可能是服务器不支持
                        eprintln!("警告: HTTP/3 失败，尝试降级: {}", e);
                    }
                }
            }
        }

        // 尝试 HTTP/2
        #[cfg(feature = "http2")]
        {
            if self.config.prefer_http2 {
                match http2::send_http2_request(host, port, path, request, &self.config) {
                    Ok(resp) => return Ok(resp),
                    Err(_e) => {
                        // 记录错误但继续尝试 HTTP/1.1
                        // 在实际生产中应该使用日志系统
                        // eprintln!("HTTP/2 尝试失败: {}，回退到 HTTP/1.1", e);
                    }
                }
            }
        }

        // 回退到 HTTP/1.1 + TLS
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
