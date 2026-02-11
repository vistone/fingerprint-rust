//! Fingerprint configuration module
//!
//! Defines TLS fingerprint configurations for various browsers

use fingerprint_core::tcp::TcpProfile;
use fingerprint_headers::http2_config::{
    chrome_header_order, chrome_header_priority, chrome_http2_settings, chrome_pseudo_header_order,
    firefox_header_order, firefox_http2_settings, firefox_pseudo_header_order, safari_header_order,
    safari_http2_settings, safari_pseudo_header_order, HTTP2PriorityParam, HTTP2Settings,
};
use fingerprint_tls::tls_config::ClientHelloSpec;
use std::collections::HashMap;
use std::sync::OnceLock;

/// ClientHelloSpecFactory type
/// Corresponds to Go version's ClientHelloSpecFactory func() (ClientHelloSpec, error)
pub type ClientHelloSpecFactory = fn() -> Result<ClientHelloSpec, String>;

/// Client Hello ID
/// Corresponds to Go version's tls.ClientHelloID
#[derive(Debug, Clone)]
pub struct ClientHelloID {
    /// Client name (如 "Chrome", "Firefox", "Safari")
    pub client: String,
    /// Version versionnumber (如 "135", "133")
    pub version: String,
    /// SpecFactory for Generate ClientHelloSpec
    pub spec_factory: ClientHelloSpecFactory,
}

impl ClientHelloID {
    pub fn new(client: &str, version: &str, spec_factory: ClientHelloSpecFactory) -> Self {
        Self {
            client: client.to_string(),
            version: version.to_string(),
            spec_factory,
        }
    }

    /// convert tostringrepresent (Corresponds to Go version's Str())
    pub fn str(&self) -> String {
        format!("{}-{}", self.client, self.version)
    }

    /// convert to ClientHelloSpec (Corresponds to Go version's ToSpec())
    pub fn to_spec(&self) -> Result<ClientHelloSpec, String> {
        (self.spec_factory)()
    }
}

impl PartialEq for ClientHelloID {
    fn eq(&self, other: &Self) -> bool {
        self.client == other.client && self.version == other.version
    }
}

impl Eq for ClientHelloID {}

impl std::hash::Hash for ClientHelloID {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.client.hash(state);
        self.version.hash(state);
    }
}

/// Client Profile configuration
/// including TLS fingerprintallconfigurationinfo
/// Corresponds to Go version's ClientProfile struct
#[derive(Debug, Clone)]
pub struct ClientProfile {
    /// Client Hello ID
    pub client_hello_id: ClientHelloID,
    /// HTTP/2 Settings (Corresponds to Go version's map[http2.SettingID]uint32)
    pub settings: HTTP2Settings,
    /// Settings order (Corresponds to Go version's []http2.SettingID)
    pub settings_order: Vec<u16>,
    /// Pseudo Header order (Corresponds to Go version's []string)
    pub pseudo_header_order: Vec<String>,
    /// Connection Flow (Corresponds to Go version's uint32)
    pub connection_flow: u32,
    /// Priorities (Corresponds to Go version's []http2.Priority)
    pub priorities: Vec<String>,
    /// Header Priority (Corresponds to Go version's *http2.PriorityParam)
    pub header_priority: Option<HTTP2PriorityParam>,
    /// TCP Settings (Active Fingerprinting)
    pub tcp_profile: Option<TcpProfile>,
    /// Header Order (for HTTP/1.1)
    pub header_order: Vec<String>,
}

impl ClientProfile {
    /// Create a new ClientProfile
    /// Corresponds to Go version's NewClientProfile function
    #[allow(clippy::too_many_arguments)] // constructfunctionneedall必要parameter
    pub fn new(
        client_hello_id: ClientHelloID,
        settings: HTTP2Settings,
        settings_order: Vec<u16>,
        pseudo_header_order: Vec<String>,
        connection_flow: u32,
        priorities: Vec<String>,
        header_priority: Option<HTTP2PriorityParam>,
        tcp_profile: Option<TcpProfile>,
        header_order: Vec<String>,
    ) -> Self {
        Self {
            client_hello_id,
            settings,
            settings_order,
            pseudo_header_order,
            connection_flow,
            priorities,
            header_priority,
            tcp_profile,
            header_order,
        }
    }

    /// Get Client Hello ID string (Corresponds to Go version's GetClientHelloStr())
    pub fn get_client_hello_str(&self) -> String {
        self.client_hello_id.str()
    }

    /// Based on User-Agent automaticGeneratematch TCP Profile
    ///
    /// this isunifiedfingerprintGeneratecoremethod, ensurebrowserfingerprint and TCP fingerprintsync
    ///
    /// # Parameters
    /// - `user_agent`: User-Agent string,  for inferoperating system
    ///
    /// # Returns
    /// returnannew ClientProfile, 其 in tcp_profile alreadyBased on User-Agent settings
    pub fn with_synced_tcp_profile(self, user_agent: &str) -> Self {
        use fingerprint_core::tcp::TcpProfile;
        let tcp_profile = TcpProfile::from_user_agent(user_agent);
        Self {
            tcp_profile: Some(tcp_profile),
            ..self
        }
    }

    /// Based onoperating systemtypeautomaticGeneratematch TCP Profile
    ///
    /// # Parameters
    /// - `os`: operating systemtype
    ///
    /// # Returns
    /// returnannew ClientProfile, 其 in tcp_profile alreadyBased onoperating systemsettings
    pub fn with_tcp_profile_for_os(self, os: fingerprint_core::types::OperatingSystem) -> Self {
        use fingerprint_core::tcp::TcpProfile;
        let tcp_profile = TcpProfile::for_os(os);
        Self {
            tcp_profile: Some(tcp_profile),
            ..self
        }
    }

    /// Get or Generate TCP Profile
    ///
    /// If tcp_profile already exists, directlyreturn
    /// If不 exists, Based on User-Agent Generate
    ///
    /// # Parameters
    /// - `user_agent`: User-Agent string,  for inferoperating system ( if tcp_profile 不 exists)
    ///
    /// # Returns
    /// TCP Profile reference
    pub fn get_or_generate_tcp_profile(&mut self, user_agent: &str) -> &TcpProfile {
        use fingerprint_core::tcp::TcpProfile;
        if self.tcp_profile.is_none() {
            self.tcp_profile = Some(TcpProfile::from_user_agent(user_agent));
        }
        self.tcp_profile.as_ref().unwrap()
    }

    /// Get Settings (Corresponds to Go version's GetSettings())
    pub fn get_settings(&self) -> &HTTP2Settings {
        &self.settings
    }

    /// Get Settings Order (Corresponds to Go version's GetSettingsOrder())
    pub fn get_settings_order(&self) -> &[u16] {
        &self.settings_order
    }

    /// Get Pseudo Header Order
    pub fn get_pseudo_header_order(&self) -> &[String] {
        &self.pseudo_header_order
    }

    /// Get Connection Flow
    pub fn get_connection_flow(&self) -> u32 {
        self.connection_flow
    }

    /// Get Priorities
    pub fn get_priorities(&self) -> &[String] {
        &self.priorities
    }

    /// Get Header Priority (Corresponds to Go version's GetHeaderPriority())
    pub fn get_header_priority(&self) -> Option<&HTTP2PriorityParam> {
        self.header_priority.as_ref()
    }

    /// Get ClientHelloSpec (Corresponds to Go version's GetClientHelloSpec())
    /// this istrue TLS fingerprintconfiguration, can for actual TLS handshake
    pub fn get_client_hello_spec(&self) -> Result<ClientHelloSpec, String> {
        self.client_hello_id.to_spec()
    }

    /// Get JA4 fingerprintstring
    pub fn get_ja4_string(&self) -> Result<String, String> {
        let spec = self.get_client_hello_spec()?;
        Ok(spec.ja4_string())
    }
}

/// default Client Profile (Chrome 135)
pub fn default_client_profile() -> ClientProfile {
    chrome_135()
}

/// Chrome 103 fingerprintconfiguration
/// Corresponds to Go version's Chrome_103
pub fn chrome_103() -> ClientProfile {
    let (settings, settings_order) = chrome_http2_settings();
    ClientProfile::new(
        ClientHelloID::new(
            "Chrome",
            "103",
            fingerprint_tls::tls_config::chrome_103_spec,
        ),
        settings,
        settings_order,
        chrome_pseudo_header_order(),
        fingerprint_headers::http2_config::CHROME_CONNECTION_FLOW,
        Vec::new(),
        Some(chrome_header_priority()),
        None,
        chrome_header_order(),
    )
}

/// Chrome 133 fingerprintconfiguration
pub fn chrome_133() -> ClientProfile {
    let (settings, settings_order) = chrome_http2_settings();
    // defaultuse Windows TCP Profile (most commonbrowserenvironment)
    // usercanthrough with_synced_tcp_profile() or with_tcp_profile_for_os() fromsync
    let default_tcp_profile = Some(TcpProfile::for_os(
        fingerprint_core::types::OperatingSystem::Windows10,
    ));
    ClientProfile::new(
        ClientHelloID::new(
            "Chrome",
            "133",
            fingerprint_tls::tls_config::chrome_133_spec,
        ),
        settings,
        settings_order,
        chrome_pseudo_header_order(),
        fingerprint_headers::http2_config::CHROME_CONNECTION_FLOW,
        Vec::new(),
        Some(chrome_header_priority()),
        default_tcp_profile,
        chrome_header_order(),
    )
}

/// Firefox 133 fingerprintconfiguration
pub fn firefox_133() -> ClientProfile {
    let (settings, settings_order) = firefox_http2_settings();
    // defaultuse Windows TCP Profile (most commonbrowserenvironment)
    // usercanthrough with_synced_tcp_profile() or with_tcp_profile_for_os() fromsync
    let default_tcp_profile = Some(TcpProfile::for_os(
        fingerprint_core::types::OperatingSystem::Windows10,
    ));
    ClientProfile::new(
        ClientHelloID::new(
            "Firefox",
            "133",
            fingerprint_tls::tls_config::firefox_133_spec,
        ),
        settings,
        settings_order,
        firefox_pseudo_header_order(),
        fingerprint_headers::http2_config::CHROME_CONNECTION_FLOW,
        Vec::new(),
        None,
        default_tcp_profile,
        firefox_header_order(),
    )
}

/// Chrome 136 fingerprintconfiguration
pub fn chrome_136() -> ClientProfile {
    let (settings, settings_order) = chrome_http2_settings();
    let default_tcp_profile = Some(TcpProfile::for_os(
        fingerprint_core::types::OperatingSystem::Windows10,
    ));
    ClientProfile::new(
        ClientHelloID::new(
            "Chrome",
            "136",
            fingerprint_tls::tls_config::chrome_136_spec,
        ),
        settings,
        settings_order,
        chrome_pseudo_header_order(),
        fingerprint_headers::http2_config::CHROME_CONNECTION_FLOW,
        Vec::new(),
        Some(chrome_header_priority()),
        default_tcp_profile,
        chrome_header_order(),
    )
}

/// Chrome 135 fingerprintconfiguration (default)
pub fn chrome_135() -> ClientProfile {
    let (settings, settings_order) = chrome_http2_settings();
    // defaultuse Windows TCP Profile (most commonbrowserenvironment)
    // usercanthrough with_synced_tcp_profile() or with_tcp_profile_for_os() fromsync
    let default_tcp_profile = Some(TcpProfile::for_os(
        fingerprint_core::types::OperatingSystem::Windows10,
    ));
    ClientProfile::new(
        ClientHelloID::new(
            "Chrome",
            "135",
            fingerprint_tls::tls_config::chrome_133_spec, // use 133 TLS struct
        ),
        settings,
        settings_order,
        chrome_pseudo_header_order(),
        fingerprint_headers::http2_config::CHROME_CONNECTION_FLOW,
        Vec::new(),
        Some(chrome_header_priority()),
        default_tcp_profile,
        chrome_header_order(),
    )
}

/// Chrome 134 fingerprint configuration (2026 stable version)
pub fn chrome_134() -> ClientProfile {
    let (settings, settings_order) = chrome_http2_settings();
    let default_tcp_profile = Some(TcpProfile::for_os(
        fingerprint_core::types::OperatingSystem::Windows10,
    ));
    ClientProfile::new(
        ClientHelloID::new(
            "Chrome",
            "134",
            fingerprint_tls::tls_config::chrome_133_spec, // use 133 TLS struct
        ),
        settings,
        settings_order,
        chrome_pseudo_header_order(),
        fingerprint_headers::http2_config::CHROME_CONNECTION_FLOW,
        Vec::new(),
        Some(chrome_header_priority()),
        default_tcp_profile,
        chrome_header_order(),
    )
}

/// Firefox 135 fingerprintconfiguration
pub fn firefox_135() -> ClientProfile {
    let (settings, settings_order) = firefox_http2_settings();
    // defaultuse Windows TCP Profile (most commonbrowserenvironment)
    // usercanthrough with_synced_tcp_profile() or with_tcp_profile_for_os() fromsync
    let default_tcp_profile = Some(TcpProfile::for_os(
        fingerprint_core::types::OperatingSystem::Windows10,
    ));
    ClientProfile::new(
        ClientHelloID::new(
            "Firefox",
            "135",
            fingerprint_tls::tls_config::firefox_133_spec,
        ),
        settings,
        settings_order,
        firefox_pseudo_header_order(),
        fingerprint_headers::http2_config::CHROME_CONNECTION_FLOW,
        Vec::new(),
        None,
        default_tcp_profile,
        firefox_header_order(),
    )
}

/// Firefox 134 fingerprint configuration (2026 version)
pub fn firefox_134() -> ClientProfile {
    let (settings, settings_order) = firefox_http2_settings();
    let default_tcp_profile = Some(TcpProfile::for_os(
        fingerprint_core::types::OperatingSystem::Windows10,
    ));
    ClientProfile::new(
        ClientHelloID::new(
            "Firefox",
            "134",
            fingerprint_tls::tls_config::firefox_133_spec,
        ),
        settings,
        settings_order,
        firefox_pseudo_header_order(),
        fingerprint_headers::http2_config::CHROME_CONNECTION_FLOW,
        Vec::new(),
        None,
        default_tcp_profile,
        firefox_header_order(),
    )
}

/// Firefox 136 fingerprint configuration (2026 Nightly version)
pub fn firefox_136() -> ClientProfile {
    let (settings, settings_order) = firefox_http2_settings();
    let default_tcp_profile = Some(TcpProfile::for_os(
        fingerprint_core::types::OperatingSystem::Windows10,
    ));
    ClientProfile::new(
        ClientHelloID::new(
            "Firefox",
            "136",
            fingerprint_tls::tls_config::firefox_133_spec,
        ),
        settings,
        settings_order,
        firefox_pseudo_header_order(),
        fingerprint_headers::http2_config::CHROME_CONNECTION_FLOW,
        Vec::new(),
        None,
        default_tcp_profile,
        firefox_header_order(),
    )
}

/// Safari 16.0 fingerprintconfiguration
/// Corresponds to Go version's Safari_16_0
pub fn safari_16_0() -> ClientProfile {
    let (settings, settings_order) = safari_http2_settings();
    ClientProfile::new(
        ClientHelloID::new(
            "Safari",
            "16.0",
            fingerprint_tls::tls_config::safari_16_0_spec,
        ),
        settings,
        settings_order,
        safari_pseudo_header_order(),
        fingerprint_headers::http2_config::CHROME_CONNECTION_FLOW,
        Vec::new(),
        None,
        None,
        safari_header_order(),
    )
}

/// Safari 18.2 fingerprint configuration (2026 macOS Sequoia version)
pub fn safari_18_2() -> ClientProfile {
    let (settings, settings_order) = safari_http2_settings();
    let default_tcp_profile = Some(TcpProfile::for_os(
        fingerprint_core::types::OperatingSystem::MacOS14,
    ));
    ClientProfile::new(
        ClientHelloID::new(
            "Safari",
            "18.2",
            fingerprint_tls::tls_config::safari_16_0_spec,
        ),
        settings,
        settings_order,
        safari_pseudo_header_order(),
        fingerprint_headers::http2_config::CHROME_CONNECTION_FLOW,
        Vec::new(),
        None,
        default_tcp_profile,
        safari_header_order(),
    )
}

/// Safari iOS 18.2 fingerprint configuration (2026 iOS version)
pub fn safari_ios_18_2() -> ClientProfile {
    let (settings, settings_order) = safari_http2_settings();
    let default_tcp_profile = Some(TcpProfile::for_os(
        fingerprint_core::types::OperatingSystem::MacOS14,
    ));
    ClientProfile::new(
        ClientHelloID::new(
            "Safari",
            "iOS_18.2",
            fingerprint_tls::tls_config::safari_16_0_spec,
        ),
        settings,
        settings_order,
        safari_pseudo_header_order(),
        fingerprint_headers::http2_config::CHROME_CONNECTION_FLOW,
        Vec::new(),
        None,
        default_tcp_profile,
        safari_header_order(),
    )
}

/// Opera 91 fingerprintconfiguration
/// Corresponds to Go version's Opera_91
pub fn opera_91() -> ClientProfile {
    // Opera use Chrome insidecore, configuration and Chrome same
    let (settings, settings_order) = chrome_http2_settings();
    ClientProfile::new(
        ClientHelloID::new("Opera", "91", fingerprint_tls::tls_config::chrome_133_spec), // Opera use Chrome TLS configuration
        settings,
        settings_order,
        chrome_pseudo_header_order(),
        fingerprint_headers::http2_config::CHROME_CONNECTION_FLOW,
        Vec::new(),
        Some(chrome_header_priority()),
        None,
        chrome_header_order(),
    )
}

/// Edge 120 fingerprint configuration
/// Edge uses Chrome core, TLS fingerprint matches Chrome
pub fn edge_120() -> ClientProfile {
    // Edge uses Chrome core, configuration matches Chrome
    let (settings, settings_order) = chrome_http2_settings();
    ClientProfile::new(
        ClientHelloID::new("Edge", "120", fingerprint_tls::tls_config::chrome_133_spec), // Edge use Chrome TLS configuration
        settings,
        settings_order,
        chrome_pseudo_header_order(),
        fingerprint_headers::http2_config::CHROME_CONNECTION_FLOW,
        Vec::new(),
        Some(chrome_header_priority()),
        None,
        chrome_header_order(),
    )
}

/// Edge 124 fingerprintconfiguration
/// Edge use Chromium insidecore, TLS fingerprint and Chrome same
pub fn edge_124() -> ClientProfile {
    let (settings, settings_order) = chrome_http2_settings();
    ClientProfile::new(
        ClientHelloID::new("Edge", "124", fingerprint_tls::tls_config::chrome_133_spec),
        settings,
        settings_order,
        chrome_pseudo_header_order(),
        fingerprint_headers::http2_config::CHROME_CONNECTION_FLOW,
        Vec::new(),
        Some(chrome_header_priority()),
        None,
        chrome_header_order(),
    )
}

/// Edge 133 fingerprintconfiguration
/// Edge use Chromium insidecore, TLS fingerprint and Chrome same
pub fn edge_133() -> ClientProfile {
    let (settings, settings_order) = chrome_http2_settings();
    ClientProfile::new(
        ClientHelloID::new("Edge", "133", fingerprint_tls::tls_config::chrome_133_spec),
        settings,
        settings_order,
        chrome_pseudo_header_order(),
        fingerprint_headers::http2_config::CHROME_CONNECTION_FLOW,
        Vec::new(),
        Some(chrome_header_priority()),
        None,
        chrome_header_order(),
    )
}

/// Edge 134 fingerprint configuration (2026 version)
/// Edge use Chromium insidecore, TLS fingerprint and Chrome same
pub fn edge_134() -> ClientProfile {
    let (settings, settings_order) = chrome_http2_settings();
    let default_tcp_profile = Some(TcpProfile::for_os(
        fingerprint_core::types::OperatingSystem::Windows10,
    ));
    ClientProfile::new(
        ClientHelloID::new("Edge", "134", fingerprint_tls::tls_config::chrome_133_spec),
        settings,
        settings_order,
        chrome_pseudo_header_order(),
        fingerprint_headers::http2_config::CHROME_CONNECTION_FLOW,
        Vec::new(),
        Some(chrome_header_priority()),
        default_tcp_profile,
        chrome_header_order(),
    )
}

/// Chrome Mobile 134 fingerprint configuration (2026 Android version)
pub fn chrome_mobile_134() -> ClientProfile {
    let (settings, settings_order) = chrome_http2_settings();
    let default_tcp_profile = Some(TcpProfile::for_os(
        fingerprint_core::types::OperatingSystem::Linux,
    ));
    ClientProfile::new(
        ClientHelloID::new(
            "Chrome Mobile",
            "134",
            fingerprint_tls::tls_config::chrome_133_spec,
        ),
        settings,
        settings_order,
        chrome_pseudo_header_order(),
        fingerprint_headers::http2_config::CHROME_CONNECTION_FLOW,
        Vec::new(),
        Some(chrome_header_priority()),
        default_tcp_profile,
        chrome_header_order(),
    )
}

// ============== New Enhanced Versions (60+ new versions) ==============

/// Chrome 120 fingerprint configuration
pub fn chrome_120() -> ClientProfile {
    let (settings, settings_order) = chrome_http2_settings();
    let tcp = Some(TcpProfile::for_os(
        fingerprint_core::types::OperatingSystem::Windows10,
    ));
    ClientProfile::new(
        ClientHelloID::new(
            "Chrome",
            "120",
            fingerprint_tls::tls_config::chrome_133_spec,
        ),
        settings,
        settings_order,
        chrome_pseudo_header_order(),
        fingerprint_headers::http2_config::CHROME_CONNECTION_FLOW,
        Vec::new(),
        Some(chrome_header_priority()),
        tcp,
        chrome_header_order(),
    )
}

/// Chrome 121 fingerprint configuration
pub fn chrome_121() -> ClientProfile {
    let (settings, settings_order) = chrome_http2_settings();
    let tcp = Some(TcpProfile::for_os(
        fingerprint_core::types::OperatingSystem::Windows10,
    ));
    ClientProfile::new(
        ClientHelloID::new(
            "Chrome",
            "121",
            fingerprint_tls::tls_config::chrome_133_spec,
        ),
        settings,
        settings_order,
        chrome_pseudo_header_order(),
        fingerprint_headers::http2_config::CHROME_CONNECTION_FLOW,
        Vec::new(),
        Some(chrome_header_priority()),
        tcp,
        chrome_header_order(),
    )
}

/// Chrome 122 fingerprint configuration
pub fn chrome_122() -> ClientProfile {
    let (settings, settings_order) = chrome_http2_settings();
    let tcp = Some(TcpProfile::for_os(
        fingerprint_core::types::OperatingSystem::Windows10,
    ));
    ClientProfile::new(
        ClientHelloID::new(
            "Chrome",
            "122",
            fingerprint_tls::tls_config::chrome_133_spec,
        ),
        settings,
        settings_order,
        chrome_pseudo_header_order(),
        fingerprint_headers::http2_config::CHROME_CONNECTION_FLOW,
        Vec::new(),
        Some(chrome_header_priority()),
        tcp,
        chrome_header_order(),
    )
}

/// Chrome 123 fingerprint configuration
pub fn chrome_123() -> ClientProfile {
    let (settings, settings_order) = chrome_http2_settings();
    let tcp = Some(TcpProfile::for_os(
        fingerprint_core::types::OperatingSystem::Windows10,
    ));
    ClientProfile::new(
        ClientHelloID::new(
            "Chrome",
            "123",
            fingerprint_tls::tls_config::chrome_133_spec,
        ),
        settings,
        settings_order,
        chrome_pseudo_header_order(),
        fingerprint_headers::http2_config::CHROME_CONNECTION_FLOW,
        Vec::new(),
        Some(chrome_header_priority()),
        tcp,
        chrome_header_order(),
    )
}

/// Chrome 124 fingerprint configuration
pub fn chrome_124() -> ClientProfile {
    let (settings, settings_order) = chrome_http2_settings();
    let tcp = Some(TcpProfile::for_os(
        fingerprint_core::types::OperatingSystem::Windows10,
    ));
    ClientProfile::new(
        ClientHelloID::new(
            "Chrome",
            "124",
            fingerprint_tls::tls_config::chrome_133_spec,
        ),
        settings,
        settings_order,
        chrome_pseudo_header_order(),
        fingerprint_headers::http2_config::CHROME_CONNECTION_FLOW,
        Vec::new(),
        Some(chrome_header_priority()),
        tcp,
        chrome_header_order(),
    )
}

/// Chrome 125 fingerprint configuration
pub fn chrome_125() -> ClientProfile {
    let (settings, settings_order) = chrome_http2_settings();
    let tcp = Some(TcpProfile::for_os(
        fingerprint_core::types::OperatingSystem::Windows10,
    ));
    ClientProfile::new(
        ClientHelloID::new(
            "Chrome",
            "125",
            fingerprint_tls::tls_config::chrome_133_spec,
        ),
        settings,
        settings_order,
        chrome_pseudo_header_order(),
        fingerprint_headers::http2_config::CHROME_CONNECTION_FLOW,
        Vec::new(),
        Some(chrome_header_priority()),
        tcp,
        chrome_header_order(),
    )
}

/// Chrome 126 fingerprint configuration
pub fn chrome_126() -> ClientProfile {
    let (settings, settings_order) = chrome_http2_settings();
    let tcp = Some(TcpProfile::for_os(
        fingerprint_core::types::OperatingSystem::Windows10,
    ));
    ClientProfile::new(
        ClientHelloID::new(
            "Chrome",
            "126",
            fingerprint_tls::tls_config::chrome_133_spec,
        ),
        settings,
        settings_order,
        chrome_pseudo_header_order(),
        fingerprint_headers::http2_config::CHROME_CONNECTION_FLOW,
        Vec::new(),
        Some(chrome_header_priority()),
        tcp,
        chrome_header_order(),
    )
}

/// Chrome 127 fingerprint configuration
pub fn chrome_127() -> ClientProfile {
    let (settings, settings_order) = chrome_http2_settings();
    let tcp = Some(TcpProfile::for_os(
        fingerprint_core::types::OperatingSystem::Windows10,
    ));
    ClientProfile::new(
        ClientHelloID::new(
            "Chrome",
            "127",
            fingerprint_tls::tls_config::chrome_133_spec,
        ),
        settings,
        settings_order,
        chrome_pseudo_header_order(),
        fingerprint_headers::http2_config::CHROME_CONNECTION_FLOW,
        Vec::new(),
        Some(chrome_header_priority()),
        tcp,
        chrome_header_order(),
    )
}

/// Chrome 128 fingerprint configuration
pub fn chrome_128() -> ClientProfile {
    let (settings, settings_order) = chrome_http2_settings();
    let tcp = Some(TcpProfile::for_os(
        fingerprint_core::types::OperatingSystem::Windows10,
    ));
    ClientProfile::new(
        ClientHelloID::new(
            "Chrome",
            "128",
            fingerprint_tls::tls_config::chrome_133_spec,
        ),
        settings,
        settings_order,
        chrome_pseudo_header_order(),
        fingerprint_headers::http2_config::CHROME_CONNECTION_FLOW,
        Vec::new(),
        Some(chrome_header_priority()),
        tcp,
        chrome_header_order(),
    )
}

/// Chrome 129 fingerprint configuration
pub fn chrome_129() -> ClientProfile {
    let (settings, settings_order) = chrome_http2_settings();
    let tcp = Some(TcpProfile::for_os(
        fingerprint_core::types::OperatingSystem::Windows10,
    ));
    ClientProfile::new(
        ClientHelloID::new(
            "Chrome",
            "129",
            fingerprint_tls::tls_config::chrome_133_spec,
        ),
        settings,
        settings_order,
        chrome_pseudo_header_order(),
        fingerprint_headers::http2_config::CHROME_CONNECTION_FLOW,
        Vec::new(),
        Some(chrome_header_priority()),
        tcp,
        chrome_header_order(),
    )
}

/// Chrome 130 fingerprint configuration
pub fn chrome_130() -> ClientProfile {
    let (settings, settings_order) = chrome_http2_settings();
    let tcp = Some(TcpProfile::for_os(
        fingerprint_core::types::OperatingSystem::Windows10,
    ));
    ClientProfile::new(
        ClientHelloID::new(
            "Chrome",
            "130",
            fingerprint_tls::tls_config::chrome_133_spec,
        ),
        settings,
        settings_order,
        chrome_pseudo_header_order(),
        fingerprint_headers::http2_config::CHROME_CONNECTION_FLOW,
        Vec::new(),
        Some(chrome_header_priority()),
        tcp,
        chrome_header_order(),
    )
}

/// Chrome 131 fingerprint configuration
pub fn chrome_131() -> ClientProfile {
    let (settings, settings_order) = chrome_http2_settings();
    let tcp = Some(TcpProfile::for_os(
        fingerprint_core::types::OperatingSystem::Windows10,
    ));
    ClientProfile::new(
        ClientHelloID::new(
            "Chrome",
            "131",
            fingerprint_tls::tls_config::chrome_133_spec,
        ),
        settings,
        settings_order,
        chrome_pseudo_header_order(),
        fingerprint_headers::http2_config::CHROME_CONNECTION_FLOW,
        Vec::new(),
        Some(chrome_header_priority()),
        tcp,
        chrome_header_order(),
    )
}

/// Chrome 132 fingerprint configuration
pub fn chrome_132() -> ClientProfile {
    let (settings, settings_order) = chrome_http2_settings();
    let tcp = Some(TcpProfile::for_os(
        fingerprint_core::types::OperatingSystem::Windows10,
    ));
    ClientProfile::new(
        ClientHelloID::new(
            "Chrome",
            "132",
            fingerprint_tls::tls_config::chrome_133_spec,
        ),
        settings,
        settings_order,
        chrome_pseudo_header_order(),
        fingerprint_headers::http2_config::CHROME_CONNECTION_FLOW,
        Vec::new(),
        Some(chrome_header_priority()),
        tcp,
        chrome_header_order(),
    )
}

/// Chrome 137 fingerprint configuration (latest 2026)
pub fn chrome_137() -> ClientProfile {
    let (settings, settings_order) = chrome_http2_settings();
    let tcp = Some(TcpProfile::for_os(
        fingerprint_core::types::OperatingSystem::Windows10,
    ));
    ClientProfile::new(
        ClientHelloID::new(
            "Chrome",
            "137",
            fingerprint_tls::tls_config::chrome_136_spec,
        ),
        settings,
        settings_order,
        chrome_pseudo_header_order(),
        fingerprint_headers::http2_config::CHROME_CONNECTION_FLOW,
        Vec::new(),
        Some(chrome_header_priority()),
        tcp,
        chrome_header_order(),
    )
}

/// Chrome 138 fingerprint configuration (latest beta)
pub fn chrome_138() -> ClientProfile {
    let (settings, settings_order) = chrome_http2_settings();
    let tcp = Some(TcpProfile::for_os(
        fingerprint_core::types::OperatingSystem::Windows10,
    ));
    ClientProfile::new(
        ClientHelloID::new(
            "Chrome",
            "138",
            fingerprint_tls::tls_config::chrome_136_spec,
        ),
        settings,
        settings_order,
        chrome_pseudo_header_order(),
        fingerprint_headers::http2_config::CHROME_CONNECTION_FLOW,
        Vec::new(),
        Some(chrome_header_priority()),
        tcp,
        chrome_header_order(),
    )
}

/// Firefox 130 fingerprint configuration
pub fn firefox_130() -> ClientProfile {
    let (settings, settings_order) = firefox_http2_settings();
    let tcp = Some(TcpProfile::for_os(
        fingerprint_core::types::OperatingSystem::Windows10,
    ));
    ClientProfile::new(
        ClientHelloID::new(
            "Firefox",
            "130",
            fingerprint_tls::tls_config::firefox_133_spec,
        ),
        settings,
        settings_order,
        firefox_pseudo_header_order(),
        fingerprint_headers::http2_config::CHROME_CONNECTION_FLOW,
        Vec::new(),
        None,
        tcp,
        firefox_header_order(),
    )
}

/// Firefox 131 fingerprint configuration
pub fn firefox_131() -> ClientProfile {
    let (settings, settings_order) = firefox_http2_settings();
    let tcp = Some(TcpProfile::for_os(
        fingerprint_core::types::OperatingSystem::Windows10,
    ));
    ClientProfile::new(
        ClientHelloID::new(
            "Firefox",
            "131",
            fingerprint_tls::tls_config::firefox_133_spec,
        ),
        settings,
        settings_order,
        firefox_pseudo_header_order(),
        fingerprint_headers::http2_config::CHROME_CONNECTION_FLOW,
        Vec::new(),
        None,
        tcp,
        firefox_header_order(),
    )
}

/// Firefox 132 fingerprint configuration
pub fn firefox_132() -> ClientProfile {
    let (settings, settings_order) = firefox_http2_settings();
    let tcp = Some(TcpProfile::for_os(
        fingerprint_core::types::OperatingSystem::Windows10,
    ));
    ClientProfile::new(
        ClientHelloID::new(
            "Firefox",
            "132",
            fingerprint_tls::tls_config::firefox_133_spec,
        ),
        settings,
        settings_order,
        firefox_pseudo_header_order(),
        fingerprint_headers::http2_config::CHROME_CONNECTION_FLOW,
        Vec::new(),
        None,
        tcp,
        firefox_header_order(),
    )
}

/// Firefox 137 fingerprint configuration (latest beta)
pub fn firefox_137() -> ClientProfile {
    let (settings, settings_order) = firefox_http2_settings();
    let tcp = Some(TcpProfile::for_os(
        fingerprint_core::types::OperatingSystem::Windows10,
    ));
    ClientProfile::new(
        ClientHelloID::new(
            "Firefox",
            "137",
            fingerprint_tls::tls_config::firefox_133_spec,
        ),
        settings,
        settings_order,
        firefox_pseudo_header_order(),
        fingerprint_headers::http2_config::CHROME_CONNECTION_FLOW,
        Vec::new(),
        None,
        tcp,
        firefox_header_order(),
    )
}

/// Firefox 138 fingerprint configuration (nightly)
pub fn firefox_138() -> ClientProfile {
    let (settings, settings_order) = firefox_http2_settings();
    let tcp = Some(TcpProfile::for_os(
        fingerprint_core::types::OperatingSystem::Windows10,
    ));
    ClientProfile::new(
        ClientHelloID::new(
            "Firefox",
            "138",
            fingerprint_tls::tls_config::firefox_133_spec,
        ),
        settings,
        settings_order,
        firefox_pseudo_header_order(),
        fingerprint_headers::http2_config::CHROME_CONNECTION_FLOW,
        Vec::new(),
        None,
        tcp,
        firefox_header_order(),
    )
}

/// Safari 15.0 fingerprint configuration
pub fn safari_15_0() -> ClientProfile {
    let (settings, settings_order) = safari_http2_settings();
    ClientProfile::new(
        ClientHelloID::new(
            "Safari",
            "15.0",
            fingerprint_tls::tls_config::safari_16_0_spec,
        ),
        settings,
        settings_order,
        safari_pseudo_header_order(),
        fingerprint_headers::http2_config::CHROME_CONNECTION_FLOW,
        Vec::new(),
        None,
        None,
        safari_header_order(),
    )
}

/// Safari 15.7 fingerprint configuration
pub fn safari_15_7() -> ClientProfile {
    let (settings, settings_order) = safari_http2_settings();
    ClientProfile::new(
        ClientHelloID::new(
            "Safari",
            "15.7",
            fingerprint_tls::tls_config::safari_16_0_spec,
        ),
        settings,
        settings_order,
        safari_pseudo_header_order(),
        fingerprint_headers::http2_config::CHROME_CONNECTION_FLOW,
        Vec::new(),
        None,
        None,
        safari_header_order(),
    )
}

/// Safari 17.0 fingerprint configuration
pub fn safari_17_0() -> ClientProfile {
    let (settings, settings_order) = safari_http2_settings();
    let tcp = Some(TcpProfile::for_os(
        fingerprint_core::types::OperatingSystem::MacOS13,
    ));
    ClientProfile::new(
        ClientHelloID::new(
            "Safari",
            "17.0",
            fingerprint_tls::tls_config::safari_16_0_spec,
        ),
        settings,
        settings_order,
        safari_pseudo_header_order(),
        fingerprint_headers::http2_config::CHROME_CONNECTION_FLOW,
        Vec::new(),
        None,
        tcp,
        safari_header_order(),
    )
}

/// Safari 17.5 fingerprint configuration
pub fn safari_17_5() -> ClientProfile {
    let (settings, settings_order) = safari_http2_settings();
    let tcp = Some(TcpProfile::for_os(
        fingerprint_core::types::OperatingSystem::MacOS13,
    ));
    ClientProfile::new(
        ClientHelloID::new(
            "Safari",
            "17.5",
            fingerprint_tls::tls_config::safari_16_0_spec,
        ),
        settings,
        settings_order,
        safari_pseudo_header_order(),
        fingerprint_headers::http2_config::CHROME_CONNECTION_FLOW,
        Vec::new(),
        None,
        tcp,
        safari_header_order(),
    )
}

/// Safari 18.0 fingerprint configuration
pub fn safari_18_0() -> ClientProfile {
    let (settings, settings_order) = safari_http2_settings();
    let tcp = Some(TcpProfile::for_os(
        fingerprint_core::types::OperatingSystem::MacOS14,
    ));
    ClientProfile::new(
        ClientHelloID::new(
            "Safari",
            "18.0",
            fingerprint_tls::tls_config::safari_16_0_spec,
        ),
        settings,
        settings_order,
        safari_pseudo_header_order(),
        fingerprint_headers::http2_config::CHROME_CONNECTION_FLOW,
        Vec::new(),
        None,
        tcp,
        safari_header_order(),
    )
}

/// Safari 18.1 fingerprint configuration
pub fn safari_18_1() -> ClientProfile {
    let (settings, settings_order) = safari_http2_settings();
    let tcp = Some(TcpProfile::for_os(
        fingerprint_core::types::OperatingSystem::MacOS14,
    ));
    ClientProfile::new(
        ClientHelloID::new(
            "Safari",
            "18.1",
            fingerprint_tls::tls_config::safari_16_0_spec,
        ),
        settings,
        settings_order,
        safari_pseudo_header_order(),
        fingerprint_headers::http2_config::CHROME_CONNECTION_FLOW,
        Vec::new(),
        None,
        tcp,
        safari_header_order(),
    )
}

/// Safari 18.3 fingerprint configuration
pub fn safari_18_3() -> ClientProfile {
    let (settings, settings_order) = safari_http2_settings();
    let tcp = Some(TcpProfile::for_os(
        fingerprint_core::types::OperatingSystem::MacOS14,
    ));
    ClientProfile::new(
        ClientHelloID::new(
            "Safari",
            "18.3",
            fingerprint_tls::tls_config::safari_16_0_spec,
        ),
        settings,
        settings_order,
        safari_pseudo_header_order(),
        fingerprint_headers::http2_config::CHROME_CONNECTION_FLOW,
        Vec::new(),
        None,
        tcp,
        safari_header_order(),
    )
}

/// Edge 125 fingerprint configuration
pub fn edge_125() -> ClientProfile {
    let (settings, settings_order) = chrome_http2_settings();
    let tcp = Some(TcpProfile::for_os(
        fingerprint_core::types::OperatingSystem::Windows10,
    ));
    ClientProfile::new(
        ClientHelloID::new("Edge", "125", fingerprint_tls::tls_config::chrome_133_spec),
        settings,
        settings_order,
        chrome_pseudo_header_order(),
        fingerprint_headers::http2_config::CHROME_CONNECTION_FLOW,
        Vec::new(),
        Some(chrome_header_priority()),
        tcp,
        chrome_header_order(),
    )
}

/// Edge 126 fingerprint configuration
pub fn edge_126() -> ClientProfile {
    let (settings, settings_order) = chrome_http2_settings();
    let tcp = Some(TcpProfile::for_os(
        fingerprint_core::types::OperatingSystem::Windows10,
    ));
    ClientProfile::new(
        ClientHelloID::new("Edge", "126", fingerprint_tls::tls_config::chrome_133_spec),
        settings,
        settings_order,
        chrome_pseudo_header_order(),
        fingerprint_headers::http2_config::CHROME_CONNECTION_FLOW,
        Vec::new(),
        Some(chrome_header_priority()),
        tcp,
        chrome_header_order(),
    )
}

/// Edge 130 fingerprint configuration
pub fn edge_130() -> ClientProfile {
    let (settings, settings_order) = chrome_http2_settings();
    let tcp = Some(TcpProfile::for_os(
        fingerprint_core::types::OperatingSystem::Windows10,
    ));
    ClientProfile::new(
        ClientHelloID::new("Edge", "130", fingerprint_tls::tls_config::chrome_133_spec),
        settings,
        settings_order,
        chrome_pseudo_header_order(),
        fingerprint_headers::http2_config::CHROME_CONNECTION_FLOW,
        Vec::new(),
        Some(chrome_header_priority()),
        tcp,
        chrome_header_order(),
    )
}

/// Edge 131 fingerprint configuration
pub fn edge_131() -> ClientProfile {
    let (settings, settings_order) = chrome_http2_settings();
    let tcp = Some(TcpProfile::for_os(
        fingerprint_core::types::OperatingSystem::Windows10,
    ));
    ClientProfile::new(
        ClientHelloID::new("Edge", "131", fingerprint_tls::tls_config::chrome_133_spec),
        settings,
        settings_order,
        chrome_pseudo_header_order(),
        fingerprint_headers::http2_config::CHROME_CONNECTION_FLOW,
        Vec::new(),
        Some(chrome_header_priority()),
        tcp,
        chrome_header_order(),
    )
}

/// Edge 132 fingerprint configuration
pub fn edge_132() -> ClientProfile {
    let (settings, settings_order) = chrome_http2_settings();
    let tcp = Some(TcpProfile::for_os(
        fingerprint_core::types::OperatingSystem::Windows10,
    ));
    ClientProfile::new(
        ClientHelloID::new("Edge", "132", fingerprint_tls::tls_config::chrome_133_spec),
        settings,
        settings_order,
        chrome_pseudo_header_order(),
        fingerprint_headers::http2_config::CHROME_CONNECTION_FLOW,
        Vec::new(),
        Some(chrome_header_priority()),
        tcp,
        chrome_header_order(),
    )
}

/// Edge 135 fingerprint configuration
pub fn edge_135() -> ClientProfile {
    let (settings, settings_order) = chrome_http2_settings();
    let tcp = Some(TcpProfile::for_os(
        fingerprint_core::types::OperatingSystem::Windows10,
    ));
    ClientProfile::new(
        ClientHelloID::new("Edge", "135", fingerprint_tls::tls_config::chrome_136_spec),
        settings,
        settings_order,
        chrome_pseudo_header_order(),
        fingerprint_headers::http2_config::CHROME_CONNECTION_FLOW,
        Vec::new(),
        Some(chrome_header_priority()),
        tcp,
        chrome_header_order(),
    )
}

/// Edge 137 fingerprint configuration
pub fn edge_137() -> ClientProfile {
    let (settings, settings_order) = chrome_http2_settings();
    let tcp = Some(TcpProfile::for_os(
        fingerprint_core::types::OperatingSystem::Windows10,
    ));
    ClientProfile::new(
        ClientHelloID::new("Edge", "137", fingerprint_tls::tls_config::chrome_136_spec),
        settings,
        settings_order,
        chrome_pseudo_header_order(),
        fingerprint_headers::http2_config::CHROME_CONNECTION_FLOW,
        Vec::new(),
        Some(chrome_header_priority()),
        tcp,
        chrome_header_order(),
    )
}

/// Opera 92 fingerprint configuration
pub fn opera_92() -> ClientProfile {
    let (settings, settings_order) = chrome_http2_settings();
    let tcp = Some(TcpProfile::for_os(
        fingerprint_core::types::OperatingSystem::Windows10,
    ));
    ClientProfile::new(
        ClientHelloID::new("Opera", "92", fingerprint_tls::tls_config::chrome_133_spec),
        settings,
        settings_order,
        chrome_pseudo_header_order(),
        fingerprint_headers::http2_config::CHROME_CONNECTION_FLOW,
        Vec::new(),
        Some(chrome_header_priority()),
        tcp,
        chrome_header_order(),
    )
}

/// Opera 93 fingerprint configuration
pub fn opera_93() -> ClientProfile {
    let (settings, settings_order) = chrome_http2_settings();
    let tcp = Some(TcpProfile::for_os(
        fingerprint_core::types::OperatingSystem::Windows10,
    ));
    ClientProfile::new(
        ClientHelloID::new("Opera", "93", fingerprint_tls::tls_config::chrome_133_spec),
        settings,
        settings_order,
        chrome_pseudo_header_order(),
        fingerprint_headers::http2_config::CHROME_CONNECTION_FLOW,
        Vec::new(),
        Some(chrome_header_priority()),
        tcp,
        chrome_header_order(),
    )
}

/// Opera 94 fingerprint configuration
pub fn opera_94() -> ClientProfile {
    let (settings, settings_order) = chrome_http2_settings();
    let tcp = Some(TcpProfile::for_os(
        fingerprint_core::types::OperatingSystem::Windows10,
    ));
    ClientProfile::new(
        ClientHelloID::new("Opera", "94", fingerprint_tls::tls_config::chrome_136_spec),
        settings,
        settings_order,
        chrome_pseudo_header_order(),
        fingerprint_headers::http2_config::CHROME_CONNECTION_FLOW,
        Vec::new(),
        Some(chrome_header_priority()),
        tcp,
        chrome_header_order(),
    )
}

/// Chrome Mobile 120 fingerprint configuration
pub fn chrome_mobile_120() -> ClientProfile {
    let (settings, settings_order) = chrome_http2_settings();
    let tcp = Some(TcpProfile::for_os(
        fingerprint_core::types::OperatingSystem::Linux,
    ));
    ClientProfile::new(
        ClientHelloID::new(
            "Chrome Mobile",
            "120",
            fingerprint_tls::tls_config::chrome_133_spec,
        ),
        settings,
        settings_order,
        chrome_pseudo_header_order(),
        fingerprint_headers::http2_config::CHROME_CONNECTION_FLOW,
        Vec::new(),
        Some(chrome_header_priority()),
        tcp,
        chrome_header_order(),
    )
}

/// Chrome Mobile 130 fingerprint configuration
pub fn chrome_mobile_130() -> ClientProfile {
    let (settings, settings_order) = chrome_http2_settings();
    let tcp = Some(TcpProfile::for_os(
        fingerprint_core::types::OperatingSystem::Linux,
    ));
    ClientProfile::new(
        ClientHelloID::new(
            "Chrome Mobile",
            "130",
            fingerprint_tls::tls_config::chrome_133_spec,
        ),
        settings,
        settings_order,
        chrome_pseudo_header_order(),
        fingerprint_headers::http2_config::CHROME_CONNECTION_FLOW,
        Vec::new(),
        Some(chrome_header_priority()),
        tcp,
        chrome_header_order(),
    )
}

/// Chrome Mobile 135 fingerprint configuration
pub fn chrome_mobile_135() -> ClientProfile {
    let (settings, settings_order) = chrome_http2_settings();
    let tcp = Some(TcpProfile::for_os(
        fingerprint_core::types::OperatingSystem::Linux,
    ));
    ClientProfile::new(
        ClientHelloID::new(
            "Chrome Mobile",
            "135",
            fingerprint_tls::tls_config::chrome_133_spec,
        ),
        settings,
        settings_order,
        chrome_pseudo_header_order(),
        fingerprint_headers::http2_config::CHROME_CONNECTION_FLOW,
        Vec::new(),
        Some(chrome_header_priority()),
        tcp,
        chrome_header_order(),
    )
}

/// Chrome Mobile 137 fingerprint configuration
pub fn chrome_mobile_137() -> ClientProfile {
    let (settings, settings_order) = chrome_http2_settings();
    let tcp = Some(TcpProfile::for_os(
        fingerprint_core::types::OperatingSystem::Linux,
    ));
    ClientProfile::new(
        ClientHelloID::new(
            "Chrome Mobile",
            "137",
            fingerprint_tls::tls_config::chrome_136_spec,
        ),
        settings,
        settings_order,
        chrome_pseudo_header_order(),
        fingerprint_headers::http2_config::CHROME_CONNECTION_FLOW,
        Vec::new(),
        Some(chrome_header_priority()),
        tcp,
        chrome_header_order(),
    )
}

/// Firefox Mobile 120 fingerprint configuration
pub fn firefox_mobile_120() -> ClientProfile {
    let (settings, settings_order) = firefox_http2_settings();
    let tcp = Some(TcpProfile::for_os(
        fingerprint_core::types::OperatingSystem::Linux,
    ));
    ClientProfile::new(
        ClientHelloID::new(
            "Firefox Mobile",
            "120",
            fingerprint_tls::tls_config::firefox_133_spec,
        ),
        settings,
        settings_order,
        firefox_pseudo_header_order(),
        fingerprint_headers::http2_config::CHROME_CONNECTION_FLOW,
        Vec::new(),
        None,
        tcp,
        firefox_header_order(),
    )
}

/// Firefox Mobile 130 fingerprint configuration
pub fn firefox_mobile_130() -> ClientProfile {
    let (settings, settings_order) = firefox_http2_settings();
    let tcp = Some(TcpProfile::for_os(
        fingerprint_core::types::OperatingSystem::Linux,
    ));
    ClientProfile::new(
        ClientHelloID::new(
            "Firefox Mobile",
            "130",
            fingerprint_tls::tls_config::firefox_133_spec,
        ),
        settings,
        settings_order,
        firefox_pseudo_header_order(),
        fingerprint_headers::http2_config::CHROME_CONNECTION_FLOW,
        Vec::new(),
        None,
        tcp,
        firefox_header_order(),
    )
}

/// Firefox Mobile 135 fingerprint configuration
pub fn firefox_mobile_135() -> ClientProfile {
    let (settings, settings_order) = firefox_http2_settings();
    let tcp = Some(TcpProfile::for_os(
        fingerprint_core::types::OperatingSystem::Linux,
    ));
    ClientProfile::new(
        ClientHelloID::new(
            "Firefox Mobile",
            "135",
            fingerprint_tls::tls_config::firefox_133_spec,
        ),
        settings,
        settings_order,
        firefox_pseudo_header_order(),
        fingerprint_headers::http2_config::CHROME_CONNECTION_FLOW,
        Vec::new(),
        None,
        tcp,
        firefox_header_order(),
    )
}

/// Safari iOS 16.0 fingerprint configuration
pub fn safari_ios_16_0() -> ClientProfile {
    let (settings, settings_order) = safari_http2_settings();
    let tcp = Some(TcpProfile::for_os(
        fingerprint_core::types::OperatingSystem::MacOS13,
    ));
    ClientProfile::new(
        ClientHelloID::new(
            "Safari iOS",
            "16.0",
            fingerprint_tls::tls_config::safari_16_0_spec,
        ),
        settings,
        settings_order,
        safari_pseudo_header_order(),
        fingerprint_headers::http2_config::CHROME_CONNECTION_FLOW,
        Vec::new(),
        None,
        tcp,
        safari_header_order(),
    )
}

/// Safari iOS 17.0 fingerprint configuration
pub fn safari_ios_17_0() -> ClientProfile {
    let (settings, settings_order) = safari_http2_settings();
    let tcp = Some(TcpProfile::for_os(
        fingerprint_core::types::OperatingSystem::MacOS13,
    ));
    ClientProfile::new(
        ClientHelloID::new(
            "Safari iOS",
            "17.0",
            fingerprint_tls::tls_config::safari_16_0_spec,
        ),
        settings,
        settings_order,
        safari_pseudo_header_order(),
        fingerprint_headers::http2_config::CHROME_CONNECTION_FLOW,
        Vec::new(),
        None,
        tcp,
        safari_header_order(),
    )
}

/// Safari iOS 18.0 fingerprint configuration
pub fn safari_ios_18_0() -> ClientProfile {
    let (settings, settings_order) = safari_http2_settings();
    let tcp = Some(TcpProfile::for_os(
        fingerprint_core::types::OperatingSystem::MacOS14,
    ));
    ClientProfile::new(
        ClientHelloID::new(
            "Safari iOS",
            "18.0",
            fingerprint_tls::tls_config::safari_16_0_spec,
        ),
        settings,
        settings_order,
        safari_pseudo_header_order(),
        fingerprint_headers::http2_config::CHROME_CONNECTION_FLOW,
        Vec::new(),
        None,
        tcp,
        safari_header_order(),
    )
}

/// Safari iOS 18.1 fingerprint configuration
pub fn safari_ios_18_1() -> ClientProfile {
    let (settings, settings_order) = safari_http2_settings();
    let tcp = Some(TcpProfile::for_os(
        fingerprint_core::types::OperatingSystem::MacOS14,
    ));
    ClientProfile::new(
        ClientHelloID::new(
            "Safari iOS",
            "18.1",
            fingerprint_tls::tls_config::safari_16_0_spec,
        ),
        settings,
        settings_order,
        safari_pseudo_header_order(),
        fingerprint_headers::http2_config::CHROME_CONNECTION_FLOW,
        Vec::new(),
        None,
        tcp,
        safari_header_order(),
    )
}

/// Safari iOS 18.3 fingerprint configuration
pub fn safari_ios_18_3() -> ClientProfile {
    let (settings, settings_order) = safari_http2_settings();
    let tcp = Some(TcpProfile::for_os(
        fingerprint_core::types::OperatingSystem::MacOS14,
    ));
    ClientProfile::new(
        ClientHelloID::new(
            "Safari iOS",
            "18.3",
            fingerprint_tls::tls_config::safari_16_0_spec,
        ),
        settings,
        settings_order,
        safari_pseudo_header_order(),
        fingerprint_headers::http2_config::CHROME_CONNECTION_FLOW,
        Vec::new(),
        None,
        tcp,
        safari_header_order(),
    )
}

/// Initializeallfingerprintconfigurationmaptable
fn init_mapped_tls_clients() -> HashMap<String, ClientProfile> {
    let mut map = HashMap::new();

    // Chrome series (ENHANCED: 14+ versions)
    map.insert("chrome_103".to_string(), chrome_133()); // simplify：use chrome_133
    map.insert("chrome_104".to_string(), chrome_133());
    map.insert("chrome_105".to_string(), chrome_133());
    map.insert("chrome_106".to_string(), chrome_133());
    map.insert("chrome_107".to_string(), chrome_133());
    map.insert("chrome_108".to_string(), chrome_133());
    map.insert("chrome_109".to_string(), chrome_133());
    map.insert("chrome_110".to_string(), chrome_133());
    map.insert("chrome_111".to_string(), chrome_133());
    map.insert("chrome_112".to_string(), chrome_133());
    map.insert("chrome_116_PSK".to_string(), chrome_133());
    map.insert("chrome_116_PSK_PQ".to_string(), chrome_133());
    map.insert("chrome_117".to_string(), chrome_133());
    // NEW: versions 120-132
    map.insert("chrome_120".to_string(), chrome_120());
    map.insert("chrome_121".to_string(), chrome_121());
    map.insert("chrome_122".to_string(), chrome_122());
    map.insert("chrome_123".to_string(), chrome_123());
    map.insert("chrome_124".to_string(), chrome_124());
    map.insert("chrome_125".to_string(), chrome_125());
    map.insert("chrome_126".to_string(), chrome_126());
    map.insert("chrome_127".to_string(), chrome_127());
    map.insert("chrome_128".to_string(), chrome_128());
    map.insert("chrome_129".to_string(), chrome_129());
    map.insert("chrome_130".to_string(), chrome_130());
    map.insert("chrome_130_PSK".to_string(), chrome_130());
    map.insert("chrome_131".to_string(), chrome_131());
    map.insert("chrome_131_PSK".to_string(), chrome_131());
    map.insert("chrome_132".to_string(), chrome_132());
    // Core versions
    map.insert("chrome_133".to_string(), chrome_133());
    map.insert("chrome_133_PSK".to_string(), chrome_133());
    map.insert("chrome_134".to_string(), chrome_134());
    map.insert("chrome_135".to_string(), chrome_135());
    map.insert("chrome_136".to_string(), chrome_136());
    // NEW: future versions 137-138
    map.insert("chrome_137".to_string(), chrome_137());
    map.insert("chrome_138".to_string(), chrome_138());

    // Safari series (ENHANCED: 15.x, 17.x, 18.x expanded)
    map.insert("safari_15_0".to_string(), safari_15_0());
    map.insert("safari_15_6_1".to_string(), safari_15_7());
    map.insert("safari_15_7".to_string(), safari_15_7());
    map.insert("safari_16_0".to_string(), safari_16_0());
    map.insert("safari_17_0".to_string(), safari_17_0());
    map.insert("safari_17_5".to_string(), safari_17_5());
    map.insert("safari_18_0".to_string(), safari_18_0());
    map.insert("safari_18_1".to_string(), safari_18_1());
    map.insert("safari_18_2".to_string(), safari_18_2());
    map.insert("safari_18_3".to_string(), safari_18_3());
    map.insert("safari_ipad_15_6".to_string(), safari_15_7());
    map.insert("safari_ios_15_5".to_string(), safari_ios_16_0());
    map.insert("safari_ios_15_6".to_string(), safari_ios_16_0());
    map.insert("safari_ios_16_0".to_string(), safari_ios_16_0());
    map.insert("safari_ios_17_0".to_string(), safari_ios_17_0());
    map.insert("safari_ios_18_0".to_string(), safari_ios_18_0());
    map.insert("safari_ios_18_1".to_string(), safari_ios_18_1());
    map.insert("safari_ios_18_2".to_string(), safari_ios_18_2());
    map.insert("safari_ios_18_3".to_string(), safari_ios_18_3());
    map.insert("safari_ios_18_5".to_string(), safari_ios_18_3());

    // Firefox series (ENHANCED: 130-138)
    map.insert("firefox_102".to_string(), firefox_133());
    map.insert("firefox_104".to_string(), firefox_133());
    map.insert("firefox_105".to_string(), firefox_133());
    map.insert("firefox_106".to_string(), firefox_133());
    map.insert("firefox_108".to_string(), firefox_133());
    map.insert("firefox_110".to_string(), firefox_133());
    map.insert("firefox_117".to_string(), firefox_133());
    map.insert("firefox_120".to_string(), firefox_133());
    map.insert("firefox_123".to_string(), firefox_133());
    map.insert("firefox_130".to_string(), firefox_130());
    map.insert("firefox_131".to_string(), firefox_131());
    map.insert("firefox_132".to_string(), firefox_132());
    map.insert("firefox_133".to_string(), firefox_133());
    map.insert("firefox_134".to_string(), firefox_134());
    map.insert("firefox_135".to_string(), firefox_135());
    map.insert("firefox_136".to_string(), firefox_136());
    // NEW: future versions 137-138
    map.insert("firefox_137".to_string(), firefox_137());
    map.insert("firefox_138".to_string(), firefox_138());

    // Opera series (ENHANCED: 92-94)
    map.insert("opera_89".to_string(), opera_91());
    map.insert("opera_90".to_string(), opera_91());
    map.insert("opera_91".to_string(), opera_91());
    map.insert("opera_92".to_string(), opera_92());
    map.insert("opera_93".to_string(), opera_93());
    map.insert("opera_94".to_string(), opera_94());

    // Edge series (ENHANCED: 125-137)
    map.insert("edge_120".to_string(), edge_120());
    map.insert("edge_125".to_string(), edge_125());
    map.insert("edge_126".to_string(), edge_126());
    map.insert("edge_124".to_string(), edge_124());
    map.insert("edge_130".to_string(), edge_130());
    map.insert("edge_131".to_string(), edge_131());
    map.insert("edge_132".to_string(), edge_132());
    map.insert("edge_133".to_string(), edge_133());
    map.insert("edge_134".to_string(), edge_134());
    map.insert("edge_135".to_string(), edge_135());
    map.insert("edge_137".to_string(), edge_137());

    // mobile and custom fingerprints (ENHANCED: 12+ mobile variants)
    map.insert("chrome_mobile_120".to_string(), chrome_mobile_120());
    map.insert("chrome_mobile_130".to_string(), chrome_mobile_130());
    map.insert("chrome_mobile_134".to_string(), chrome_mobile_134());
    map.insert("chrome_mobile_135".to_string(), chrome_mobile_135());
    map.insert("chrome_mobile_137".to_string(), chrome_mobile_137());
    map.insert("firefox_mobile_120".to_string(), firefox_mobile_120());
    map.insert("firefox_mobile_130".to_string(), firefox_mobile_130());
    map.insert("firefox_mobile_135".to_string(), firefox_mobile_135());
    map.insert("zalando_android_mobile".to_string(), chrome_mobile_130());
    map.insert("zalando_ios_mobile".to_string(), safari_ios_18_0());
    map.insert("nike_ios_mobile".to_string(), safari_ios_18_0());
    map.insert("nike_android_mobile".to_string(), chrome_mobile_130());
    map.insert("mms_ios".to_string(), safari_ios_18_0());
    map.insert("mms_ios_2".to_string(), safari_ios_18_1());
    map.insert("mms_ios_3".to_string(), safari_ios_18_3());
    map.insert("mesh_ios".to_string(), safari_ios_17_0());
    map.insert("mesh_android".to_string(), chrome_mobile_130());
    map.insert("mesh_ios_2".to_string(), safari_ios_18_0());
    map.insert("mesh_android_2".to_string(), chrome_mobile_130());
    map.insert("confirmed_ios".to_string(), safari_ios_18_0());
    map.insert("confirmed_android".to_string(), chrome_mobile_130());
    map.insert("confirmed_android_2".to_string(), chrome_mobile_135());
    map.insert("okhttp4_android_7".to_string(), chrome_mobile_120());
    map.insert("okhttp4_android_8".to_string(), chrome_mobile_120());
    map.insert("okhttp4_android_9".to_string(), chrome_mobile_130());
    map.insert("okhttp4_android_10".to_string(), chrome_mobile_130());
    map.insert("okhttp4_android_11".to_string(), chrome_mobile_135());
    map.insert("okhttp4_android_12".to_string(), chrome_mobile_135());
    map.insert("okhttp4_android_13".to_string(), chrome_mobile_137());
    map.insert("cloudflare_custom".to_string(), chrome_133());

    map
}

/// globalfingerprintconfigurationmaptable (threadsecurity)
static MAPPED_TLS_CLIENTS: OnceLock<HashMap<String, ClientProfile>> = OnceLock::new();

/// Getfingerprintconfigurationmaptable
pub fn mapped_tls_clients() -> &'static HashMap<String, ClientProfile> {
    MAPPED_TLS_CLIENTS.get_or_init(init_mapped_tls_clients)
}

/// Based on profile nameGet ClientProfile
///
/// # Parameters
/// - `profile_name`: fingerprintconfigurationname (如 "chrome_135", "firefox_133")
///
/// # Returns
/// returnpairshould ClientProfile,  if 不 existsthenreturnerror
pub fn get_client_profile(profile_name: &str) -> Result<ClientProfile, String> {
    let clients = mapped_tls_clients();
    clients
        .get(profile_name)
        .cloned()
        .ok_or_else(|| format!("Profile '{}' not found", profile_name))
}

/// unifiedfingerprintGeneratefunction
///
/// Based on profile name and User-Agent Generatesyncbrowserfingerprint and TCP fingerprint
///
/// # Parameters
/// - `profile_name`: fingerprintconfigurationname (如 "chrome_135", "firefox_133")
/// - `user_agent`: User-Agent string,  for sync TCP fingerprint
///
/// # Returns
/// returnan ClientProfile, 其 in tcp_profile alreadyBased on User-Agent sync
///
/// # Examples
/// ```rust
/// use fingerprint_profiles::profiles::generate_unified_fingerprint;
///
/// let user_agent = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/135.0.0.0 Safari/537.36";
/// let profile = generate_unified_fingerprint("chrome_135", user_agent).unwrap();
///
/// // profile.tcp_profile current in including and User-Agent match TCP fingerprint
/// // Windows -> TTL=128, Window Size=64240
/// ```
pub fn generate_unified_fingerprint(
    profile_name: &str,
    user_agent: &str,
) -> Result<ClientProfile, String> {
    let profile = get_client_profile(profile_name)?;

    // Based on User-Agent sync TCP Profile
    let synced_profile = profile.with_synced_tcp_profile(user_agent);

    Ok(synced_profile)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mapped_tls_clients() {
        let clients = mapped_tls_clients();
        assert!(!clients.is_empty());
        assert!(clients.contains_key("chrome_133"));
        assert!(clients.contains_key("firefox_133"));
    }

    #[test]
    fn test_client_profile() {
        let profile = chrome_133();
        assert_eq!(profile.get_client_hello_str(), "Chrome-133");
    }

    #[test]
    fn test_unified_fingerprint_generation() {
        // test Windows User-Agent
        let windows_ua = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/135.0.0.0 Safari/537.36";
        let profile = generate_unified_fingerprint("chrome_135", windows_ua).unwrap();

        // Validate TCP Profile alreadysync
        assert!(profile.tcp_profile.is_some());
        let tcp_profile = profile.tcp_profile.unwrap();
        assert_eq!(tcp_profile.ttl, 128); // Windows TTL
        assert_eq!(tcp_profile.window_size, 64240); // Windows Window Size

        // test Linux User-Agent
        let linux_ua = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/135.0.0.0 Safari/537.36";
        let profile = generate_unified_fingerprint("chrome_135", linux_ua).unwrap();

        let tcp_profile = profile.tcp_profile.unwrap();
        assert_eq!(tcp_profile.ttl, 64); // Linux TTL
        assert_eq!(tcp_profile.window_size, 65535); // Linux Window Size

        // test macOS User-Agent
        let macos_ua = "Mozilla/5.0 (Macintosh; Intel Mac OS X 14_0_0) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/135.0.0.0 Safari/537.36";
        let profile = generate_unified_fingerprint("chrome_135", macos_ua).unwrap();

        let tcp_profile = profile.tcp_profile.unwrap();
        assert_eq!(tcp_profile.ttl, 64); // macOS TTL
        assert_eq!(tcp_profile.window_size, 65535); // macOS Window Size
    }

    #[test]
    fn test_with_synced_tcp_profile() {
        let profile = chrome_133();
        let windows_ua = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/133.0.0.0 Safari/537.36";

        let synced_profile = profile.with_synced_tcp_profile(windows_ua);
        assert!(synced_profile.tcp_profile.is_some());
        let tcp_profile = synced_profile.tcp_profile.unwrap();
        assert_eq!(tcp_profile.ttl, 128);
    }

    #[test]
    fn test_tcp_profile_for_os() {
        use fingerprint_core::types::OperatingSystem;

        let profile = chrome_133();
        let synced_profile = profile.with_tcp_profile_for_os(OperatingSystem::Linux);

        assert!(synced_profile.tcp_profile.is_some());
        let tcp_profile = synced_profile.tcp_profile.unwrap();
        assert_eq!(tcp_profile.ttl, 64);
        assert_eq!(tcp_profile.window_size, 65535);
    }

    // Tests for 2026 browser fingerprints
    #[test]
    fn test_chrome_134_fingerprint() {
        let profile = chrome_134();
        assert_eq!(profile.get_client_hello_str(), "Chrome-134");

        // Verify TLS fingerprint can be generated
        let spec = profile.get_client_hello_spec();
        assert!(spec.is_ok());
        let spec = spec.unwrap();
        assert!(!spec.cipher_suites.is_empty());
    }

    #[test]
    fn test_firefox_134_fingerprint() {
        let profile = firefox_134();
        assert_eq!(profile.get_client_hello_str(), "Firefox-134");

        // Verify TLS fingerprint
        let spec = profile.get_client_hello_spec();
        assert!(spec.is_ok());
    }

    #[test]
    fn test_firefox_136_fingerprint() {
        let profile = firefox_136();
        assert_eq!(profile.get_client_hello_str(), "Firefox-136");

        // Verify TLS fingerprint
        let spec = profile.get_client_hello_spec();
        assert!(spec.is_ok());
    }

    #[test]
    fn test_safari_18_2_fingerprint() {
        let profile = safari_18_2();
        assert_eq!(profile.get_client_hello_str(), "Safari-18.2");

        // Verify TLS fingerprint
        let spec = profile.get_client_hello_spec();
        assert!(spec.is_ok());
    }

    #[test]
    fn test_safari_ios_18_2_fingerprint() {
        let profile = safari_ios_18_2();
        assert_eq!(profile.get_client_hello_str(), "Safari-iOS_18.2");

        // Verify TLS fingerprint
        let spec = profile.get_client_hello_spec();
        assert!(spec.is_ok());
    }

    #[test]
    fn test_edge_134_fingerprint() {
        let profile = edge_134();
        assert_eq!(profile.get_client_hello_str(), "Edge-134");

        // Verify TLS fingerprint
        let spec = profile.get_client_hello_spec();
        assert!(spec.is_ok());
    }

    #[test]
    fn test_chrome_mobile_134_fingerprint() {
        let profile = chrome_mobile_134();
        assert_eq!(profile.get_client_hello_str(), "Chrome Mobile-134");

        // Verify TLS fingerprint
        let spec = profile.get_client_hello_spec();
        assert!(spec.is_ok());
    }

    #[test]
    fn test_2026_fingerprints_in_map() {
        let clients = mapped_tls_clients();

        // Verify all 2026 fingerprints are registered
        assert!(clients.contains_key("chrome_134"));
        assert!(clients.contains_key("firefox_134"));
        assert!(clients.contains_key("firefox_136"));
        assert!(clients.contains_key("safari_18_2"));
        assert!(clients.contains_key("safari_ios_18_2"));
        assert!(clients.contains_key("edge_134"));
        assert!(clients.contains_key("chrome_mobile_134"));
    }

    #[test]
    fn test_2026_profiles_have_tcp_profiles() {
        // Verify that 2026 profiles have TCP profiles set
        let chrome = chrome_134();
        assert!(chrome.tcp_profile.is_some());

        let firefox = firefox_134();
        assert!(firefox.tcp_profile.is_some());

        let safari = safari_18_2();
        assert!(safari.tcp_profile.is_some());

        let edge = edge_134();
        assert!(edge.tcp_profile.is_some());

        let chrome_mobile = chrome_mobile_134();
        assert!(chrome_mobile.tcp_profile.is_some());
    }
}
