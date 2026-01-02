//! TCP fingerprint核心type
//!
//! 定义 TCP fingerprint的核心count据struct。

use crate::fingerprint::{Fingerprint, FingerprintType};
use crate::metadata::FingerprintMetadata;
use std::hash::{Hash, Hasher};

/// TCP configuration描述file
///  for 主动configuration出口connection TCP parameter
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
            window_size: 64240, // 典型value
            mss: None,          // operating systemdefault
            window_scale: None, // operating systemdefault
        }
    }
}

impl TcpProfile {
    /// Based onoperating systemtypeGenerates corresponding TCP Profile
    ///
    /// 确保 TCP fingerprint and browserfingerprint（User-Agent）一致
    pub fn for_os(os: crate::types::OperatingSystem) -> Self {
        match os {
            crate::types::OperatingSystem::Windows10 | crate::types::OperatingSystem::Windows11 => {
                // Windows: TTL=128, Window Size=64240 (Windows 10/11 典型value)
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
                // macOS: TTL=64, Window Size=65535 (macOS 典型value)
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
                // Linux: TTL=64, Window Size=65535 (Linux 典型value)
                Self {
                    ttl: 64,
                    window_size: 65535,
                    mss: Some(1460),
                    window_scale: Some(7),
                }
            }
        }
    }

    ///  from  User-Agent string推断operating system并Generates corresponding TCP Profile
    ///
    /// 这是统一fingerprintGenerate的核心function，确保browserfingerprint and TCP fingerprintsync
    pub fn from_user_agent(user_agent: &str) -> Self {
        use crate::types::OperatingSystem;

        //  from  User-Agent 推断operating system
        // Note: iPhone/iPad  User-Agent including "Mac OS X"，need先Check移动设备
        let os = if user_agent.contains("iPhone") || user_agent.contains("iPad") {
            // iOS 设备：use macOS  TCP fingerprint（iOS 基于 macOS）
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
            // defaultuse Windows（最常见的browser环境）
            OperatingSystem::Windows10
        };

        Self::for_os(os)
    }

    ///  from platformstring（如 "Windows", "macOS", "Linux"）Generate TCP Profile
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
    /// fingerprint ID（基于 TCP trait的hash）
    pub id: String,

    /// TTL
    pub ttl: u8,

    /// Window Size
    pub window_size: u16,

    /// MSS (Maximum Segment Size)
    pub mss: Option<u16>,

    /// Window Scale
    pub window_scale: Option<u8>,

    /// TCP optionsstring（ for  p0f 兼容）
    pub options_str: Option<String>,

    /// metadata
    pub metadata: FingerprintMetadata,
}

impl TcpFingerprint {
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

    /// Createcomplete TCP fingerprint
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

    /// 推断initialbeginning TTL
    pub fn infer_initial_ttl(&self) -> u8 {
        // Based on TTL 推断initialbeginning TTL
        // 常见的initialbeginning TTL value：64 (Linux), 128 (Windows), 255 (Unix)
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

        // TCP fingerprint的相似度判断：允许一定的容差
        // 这里简化process，实际should考虑 TTL 的推断value、Window Size 的倍count关系等
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
}
