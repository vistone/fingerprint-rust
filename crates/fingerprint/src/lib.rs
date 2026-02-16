//! # fingerprint
//!
//! An independent browser TLS fingerprint library, migrated from golang version.
//!
//! ## Features
//!
//! - ✅ **Real browser fingerprints**: 90+ real browser fingerprints (Chrome, Firefox, Safari, Opera, Edge)
//! - ✅ **Real TLS configuration**: Complete TLS Client Hello Spec (cipher suite, elliptic curve, extension, etc.)
//! - ✅ **JA4 fingerprint generation**: Complete JA4 TLS client fingerprint generation (sorted and unsorted versions)
//! - ✅ **Fingerprint comparison**: Support fingerprint similarity comparison and best match finding
//! - ✅ **GREASE processing**: Complete GREASE value filtering and handling
//! - ✅ **Mobile support**: iOS and Android mobile device fingerprints
//! - ✅ **HTTP/2 & HTTP/3**: Complete HTTP/2 configuration with HTTP/3 compatibility
//! - ✅ **User-Agent matching**: Automatic matching of User-Agent strings
//! - ✅ **Standard HTTP Headers**: Complete standard HTTP request headers
//! - ✅ **Global language support**: 30+ languages for Accept-Language
//! - ✅ **Operating system randomization**: Randomly selectable operating systems
//! - ✅ **High performance**: Zero-allocation on critical paths, concurrent safe
//! - ✅ **Rust standards**: Strictly follows Rust language standards and best practices
//! - ✅ **Independent library**: Does not depend on other TLS client libraries
//! - ✅ **Code quality**: Passes all Clippy checks, follows Rust best practices

#[cfg(feature = "export")]
pub mod export;
pub mod random;
/// Re-export types module from fingerprint_core for backward compatibility
pub mod types {
    pub use fingerprint_core::types::*;
}

// reexportallpublic API
pub use fingerprint_core::{
    extract_chrome_version, extract_platform, infer_browser_from_profile_name, is_mobile_profile,
    random_choice, random_choice_string, BrowserType, OperatingSystem, OperatingSystems,
    UserAgentTemplate,
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
