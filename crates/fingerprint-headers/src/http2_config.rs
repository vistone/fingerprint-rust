//! HTTP/2 configuration module
//!
//! Provides HTTP/2 Settings, Pseudo Header Order, and other configurations
//! Corresponds to Go version's http2.Settings and http2.PriorityParam

use std::collections::HashMap;

/// HTTP/2 Setting ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HTTP2SettingID {
    HeaderTableSize = 1,
    EnablePush = 2,
    MaxConcurrentStreams = 3,
    InitialWindowSize = 4,
    MaxFrameSize = 5,
    MaxHeaderListSize = 6,
    EnableConnectProtocol = 8,
}

impl HTTP2SettingID {
    pub fn as_u16(&self) -> u16 {
        *self as u16
    }
}

/// HTTP/2 Settings
/// 对应 Go 版本的 map[http2.SettingID]uint32
pub type HTTP2Settings = HashMap<u16, u32>;

/// HTTP/2 Priority
/// 对应 Go 版本的 http2.Priority
#[derive(Debug, Clone)]
pub struct HTTP2Priority {
    pub stream_id: u32,
    pub exclusive: bool,
    pub weight: u8,
    pub stream_dependency: u32,
}

/// HTTP/2 Priority Param
/// 对应 Go 版本的 http2.PriorityParam
#[derive(Debug, Clone)]
pub struct HTTP2PriorityParam {
    pub weight: u8,
    pub stream_dependency: u32,
    pub exclusive: bool,
}

impl HTTP2PriorityParam {
    pub fn new(weight: u8, stream_dependency: u32, exclusive: bool) -> Self {
        Self {
            weight,
            stream_dependency,
            exclusive,
        }
    }
}

/// 创建 Chrome 的 HTTP/2 Settings
pub fn chrome_http2_settings() -> (HTTP2Settings, Vec<u16>) {
    let mut settings = HashMap::new();

    // Chrome 的 HTTP/2 Settings
    settings.insert(HTTP2SettingID::HeaderTableSize.as_u16(), 65536);
    settings.insert(HTTP2SettingID::EnablePush.as_u16(), 0); // 禁用 Server Push
    settings.insert(HTTP2SettingID::MaxConcurrentStreams.as_u16(), 1000);
    settings.insert(HTTP2SettingID::InitialWindowSize.as_u16(), 6291456);
    settings.insert(HTTP2SettingID::MaxFrameSize.as_u16(), 16384);
    settings.insert(HTTP2SettingID::MaxHeaderListSize.as_u16(), 262144);

    // Settings 顺序（Chrome 的顺序）
    let settings_order = vec![
        HTTP2SettingID::HeaderTableSize.as_u16(),
        HTTP2SettingID::EnablePush.as_u16(),
        HTTP2SettingID::MaxConcurrentStreams.as_u16(),
        HTTP2SettingID::InitialWindowSize.as_u16(),
        HTTP2SettingID::MaxFrameSize.as_u16(),
        HTTP2SettingID::MaxHeaderListSize.as_u16(),
    ];

    (settings, settings_order)
}

/// 创建 Firefox 的 HTTP/2 Settings
pub fn firefox_http2_settings() -> (HTTP2Settings, Vec<u16>) {
    let mut settings = HashMap::new();

    // Firefox 的 HTTP/2 Settings（与 Chrome 略有不同）
    settings.insert(HTTP2SettingID::HeaderTableSize.as_u16(), 65536);
    settings.insert(HTTP2SettingID::EnablePush.as_u16(), 0);
    settings.insert(HTTP2SettingID::MaxConcurrentStreams.as_u16(), 1000);
    settings.insert(HTTP2SettingID::InitialWindowSize.as_u16(), 131072);
    settings.insert(HTTP2SettingID::MaxFrameSize.as_u16(), 16384);
    settings.insert(HTTP2SettingID::MaxHeaderListSize.as_u16(), 262144);

    let settings_order = vec![
        HTTP2SettingID::HeaderTableSize.as_u16(),
        HTTP2SettingID::EnablePush.as_u16(),
        HTTP2SettingID::MaxConcurrentStreams.as_u16(),
        HTTP2SettingID::InitialWindowSize.as_u16(),
        HTTP2SettingID::MaxFrameSize.as_u16(),
        HTTP2SettingID::MaxHeaderListSize.as_u16(),
    ];

    (settings, settings_order)
}

/// 创建 Safari 的 HTTP/2 Settings
pub fn safari_http2_settings() -> (HTTP2Settings, Vec<u16>) {
    let mut settings = HashMap::new();

    // Safari 的 HTTP/2 Settings
    settings.insert(HTTP2SettingID::HeaderTableSize.as_u16(), 65536);
    settings.insert(HTTP2SettingID::EnablePush.as_u16(), 0);
    settings.insert(HTTP2SettingID::MaxConcurrentStreams.as_u16(), 100);
    settings.insert(HTTP2SettingID::InitialWindowSize.as_u16(), 65535);
    settings.insert(HTTP2SettingID::MaxFrameSize.as_u16(), 16777215);
    settings.insert(HTTP2SettingID::MaxHeaderListSize.as_u16(), 262144);

    let settings_order = vec![
        HTTP2SettingID::HeaderTableSize.as_u16(),
        HTTP2SettingID::EnablePush.as_u16(),
        HTTP2SettingID::MaxConcurrentStreams.as_u16(),
        HTTP2SettingID::InitialWindowSize.as_u16(),
        HTTP2SettingID::MaxFrameSize.as_u16(),
        HTTP2SettingID::MaxHeaderListSize.as_u16(),
    ];

    (settings, settings_order)
}

/// Chrome 的 Pseudo Header Order
pub fn chrome_pseudo_header_order() -> Vec<String> {
    vec![
        ":method".to_string(),
        ":authority".to_string(),
        ":scheme".to_string(),
        ":path".to_string(),
    ]
}

/// Firefox 的 Pseudo Header Order
pub fn firefox_pseudo_header_order() -> Vec<String> {
    vec![
        ":method".to_string(),
        ":path".to_string(),
        ":authority".to_string(),
        ":scheme".to_string(),
    ]
}

/// Safari 的 Pseudo Header Order
pub fn safari_pseudo_header_order() -> Vec<String> {
    vec![
        ":method".to_string(),
        ":scheme".to_string(),
        ":path".to_string(),
        ":authority".to_string(),
    ]
}

/// Chrome 的 Connection Flow
pub const CHROME_CONNECTION_FLOW: u32 = 15663105;

/// Chrome 的 Header Priority
/// weight 在 HTTP/2 中是 1-256，但在 Rust 中我们使用 u8 (0-255)
/// 实际使用时需要转换为 HTTP/2 的 weight 值（weight = value + 1）
pub fn chrome_header_priority() -> HTTP2PriorityParam {
    HTTP2PriorityParam::new(255, 0, false) // 对应 HTTP/2 weight = 256
}

/// Chrome 的标准 Header 顺序 (HTTP/1.1)
pub fn chrome_header_order() -> Vec<String> {
    vec![
        "Host".to_string(),
        "Connection".to_string(),
        "sec-ch-ua".to_string(),
        "sec-ch-ua-mobile".to_string(),
        "sec-ch-ua-platform".to_string(),
        "Upgrade-Insecure-Requests".to_string(),
        "User-Agent".to_string(),
        "Accept".to_string(),
        "Sec-Fetch-Site".to_string(),
        "Sec-Fetch-Mode".to_string(),
        "Sec-Fetch-User".to_string(),
        "Sec-Fetch-Dest".to_string(),
        "Accept-Encoding".to_string(),
        "Accept-Language".to_string(),
    ]
}

/// Firefox 的标准 Header 顺序 (HTTP/1.1)
pub fn firefox_header_order() -> Vec<String> {
    vec![
        "Host".to_string(),
        "User-Agent".to_string(),
        "Accept".to_string(),
        "Accept-Language".to_string(),
        "Accept-Encoding".to_string(),
        "Connection".to_string(),
        "Upgrade-Insecure-Requests".to_string(),
        "Sec-Fetch-Dest".to_string(),
        "Sec-Fetch-Mode".to_string(),
        "Sec-Fetch-Site".to_string(),
        "Sec-Fetch-User".to_string(),
        "Priority".to_string(),
    ]
}

/// Safari 的标准 Header 顺序 (HTTP/1.1)
pub fn safari_header_order() -> Vec<String> {
    vec![
        "Host".to_string(),
        "Accept".to_string(),
        "Accept-Language".to_string(),
        "Connection".to_string(),
        "Accept-Encoding".to_string(),
        "User-Agent".to_string(),
    ]
}
