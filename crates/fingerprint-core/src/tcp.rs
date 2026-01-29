//! TCP fingerprintcoretype
//!
//! define TCP fingerprintcorecountdatastruct.

use crate::fingerprint::{Fingerprint, FingerprintType};
use crate::metadata::FingerprintMetadata;
use std::hash::{Hash, Hasher};

/// TCP configurationdescribefile
/// for main动configurationexitconnection TCP parameter
#[derive(Debug, Clone, Copy)]
pub struct TcpProfile {
    /// initialbeginning TTL
    pub ttl: u8,

    /// initialbeginningwindowsize
    pub window_size: u16,

    /// MSS (Maximum Segment Size)
    pub mss: Option<u16>,

    /// Window Scale
    pub window_scale: Option<u8>,
}

impl Default for TcpProfile {
    fn default() -> Self {
        Self {
            ttl: 64,            // Linux default
            window_size: 64240, // typicalvalue
            mss: None,          // operating systemdefault
            window_scale: None, // operating systemdefault
        }
    }
}

impl TcpProfile {
    /// Based onoperating systemtypeGenerates corresponding TCP Profile
    ///
    /// ensure TCP fingerprint and browserfingerprint (User-Agent)consistent
    pub fn for_os(os: crate::types::OperatingSystem) -> Self {
        match os {
            crate::types::OperatingSystem::Windows10 | crate::types::OperatingSystem::Windows11 => {
                // Windows: TTL=128, Window Size=64240 (Windows 10/11 typicalvalue)
                Self {
                    ttl: 128,
                    window_size: 64240,
                    mss: Some(1460),
                    window_scale: Some(8),
                }
            }
            crate::types::OperatingSystem::MacOS13
            | crate::types::OperatingSystem::MacOS14
            | crate::types::OperatingSystem::MacOS15 => {
                // macOS: TTL=64, Window Size=65535 (macOS typicalvalue)
                Self {
                    ttl: 64,
                    window_size: 65535,
                    mss: Some(1460),
                    window_scale: Some(6),
                }
            }
            crate::types::OperatingSystem::Linux
            | crate::types::OperatingSystem::LinuxUbuntu
            | crate::types::OperatingSystem::LinuxDebian => {
                // Linux: TTL=64, Window Size=65535 (Linux typicalvalue)
                Self {
                    ttl: 64,
                    window_size: 65535,
                    mss: Some(1460),
                    window_scale: Some(7),
                }
            }
        }
    }

    /// from User-Agent stringinferoperating system并Generates corresponding TCP Profile
    ///
    /// this isunifiedfingerprintGeneratecorefunction, ensurebrowserfingerprint and TCP fingerprintsync
    pub fn from_user_agent(user_agent: &str) -> Self {
        use crate::types::OperatingSystem;

        // from User-Agent inferoperating system
        // Note: iPhone/iPad User-Agent including "Mac OS X", need先Checkmovedevice
        let os = if user_agent.contains("iPhone") || user_agent.contains("iPad") {
            // iOS device：use macOS TCP fingerprint (iOS based on macOS)
            OperatingSystem::MacOS14
        } else if user_agent.contains("Windows NT 10.0") {
            OperatingSystem::Windows10
        } else if user_agent.contains("Windows NT 11.0") {
            OperatingSystem::Windows11
        } else if user_agent.contains("Mac OS X 13")
            || user_agent.contains("Macintosh; Intel Mac OS X 13")
        {
            OperatingSystem::MacOS13
        } else if user_agent.contains("Mac OS X 14")
            || user_agent.contains("Macintosh; Intel Mac OS X 14")
        {
            OperatingSystem::MacOS14
        } else if user_agent.contains("Mac OS X 15")
            || user_agent.contains("Macintosh; Intel Mac OS X 15")
        {
            OperatingSystem::MacOS15
        } else if user_agent.contains("Linux") || user_agent.contains("Android") {
            OperatingSystem::Linux
        } else {
            // defaultuse Windows (most commonbrowserenvironment)
            OperatingSystem::Windows10
        };

        Self::for_os(os)
    }

    /// from platformstring (如 "Windows", "macOS", "Linux")Generate TCP Profile
    pub fn from_platform(platform: &str) -> Self {
        use crate::types::OperatingSystem;

        let os = match platform.to_lowercase().as_str() {
            "windows" | r#""Windows""# => OperatingSystem::Windows10,
            "macos" | r#""macOS""# => OperatingSystem::MacOS14,
            "linux" | r#""Linux""# => OperatingSystem::Linux,
            _ => OperatingSystem::Windows10, // default
        };

        Self::for_os(os)
    }
}

/// TCP fingerprint
#[derive(Debug, Clone)]
pub struct TcpFingerprint {
    /// fingerprint ID (based on TCP traithash)
    pub id: String,

    /// TTL
    pub ttl: u8,

    /// Window Size
    pub window_size: u16,

    /// MSS (Maximum Segment Size)
    pub mss: Option<u16>,

    /// Window Scale
    pub window_scale: Option<u8>,

    /// TCP optionsstring ( for p0f compatible)
    pub options_str: Option<String>,

    /// metadata
    pub metadata: FingerprintMetadata,
}

impl TcpFingerprint {
    /// Valid MSS range (minimum Ethernet-compatible MTU minus IP+TCP headers)
    /// Minimum: 536 (RFC 879 minimum), Maximum: 9000 (jumbo frames)
    const MIN_MSS: u16 = 536;
    const MAX_MSS: u16 = 9000; // Jumbo frame MTU (9000) minus headers

    /// Valid window scale range (0-14 per RFC 7323)
    const MAX_WINDOW_SCALE: u8 = 14;

    /// Create a new TCP fingerprint
    pub fn new(ttl: u8, window_size: u16) -> Self {
        let id = Self::calculate_id(ttl, window_size, None, None);
        Self {
            id,
            ttl,
            window_size,
            mss: None,
            window_scale: None,
            options_str: None,
            metadata: FingerprintMetadata::new(),
        }
    }

    /// Create a TCP fingerprint with validation
    ///
    /// Returns an error if parameters are outside valid ranges:
    /// - MSS: 536-65535 (RFC 879 minimum)
    /// - Window Scale: 0-14 (RFC 7323)
    ///
    /// # Example
    ///
    /// ```
    /// use fingerprint_core::tcp::TcpFingerprint;
    /// 
    /// let fp = TcpFingerprint::new_validated(64, 65535, Some(1460), Some(7));
    /// assert!(fp.is_ok());
    ///
    /// let invalid = TcpFingerprint::new_validated(64, 65535, Some(100), Some(7));
    /// assert!(invalid.is_err()); // MSS too small
    /// ```
    pub fn new_validated(
        ttl: u8,
        window_size: u16,
        mss: Option<u16>,
        window_scale: Option<u8>,
    ) -> Result<Self, String> {
        // Validate MSS if provided
        if let Some(mss_val) = mss {
            if mss_val < Self::MIN_MSS {
                return Err(format!(
                    "MSS value {} is below minimum {} (RFC 879)",
                    mss_val,
                    Self::MIN_MSS
                ));
            }
            if mss_val > Self::MAX_MSS {
                return Err(format!(
                    "MSS value {} exceeds maximum {}",
                    mss_val,
                    Self::MAX_MSS
                ));
            }
        }

        // Validate window scale if provided
        if let Some(ws_val) = window_scale {
            if ws_val > Self::MAX_WINDOW_SCALE {
                return Err(format!(
                    "Window scale {} exceeds maximum {} (RFC 7323)",
                    ws_val,
                    Self::MAX_WINDOW_SCALE
                ));
            }
        }

        Ok(Self::with_options(ttl, window_size, mss, window_scale))
    }

    /// Create complete TCP fingerprint (no validation, use new_validated for safe construction)
    pub fn with_options(
        ttl: u8,
        window_size: u16,
        mss: Option<u16>,
        window_scale: Option<u8>,
    ) -> Self {
        let id = Self::calculate_id(ttl, window_size, mss, window_scale);
        Self {
            id,
            ttl,
            window_size,
            mss,
            window_scale,
            options_str: None,
            metadata: FingerprintMetadata::new(),
        }
    }

    /// Calculatefingerprint ID
    fn calculate_id(
        ttl: u8,
        window_size: u16,
        mss: Option<u16>,
        window_scale: Option<u8>,
    ) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update([ttl]);
        hasher.update(window_size.to_be_bytes());
        if let Some(mss_val) = mss {
            hasher.update(mss_val.to_be_bytes());
        }
        if let Some(ws_val) = window_scale {
            hasher.update([ws_val]);
        }
        format!("{:x}", hasher.finalize())
    }

    /// inferinitialbeginning TTL
    pub fn infer_initial_ttl(&self) -> u8 {
        // Based on TTL inferinitialbeginning TTL
        // commoninitialbeginning TTL value：64 (Linux), 128 (Windows), 255 (Unix)
        if self.ttl <= 64 {
            64
        } else if self.ttl <= 128 {
            128
        } else {
            255
        }
    }
}

impl Fingerprint for TcpFingerprint {
    fn fingerprint_type(&self) -> FingerprintType {
        FingerprintType::Tcp
    }

    fn id(&self) -> String {
        self.id.clone()
    }

    fn metadata(&self) -> &FingerprintMetadata {
        &self.metadata
    }

    fn metadata_mut(&mut self) -> &mut FingerprintMetadata {
        &mut self.metadata
    }

    fn hash(&self) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        let mut hasher = DefaultHasher::new();
        self.ttl.hash(&mut hasher);
        self.window_size.hash(&mut hasher);
        self.mss.hash(&mut hasher);
        self.window_scale.hash(&mut hasher);
        hasher.finish()
    }

    fn similar_to(&self, other: &dyn Fingerprint) -> bool {
        if other.fingerprint_type() != FingerprintType::Tcp {
            return false;
        }

        // TCP fingerprintsimilardegreejudge：allowcertain tolerance
        // heresimplifyprocess, actualshouldconsider TTL infervalue, Window Size 倍countclosesystem etc.
        self.hash() == other.hash()
    }

    fn to_string(&self) -> String {
        format!(
            "TcpFingerprint(id={}, ttl={}, window={})",
            self.id, self.ttl, self.window_size
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tcp_fingerprint_new() {
        let fp = TcpFingerprint::new(64, 65535);
        assert!(!fp.id.is_empty());
        assert_eq!(fp.ttl, 64);
        assert_eq!(fp.window_size, 65535);
    }

    #[test]
    fn test_tcp_fingerprint_with_options() {
        let fp = TcpFingerprint::with_options(64, 65535, Some(1460), Some(7));
        assert_eq!(fp.mss, Some(1460));
        assert_eq!(fp.window_scale, Some(7));
    }

    #[test]
    fn test_infer_initial_ttl() {
        let fp1 = TcpFingerprint::new(64, 65535);
        assert_eq!(fp1.infer_initial_ttl(), 64);

        let fp2 = TcpFingerprint::new(128, 65535);
        assert_eq!(fp2.infer_initial_ttl(), 128);

        let fp3 = TcpFingerprint::new(200, 65535);
        assert_eq!(fp3.infer_initial_ttl(), 255);
    }

    #[test]
    fn test_new_validated_valid_params() {
        let fp = TcpFingerprint::new_validated(64, 65535, Some(1460), Some(7));
        assert!(fp.is_ok());
        let fp = fp.unwrap();
        assert_eq!(fp.ttl, 64);
        assert_eq!(fp.window_size, 65535);
        assert_eq!(fp.mss, Some(1460));
        assert_eq!(fp.window_scale, Some(7));
    }

    #[test]
    fn test_new_validated_mss_too_small() {
        let fp = TcpFingerprint::new_validated(64, 65535, Some(100), Some(7));
        assert!(fp.is_err());
        assert!(fp.unwrap_err().contains("MSS"));
    }

    #[test]
    fn test_new_validated_window_scale_too_large() {
        let fp = TcpFingerprint::new_validated(64, 65535, Some(1460), Some(15));
        assert!(fp.is_err());
        assert!(fp.unwrap_err().contains("Window scale"));
    }

    #[test]
    fn test_new_validated_none_options() {
        let fp = TcpFingerprint::new_validated(128, 32768, None, None);
        assert!(fp.is_ok());
    }
}
