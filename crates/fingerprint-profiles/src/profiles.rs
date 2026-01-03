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
 /// Client name (such as "Chrome", "Firefox", "Safari")
 pub client: String,
 /// Version version number (such as "135", "133")
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
/// including TLS fingerprint all configurationinfo
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
 /// create a new ClientProfile
 /// Corresponds to Go version's NewClientProfile function
 #[ all ow (clippy::too_many_arguments)] // constructfunctionneed all 必 need parameter
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
 /// this isunifiedfingerprintGeneratecoremethod，ensurebrowserfingerprint and TCP fingerprintsync
 ///
 /// # Parameters
 /// - `user_agent`: User-Agent string， for inferoperating system 
 ///
 /// # Returns
 /// returnannew ClientProfile， its in tcp_profile alreadyBased on User-Agent settings
 pub fn with_synced_tcp_profile(self, user_agent: &str) -> Self {
 use fingerprint_core::tcp::TcpProfile;
 let tcp_profile = TcpProfile::from_user_agent(user_agent);
 Self {
 tcp_profile: Some(tcp_profile),
..self
 }
 }

 /// Based onoperating system typeautomaticGeneratematch TCP Profile
 ///
 /// # Parameters
 /// - `os`: operating system type
 ///
 /// # Returns
 /// returnannew ClientProfile， its in tcp_profile alreadyBased onoperating system settings
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
 /// If not exists, Based on User-Agent Generate
 ///
 /// # Parameters
 /// - `user_agent`: User-Agent string， for inferoperating system (if tcp_profile not exists)
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
 /// this istrue TLS fingerprintconfiguration，can for actual TLS handshake
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
 // usercanthrough with_synced_tcp_profile() or with_tcp_profile_for_os() from sync
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
 // usercanthrough with_synced_tcp_profile() or with_tcp_profile_for_os() from sync
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
 // usercanthrough with_synced_tcp_profile() or with_tcp_profile_for_os() from sync
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

/// Firefox 135 fingerprintconfiguration
pub fn firefox_135() -> ClientProfile {
 let (settings, settings_order) = firefox_http2_settings();
 // defaultuse Windows TCP Profile (most commonbrowserenvironment)
 // usercanthrough with_synced_tcp_profile() or with_tcp_profile_for_os() from sync
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

/// Opera 91 fingerprintconfiguration
/// Corresponds to Go version's Opera_91
pub fn opera_91() -> ClientProfile {
 // Opera use Chrome core，configuration and Chrome same
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

/// Edge 120 fingerprintconfiguration
/// Edge use Chromium core，TLS fingerprint and Chrome same
pub fn edge_120() -> ClientProfile {
 // Edge use Chrome core，configuration and Chrome same
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
/// Edge use Chromium core，TLS fingerprint and Chrome same
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
/// Edge use Chromium core，TLS fingerprint and Chrome same
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

/// Initialize all fingerprintconfigurationmap表
fn init_mapped_tls_clients() -> HashMap<String, ClientProfile> {
 let mut map = HashMap::new();

 // Chrome series
 // Note: heresimplifyprocess，actualshould as eachversionCreateindependentconfiguration
 // in order tomatch Go version，weuse chrome_133 asdefaultconfiguration
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
 map.insert("chrome_120".to_string(), chrome_133());
 map.insert("chrome_124".to_string(), chrome_133());
 map.insert("chrome_130_PSK".to_string(), chrome_133());
 map.insert("chrome_131".to_string(), chrome_133());
 map.insert("chrome_131_PSK".to_string(), chrome_133());
 map.insert("chrome_133".to_string(), chrome_133());
 map.insert("chrome_133_PSK".to_string(), chrome_133());
 map.insert("chrome_134".to_string(), chrome_135());
 map.insert("chrome_135".to_string(), chrome_135());
 map.insert("chrome_136".to_string(), chrome_136());

 // Safari series
 map.insert("safari_15_6_1".to_string(), safari_16_0());
 map.insert("safari_16_0".to_string(), safari_16_0());
 map.insert("safari_ipad_15_6".to_string(), safari_16_0());
 map.insert("safari_ios_15_5".to_string(), safari_16_0());
 map.insert("safari_ios_15_6".to_string(), safari_16_0());
 map.insert("safari_ios_16_0".to_string(), safari_16_0());
 map.insert("safari_ios_17_0".to_string(), safari_16_0());
 map.insert("safari_ios_18_0".to_string(), safari_16_0());
 map.insert("safari_ios_18_5".to_string(), safari_16_0());

 // Firefox series
 map.insert("firefox_102".to_string(), firefox_133());
 map.insert("firefox_104".to_string(), firefox_133());
 map.insert("firefox_105".to_string(), firefox_133());
 map.insert("firefox_106".to_string(), firefox_133());
 map.insert("firefox_108".to_string(), firefox_133());
 map.insert("firefox_110".to_string(), firefox_133());
 map.insert("firefox_117".to_string(), firefox_133());
 map.insert("firefox_120".to_string(), firefox_133());
 map.insert("firefox_123".to_string(), firefox_133());
 map.insert("firefox_132".to_string(), firefox_133());
 map.insert("firefox_133".to_string(), firefox_133());
 map.insert("firefox_134".to_string(), firefox_135());
 map.insert("firefox_135".to_string(), firefox_135());

 // Opera series
 map.insert("opera_89".to_string(), opera_91());
 map.insert("opera_90".to_string(), opera_91());
 map.insert("opera_91".to_string(), opera_91());

 // Edge series (use Chromium core，TLS fingerprint and Chrome same)
 map.insert("edge_120".to_string(), edge_120());
 map.insert("edge_124".to_string(), edge_124());
 map.insert("edge_133".to_string(), edge_133());

 // mobile and customfingerprint
 map.insert("zalando_android_ mobile ".to_string(), chrome_133());
 map.insert("zalando_ios_ mobile ".to_string(), safari_16_0());
 map.insert("nike_ios_ mobile ".to_string(), safari_16_0());
 map.insert("nike_android_ mobile ".to_string(), chrome_133());
 map.insert("mms_ios".to_string(), safari_16_0());
 map.insert("mms_ios_2".to_string(), safari_16_0());
 map.insert("mms_ios_3".to_string(), safari_16_0());
 map.insert("mesh_ios".to_string(), safari_16_0());
 map.insert("mesh_android".to_string(), chrome_133());
 map.insert("mesh_ios_2".to_string(), safari_16_0());
 map.insert("mesh_android_2".to_string(), chrome_133());
 map.insert("confirmed_ios".to_string(), safari_16_0());
 map.insert("confirmed_android".to_string(), chrome_133());
 map.insert("confirmed_android_2".to_string(), chrome_133());
 map.insert("okhttp4_android_7".to_string(), chrome_133());
 map.insert("okhttp4_android_8".to_string(), chrome_133());
 map.insert("okhttp4_android_9".to_string(), chrome_133());
 map.insert("okhttp4_android_10".to_string(), chrome_133());
 map.insert("okhttp4_android_11".to_string(), chrome_133());
 map.insert("okhttp4_android_12".to_string(), chrome_133());
 map.insert("okhttp4_android_13".to_string(), chrome_133());
 map.insert("cloudflare_custom".to_string(), chrome_133());

 map
}

/// globalfingerprintconfigurationmap表 (threadsecurity)
static MAPPED_TLS_CLIENTS: OnceLock<HashMap<String, ClientProfile>> = OnceLock::new();

/// Getfingerprintconfigurationmap表
pub fn mapped_tls_clients() -> &'static HashMap<String, ClientProfile> {
 MAPPED_TLS_CLIENTS.get_or_init(init_mapped_tls_clients)
}

/// Based on profile nameGet ClientProfile
///
/// # Parameters
/// - `profile_name`: fingerprintconfigurationname (such as "chrome_135", "firefox_133")
///
/// # Returns
/// returnpair should ClientProfile， if not exists then returnerror
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
/// - `profile_name`: fingerprintconfigurationname (such as "chrome_135", "firefox_133")
/// - `user_agent`: User-Agent string， for sync TCP fingerprint
///
/// # Returns
/// returnan ClientProfile， its in tcp_profile alreadyBased on User-Agent sync
///
/// # Examples
/// ```rust
/// use fingerprint_profiles::profiles::generate_unified_fingerprint;
///
/// let user_agent = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/135.0.0.0 Safari/537.36";
/// let profile = generate_unified_fingerprint("chrome_135", user_agent).unwrap();
///
/// // profile.tcp_profile 现 in including and User-Agent match TCP fingerprint
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
 let window s_ua = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/135.0.0.0 Safari/537.36";
 let profile = generate_unified_fingerprint("chrome_135", window s_ua).unwrap();

 // Validate TCP Profile alreadysync
 assert!(profile.tcp_profile.is_some());
 let tcp_profile = profile.tcp_profile.unwrap();
 assert_eq!(tcp_profile.ttl, 128); // Windows TTL
 assert_eq!(tcp_profile. window _size, 64240); // Windows Window Size

 // test Linux User-Agent
 let linux_ua = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/135.0.0.0 Safari/537.36";
 let profile = generate_unified_fingerprint("chrome_135", linux_ua).unwrap();

 let tcp_profile = profile.tcp_profile.unwrap();
 assert_eq!(tcp_profile.ttl, 64); // Linux TTL
 assert_eq!(tcp_profile. window _size, 65535); // Linux Window Size

 // test macOS User-Agent
 let macos_ua = "Mozilla/5.0 (Macintosh; Intel Mac OS X 14_0_0) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/135.0.0.0 Safari/537.36";
 let profile = generate_unified_fingerprint("chrome_135", macos_ua).unwrap();

 let tcp_profile = profile.tcp_profile.unwrap();
 assert_eq!(tcp_profile.ttl, 64); // macOS TTL
 assert_eq!(tcp_profile. window _size, 65535); // macOS Window Size
 }

 #[test]
 fn test_with_synced_tcp_profile() {
 let profile = chrome_133();
 let window s_ua = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/133.0.0.0 Safari/537.36";

 let synced_profile = profile.with_synced_tcp_profile(window s_ua);
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
 assert_eq!(tcp_profile. window _size, 65535);
 }
}
