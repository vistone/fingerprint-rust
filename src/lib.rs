//! # fingerprint
//!
//! 一个独立的浏览器 TLS 指纹库，从 golang 版本迁移而来。
//!
//! ## 特性
//!
//! - ✅ **真实浏览器指纹**：66 个真实浏览器指纹（Chrome、Firefox、Safari、Opera）
//! - ✅ **移动端支持**：iOS、Android 移动端指纹
//! - ✅ **HTTP/2 & HTTP/3**：完整的 HTTP/2 配置，兼容 HTTP/3
//! - ✅ **User-Agent 匹配**：自动生成匹配的 User-Agent
//! - ✅ **标准 HTTP Headers**：完整的标准 HTTP 请求头
//! - ✅ **全球语言支持**：30+ 种语言的 Accept-Language
//! - ✅ **操作系统随机化**：随机选择操作系统
//! - ✅ **高性能**：零分配的关键操作，并发安全
//! - ✅ **独立库**：不依赖其他 TLS 客户端库

pub mod dicttls;
pub mod headers;
pub mod http2_config;
pub mod profiles;
pub mod random;
pub mod tls_config;
pub mod tls_extensions;
pub mod types;
pub mod useragent;
pub mod utils;

pub use headers::HTTPHeaders;
pub use http2_config::{
    chrome_header_priority, chrome_http2_settings, chrome_pseudo_header_order,
    firefox_http2_settings, firefox_pseudo_header_order, safari_http2_settings,
    safari_pseudo_header_order, HTTP2Priority, HTTP2PriorityParam, HTTP2SettingID, HTTP2Settings,
};
pub use profiles::{mapped_tls_clients, ClientProfile, ClientHelloID};
pub use tls_config::{
    ClientHelloSpec, ClientHelloSpecBuilder, ClientHelloSignature, TlsClientObserved,
    compare_signatures, compare_specs, extract_signature, find_best_match,
    filter_grease_values, is_grease_value, remove_grease_values, FingerprintMatch, FingerprintStats,
    TLS_GREASE_VALUES,
};
pub use tls_extensions::{
    ALPNExtension, ApplicationSettingsExtensionNew, ExtendedMasterSecretExtension,
    GREASEEncryptedClientHelloExtension, KeyShare, KeyShareExtension, PSKKeyExchangeModesExtension,
    RenegotiationInfoExtension, SCTExtension, SNIExtension, SignatureAlgorithmsExtension,
    StatusRequestExtension, SupportedCurvesExtension, SupportedPointsExtension,
    SupportedVersionsExtension, TLSExtension, TLSExtensionWriter, UtlsCompressCertExtension,
    UtlsGREASEExtension, UtlsPaddingExtension, UtlsPreSharedKeyExtension, extension_from_id,
};
pub use random::{
    get_random_fingerprint, get_random_fingerprint_by_browser,
    get_random_fingerprint_by_browser_with_os, get_random_fingerprint_with_os,
    FingerprintResult,
};
pub use types::{BrowserType, OperatingSystem, OperatingSystems};
pub use useragent::{
    get_user_agent_by_profile_name, get_user_agent_by_profile_name_with_os,
    random_os, UserAgentGenerator,
};
pub use headers::random_language;
