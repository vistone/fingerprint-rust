/// TCP Handshake Fingerprinting Module
///
/// Analyzes TCP three-way handshake (SYN, SYN-ACK, ACK) to identify
/// operating system and browser characteristics.
///
/// TCP握手include三个关键阶段：
/// 1. SYN (Client → Server)
/// 2. SYN-ACK (Server → Client)
/// 3. ACK (Client → Server)
///
/// 每个阶段ofTCPoption顺序、窗口size等特performancerecognition浏览器/OSfingerprint
use serde::{Deserialize, Serialize};

/// TCP Option types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TcpOptionType {
    /// MSS (Maximum Segment Size)
    MSS = 2,
    /// Window Scale
    WSCALE = 3,
    /// SACK Permitted
    SACK = 4,
    /// Timestamp
    Timestamp = 8,
    /// TCP Fast Open
    TFO = 34,
}

/// TCP Option in handshake
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TcpOption {
    /// Option type
    pub option_type: TcpOptionType,
    /// Option length
    pub length: u8,
    /// Option value (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<Vec<u8>>,
}

impl TcpOption {
    /// Create MSS option
    pub fn mss(mss_value: u16) -> Self {
        Self {
            option_type: TcpOptionType::MSS,
            length: 4,
            value: Some(mss_value.to_be_bytes().to_vec()),
        }
    }

    /// Create Window Scale option
    pub fn wscale(shift: u8) -> Self {
        Self {
            option_type: TcpOptionType::WSCALE,
            length: 3,
            value: Some(vec![shift]),
        }
    }

    /// Create Timestamp option
    pub fn timestamp(ts_value: u32, ts_echo: u32) -> Self {
        let mut value = ts_value.to_be_bytes().to_vec();
        value.extend_from_slice(&ts_echo.to_be_bytes());
        Self {
            option_type: TcpOptionType::Timestamp,
            length: 10,
            value: Some(value),
        }
    }

    /// Create SACK Permitted option
    pub fn sack_permitted() -> Self {
        Self {
            option_type: TcpOptionType::SACK,
            length: 2,
            value: None,
        }
    }

    /// Create TCP Fast Open option
    pub fn tfo() -> Self {
        Self {
            option_type: TcpOptionType::TFO,
            length: 2,
            value: None,
        }
    }
}

/// TCP packet flags
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct TcpFlags {
    /// SYN flag
    pub syn: bool,
    /// ACK flag
    pub ack: bool,
    /// FIN flag
    pub fin: bool,
    /// RST flag
    pub rst: bool,
    /// PSH flag
    pub psh: bool,
    /// URG flag
    pub urg: bool,
}

/// IP packet characteristics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpCharacteristics {
    /// TTL (Time To Live)
    pub ttl: u8,
    /// DF (Don't Fragment) flag
    pub dont_fragment: bool,
    /// IP ID value (or increment pattern)
    pub ip_id: u32,
    /// IP ID increment between packets
    pub ip_id_increment: Option<u16>,
}

/// SYN packet characteristics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SynCharacteristics {
    /// IP layer characteristics
    pub ip: IpCharacteristics,
    /// TCP flags (SYN should be set)
    pub flags: TcpFlags,
    /// TCP window size
    pub window_size: u16,
    /// TCP options in order
    pub options: Vec<TcpOption>,
    /// Option order signature (e.g., "MSS,WSCALE,SACK,Timestamp")
    pub option_order: String,
}

/// SYN-ACK packet characteristics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SynAckCharacteristics {
    /// IP layer characteristics
    pub ip: IpCharacteristics,
    /// TCP flags (SYN, ACK should be set)
    pub flags: TcpFlags,
    /// TCP window size
    pub window_size: u16,
    /// TCP options in order
    pub options: Vec<TcpOption>,
    /// Option order signature
    pub option_order: String,
}

/// ACK packet characteristics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AckCharacteristics {
    /// IP layer characteristics
    pub ip: IpCharacteristics,
    /// TCP flags (ACK should be set)
    pub flags: TcpFlags,
    /// TCP window size
    pub window_size: u16,
    /// TCP options in order
    pub options: Vec<TcpOption>,
    /// Option order signature
    pub option_order: String,
}

/// Complete TCP three-way handshake fingerprint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TcpHandshakeFingerprint {
    /// SYN packet characteristics
    pub syn: SynCharacteristics,
    /// SYN-ACK packet characteristics
    pub syn_ack: SynAckCharacteristics,
    /// ACK packet characteristics
    pub ack: AckCharacteristics,
    /// Identified operating system
    pub detected_os: Option<String>,
    /// Identified browser
    pub detected_browser: Option<String>,
    /// Confidence level (0.0-1.0)
    pub confidence: f64,
}

impl TcpHandshakeFingerprint {
    /// Create a new handshake fingerprint
    #[must_use]
    pub fn new(
        syn: SynCharacteristics,
        syn_ack: SynAckCharacteristics,
        ack: AckCharacteristics,
    ) -> Self {
        Self {
            syn,
            syn_ack,
            ack,
            detected_os: None,
            detected_browser: None,
            confidence: 0.0,
        }
    }

    /// Get unique handshake signature
    pub fn signature(&self) -> String {
        format!(
            "{}-{}-{}",
            self.syn.option_order, self.syn_ack.option_order, self.ack.option_order
        )
    }

    /// Get TTL sequence
    pub fn ttl_sequence(&self) -> (u8, u8, u8) {
        (self.syn.ip.ttl, self.syn_ack.ip.ttl, self.ack.ip.ttl)
    }

    /// Get window size sequence
    pub fn window_sequence(&self) -> (u16, u16, u16) {
        (
            self.syn.window_size,
            self.syn_ack.window_size,
            self.ack.window_size,
        )
    }
}

/// TCP Handshake Finder/Analyzer
pub struct TcpHandshakeAnalyzer;

impl TcpHandshakeAnalyzer {
    /// Detect operating system from TCP handshake
    ///
    /// # Examples
    ///
    /// ```
    /// use fingerprint_core::tcp_handshake::*;
    ///
    /// let ttl_sequence = (64, 64, 64);  // Linux/macOS
    /// let os = TcpHandshakeAnalyzer::detect_os(ttl_sequence);
    /// assert!(os.is_some());
    /// ```
    pub fn detect_os(ttl_sequence: (u8, u8, u8)) -> Option<String> {
        match ttl_sequence {
            // Windows: TTL=128 or 127 (after one hop)
            (128, 128, 128) | (127, 127, 127) | (128, 127, 127) => Some("Windows".to_string()),
            // Unix-like (macOS, Linux, iOS, Android): TTL=64
            (64, 64, 64) => Some("Unix-like".to_string()),
            // Mixed TTL values detected
            _ => None,
        }
    }

    /// Detect browser from TCP options order
    pub fn detect_browser(option_order: &str) -> Option<String> {
        match option_order {
            // Chrome/Chromium/Edge: MSS,WSCALE,SACK,Timestamp
            "MSS,WSCALE,SACK,Timestamp" => Some("Chrome/Chromium/Edge".to_string()),
            // Firefox: MSS,WSCALE,Timestamp,SACK
            "MSS,WSCALE,Timestamp,SACK" => Some("Firefox".to_string()),
            // Safari: MSS,WSCALE,Timestamp
            "MSS,WSCALE,Timestamp" => Some("Safari".to_string()),
            _ => None,
        }
    }

    /// Detect browser from MSS (Maximum Segment Size)
    pub fn detect_from_mss(mss: u16) -> Option<String> {
        match mss {
            // Chrome/Edge/Opera: 1460
            1460 => Some("Chrome/Edge/Opera".to_string()),
            // Firefox: 1440
            1440 => Some("Firefox".to_string()),
            // Custom values
            _ => None,
        }
    }

    /// Detect browser from Window Size
    pub fn detect_from_window_size(window_size: u16) -> Option<String> {
        match window_size {
            // Windows Chrome: 64240
            64240 => Some("Windows Chrome".to_string()),
            // macOS Chrome: 65535 or 45928
            65535 => Some("macOS Chrome/Safari".to_string()),
            // Linux Chrome/Firefox: 65535
            _ => None,
        }
    }

    /// Get detailed TCP fingerprint info
    pub fn get_fingerprint_info(fingerprint: &TcpHandshakeFingerprint) -> String {
        let syn_opts = fingerprint
            .syn
            .options
            .iter()
            .map(|o| format!("{:?}", o.option_type))
            .collect::<Vec<_>>()
            .join(",");

        let syn_ack_opts = fingerprint
            .syn_ack
            .options
            .iter()
            .map(|o| format!("{:?}", o.option_type))
            .collect::<Vec<_>>()
            .join(",");

        format!(
            "TCP Handshake Fingerprint\n\
             ├─ SYN: TTL={}, Window={}, Options=[{}]\n\
             ├─ SYN-ACK: TTL={}, Window={}, Options=[{}]\n\
             ├─ ACK: TTL={}, Window={}, Options=[{}]\n\
             ├─ Signature: {}\n\
             ├─ Detected OS: {}\n\
             ├─ Detected Browser: {}\n\
             └─ Confidence: {:.2}%",
            fingerprint.syn.ip.ttl,
            fingerprint.syn.window_size,
            syn_opts,
            fingerprint.syn_ack.ip.ttl,
            fingerprint.syn_ack.window_size,
            syn_ack_opts,
            fingerprint.ack.ip.ttl,
            fingerprint.ack.window_size,
            fingerprint
                .ack
                .options
                .iter()
                .map(|o| format!("{:?}", o.option_type))
                .collect::<Vec<_>>()
                .join(","),
            fingerprint.signature(),
            fingerprint.detected_os.as_deref().unwrap_or("Unknown"),
            fingerprint.detected_browser.as_deref().unwrap_or("Unknown"),
            fingerprint.confidence * 100.0
        )
    }
}

/// Common TCP handshake signatures for known OS/browsers
pub mod signatures {
    /// Chrome on Windows 11
    pub const CHROME_WIN11: &str =
        "MSS,WSCALE,SACK,Timestamp-MSS,WSCALE,SACK,Timestamp-MSS,WSCALE,SACK,Timestamp";

    /// Chrome on macOS
    pub const CHROME_MACOS: &str = "MSS,WSCALE,Timestamp-MSS,WSCALE,Timestamp-MSS,WSCALE,Timestamp";

    /// Firefox on Windows
    pub const FIREFOX_WIN: &str =
        "MSS,WSCALE,Timestamp,SACK-MSS,WSCALE,Timestamp,SACK-MSS,WSCALE,Timestamp,SACK";

    /// Firefox on Linux
    pub const FIREFOX_LINUX: &str =
        "MSS,WSCALE,Timestamp,SACK-MSS,WSCALE,Timestamp,SACK-MSS,WSCALE,Timestamp,SACK";

    /// Safari on macOS
    pub const SAFARI_MACOS: &str = "MSS,WSCALE,Timestamp-MSS,WSCALE,Timestamp-MSS,WSCALE,Timestamp";

    /// Safari on iOS
    pub const SAFARI_IOS: &str = "MSS,WSCALE,Timestamp-MSS,WSCALE,Timestamp-MSS,WSCALE,Timestamp";

    /// Edge on Windows
    pub const EDGE_WIN: &str =
        "MSS,WSCALE,SACK,Timestamp-MSS,WSCALE,SACK,Timestamp-MSS,WSCALE,SACK,Timestamp";

    /// Get signature for browser/OS combination
    pub fn get_signature(browser: &str, os: &str) -> Option<&'static str> {
        match (browser.to_lowercase().as_str(), os.to_lowercase().as_str()) {
            ("chrome", "windows") => Some(CHROME_WIN11),
            ("chrome", "macos") => Some(CHROME_MACOS),
            ("firefox", "windows") => Some(FIREFOX_WIN),
            ("firefox", "linux") => Some(FIREFOX_LINUX),
            ("safari", "macos") => Some(SAFARI_MACOS),
            ("safari", "ios") => Some(SAFARI_IOS),
            ("edge", "windows") => Some(EDGE_WIN),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tcp_option_creation() {
        let mss = TcpOption::mss(1460);
        assert_eq!(mss.option_type, TcpOptionType::MSS);
        assert_eq!(mss.length, 4);

        let wscale = TcpOption::wscale(8);
        assert_eq!(wscale.option_type, TcpOptionType::WSCALE);
    }

    #[test]
    fn test_tcp_flags() {
        let flags = TcpFlags {
            syn: true,
            ..Default::default()
        };
        assert!(flags.syn);
        assert!(!flags.ack);
    }

    #[test]
    fn test_os_detection() {
        let os = TcpHandshakeAnalyzer::detect_os((128, 128, 128));
        assert!(os.is_some());
        assert_eq!(os.unwrap(), "Windows");

        let os = TcpHandshakeAnalyzer::detect_os((64, 64, 64));
        assert!(os.is_some());
    }

    #[test]
    fn test_browser_detection_from_options() {
        let browser = TcpHandshakeAnalyzer::detect_browser("MSS,WSCALE,SACK,Timestamp");
        assert!(browser.is_some());
        assert_eq!(browser.unwrap(), "Chrome/Chromium/Edge");

        let browser = TcpHandshakeAnalyzer::detect_browser("MSS,WSCALE,Timestamp,SACK");
        assert!(browser.is_some());
        assert_eq!(browser.unwrap(), "Firefox");
    }

    #[test]
    fn test_mss_detection() {
        let browser = TcpHandshakeAnalyzer::detect_from_mss(1460);
        assert!(browser.is_some());

        let browser = TcpHandshakeAnalyzer::detect_from_mss(1440);
        assert!(browser.is_some());
    }

    #[test]
    fn test_handshake_signature() {
        let syn = SynCharacteristics {
            ip: IpCharacteristics {
                ttl: 64,
                dont_fragment: true,
                ip_id: 0,
                ip_id_increment: None,
            },
            flags: TcpFlags {
                syn: true,
                ..Default::default()
            },
            window_size: 65535,
            options: vec![TcpOption::mss(1460), TcpOption::wscale(8)],
            option_order: "MSS,WSCALE".to_string(),
        };

        let syn_ack = SynAckCharacteristics {
            ip: IpCharacteristics {
                ttl: 64,
                dont_fragment: true,
                ip_id: 1,
                ip_id_increment: Some(1),
            },
            flags: TcpFlags {
                syn: true,
                ack: true,
                ..Default::default()
            },
            window_size: 65535,
            options: vec![TcpOption::mss(1460), TcpOption::wscale(8)],
            option_order: "MSS,WSCALE".to_string(),
        };

        let ack = AckCharacteristics {
            ip: IpCharacteristics {
                ttl: 64,
                dont_fragment: true,
                ip_id: 2,
                ip_id_increment: Some(1),
            },
            flags: TcpFlags {
                ack: true,
                ..Default::default()
            },
            window_size: 65535,
            options: vec![],
            option_order: "".to_string(),
        };

        let fingerprint = TcpHandshakeFingerprint::new(syn, syn_ack, ack);
        assert_eq!(fingerprint.signature(), "MSS,WSCALE-MSS,WSCALE-");
    }

    #[test]
    fn test_ttl_sequence() {
        let syn = SynCharacteristics {
            ip: IpCharacteristics {
                ttl: 64,
                dont_fragment: true,
                ip_id: 0,
                ip_id_increment: None,
            },
            flags: TcpFlags {
                syn: true,
                ..Default::default()
            },
            window_size: 65535,
            options: Vec::new(),
            option_order: "".to_string(),
        };

        let syn_ack = SynAckCharacteristics {
            ip: IpCharacteristics {
                ttl: 64,
                dont_fragment: true,
                ip_id: 1,
                ip_id_increment: None,
            },
            flags: TcpFlags {
                syn: true,
                ack: true,
                ..Default::default()
            },
            window_size: 65535,
            options: Vec::new(),
            option_order: "".to_string(),
        };

        let ack = AckCharacteristics {
            ip: IpCharacteristics {
                ttl: 64,
                dont_fragment: true,
                ip_id: 2,
                ip_id_increment: None,
            },
            flags: TcpFlags {
                ack: true,
                ..Default::default()
            },
            window_size: 65535,
            options: Vec::new(),
            option_order: "".to_string(),
        };

        let fingerprint = TcpHandshakeFingerprint::new(syn, syn_ack, ack);
        let ttl_seq = fingerprint.ttl_sequence();
        assert_eq!(ttl_seq, (64, 64, 64));
    }
}
