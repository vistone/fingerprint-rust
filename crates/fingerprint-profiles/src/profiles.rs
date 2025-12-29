//! 指纹配置模块
//!
//! 定义了各种浏览器的 TLS 指纹配置

use fingerprint_headers::http2_config::{
    chrome_header_priority, chrome_http2_settings, chrome_pseudo_header_order,
    firefox_http2_settings, firefox_pseudo_header_order, safari_http2_settings,
    safari_pseudo_header_order, HTTP2PriorityParam, HTTP2Settings,
};
use fingerprint_tls::tls_config::ClientHelloSpec;
use std::collections::HashMap;
use std::sync::OnceLock;

/// ClientHelloSpecFactory 类型
/// 对应 Go 版本的 ClientHelloSpecFactory func() (ClientHelloSpec, error)
pub type ClientHelloSpecFactory = fn() -> Result<ClientHelloSpec, String>;

/// Client Hello ID
/// 对应 Go 版本的 tls.ClientHelloID
#[derive(Debug, Clone)]
pub struct ClientHelloID {
    /// Client 名称（如 "Chrome", "Firefox", "Safari"）
    pub client: String,
    /// Version 版本号（如 "133", "120"）
    pub version: String,
    /// SpecFactory 用于生成 ClientHelloSpec
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

    /// 转换为字符串表示（对应 Go 版本的 Str()）
    pub fn str(&self) -> String {
        format!("{}-{}", self.client, self.version)
    }

    /// 转换为 ClientHelloSpec（对应 Go 版本的 ToSpec()）
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

/// Client Profile 配置
/// 包含 TLS 指纹的所有配置信息
/// 对应 Go 版本的 ClientProfile 结构
#[derive(Debug, Clone)]
pub struct ClientProfile {
    /// Client Hello ID
    pub client_hello_id: ClientHelloID,
    /// HTTP/2 Settings（对应 Go 版本的 map[http2.SettingID]uint32）
    pub settings: HTTP2Settings,
    /// Settings 顺序（对应 Go 版本的 []http2.SettingID）
    pub settings_order: Vec<u16>,
    /// Pseudo Header 顺序（对应 Go 版本的 []string）
    pub pseudo_header_order: Vec<String>,
    /// Connection Flow（对应 Go 版本的 uint32）
    pub connection_flow: u32,
    /// Priorities（对应 Go 版本的 []http2.Priority）
    pub priorities: Vec<String>,
    /// Header Priority（对应 Go 版本的 *http2.PriorityParam）
    pub header_priority: Option<HTTP2PriorityParam>,
}

impl ClientProfile {
    /// 创建新的 ClientProfile
    /// 对应 Go 版本的 NewClientProfile 函数
    pub fn new(
        client_hello_id: ClientHelloID,
        settings: HTTP2Settings,
        settings_order: Vec<u16>,
        pseudo_header_order: Vec<String>,
        connection_flow: u32,
        priorities: Vec<String>,
        header_priority: Option<HTTP2PriorityParam>,
    ) -> Self {
        Self {
            client_hello_id,
            settings,
            settings_order,
            pseudo_header_order,
            connection_flow,
            priorities,
            header_priority,
        }
    }

    /// 获取 Client Hello ID 字符串（对应 Go 版本的 GetClientHelloStr()）
    pub fn get_client_hello_str(&self) -> String {
        self.client_hello_id.str()
    }

    /// 获取 Settings（对应 Go 版本的 GetSettings()）
    pub fn get_settings(&self) -> &HTTP2Settings {
        &self.settings
    }

    /// 获取 Settings Order（对应 Go 版本的 GetSettingsOrder()）
    pub fn get_settings_order(&self) -> &[u16] {
        &self.settings_order
    }

    /// 获取 Pseudo Header Order
    pub fn get_pseudo_header_order(&self) -> &[String] {
        &self.pseudo_header_order
    }

    /// 获取 Connection Flow
    pub fn get_connection_flow(&self) -> u32 {
        self.connection_flow
    }

    /// 获取 Priorities
    pub fn get_priorities(&self) -> &[String] {
        &self.priorities
    }

    /// 获取 Header Priority（对应 Go 版本的 GetHeaderPriority()）
    pub fn get_header_priority(&self) -> Option<&HTTP2PriorityParam> {
        self.header_priority.as_ref()
    }

    /// 获取 ClientHelloSpec（对应 Go 版本的 GetClientHelloSpec()）
    /// 这是真正的 TLS 指纹配置，可以用于实际的 TLS 握手
    pub fn get_client_hello_spec(&self) -> Result<ClientHelloSpec, String> {
        self.client_hello_id.to_spec()
    }
}

/// 默认的 Client Profile（Chrome 133）
pub fn default_client_profile() -> ClientProfile {
    chrome_133()
}

/// Chrome 103 指纹配置
/// 对应 Go 版本的 Chrome_103
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
    )
}

/// Chrome 133 指纹配置（默认）
/// 对应 Go 版本的 Chrome_133
pub fn chrome_133() -> ClientProfile {
    let (settings, settings_order) = chrome_http2_settings();
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
    )
}

/// Firefox 133 指纹配置
/// 对应 Go 版本的 Firefox_133
pub fn firefox_133() -> ClientProfile {
    let (settings, settings_order) = firefox_http2_settings();
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
    )
}

/// Safari 16.0 指纹配置
/// 对应 Go 版本的 Safari_16_0
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
    )
}

/// Opera 91 指纹配置
/// 对应 Go 版本的 Opera_91
pub fn opera_91() -> ClientProfile {
    // Opera 使用 Chrome 内核，配置与 Chrome 相同
    let (settings, settings_order) = chrome_http2_settings();
    ClientProfile::new(
        ClientHelloID::new("Opera", "91", fingerprint_tls::tls_config::chrome_133_spec), // Opera 使用 Chrome 的 TLS 配置
        settings,
        settings_order,
        chrome_pseudo_header_order(),
        fingerprint_headers::http2_config::CHROME_CONNECTION_FLOW,
        Vec::new(),
        Some(chrome_header_priority()),
    )
}

/// Edge 120 指纹配置
/// Edge 使用 Chromium 内核，TLS 指纹与 Chrome 相同
pub fn edge_120() -> ClientProfile {
    // Edge 使用 Chrome 内核，配置与 Chrome 相同
    let (settings, settings_order) = chrome_http2_settings();
    ClientProfile::new(
        ClientHelloID::new("Edge", "120", fingerprint_tls::tls_config::chrome_133_spec), // Edge 使用 Chrome 的 TLS 配置
        settings,
        settings_order,
        chrome_pseudo_header_order(),
        fingerprint_headers::http2_config::CHROME_CONNECTION_FLOW,
        Vec::new(),
        Some(chrome_header_priority()),
    )
}

/// Edge 124 指纹配置
/// Edge 使用 Chromium 内核，TLS 指纹与 Chrome 相同
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
    )
}

/// Edge 133 指纹配置
/// Edge 使用 Chromium 内核，TLS 指纹与 Chrome 相同
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
    )
}

/// 初始化所有指纹配置的映射表
fn init_mapped_tls_clients() -> HashMap<String, ClientProfile> {
    let mut map = HashMap::new();

    // Chrome 系列
    // 注意：这里简化处理，实际应该为每个版本创建独立的配置
    // 为了匹配 Go 版本，我们使用 chrome_133 作为默认配置
    map.insert("chrome_103".to_string(), chrome_133()); // 简化：使用 chrome_133
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

    // Safari 系列
    map.insert("safari_15_6_1".to_string(), safari_16_0());
    map.insert("safari_16_0".to_string(), safari_16_0());
    map.insert("safari_ipad_15_6".to_string(), safari_16_0());
    map.insert("safari_ios_15_5".to_string(), safari_16_0());
    map.insert("safari_ios_15_6".to_string(), safari_16_0());
    map.insert("safari_ios_16_0".to_string(), safari_16_0());
    map.insert("safari_ios_17_0".to_string(), safari_16_0());
    map.insert("safari_ios_18_0".to_string(), safari_16_0());
    map.insert("safari_ios_18_5".to_string(), safari_16_0());

    // Firefox 系列
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
    map.insert("firefox_135".to_string(), firefox_133());

    // Opera 系列
    map.insert("opera_89".to_string(), opera_91());
    map.insert("opera_90".to_string(), opera_91());
    map.insert("opera_91".to_string(), opera_91());

    // Edge 系列（使用 Chromium 内核，TLS 指纹与 Chrome 相同）
    map.insert("edge_120".to_string(), edge_120());
    map.insert("edge_124".to_string(), edge_124());
    map.insert("edge_133".to_string(), edge_133());

    // 移动端和自定义指纹
    map.insert("zalando_android_mobile".to_string(), chrome_133());
    map.insert("zalando_ios_mobile".to_string(), safari_16_0());
    map.insert("nike_ios_mobile".to_string(), safari_16_0());
    map.insert("nike_android_mobile".to_string(), chrome_133());
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

/// 全局指纹配置映射表（线程安全）
static MAPPED_TLS_CLIENTS: OnceLock<HashMap<String, ClientProfile>> = OnceLock::new();

/// 获取指纹配置映射表
pub fn mapped_tls_clients() -> &'static HashMap<String, ClientProfile> {
    MAPPED_TLS_CLIENTS.get_or_init(init_mapped_tls_clients)
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
}
