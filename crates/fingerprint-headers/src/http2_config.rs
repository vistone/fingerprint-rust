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
/// Corresponds to Go version's map[http2.SettingID]uint32
pub type HTTP2Settings = HashMap<u16, u32>;

/// HTTP/2 Priority
/// Corresponds to Go version's http2.Priority
#[derive(Debug, Clone)]
pub struct HTTP2Priority {
 pub stream_id: u32,
 pub exclusive: bool,
 pub weight: u8,
 pub stream_dependency: u32,
}

/// HTTP/2 Priority Param
/// Corresponds to Go version's http2.PriorityParam
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

/// Create Chrome HTTP/2 Settings
pub fn chrome_http2_settings() -> (HTTP2Settings, Vec<u16>) {
 let mut settings = HashMap::new();

 // Chrome HTTP/2 Settings
 settings.insert(HTTP2SettingID::HeaderTableSize.as_u16(), 65536);
 settings.insert(HTTP2SettingID::EnablePush.as_u16(), 0); // Disable Server Push
 settings.insert(HTTP2SettingID::MaxConcurrentStreams.as_u16(), 1000);
 settings.insert(HTTP2SettingID::InitialWindowSize.as_u16(), 6291456);
 settings.insert(HTTP2SettingID::MaxFrameSize.as_u16(), 16384);
 settings.insert(HTTP2SettingID::MaxHeaderListSize.as_u16(), 262144);

 // Settings order (Chrome\'s order)
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

/// Create Firefox HTTP/2 Settings
pub fn firefox_http2_settings() -> (HTTP2Settings, Vec<u16>) {
 let mut settings = HashMap::new();

 // Firefox HTTP/2 Settings ( and Chrome slightly different)
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

/// Create Safari HTTP/2 Settings
pub fn safari_http2_settings() -> (HTTP2Settings, Vec<u16>) {
 let mut settings = HashMap::new();

 // Safari HTTP/2 Settings
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

/// Chrome Pseudo Header Order
pub fn chrome_pseudo_header_order() -> Vec<String> {
 vec![
 ":method".to_string(),
 ":authority".to_string(),
 ":scheme".to_string(),
 ":path".to_string(),
 ]
}

/// Firefox Pseudo Header Order
pub fn firefox_pseudo_header_order() -> Vec<String> {
 vec![
 ":method".to_string(),
 ":path".to_string(),
 ":authority".to_string(),
 ":scheme".to_string(),
 ]
}

/// Safari Pseudo Header Order
pub fn safari_pseudo_header_order() -> Vec<String> {
 vec![
 ":method".to_string(),
 ":scheme".to_string(),
 ":path".to_string(),
 ":authority".to_string(),
 ]
}

/// Chrome Connection Flow
pub const CHROME_CONNECTION_FLOW: u32 = 15663105;

/// Chrome Header Priority
/// weight in HTTP/2 is 1-256,  but in Rust we use u8 (0-255)
/// actualwhen used needconvert to HTTP/2 weight value (weight = value + 1)
pub fn chrome_header_priority() -> HTTP2PriorityParam {
 HTTP2PriorityParam::new(255, 0, false) // Corresponds to HTTP/2 weight = 256
}

/// Chrome's standard Header order (HTTP/1.1)
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

/// Firefox's standard Header order (HTTP/1.1)
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

/// Safari's standard Header order (HTTP/1.1)
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
