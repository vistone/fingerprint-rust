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
pub mod http2;
pub mod http3;
pub mod pool;
pub mod proxy;
pub mod reporter;
pub mod request;
pub mod response;
pub mod tls;

pub use cookie::{Cookie, CookieStore, SameSite};
pub use pool::{ConnectionPoolManager, PoolManagerConfig, PoolStats};
pub use proxy::{ProxyConfig, ProxyType};
pub use reporter::{ReportFormat, ReportSection, ValidationReport};
pub use request::{HttpMethod, HttpRequest};
pub use response::HttpResponse;
pub use tls::TlsConnector;

use crate::{ClientProfile, HTTPHeaders};
use std::io;
use std::time::Duration;

/// HTTP 客户端错误
#[derive(Debug)]
pub enum HttpClientError {
    Io(io::Error),
    InvalidUrl(String),
    InvalidResponse(String),
    TlsError(String),
    ConnectionFailed(String),
    Timeout,
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
        }
    }
}

impl std::error::Error for HttpClientError {}

impl From<io::Error> for HttpClientError {
    fn from(err: io::Error) -> Self {
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
        let mut config = HttpClientConfig::default();
        config.profile = Some(profile);
        config.headers = headers;
        config.user_agent = user_agent;
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

    /// 发送自定义请求
    pub fn send_request(&self, request: &HttpRequest) -> Result<HttpResponse> {
        // 解析 URL
        let (scheme, host, port, path) = self.parse_url(&request.url)?;

        // 根据协议选择处理方式
        match scheme.as_str() {
            "http" => self.send_http_request(&host, port, &path, request),
            "https" => self.send_https_request(&host, port, &path, request),
            _ => Err(HttpClientError::InvalidUrl(format!(
                "不支持的协议: {}",
                scheme
            ))),
        }
    }

    /// 解析 URL
    fn parse_url(&self, url: &str) -> Result<(String, String, u16, String)> {
        // 简单的 URL 解析
        let url = url.trim();

        let (scheme, rest) = if url.starts_with("https://") {
            ("https", &url[8..])
        } else if url.starts_with("http://") {
            ("http", &url[7..])
        } else {
            return Err(HttpClientError::InvalidUrl("缺少协议".to_string()));
        };

        let (host_port, path) = if let Some(pos) = rest.find('/') {
            (&rest[..pos], &rest[pos..])
        } else {
            (rest, "/")
        };

        let (host, port) = if let Some(pos) = host_port.find(':') {
            let host = host_port[..pos].to_string();
            let port = host_port[pos + 1..]
                .parse::<u16>()
                .map_err(|_| HttpClientError::InvalidUrl("无效的端口".to_string()))?;
            (host, port)
        } else {
            let default_port = if scheme == "https" { 443 } else { 80 };
            (host_port.to_string(), default_port)
        };

        Ok((scheme.to_string(), host, port, path.to_string()))
    }

    /// 发送 HTTP 请求
    fn send_http_request(
        &self,
        host: &str,
        port: u16,
        path: &str,
        request: &HttpRequest,
    ) -> Result<HttpResponse> {
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
        // 优先级：HTTP/3 > HTTP/2 > HTTP/1.1

        // 尝试 HTTP/3
        #[cfg(feature = "http3")]
        {
            if self.config.prefer_http3 {
                return http3::send_http3_request(host, port, path, request, &self.config);
            }
        }

        // 尝试 HTTP/2
        #[cfg(feature = "http2")]
        {
            if self.config.prefer_http2 {
                return http2::send_http2_request(host, port, path, request, &self.config);
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
