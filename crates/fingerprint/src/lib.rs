//! # fingerprint
//!
//! an独立的browser TLS fingerprint库， from  golang version迁移而来。
//!
//! ## Features
//!
//! - ✅ **realbrowserfingerprint**：69+ 个realbrowserfingerprint（Chrome、Firefox、Safari、Opera、Edge）
//! - ✅ **real TLS configuration**：complete TLS Client Hello Spec（cipher suite、椭圆曲线、extension等）
//! - ✅ **JA4 fingerprintGenerate**：complete JA4 TLS clientfingerprintGenerate（sorted  and unsorted version）
//! - ✅ **fingerprint比较**：supportfingerprint相似度比较 and 最佳match查找
//! - ✅ **GREASE process**：complete GREASE value过滤 and process
//! - ✅ **mobilesupport**：iOS、Android mobilefingerprint
//! - ✅ **HTTP/2 & HTTP/3**：complete HTTP/2 configuration，兼容 HTTP/3
//! - ✅ **User-Agent match**：automaticGeneratematch User-Agent
//! - ✅ **standard HTTP Headers**：complete's standard HTTP requestheader
//! - ✅ **全球语言support**：30+ 种语言 Accept-Language
//! - ✅ **operating systemrandom化**：randomly selectoperating system
//! - ✅ **高性能**：零分配的关key操作，并发security
//! - ✅ **Rust standard**：严格遵循 Rust 语言standard and 最佳实践
//! - ✅ **独立库**：不依赖其他 TLS client库
//! - ✅ **代码质量**：throughall Clippy Check，遵循 Rust 最佳实践

#[cfg(feature = "export")]
pub mod export;
pub mod random;

// 重新exportall公共 API
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

#[cfg(feature = "defense")]
pub use fingerprint_defense::{
    HttpFingerprint, Packet, PacketParser, PassiveAnalysisResult, PassiveAnalyzer, PassiveError,
    TcpFingerprint, TlsFingerprint,
};
