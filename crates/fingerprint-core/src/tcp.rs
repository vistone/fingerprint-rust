//! TCP Fingerprint Type
//!
//! Defines the TCP fingerprint data structure.

use crate::fingerprint::{Fingerprint, FingerprintType};
use crate::metadata::FingerprintMetadata;
use std::hash::{Hash, Hasher};

/// TCP configuration
/// for configuring TCP parameters for connections
#[derive(Debug, Clone, Copy)]
pub struct TcpProfile {
    /// Initial Time-To-Live value
    pub ttl: u8,

    /// Initial window size
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
            window_size: 64240, // Typical value for Windows
            mss: None,          // Operating system default
            window_scale: None, // Operating system default
        }
    }
}

impl TcpProfile {
    /// Generates a TCP profile based on the operating system type
    ///
    /// Ensures TCP fingerprint matches browser fingerprint (User-Agent)
    pub fn for_os(os: crate::types::OperatingSystem) -> Self {
        match os {
            crate::types::OperatingSystem::Windows10 | crate::types::OperatingSystem::Windows11 => {
                // Windows: TTL=128, Window Size=64240 (Windows 10/11 typical values)
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
                // macOS: TTL=64, Window Size=65535 (macOS typical values)
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
                // Linux: TTL=64, Window Size=65535 (Linux typical values)
                Self {
                    ttl: 64,
                    window_size: 65535,
                    mss: Some(1460),
                    window_scale: Some(7),
                }
            }
        }
    }

    /// Generates a TCP profile by inferring the operating system from User-Agent string
    ///
    /// This is a core function for unified fingerprint generation, ensuring browser fingerprint and TCP fingerprint are synchronized
    pub fn from_user_agent(user_agent: &str) -> Self {
        use crate::types::OperatingSystem;

        // Infer operating system from User-Agent
        // Note: iPhone/iPad User-Agent contains "Mac OS X", need to check mobile device first
        let os = if user_agent.contains("iPhone") || user_agent.contains("iPad") {
            // iOS device: use macOS TCP fingerprint (iOS based on macOS)
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
            // Default to Windows (most common browser environment)
            OperatingSystem::Windows10
        };

        Self::for_os(os)
    }

    /// Generates a TCP profile from platform string (e.g., "Windows", "macOS", "Linux")
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
    /// Fingerprint ID (based on TCP trait hash)
    pub id: String,

    /// Time-To-Live value
    pub ttl: u8,

    /// Transmission window size
    pub window_size: u16,

    /// MSS (Maximum Segment Size)
    pub mss: Option<u16>,

    /// Window scale option
    pub window_scale: Option<u8>,

    /// TCP options string (for p0f compatibility)
    pub options_str: Option<String>,

    /// metadata
    pub metadata: FingerprintMetadata,
}

impl TcpFingerprint {
    /// Valid MSS range (minimum Ethernet-compatible MTU minus IP+TCP headers)
    /// Minimum: 536 (RFC 879 minimum), Maximum: 9000 (practical jumbo frame limit)
    const MIN_MSS: u16 = 536;
    const MAX_MSS: u16 = 9000; // Conservative jumbo frame MSS limit

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
    /// - MSS: 536-9000 (RFC 879 minimum to practical jumbo frame limit)
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

    /// Calculates the fingerprint ID
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

    /// Infers the initial TTL according to common OS defaults
    pub fn infer_initial_ttl(&self) -> u8 {
        // Based on current TTL, infer the initial TTL
        // Common initial TTL values: 64 (Linux), 128 (Windows), 255 (Unix)
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

        // TCP fingerprint similarity judgment: allow certain tolerance
        // Simplified process here, should actually consider TTL inferred value, Window Size multiple close system etc.
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
    fn test_new_validated_mss_at_boundaries() {
        // Test MSS at minimum boundary (should pass)
        let fp_min = TcpFingerprint::new_validated(64, 65535, Some(536), Some(7));
        assert!(fp_min.is_ok());

        // Test MSS at maximum boundary (should pass)
        let fp_max = TcpFingerprint::new_validated(64, 65535, Some(9000), Some(7));
        assert!(fp_max.is_ok());

        // Test MSS just above maximum (should fail)
        let fp_over = TcpFingerprint::new_validated(64, 65535, Some(9001), Some(7));
        assert!(fp_over.is_err());

        // Test MSS just below minimum (should fail)
        let fp_under = TcpFingerprint::new_validated(64, 65535, Some(535), Some(7));
        assert!(fp_under.is_err());
    }

    #[test]
    fn test_new_validated_window_scale_too_large() {
        let fp = TcpFingerprint::new_validated(64, 65535, Some(1460), Some(15));
        assert!(fp.is_err());
        assert!(fp.unwrap_err().contains("Window scale"));
    }

    #[test]
    fn test_new_validated_window_scale_at_boundary() {
        // Test window scale at maximum (should pass)
        let fp_max = TcpFingerprint::new_validated(64, 65535, Some(1460), Some(14));
        assert!(fp_max.is_ok());

        // Test window scale at zero (should pass)
        let fp_zero = TcpFingerprint::new_validated(64, 65535, Some(1460), Some(0));
        assert!(fp_zero.is_ok());
    }

    #[test]
    fn test_new_validated_none_options() {
        let fp = TcpFingerprint::new_validated(128, 32768, None, None);
        assert!(fp.is_ok());
    }
}
