//! HTTP/2 Frame Parser
//!
//! Parses HTTP/2 frames from TCP payloads for browser fingerprinting.
//! Focuses on SETTINGS frames which contain browser-specific configurations.

use std::collections::HashMap;
use std::fmt;

/// HTTP/2 Frame Type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Http2FrameType {
    Data = 0x0,
    Headers = 0x1,
    Priority = 0x2,
    RstStream = 0x3,
    Settings = 0x4,
    PushPromise = 0x5,
    Ping = 0x6,
    GoAway = 0x7,
    WindowUpdate = 0x8,
    Continuation = 0x9,
}

impl TryFrom<u8> for Http2FrameType {
    type Error = Http2ParseError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x0 => Ok(Http2FrameType::Data),
            0x1 => Ok(Http2FrameType::Headers),
            0x2 => Ok(Http2FrameType::Priority),
            0x3 => Ok(Http2FrameType::RstStream),
            0x4 => Ok(Http2FrameType::Settings),
            0x5 => Ok(Http2FrameType::PushPromise),
            0x6 => Ok(Http2FrameType::Ping),
            0x7 => Ok(Http2FrameType::GoAway),
            0x8 => Ok(Http2FrameType::WindowUpdate),
            0x9 => Ok(Http2FrameType::Continuation),
            _ => Err(Http2ParseError::UnknownFrameType(value)),
        }
    }
}

/// HTTP/2 Parse Error
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Http2ParseError {
    TooShort,
    UnknownFrameType(u8),
    NotSettingsFrame,
    InvalidPayloadLength,
}

impl fmt::Display for Http2ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Http2ParseError::TooShort => write!(f, "Buffer too short for HTTP/2 frame"),
            Http2ParseError::UnknownFrameType(t) => write!(f, "Unknown frame type: {}", t),
            Http2ParseError::NotSettingsFrame => write!(f, "Not a SETTINGS frame"),
            Http2ParseError::InvalidPayloadLength => write!(f, "Invalid payload length"),
        }
    }
}

impl std::error::Error for Http2ParseError {}

/// HTTP/2 Frame Header (9 bytes)
///
/// ```text
/// +-----------------------------------------------+
/// |                 Length (24)                   |
/// +---------------+---------------+---------------+
/// |   Type (8)    |   Flags (8)   |
/// +-+-------------+---------------+-------------------------------+
/// |R|                 Stream Identifier (31)                      |
/// +=+=============================================================+
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Http2FrameHeader {
    pub length: u32,
    pub frame_type: u8,
    pub flags: u8,
    pub stream_id: u32,
}

impl Http2FrameHeader {
    /// Parse HTTP/2 frame header from bytes
    pub fn parse(data: &[u8]) -> Result<Self, Http2ParseError> {
        if data.len() < 9 {
            return Err(Http2ParseError::TooShort);
        }

        // Length (24 bits, big-endian)
        let length = u32::from_be_bytes([0, data[0], data[1], data[2]]);

        // Type and Flags (8 bits each)
        let frame_type = data[3];
        let flags = data[4];

        // Stream ID (31 bits, highest bit reserved)
        let stream_id = u32::from_be_bytes([data[5], data[6], data[7], data[8]]) & 0x7FFF_FFFF;

        Ok(Http2FrameHeader {
            length,
            frame_type,
            flags,
            stream_id,
        })
    }

    /// Check if this is a SETTINGS frame
    pub fn is_settings(&self) -> bool {
        self.frame_type == Http2FrameType::Settings as u8
    }

    /// Get frame type (if known)
    pub fn get_type(&self) -> Result<Http2FrameType, Http2ParseError> {
        Http2FrameType::try_from(self.frame_type)
    }
}

/// HTTP/2 SETTINGS Frame
///
/// SETTINGS frames contain browser-specific configurations that can be used
/// for fingerprinting. Most distinctive is INITIAL_WINDOW_SIZE:
/// - Chrome: 6291456 (6MB)
/// - Firefox: 131072 (128KB)
/// - Safari: 2097152 (2MB)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Http2SettingsFrame {
    pub header: Http2FrameHeader,
    pub settings: Vec<(u16, u32)>,
}

impl Http2SettingsFrame {
    /// Parse SETTINGS frame from bytes
    pub fn parse(data: &[u8]) -> Result<Self, Http2ParseError> {
        // Parse frame header
        let header = Http2FrameHeader::parse(data)?;

        if !header.is_settings() {
            return Err(Http2ParseError::NotSettingsFrame);
        }

        // SETTINGS payload must be multiple of 6 bytes
        if header.length % 6 != 0 {
            return Err(Http2ParseError::InvalidPayloadLength);
        }

        // Check buffer has enough data
        if data.len() < 9 + header.length as usize {
            return Err(Http2ParseError::TooShort);
        }

        // Parse SETTINGS parameters (each 6 bytes)
        let payload = &data[9..9 + header.length as usize];
        let mut settings = Vec::new();

        for chunk in payload.chunks_exact(6) {
            let identifier = u16::from_be_bytes([chunk[0], chunk[1]]);
            let value = u32::from_be_bytes([chunk[2], chunk[3], chunk[4], chunk[5]]);
            settings.push((identifier, value));
        }

        Ok(Http2SettingsFrame { header, settings })
    }

    /// Convert settings to HashMap for easier lookup
    pub fn to_map(&self) -> HashMap<u16, u32> {
        self.settings.iter().cloned().collect()
    }

    /// Get SETTINGS parameter order (useful for fingerprinting)
    pub fn get_order(&self) -> Vec<u16> {
        self.settings.iter().map(|(id, _)| *id).collect()
    }

    /// Get specific setting value
    pub fn get(&self, identifier: u16) -> Option<u32> {
        self.settings
            .iter()
            .find(|(id, _)| *id == identifier)
            .map(|(_, val)| *val)
    }
}

/// HTTP/2 Connection Preface (magic string)
pub const HTTP2_PREFACE: &[u8] = b"PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n";

/// Check if data starts with HTTP/2 connection preface
pub fn is_http2_connection(data: &[u8]) -> bool {
    data.len() >= HTTP2_PREFACE.len() && data.starts_with(HTTP2_PREFACE)
}

/// Find first SETTINGS frame in TCP payload
///
/// Scans through HTTP/2 frames to find the first SETTINGS frame.
/// Automatically skips the connection preface if present.
pub fn find_settings_frame(data: &[u8]) -> Option<Http2SettingsFrame> {
    // Skip HTTP/2 Preface if present (24 bytes)
    let offset = if is_http2_connection(data) {
        24
    } else {
        0
    };

    let mut pos = offset;

    // Scan frames looking for SETTINGS
    while pos + 9 <= data.len() {
        if let Ok(header) = Http2FrameHeader::parse(&data[pos..]) {
            if header.is_settings() && pos + 9 + header.length as usize <= data.len() {
                return Http2SettingsFrame::parse(&data[pos..]).ok();
            }
            // Skip to next frame
            pos += 9 + header.length as usize;
        } else {
            break;
        }
    }

    None
}

/// Browser Type for HTTP/2 fingerprinting
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrowserType {
    Chrome,
    Firefox,
    Safari,
    Edge,
    Opera,
    Unknown,
}

impl fmt::Display for BrowserType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BrowserType::Chrome => write!(f, "Chrome"),
            BrowserType::Firefox => write!(f, "Firefox"),
            BrowserType::Safari => write!(f, "Safari"),
            BrowserType::Edge => write!(f, "Edge"),
            BrowserType::Opera => write!(f, "Opera"),
            BrowserType::Unknown => write!(f, "Unknown"),
        }
    }
}

/// HTTP/2 SETTINGS Browser Matcher
///
/// Matches browser types based on HTTP/2 SETTINGS values.
/// Key discriminator: INITIAL_WINDOW_SIZE (setting ID 4)
/// - Chrome: 6291456 (6MB)
/// - Firefox: 131072 (128KB)  
/// - Safari: 2097152 (2MB)
pub struct Http2SettingsMatcher {
    chrome_settings: HashMap<u16, u32>,
    firefox_settings: HashMap<u16, u32>,
    safari_settings: HashMap<u16, u32>,
}

impl Http2SettingsMatcher {
    /// Create new matcher with default browser SETTINGS
    pub fn new() -> Self {
        let mut chrome_settings = HashMap::new();
        chrome_settings.insert(1, 65536); // HEADER_TABLE_SIZE
        chrome_settings.insert(2, 0); // ENABLE_PUSH
        chrome_settings.insert(3, 1000); // MAX_CONCURRENT_STREAMS
        chrome_settings.insert(4, 6291456); // INITIAL_WINDOW_SIZE (6MB)
        chrome_settings.insert(5, 16384); // MAX_FRAME_SIZE
        chrome_settings.insert(6, 262144); // MAX_HEADER_LIST_SIZE

        let mut firefox_settings = HashMap::new();
        firefox_settings.insert(1, 65536);
        firefox_settings.insert(2, 0);
        firefox_settings.insert(3, 1000);
        firefox_settings.insert(4, 131072); // INITIAL_WINDOW_SIZE (128KB)
        firefox_settings.insert(5, 16384);
        firefox_settings.insert(6, 262144);

        let mut safari_settings = HashMap::new();
        safari_settings.insert(1, 65536);
        safari_settings.insert(2, 1); // ENABLE_PUSH enabled
        safari_settings.insert(3, 100); // Lower MAX_CONCURRENT_STREAMS
        safari_settings.insert(4, 2097152); // INITIAL_WINDOW_SIZE (2MB)
        safari_settings.insert(5, 16384);

        Self {
            chrome_settings,
            firefox_settings,
            safari_settings,
        }
    }

    /// Match browser type from SETTINGS
    ///
    /// Returns (BrowserType, confidence) tuple.
    /// Confidence ranges from 0.0 to 1.0.
    pub fn match_browser(&self, settings: &HashMap<u16, u32>) -> (BrowserType, f64) {
        // Quick check: INITIAL_WINDOW_SIZE (ID 4) is the strongest discriminator
        if let Some(&window_size) = settings.get(&4) {
            return match window_size {
                6291456 => (BrowserType::Chrome, 0.95),
                131072 => (BrowserType::Firefox, 0.95),
                2097152 => (BrowserType::Safari, 0.95),
                _ => {
                    // Fall back to full comparison
                    self.match_browser_full(settings)
                }
            };
        }

        // No INITIAL_WINDOW_SIZE, use full comparison
        self.match_browser_full(settings)
    }

    /// Full browser matching with all SETTINGS
    fn match_browser_full(&self, settings: &HashMap<u16, u32>) -> (BrowserType, f64) {
        let chrome_score = self.calculate_similarity(settings, &self.chrome_settings);
        let firefox_score = self.calculate_similarity(settings, &self.firefox_settings);
        let safari_score = self.calculate_similarity(settings, &self.safari_settings);

        let max_score = chrome_score.max(firefox_score).max(safari_score);

        if max_score < 0.70 {
            return (BrowserType::Unknown, max_score);
        }

        if chrome_score == max_score {
            (BrowserType::Chrome, chrome_score)
        } else if firefox_score == max_score {
            (BrowserType::Firefox, firefox_score)
        } else {
            (BrowserType::Safari, safari_score)
        }
    }

    /// Calculate similarity between two SETTINGS maps (0.0 - 1.0)
    fn calculate_similarity(&self, actual: &HashMap<u16, u32>, expected: &HashMap<u16, u32>) -> f64 {
        if actual.is_empty() || expected.is_empty() {
            return 0.0;
        }

        let mut matched = 0;
        let mut total = 0;

        // Compare each expected setting
        for (key, expected_value) in expected {
            total += 1;
            if let Some(actual_value) = actual.get(key) {
                if actual_value == expected_value {
                    matched += 1;
                } else if *key == 4 && is_valid_window_size(*actual_value) {
                    // Partial match for INITIAL_WINDOW_SIZE variants
                    matched += 1;
                }
            }
        }

        matched as f64 / total as f64
    }
}

impl Default for Http2SettingsMatcher {
    fn default() -> Self {
        Self::new()
    }
}

/// Check if window size is a valid browser value
fn is_valid_window_size(size: u32) -> bool {
    matches!(
        size,
        65535 | 131072 | 262144 | 524288 | 1048576 | 2097152 | 4194304 | 6291456
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_frame_header() {
        let data = vec![
            0x00, 0x00, 0x0C, // Length: 12
            0x04, // Type: SETTINGS
            0x00, // Flags: none
            0x00, 0x00, 0x00, 0x00, // Stream ID: 0
        ];

        let header = Http2FrameHeader::parse(&data).unwrap();
        assert_eq!(header.length, 12);
        assert_eq!(header.frame_type, 4);
        assert_eq!(header.flags, 0);
        assert_eq!(header.stream_id, 0);
        assert!(header.is_settings());
    }

    #[test]
    fn test_parse_settings_frame() {
        let data = vec![
            // Frame Header
            0x00, 0x00, 0x0C, // Length: 12 (2 settings)
            0x04, // Type: SETTINGS
            0x00, // Flags
            0x00, 0x00, 0x00, 0x00, // Stream ID: 0
            // Setting 1: HEADER_TABLE_SIZE = 65536
            0x00, 0x01, // ID: 1
            0x00, 0x01, 0x00, 0x00, // Value: 65536
            // Setting 2: ENABLE_PUSH = 0
            0x00, 0x02, // ID: 2
            0x00, 0x00, 0x00, 0x00, // Value: 0
        ];

        let frame = Http2SettingsFrame::parse(&data).unwrap();
        assert_eq!(frame.settings.len(), 2);
        assert_eq!(frame.settings[0], (1, 65536));
        assert_eq!(frame.settings[1], (2, 0));

        let map = frame.to_map();
        assert_eq!(map.get(&1), Some(&65536));
        assert_eq!(map.get(&2), Some(&0));
    }

    #[test]
    fn test_http2_preface() {
        let data = b"PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n";
        assert!(is_http2_connection(data));

        let data = b"GET / HTTP/1.1\r\n";
        assert!(!is_http2_connection(data));
    }

    #[test]
    fn test_find_settings_frame() {
        // HTTP/2 preface + SETTINGS frame
        let mut data = Vec::new();
        data.extend_from_slice(HTTP2_PREFACE);
        data.extend_from_slice(&[
            0x00, 0x00, 0x06, // Length: 6
            0x04, // Type: SETTINGS
            0x00, // Flags
            0x00, 0x00, 0x00, 0x00, // Stream ID: 0
            0x00, 0x04, // ID: INITIAL_WINDOW_SIZE
            0x00, 0x60, 0x00, 0x00, // Value: 6291456 (Chrome)
        ]);

        let frame = find_settings_frame(&data).unwrap();
        assert_eq!(frame.settings.len(), 1);
        assert_eq!(frame.get(4), Some(6291456));
    }

    #[test]
    fn test_match_chrome() {
        let matcher = Http2SettingsMatcher::new();
        
        let mut settings = HashMap::new();
        settings.insert(4, 6291456); // Chrome INITIAL_WINDOW_SIZE
        
        let (browser, confidence) = matcher.match_browser(&settings);
        assert_eq!(browser, BrowserType::Chrome);
        assert!(confidence >= 0.90);
    }

    #[test]
    fn test_match_firefox() {
        let matcher = Http2SettingsMatcher::new();
        
        let mut settings = HashMap::new();
        settings.insert(4, 131072); // Firefox INITIAL_WINDOW_SIZE
        
        let (browser, confidence) = matcher.match_browser(&settings);
        assert_eq!(browser, BrowserType::Firefox);
        assert!(confidence >= 0.90);
    }

    #[test]
    fn test_match_safari() {
        let matcher = Http2SettingsMatcher::new();
        
        let mut settings = HashMap::new();
        settings.insert(4, 2097152); // Safari INITIAL_WINDOW_SIZE
        
        let (browser, confidence) = matcher.match_browser(&settings);
        assert_eq!(browser, BrowserType::Safari);
        assert!(confidence >= 0.90);
    }

    #[test]
    fn test_match_unknown() {
        let matcher = Http2SettingsMatcher::new();
        
        let mut settings = HashMap::new();
        settings.insert(4, 999999); // Unknown INITIAL_WINDOW_SIZE
        
        let (browser, _) = matcher.match_browser(&settings);
        assert_eq!(browser, BrowserType::Unknown);
    }
}
