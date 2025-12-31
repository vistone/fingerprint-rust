//! TCP 指纹核心类型
//!
//! 定义 TCP 指纹的核心数据结构。

use crate::fingerprint::{Fingerprint, FingerprintType};
use crate::metadata::FingerprintMetadata;
use std::hash::{Hash, Hasher};

/// TCP 配置描述文件
/// 用于主动配置出口连接的 TCP 参数
#[derive(Debug, Clone, Copy)]
pub struct TcpProfile {
    /// 初始 TTL
    pub ttl: u8,

    /// 初始窗口大小
    pub window_size: u16,

    /// MSS (Maximum Segment Size)
    pub mss: Option<u16>,

    /// Window Scale
    pub window_scale: Option<u8>,
}

impl Default for TcpProfile {
    fn default() -> Self {
        Self {
            ttl: 64,            // Linux 默认
            window_size: 64240, // 典型值
            mss: None,          // 操作系统默认
            window_scale: None, // 操作系统默认
        }
    }
}

impl TcpProfile {
    /// 根据操作系统类型生成对应的 TCP Profile
    ///
    /// 确保 TCP 指纹与浏览器指纹（User-Agent）一致
    pub fn for_os(os: crate::types::OperatingSystem) -> Self {
        match os {
            crate::types::OperatingSystem::Windows10 | crate::types::OperatingSystem::Windows11 => {
                // Windows: TTL=128, Window Size=64240 (Windows 10/11 典型值)
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
                // macOS: TTL=64, Window Size=65535 (macOS 典型值)
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
                // Linux: TTL=64, Window Size=65535 (Linux 典型值)
                Self {
                    ttl: 64,
                    window_size: 65535,
                    mss: Some(1460),
                    window_scale: Some(7),
                }
            }
        }
    }

    /// 从 User-Agent 字符串推断操作系统并生成对应的 TCP Profile
    ///
    /// 这是统一指纹生成的核心函数，确保浏览器指纹和 TCP 指纹同步
    pub fn from_user_agent(user_agent: &str) -> Self {
        use crate::types::OperatingSystem;

        // 从 User-Agent 推断操作系统
        // 注意：iPhone/iPad 的 User-Agent 包含 "Mac OS X"，需要先检查移动设备
        let os = if user_agent.contains("iPhone") || user_agent.contains("iPad") {
            // iOS 设备：使用 macOS 的 TCP 指纹（iOS 基于 macOS）
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
            // 默认使用 Windows（最常见的浏览器环境）
            OperatingSystem::Windows10
        };

        Self::for_os(os)
    }

    /// 从平台字符串（如 "Windows", "macOS", "Linux"）生成 TCP Profile
    pub fn from_platform(platform: &str) -> Self {
        use crate::types::OperatingSystem;

        let os = match platform.to_lowercase().as_str() {
            "windows" | r#""Windows""# => OperatingSystem::Windows10,
            "macos" | r#""macOS""# => OperatingSystem::MacOS14,
            "linux" | r#""Linux""# => OperatingSystem::Linux,
            _ => OperatingSystem::Windows10, // 默认
        };

        Self::for_os(os)
    }
}

/// TCP 指纹
#[derive(Debug, Clone)]
pub struct TcpFingerprint {
    /// 指纹 ID（基于 TCP 特征的哈希）
    pub id: String,

    /// TTL
    pub ttl: u8,

    /// Window Size
    pub window_size: u16,

    /// MSS (Maximum Segment Size)
    pub mss: Option<u16>,

    /// Window Scale
    pub window_scale: Option<u8>,

    /// TCP 选项字符串（用于 p0f 兼容）
    pub options_str: Option<String>,

    /// 元数据
    pub metadata: FingerprintMetadata,
}

impl TcpFingerprint {
    /// 创建新的 TCP 指纹
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

    /// 创建完整的 TCP 指纹
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

    /// 计算指纹 ID
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

    /// 推断初始 TTL
    pub fn infer_initial_ttl(&self) -> u8 {
        // 根据 TTL 推断初始 TTL
        // 常见的初始 TTL 值：64 (Linux), 128 (Windows), 255 (Unix)
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

        // TCP 指纹的相似度判断：允许一定的容差
        // 这里简化处理，实际应该考虑 TTL 的推断值、Window Size 的倍数关系等
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
