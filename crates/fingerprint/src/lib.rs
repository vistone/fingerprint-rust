//! # fingerprint
//!
//! anindependentbrowser TLS fingerprintlibrary,  from golang versionmigrate而from.
//!
//! ## Features
//!
//! - ✅ **realbrowserfingerprint**：69+ realbrowserfingerprint (Chrome, Firefox, Safari, Opera, Edge)
//! - ✅ **real TLS configuration**：complete TLS Client Hello Spec (cipher suite, elliptic curve, extension etc.)
//! - ✅ **JA4 fingerprintGenerate**：complete JA4 TLS clientfingerprintGenerate (sorted and unsorted version)
//! - ✅ **fingerprintcompare**：supportfingerprintsimilardegreecompare and most佳matchfind
//! - ✅ **GREASE process**：complete GREASE valuefilter and process
//! - ✅ **mobilesupport**：iOS, Android mobilefingerprint
//! - ✅ **HTTP/2 & HTTP/3**：complete HTTP/2 configuration, compatible HTTP/3
//! - ✅ **User-Agent match**：automaticGeneratematch User-Agent
//! - ✅ **standard HTTP Headers**：complete's standard HTTP requestheader
//! - ✅ **globallanguagesupport**：30+ 种language Accept-Language
//! - ✅ **operating systemrandom化**：randomly selectoperating system
//! - ✅ **highperformance**：零allocateclosekeyoperation, concurrentsecurity
//! - ✅ **Rust standard**：strictfollow Rust languagestandard and best practice
//! - ✅ **independentlibrary**：不dependother TLS clientlibrary
//! - ✅ **codequalityquantity**：throughall Clippy Check, follow Rust best practice

#[cfg(feature = "export")]
pub mod export;
pub mod random;
pub mod types {
    pub use fingerprint_core::types::OPERATING_SYSTEMS;
}

// reexportallpublic API
pub use fingerprint_core::{
    extract_chrome_version, extract_platform, infer_browser_from_profile_name, is_mobile_profile,
    random_choice, random_choice_string, BrowserType, OperatingSystem, OperatingSystems,
    UserAgentTemplate, OPERATING_SYSTEMS,
};
pub use fingerprint_headers::{
    chrome_header_priority, chrome_http2_settings, chrome_pseudo_header_order,
    firefox_http2_settings, firefox_pseudo_header_order, generate_headers,
    get_user_agent_by_profile_name, get_user_agent_by_profile_name_with_os, random_language,
    random_os, safari_http2_settings, safari_pseudo_header_order, HTTP2Priority,
    HTTP2PriorityParam, HTTP2SettingID, HTTP2Settings, HTTPHeaders, UserAgentGenerator,
    CHROME_CONNECTION_FLOW,
};
pub use fingerprint_http::{
    Cookie, CookieStore, DNSHelper, HttpClient, HttpClientConfig, HttpClientError, HttpMethod,
    HttpRequest, HttpResponse, ProxyConfig, ProxyType, ReportFormat, ReportSection, SameSite,
    TlsConnector, ValidationReport,
};

#[cfg(feature = "connection-pool")]
pub use fingerprint_http::{ConnectionPoolManager, PoolManagerConfig, PoolStats};

pub use fingerprint_profiles::*;
pub use fingerprint_tls::*;
pub use random::{
    get_random_fingerprint, get_random_fingerprint_by_browser,
    get_random_fingerprint_by_browser_with_os, get_random_fingerprint_with_os, FingerprintResult,
};

#[cfg(feature = "dns")]
pub use fingerprint_dns::{
    load_config as load_dns_config, CachedDNSResolver, DNSCache, DNSConfig, DNSError, DNSResolver,
    DNSResolverTrait, DNSResult, DomainIPs, IPInfo, ServerCollector, ServerPool,
    Service as DNSService,
};

#[cfg(feature = "defense")]
pub use fingerprint_defense::{
    HttpFingerprint, Packet, PacketParser, PassiveAnalysisResult, PassiveAnalyzer, PassiveError,
    TcpFingerprint, TlsFingerprint,
};

#[cfg(feature = "api-noise")]
pub use fingerprint_api_noise as api_noise;
