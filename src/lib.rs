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

pub mod headers;
pub mod profiles;
pub mod random;
pub mod types;
pub mod useragent;
pub mod utils;

pub use headers::HTTPHeaders;
pub use profiles::{mapped_tls_clients, ClientProfile};
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
