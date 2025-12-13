//! 指纹配置模块
//!
//! 定义了各种浏览器的 TLS 指纹配置

use crate::http2_config::{
    chrome_header_priority, chrome_http2_settings, chrome_pseudo_header_order,
    firefox_http2_settings, firefox_pseudo_header_order, safari_http2_settings,
    safari_pseudo_header_order, HTTP2PriorityParam, HTTP2Settings,
};
use crate::tls_config::ClientHelloSpec;
use std::collections::HashMap;
use std::sync::OnceLock;

/// Client Hello ID 字符串表示
/// 对应 Go 版本的 tls.ClientHelloID
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ClientHelloID {
    pub id: String,
}

impl ClientHelloID {
    pub fn new(id: &str) -> Self {
        Self { id: id.to_string() }
    }

    pub fn as_str(&self) -> &str {
        &self.id
    }

    /// 转换为 ClientHelloSpec（对应 Go 版本的 ToSpec()）
    pub fn to_spec(&self) -> Result<ClientHelloSpec, String> {
        match self.id.as_str() {
            "chrome_103" | "chrome_104" | "chrome_105" | "chrome_106" 
            | "chrome_107" | "chrome_108" | "chrome_109" | "chrome_110"
            | "chrome_111" | "chrome_112" | "chrome_116_PSK" | "chrome_116_PSK_PQ"
            | "chrome_117" | "chrome_120" | "chrome_124" | "chrome_130_PSK"
            | "chrome_131" | "chrome_131_PSK" | "chrome_133" | "chrome_133_PSK"
            | "zalando_android_mobile" | "nike_android_mobile" | "mesh_android"
            | "mesh_android_2" | "confirmed_android" | "confirmed_android_2"
            | "okhttp4_android_7" | "okhttp4_android_8" | "okhttp4_android_9"
            | "okhttp4_android_10" | "okhttp4_android_11" | "okhttp4_android_12"
            | "okhttp4_android_13" | "cloudflare_custom" => {
                Ok(ClientHelloSpec::chrome_133())
            }
            "firefox_102" | "firefox_104" | "firefox_105" | "firefox_106"
            | "firefox_108" | "firefox_110" | "firefox_117" | "firefox_120"
            | "firefox_123" | "firefox_132" | "firefox_133" | "firefox_135" => {
                Ok(ClientHelloSpec::firefox_133())
            }
            "safari_15_6_1" | "safari_16_0" | "safari_ipad_15_6"
            | "safari_ios_15_5" | "safari_ios_15_6" | "safari_ios_16_0"
            | "safari_ios_17_0" | "safari_ios_18_0" | "safari_ios_18_5"
            | "zalando_ios_mobile" | "nike_ios_mobile" | "mms_ios" | "mms_ios_2"
            | "mms_ios_3" | "mesh_ios" | "mesh_ios_2" | "confirmed_ios" => {
                Ok(ClientHelloSpec::safari_16_0())
            }
            "opera_89" | "opera_90" | "opera_91" => {
                // Opera 使用 Chrome 内核
                Ok(ClientHelloSpec::chrome_133())
            }
            _ => Err(format!("Unknown client hello ID: {}", self.id)),
        }
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

    /// 获取 Client Hello ID 字符串
    pub fn get_client_hello_str(&self) -> &str {
        self.client_hello_id.as_str()
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
pub fn chrome_103() -> ClientProfile {
    let (settings, settings_order) = chrome_http2_settings();
    ClientProfile::new(
        ClientHelloID::new("chrome_103"),
        settings,
        settings_order,
        chrome_pseudo_header_order(),
        crate::http2_config::CHROME_CONNECTION_FLOW,
        Vec::new(),
        Some(chrome_header_priority()),
    )
}

/// Chrome 133 指纹配置（默认）
pub fn chrome_133() -> ClientProfile {
    let (settings, settings_order) = chrome_http2_settings();
    ClientProfile::new(
        ClientHelloID::new("chrome_133"),
        settings,
        settings_order,
        chrome_pseudo_header_order(),
        crate::http2_config::CHROME_CONNECTION_FLOW,
        Vec::new(),
        Some(chrome_header_priority()),
    )
}

/// Firefox 133 指纹配置
pub fn firefox_133() -> ClientProfile {
    let (settings, settings_order) = firefox_http2_settings();
    ClientProfile::new(
        ClientHelloID::new("firefox_133"),
        settings,
        settings_order,
        firefox_pseudo_header_order(),
        crate::http2_config::CHROME_CONNECTION_FLOW,
        Vec::new(),
        None,
    )
}

/// Safari 16.0 指纹配置
pub fn safari_16_0() -> ClientProfile {
    let (settings, settings_order) = safari_http2_settings();
    ClientProfile::new(
        ClientHelloID::new("safari_16_0"),
        settings,
        settings_order,
        safari_pseudo_header_order(),
        crate::http2_config::CHROME_CONNECTION_FLOW,
        Vec::new(),
        None,
    )
}

/// Opera 91 指纹配置
pub fn opera_91() -> ClientProfile {
    // Opera 使用 Chrome 内核，配置与 Chrome 相同
    let (settings, settings_order) = chrome_http2_settings();
    ClientProfile::new(
        ClientHelloID::new("opera_91"),
        settings,
        settings_order,
        chrome_pseudo_header_order(),
        crate::http2_config::CHROME_CONNECTION_FLOW,
        Vec::new(),
        Some(chrome_header_priority()),
    )
}

/// 初始化所有指纹配置的映射表
fn init_mapped_tls_clients() -> HashMap<String, ClientProfile> {
    let mut map = HashMap::new();

    // Chrome 系列
    map.insert("chrome_103".to_string(), chrome_103());
    map.insert("chrome_104".to_string(), chrome_103()); // 简化处理，实际应该有不同配置
    map.insert("chrome_105".to_string(), chrome_103());
    map.insert("chrome_106".to_string(), chrome_103());
    map.insert("chrome_107".to_string(), chrome_103());
    map.insert("chrome_108".to_string(), chrome_103());
    map.insert("chrome_109".to_string(), chrome_103());
    map.insert("chrome_110".to_string(), chrome_103());
    map.insert("chrome_111".to_string(), chrome_103());
    map.insert("chrome_112".to_string(), chrome_103());
    map.insert("chrome_116_PSK".to_string(), chrome_103());
    map.insert("chrome_116_PSK_PQ".to_string(), chrome_103());
    map.insert("chrome_117".to_string(), chrome_103());
    map.insert("chrome_120".to_string(), chrome_103());
    map.insert("chrome_124".to_string(), chrome_103());
    map.insert("chrome_130_PSK".to_string(), chrome_103());
    map.insert("chrome_131".to_string(), chrome_103());
    map.insert("chrome_131_PSK".to_string(), chrome_103());
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

    // 移动端和自定义指纹
    map.insert("zalando_android_mobile".to_string(), chrome_103());
    map.insert("zalando_ios_mobile".to_string(), safari_16_0());
    map.insert("nike_ios_mobile".to_string(), safari_16_0());
    map.insert("nike_android_mobile".to_string(), chrome_103());
    map.insert("mms_ios".to_string(), safari_16_0());
    map.insert("mms_ios_2".to_string(), safari_16_0());
    map.insert("mms_ios_3".to_string(), safari_16_0());
    map.insert("mesh_ios".to_string(), safari_16_0());
    map.insert("mesh_android".to_string(), chrome_103());
    map.insert("mesh_ios_2".to_string(), safari_16_0());
    map.insert("mesh_android_2".to_string(), chrome_103());
    map.insert("confirmed_ios".to_string(), safari_16_0());
    map.insert("confirmed_android".to_string(), chrome_103());
    map.insert("confirmed_android_2".to_string(), chrome_103());
    map.insert("okhttp4_android_7".to_string(), chrome_103());
    map.insert("okhttp4_android_8".to_string(), chrome_103());
    map.insert("okhttp4_android_9".to_string(), chrome_103());
    map.insert("okhttp4_android_10".to_string(), chrome_103());
    map.insert("okhttp4_android_11".to_string(), chrome_103());
    map.insert("okhttp4_android_12".to_string(), chrome_103());
    map.insert("okhttp4_android_13".to_string(), chrome_103());
    map.insert("cloudflare_custom".to_string(), chrome_103());

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
        assert_eq!(profile.get_client_hello_str(), "chrome_133");
    }
}
