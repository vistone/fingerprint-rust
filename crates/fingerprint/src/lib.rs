//! # fingerprint
//!
//! 一个独立的浏览器 TLS 指纹库，从 golang 版本迁移而来。
//!
//! ## 特性
//!
//! - ✅ **真实浏览器指纹**：69+ 个真实浏览器指纹（Chrome、Firefox、Safari、Opera、Edge）
//! - ✅ **真实 TLS 配置**：完整的 TLS Client Hello Spec（密码套件、椭圆曲线、扩展等）
//! - ✅ **JA4 指纹生成**：完整的 JA4 TLS 客户端指纹生成（sorted 和 unsorted 版本）
//! - ✅ **指纹比较**：支持指纹相似度比较和最佳匹配查找
//! - ✅ **GREASE 处理**：完整的 GREASE 值过滤和处理
//! - ✅ **移动端支持**：iOS、Android 移动端指纹
//! - ✅ **HTTP/2 & HTTP/3**：完整的 HTTP/2 配置，兼容 HTTP/3
//! - ✅ **User-Agent 匹配**：自动生成匹配的 User-Agent
//! - ✅ **标准 HTTP Headers**：完整的标准 HTTP 请求头
//! - ✅ **全球语言支持**：30+ 种语言的 Accept-Language
//! - ✅ **操作系统随机化**：随机选择操作系统
//! - ✅ **高性能**：零分配的关键操作，并发安全
//! - ✅ **Rust 标准**：严格遵循 Rust 语言标准和最佳实践
//! - ✅ **独立库**：不依赖其他 TLS 客户端库
//! - ✅ **代码质量**：通过所有 Clippy 检查，遵循 Rust 最佳实践

#[cfg(feature = "export")]
pub mod export;
pub mod random;

// 重新导出所有公共 API
pub use fingerprint_core::{
    extract_chrome_version, extract_platform, infer_browser_from_profile_name, is_mobile_profile,
    random_choice, random_choice_string, BrowserType, OperatingSystem, OperatingSystems,
    UserAgentTemplate,
};
pub use fingerprint_headers::{
    chrome_header_priority, chrome_http2_settings, chrome_pseudo_header_order,
    firefox_http2_settings, firefox_pseudo_header_order, get_user_agent_by_profile_name,
    get_user_agent_by_profile_name_with_os, random_language, random_os, safari_http2_settings,
    safari_pseudo_header_order, HTTP2Priority, HTTP2PriorityParam, HTTP2SettingID, HTTP2Settings,
    HTTPHeaders, UserAgentGenerator,
};
pub use fingerprint_http::{
    Cookie, CookieStore, HttpClient, HttpClientConfig, HttpClientError, HttpMethod, HttpRequest,
    HttpResponse, ProxyConfig, ProxyType, ReportFormat, ReportSection, SameSite, TlsConnector,
    ValidationReport,
};
pub use fingerprint_profiles::*;
pub use fingerprint_tls::*;
pub use random::{
    get_random_fingerprint, get_random_fingerprint_by_browser,
    get_random_fingerprint_by_browser_with_os, get_random_fingerprint_with_os, FingerprintResult,
};

#[cfg(feature = "dns")]
pub use fingerprint_dns::{
    load_config as load_dns_config, DNSConfig, DNSError, DNSResult, DomainIPs, IPInfo,
    ServerCollector, ServerPool, Service as DNSService,
};
