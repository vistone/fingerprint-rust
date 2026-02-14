//! Browser Fingerprint Profiles Module
//!
//! Provides complete browser fingerprint configurations including TLS and HTTP settings
//! for various browser versions.

use fingerprint_core::types::BrowserType;
use fingerprint_headers::{
    generate_headers,
    http2_config::{
        chrome_http2_settings, firefox_http2_settings, safari_http2_settings, HTTP2Settings,
    },
    HTTPHeaders,
};
use fingerprint_tls::tls_config::ClientHelloSpec;

/// Complete browser fingerprint profile
///
/// Combines TLS Client Hello configuration with HTTP headers
/// to provide a comprehensive browser fingerprint.
#[derive(Debug)]
pub struct BrowserProfile {
    /// TLS Client Hello specification
    pub tls_config: ClientHelloSpec,

    /// HTTP request headers
    pub http_headers: HTTPHeaders,

    /// HTTP/2 settings and frame order
    pub http2_settings: HTTP2Settings,

    /// HTTP/2 settings order
    pub http2_settings_order: Vec<u16>,

    /// Browser name and version metadata
    pub metadata: ProfileMetadata,
}

/// Profile metadata
#[derive(Debug, Clone)]
pub struct ProfileMetadata {
    /// Browser name (e.g., "Chrome", "Firefox", "Safari")
    pub browser_name: String,

    /// Browser version (e.g., 133, 136)
    pub browser_version: u32,

    /// User-Agent string
    pub user_agent: String,

    /// Platform (e.g., "Windows", "macOS", "Linux")
    pub platform: String,

    /// Whether this is a mobile profile
    pub is_mobile: bool,

    /// Full version string (e.g., "15.6.1" for Safari, "134" for Chrome)
    pub version_string: String,
}

impl BrowserProfile {
    /// Create a new browser profile with simplified API
    fn create(
        tls_config: ClientHelloSpec,
        browser_type: BrowserType,
        browser_version: u32,
        user_agent: String,
        platform: String,
        is_mobile: bool,
    ) -> Self {
        Self::create_with_version_string(
            tls_config,
            browser_type,
            browser_version,
            user_agent,
            platform,
            is_mobile,
            browser_version.to_string(),
        )
    }

    /// Create a new browser profile with explicit version string
    fn create_with_version_string(
        tls_config: ClientHelloSpec,
        browser_type: BrowserType,
        browser_version: u32,
        user_agent: String,
        platform: String,
        is_mobile: bool,
        version_string: String,
    ) -> Self {
        let http_headers = generate_headers(browser_type, &user_agent, is_mobile);

        let (http2_settings, http2_settings_order) = match browser_type {
            BrowserType::Chrome | BrowserType::Edge | BrowserType::Opera => chrome_http2_settings(),
            BrowserType::Firefox => firefox_http2_settings(),
            BrowserType::Safari => safari_http2_settings(),
        };

        let metadata = ProfileMetadata {
            browser_name: browser_type.to_string(),
            browser_version,
            user_agent,
            platform,
            is_mobile,
            version_string,
        };

        Self {
            tls_config,
            http_headers,
            http2_settings,
            http2_settings_order,
            metadata,
        }
    }

    /// Get a unique identifier for this profile (e.g., "chrome_133", "chrome_mobile_134")
    pub fn id(&self) -> String {
        let browser_name = if self.metadata.is_mobile {
            match self.metadata.platform.as_str() {
                "iOS" | "iPadOS" => format!(
                    "{}_ios",
                    self.metadata.browser_name.to_lowercase().replace(' ', "_")
                ),
                _ => format!(
                    "{}_mobile",
                    self.metadata.browser_name.to_lowercase().replace(' ', "_")
                ),
            }
        } else {
            self.metadata.browser_name.to_lowercase().replace(' ', "_")
        };
        let version = self.metadata.version_string.replace('.', "_");
        format!("{}_{}", browser_name, version)
    }
}

// Helper macro to create Chrome profiles
macro_rules! define_chrome_version {
    ($fn_name:ident, $version:expr, $tls_fn:expr) => {
        pub fn $fn_name() -> BrowserProfile {
            BrowserProfile::create(
                $tls_fn(),
                BrowserType::Chrome,
                $version,
                format!(
                    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/{}.0.0.0 Safari/537.36",
                    $version
                ),
                "Windows".to_string(),
                false,
            )
        }
    };
}

// ============================================================================
// Chrome Profiles
// ============================================================================

define_chrome_version!(chrome_103, 103, ClientHelloSpec::chrome_103);
define_chrome_version!(chrome_104, 104, ClientHelloSpec::chrome_103);
define_chrome_version!(chrome_105, 105, ClientHelloSpec::chrome_103);
define_chrome_version!(chrome_106, 106, ClientHelloSpec::chrome_103);
define_chrome_version!(chrome_107, 107, ClientHelloSpec::chrome_103);
define_chrome_version!(chrome_108, 108, ClientHelloSpec::chrome_103);
define_chrome_version!(chrome_109, 109, ClientHelloSpec::chrome_103);
define_chrome_version!(chrome_110, 110, ClientHelloSpec::chrome_103);
define_chrome_version!(chrome_111, 111, ClientHelloSpec::chrome_103);
define_chrome_version!(chrome_112, 112, ClientHelloSpec::chrome_103);

pub fn chrome_116_psk() -> BrowserProfile {
    BrowserProfile::create(
        ClientHelloSpec::chrome_133_psk(),
        BrowserType::Chrome,
        116,
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/116.0.0.0 Safari/537.36".to_string(),
        "Windows".to_string(),
        false,
    )
}

pub fn chrome_116_psk_pq() -> BrowserProfile {
    BrowserProfile::create(
        ClientHelloSpec::chrome_133_psk(),
        BrowserType::Chrome,
        116,
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/116.0.0.0 Safari/537.36".to_string(),
        "Windows".to_string(),
        false,
    )
}

define_chrome_version!(chrome_117, 117, ClientHelloSpec::chrome_133);
define_chrome_version!(chrome_120, 120, ClientHelloSpec::chrome_133);
define_chrome_version!(chrome_121, 121, ClientHelloSpec::chrome_133);
define_chrome_version!(chrome_122, 122, ClientHelloSpec::chrome_133);
define_chrome_version!(chrome_123, 123, ClientHelloSpec::chrome_133);
define_chrome_version!(chrome_124, 124, ClientHelloSpec::chrome_133);
define_chrome_version!(chrome_125, 125, ClientHelloSpec::chrome_133);
define_chrome_version!(chrome_126, 126, ClientHelloSpec::chrome_133);
define_chrome_version!(chrome_127, 127, ClientHelloSpec::chrome_133);
define_chrome_version!(chrome_128, 128, ClientHelloSpec::chrome_133);
define_chrome_version!(chrome_129, 129, ClientHelloSpec::chrome_133);
define_chrome_version!(chrome_130, 130, ClientHelloSpec::chrome_133);
define_chrome_version!(chrome_131, 131, ClientHelloSpec::chrome_133);
define_chrome_version!(chrome_132, 132, ClientHelloSpec::chrome_133);
define_chrome_version!(chrome_133, 133, ClientHelloSpec::chrome_133);

pub fn chrome_133_psk() -> BrowserProfile {
    BrowserProfile::create(
        ClientHelloSpec::chrome_133_psk(),
        BrowserType::Chrome,
        133,
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/133.0.0.0 Safari/537.36".to_string(),
        "Windows".to_string(),
        false,
    )
}

pub fn chrome_133_0rtt() -> BrowserProfile {
    BrowserProfile::create(
        ClientHelloSpec::chrome_133_0rtt(),
        BrowserType::Chrome,
        133,
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/133.0.0.0 Safari/537.36".to_string(),
        "Windows".to_string(),
        false,
    )
}

pub fn chrome_133_psk_0rtt() -> BrowserProfile {
    BrowserProfile::create(
        ClientHelloSpec::chrome_133_psk_0rtt(),
        BrowserType::Chrome,
        133,
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/133.0.0.0 Safari/537.36".to_string(),
        "Windows".to_string(),
        false,
    )
}

define_chrome_version!(chrome_134, 134, ClientHelloSpec::chrome_133);
define_chrome_version!(chrome_135, 135, ClientHelloSpec::chrome_133);
define_chrome_version!(chrome_136, 136, ClientHelloSpec::chrome_136);
define_chrome_version!(chrome_137, 137, ClientHelloSpec::chrome_136);
define_chrome_version!(chrome_138, 138, ClientHelloSpec::chrome_136);

// ============================================================================
// Firefox Profiles
// ============================================================================

macro_rules! define_firefox_version {
    ($fn_name:ident, $version:expr, $tls_fn:expr) => {
        pub fn $fn_name() -> BrowserProfile {
            BrowserProfile::create(
                $tls_fn(),
                BrowserType::Firefox,
                $version,
                format!(
                    "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:{}.0) Gecko/20100101 Firefox/{}.0",
                    $version, $version
                ),
                "Windows".to_string(),
                false,
            )
        }
    };
}

define_firefox_version!(firefox_102, 102, ClientHelloSpec::firefox_133);
define_firefox_version!(firefox_104, 104, ClientHelloSpec::firefox_133);
define_firefox_version!(firefox_105, 105, ClientHelloSpec::firefox_133);
define_firefox_version!(firefox_106, 106, ClientHelloSpec::firefox_133);
define_firefox_version!(firefox_108, 108, ClientHelloSpec::firefox_133);
define_firefox_version!(firefox_110, 110, ClientHelloSpec::firefox_133);
define_firefox_version!(firefox_117, 117, ClientHelloSpec::firefox_133);
define_firefox_version!(firefox_120, 120, ClientHelloSpec::firefox_133);
define_firefox_version!(firefox_123, 123, ClientHelloSpec::firefox_133);
define_firefox_version!(firefox_130, 130, ClientHelloSpec::firefox_133);
define_firefox_version!(firefox_131, 131, ClientHelloSpec::firefox_133);
define_firefox_version!(firefox_132, 132, ClientHelloSpec::firefox_133);
define_firefox_version!(firefox_133, 133, ClientHelloSpec::firefox_133);
define_firefox_version!(firefox_134, 134, ClientHelloSpec::firefox_133);
define_firefox_version!(firefox_135, 135, ClientHelloSpec::firefox_133);
define_firefox_version!(firefox_136, 136, ClientHelloSpec::firefox_133);
define_firefox_version!(firefox_137, 137, ClientHelloSpec::firefox_133);
define_firefox_version!(firefox_138, 138, ClientHelloSpec::firefox_133);

// ============================================================================
// Safari Profiles
// ============================================================================

macro_rules! define_safari_version {
    ($fn_name:ident, $version_str:expr, $version_num:expr, $tls_fn:expr) => {
        pub fn $fn_name() -> BrowserProfile {
            BrowserProfile::create_with_version_string(
                $tls_fn(),
                BrowserType::Safari,
                $version_num,
                format!(
                    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/{} Safari/605.1.15",
                    $version_str
                ),
                "macOS".to_string(),
                false,
                $version_str.to_string(),
            )
        }
    };
}

define_safari_version!(safari_15_0, "15.0", 150, ClientHelloSpec::safari_16_0);
define_safari_version!(safari_15_6_1, "15.6.1", 1561, ClientHelloSpec::safari_16_0);
define_safari_version!(safari_15_7, "15.7", 157, ClientHelloSpec::safari_16_0);
define_safari_version!(safari_16_0, "16.0", 160, ClientHelloSpec::safari_16_0);
define_safari_version!(safari_17_0, "17.0", 170, ClientHelloSpec::safari_16_0);
define_safari_version!(safari_17_5, "17.5", 175, ClientHelloSpec::safari_16_0);
define_safari_version!(safari_18_0, "18.0", 180, ClientHelloSpec::safari_16_0);
define_safari_version!(safari_18_1, "18.1", 181, ClientHelloSpec::safari_16_0);
define_safari_version!(safari_18_2, "18.2", 182, ClientHelloSpec::safari_16_0);
define_safari_version!(safari_18_3, "18.3", 183, ClientHelloSpec::safari_16_0);

// Safari iOS profiles
macro_rules! define_safari_ios_version {
    ($fn_name:ident, $version_str:expr, $version_num:expr, $tls_fn:expr) => {
        pub fn $fn_name() -> BrowserProfile {
            BrowserProfile::create_with_version_string(
                $tls_fn(),
                BrowserType::Safari,
                $version_num,
                format!(
                    "Mozilla/5.0 (iPhone; CPU iPhone OS {} like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/{} Mobile/15E148 Safari/604.1",
                    $version_str.replace('.', "_"),
                    $version_str
                ),
                "iOS".to_string(),
                true,
                $version_str.to_string(),
            )
        }
    };
}

define_safari_ios_version!(safari_ios_15_5, "15.5", 155, ClientHelloSpec::safari_16_0);
define_safari_ios_version!(safari_ios_15_6, "15.6", 156, ClientHelloSpec::safari_16_0);
define_safari_ios_version!(safari_ios_16_0, "16.0", 160, ClientHelloSpec::safari_16_0);
define_safari_ios_version!(safari_ios_17_0, "17.0", 170, ClientHelloSpec::safari_16_0);
define_safari_ios_version!(safari_ios_18_0, "18.0", 180, ClientHelloSpec::safari_16_0);
define_safari_ios_version!(safari_ios_18_1, "18.1", 181, ClientHelloSpec::safari_16_0);
define_safari_ios_version!(safari_ios_18_2, "18.2", 182, ClientHelloSpec::safari_16_0);
define_safari_ios_version!(safari_ios_18_3, "18.3", 183, ClientHelloSpec::safari_16_0);
define_safari_ios_version!(safari_ios_18_5, "18.5", 185, ClientHelloSpec::safari_16_0);

pub fn safari_ipad_15_6() -> BrowserProfile {
    BrowserProfile::create(
        ClientHelloSpec::safari_16_0(),
        BrowserType::Safari,
        156,
        "Mozilla/5.0 (iPad; CPU OS 15_6 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/15.6 Mobile/15E148 Safari/604.1".to_string(),
        "iPadOS".to_string(),
        true,
    )
}

// ============================================================================
// Edge Profiles
// ============================================================================

macro_rules! define_edge_version {
    ($fn_name:ident, $version:expr, $tls_fn:expr) => {
        pub fn $fn_name() -> BrowserProfile {
            BrowserProfile::create(
                $tls_fn(),
                BrowserType::Edge,
                $version,
                format!(
                    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/{}.0.0.0 Safari/537.36 Edg/{}.0.0.0",
                    $version, $version
                ),
                "Windows".to_string(),
                false,
            )
        }
    };
}

define_edge_version!(edge_120, 120, ClientHelloSpec::chrome_133);
define_edge_version!(edge_124, 124, ClientHelloSpec::chrome_133);
define_edge_version!(edge_125, 125, ClientHelloSpec::chrome_133);
define_edge_version!(edge_126, 126, ClientHelloSpec::chrome_133);
define_edge_version!(edge_130, 130, ClientHelloSpec::chrome_133);
define_edge_version!(edge_131, 131, ClientHelloSpec::chrome_133);
define_edge_version!(edge_132, 132, ClientHelloSpec::chrome_133);
define_edge_version!(edge_133, 133, ClientHelloSpec::chrome_133);
define_edge_version!(edge_134, 134, ClientHelloSpec::chrome_133);
define_edge_version!(edge_135, 135, ClientHelloSpec::chrome_133);
define_edge_version!(edge_137, 137, ClientHelloSpec::chrome_136);

// ============================================================================
// Opera Profiles
// ============================================================================

macro_rules! define_opera_version {
    ($fn_name:ident, $version:expr, $chrome_version:expr, $tls_fn:expr) => {
        pub fn $fn_name() -> BrowserProfile {
            BrowserProfile::create(
                $tls_fn(),
                BrowserType::Opera,
                $version,
                format!(
                    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/{}.0.0.0 Safari/537.36 OPR/{}.0.0.0",
                    $chrome_version, $version
                ),
                "Windows".to_string(),
                false,
            )
        }
    };
}

define_opera_version!(opera_89, 89, 103, ClientHelloSpec::chrome_103);
define_opera_version!(opera_90, 90, 104, ClientHelloSpec::chrome_103);
define_opera_version!(opera_91, 91, 105, ClientHelloSpec::chrome_103);
define_opera_version!(opera_92, 92, 106, ClientHelloSpec::chrome_103);
define_opera_version!(opera_93, 93, 107, ClientHelloSpec::chrome_103);
define_opera_version!(opera_94, 94, 108, ClientHelloSpec::chrome_103);

// ============================================================================
// Mobile Chrome Profiles
// ============================================================================

macro_rules! define_chrome_mobile_version {
    ($fn_name:ident, $version:expr, $tls_fn:expr) => {
        pub fn $fn_name() -> BrowserProfile {
            BrowserProfile::create(
                $tls_fn(),
                BrowserType::Chrome,
                $version,
                format!(
                    "Mozilla/5.0 (Linux; Android 10) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/{}.0.0.0 Mobile Safari/537.36",
                    $version
                ),
                "Android".to_string(),
                true,
            )
        }
    };
}

define_chrome_mobile_version!(chrome_mobile_120, 120, ClientHelloSpec::chrome_133);
define_chrome_mobile_version!(chrome_mobile_130, 130, ClientHelloSpec::chrome_133);
define_chrome_mobile_version!(chrome_mobile_134, 134, ClientHelloSpec::chrome_133);
define_chrome_mobile_version!(chrome_mobile_135, 135, ClientHelloSpec::chrome_133);
define_chrome_mobile_version!(chrome_mobile_137, 137, ClientHelloSpec::chrome_136);

// ============================================================================
// Mobile Firefox Profiles
// ============================================================================

macro_rules! define_firefox_mobile_version {
    ($fn_name:ident, $version:expr, $tls_fn:expr) => {
        pub fn $fn_name() -> BrowserProfile {
            BrowserProfile::create(
                $tls_fn(),
                BrowserType::Firefox,
                $version,
                format!(
                    "Mozilla/5.0 (Android 10; Mobile; rv:{}.0) Gecko/{}.0 Firefox/{}.0",
                    $version, $version, $version
                ),
                "Android".to_string(),
                true,
            )
        }
    };
}

define_firefox_mobile_version!(firefox_mobile_120, 120, ClientHelloSpec::firefox_133);
define_firefox_mobile_version!(firefox_mobile_130, 130, ClientHelloSpec::firefox_133);
define_firefox_mobile_version!(firefox_mobile_135, 135, ClientHelloSpec::firefox_133);

/// Get all available browser profiles as a map
pub fn mapped_tls_clients() -> std::collections::HashMap<String, BrowserProfile> {
    let mut map = std::collections::HashMap::new();

    // Add Chrome profiles
    map.insert("chrome_133".to_string(), chrome_133());
    map.insert("chrome_136".to_string(), chrome_136());

    // Add Firefox profiles
    map.insert("firefox_133".to_string(), firefox_133());

    // Add Safari profiles
    map.insert("safari_16_0".to_string(), safari_16_0());

    map
}
