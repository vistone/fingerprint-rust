//! HASSH SSH 指纹实现
//!
//! HASSH 是 Salesforce 开发的 SSH 客户端/服务器指纹识别方法。
//! 类似于 JA3 for TLS，HASSH 用于识别 SSH 客户端和服务器。
//!
//! ## 参考
//! - 论文: "HASSH - Profiling Method for SSH Clients and Servers" (Salesforce, 2018)
//! - GitHub: https://github.com/salesforce/hassh
//!
//! ## 算法
//! HASSH = MD5(Client KEX Algorithms;Encryption Algorithms;MAC Algorithms;Compression Algorithms)

use serde::{Deserialize, Serialize};

/// HASSH SSH 客户端指纹
///
/// 格式: MD5(KEX;EncryptionAlgs;MACAlgs;CompressionAlgs)
///
/// ## 示例
/// ```
/// use fingerprint_core::hassh::HASSH;
///
/// let hassh = HASSH::generate(
///     &["diffie-hellman-group14-sha1", "diffie-hellman-group-exchange-sha256"],
///     &["aes128-ctr", "aes256-ctr"],
///     &["hmac-sha2-256", "hmac-sha2-512"],
///     &["none", "zlib@openssh.com"],
/// );
/// assert!(!hassh.fingerprint.is_empty());
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HASSH {
    /// 密钥交换算法列表（分号分隔）
    pub kex_algorithms: String,
    
    /// 加密算法列表（分号分隔）
    pub encryption_algorithms: String,
    
    /// MAC 算法列表（分号分隔）
    pub mac_algorithms: String,
    
    /// 压缩算法列表（分号分隔）
    pub compression_algorithms: String,
    
    /// 完整的 HASSH 字符串（用于计算哈希）
    pub hassh_string: String,
    
    /// HASSH 指纹（MD5 哈希）
    pub fingerprint: String,
    
    /// SSH 客户端类型（推断）
    pub client_type: Option<String>,
}

impl HASSH {
    /// 生成 HASSH 指纹
    ///
    /// # 参数
    /// - `kex_algorithms`: 密钥交换算法列表
    /// - `encryption_algorithms`: 加密算法列表
    /// - `mac_algorithms`: MAC 算法列表
    /// - `compression_algorithms`: 压缩算法列表
    ///
    /// # 返回
    /// HASSH 指纹结构
    pub fn generate(
        kex_algorithms: &[&str],
        encryption_algorithms: &[&str],
        mac_algorithms: &[&str],
        compression_algorithms: &[&str],
    ) -> Self {
        // 连接算法列表（使用分号分隔）
        let kex_str = kex_algorithms.join(";");
        let enc_str = encryption_algorithms.join(";");
        let mac_str = mac_algorithms.join(";");
        let comp_str = compression_algorithms.join(";");

        // 构建 HASSH 字符串
        let hassh_string = format!("{};{};{};{}", kex_str, enc_str, mac_str, comp_str);

        // 计算 MD5 哈希
        let fingerprint = Self::md5_hash(&hassh_string);

        // 推断客户端类型
        let client_type = Self::infer_client_type(kex_algorithms, encryption_algorithms);

        Self {
            kex_algorithms: kex_str,
            encryption_algorithms: enc_str,
            mac_algorithms: mac_str,
            compression_algorithms: comp_str,
            hassh_string,
            fingerprint,
            client_type,
        }
    }

    /// 计算 MD5 哈希
    fn md5_hash(input: &str) -> String {
        let digest = md5::compute(input.as_bytes());
        format!("{:x}", digest)
    }

    /// 推断 SSH 客户端类型
    fn infer_client_type(
        kex_algorithms: &[&str],
        encryption_algorithms: &[&str],
    ) -> Option<String> {
        // 基于算法特征推断客户端类型
        
        // OpenSSH 特征
        if kex_algorithms.iter().any(|&k| k.contains("curve25519-sha256")) {
            if encryption_algorithms
                .iter()
                .any(|&e| e.contains("chacha20-poly1305"))
            {
                return Some("OpenSSH".to_string());
            }
        }

        // PuTTY 特征
        if kex_algorithms.iter().any(|&k| k.contains("ecdh-sha2-nistp256"))
            && encryption_algorithms
                .iter()
                .any(|&e| e.contains("aes256-ctr"))
        {
            return Some("PuTTY".to_string());
        }

        // Paramiko (Python) 特征
        if kex_algorithms
            .iter()
            .any(|&k| k.contains("diffie-hellman-group14-sha1"))
            && !kex_algorithms.iter().any(|&k| k.contains("curve25519"))
        {
            return Some("Paramiko".to_string());
        }

        // libssh 特征
        if kex_algorithms
            .iter()
            .any(|&k| k.contains("ecdh-sha2-nistp521"))
        {
            return Some("libssh".to_string());
        }

        None
    }

    /// 从 SSH KEX_INIT 消息解析 HASSH
    ///
    /// SSH KEX_INIT 消息包含算法协商信息
    pub fn from_kex_init(kex_init: &SSHKexInit) -> Self {
        Self::generate(
            &kex_init
                .kex_algorithms
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<_>>(),
            &kex_init
                .encryption_algorithms_client_to_server
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<_>>(),
            &kex_init
                .mac_algorithms_client_to_server
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<_>>(),
            &kex_init
                .compression_algorithms_client_to_server
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<_>>(),
        )
    }
}

impl std::fmt::Display for HASSH {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.fingerprint)
    }
}

/// HASSH Server - SSH 服务器指纹
///
/// 格式: MD5(Server KEX;EncryptionAlgs;MACAlgs;CompressionAlgs)
///
/// ## 示例
/// ```
/// use fingerprint_core::hassh::HASSHServer;
///
/// let hassh_server = HASSHServer::generate(
///     &["diffie-hellman-group14-sha256"],
///     &["aes256-ctr"],
///     &["hmac-sha2-512"],
///     &["none"],
/// );
/// assert!(!hassh_server.fingerprint.is_empty());
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HASSHServer {
    /// 密钥交换算法列表（分号分隔）
    pub kex_algorithms: String,
    
    /// 加密算法列表（分号分隔）
    pub encryption_algorithms: String,
    
    /// MAC 算法列表（分号分隔）
    pub mac_algorithms: String,
    
    /// 压缩算法列表（分号分隔）
    pub compression_algorithms: String,
    
    /// 完整的 HASSH Server 字符串
    pub hassh_server_string: String,
    
    /// HASSH Server 指纹（MD5 哈希）
    pub fingerprint: String,
    
    /// SSH 服务器类型（推断）
    pub server_type: Option<String>,
}

impl HASSHServer {
    /// 生成 HASSH Server 指纹
    pub fn generate(
        kex_algorithms: &[&str],
        encryption_algorithms: &[&str],
        mac_algorithms: &[&str],
        compression_algorithms: &[&str],
    ) -> Self {
        let kex_str = kex_algorithms.join(";");
        let enc_str = encryption_algorithms.join(";");
        let mac_str = mac_algorithms.join(";");
        let comp_str = compression_algorithms.join(";");

        let hassh_server_string = format!("{};{};{};{}", kex_str, enc_str, mac_str, comp_str);

        let fingerprint = Self::md5_hash(&hassh_server_string);

        let server_type = Self::infer_server_type(kex_algorithms, encryption_algorithms);

        Self {
            kex_algorithms: kex_str,
            encryption_algorithms: enc_str,
            mac_algorithms: mac_str,
            compression_algorithms: comp_str,
            hassh_server_string,
            fingerprint,
            server_type,
        }
    }

    /// 计算 MD5 哈希
    fn md5_hash(input: &str) -> String {
        let digest = md5::compute(input.as_bytes());
        format!("{:x}", digest)
    }

    /// 推断 SSH 服务器类型
    fn infer_server_type(
        kex_algorithms: &[&str],
        encryption_algorithms: &[&str],
    ) -> Option<String> {
        // OpenSSH Server
        if kex_algorithms.iter().any(|&k| k.contains("curve25519-sha256"))
            && encryption_algorithms
                .iter()
                .any(|&e| e.contains("chacha20-poly1305"))
        {
            return Some("OpenSSH".to_string());
        }

        // Dropbear
        if kex_algorithms
            .iter()
            .any(|&k| k.contains("diffie-hellman-group1-sha1"))
            && encryption_algorithms.len() < 5
        {
            return Some("Dropbear".to_string());
        }

        // libssh
        if kex_algorithms
            .iter()
            .any(|&k| k.contains("ecdh-sha2-nistp521"))
        {
            return Some("libssh".to_string());
        }

        None
    }

    /// 从 SSH KEX_INIT 消息解析 HASSH Server
    pub fn from_kex_init(kex_init: &SSHKexInit) -> Self {
        Self::generate(
            &kex_init
                .kex_algorithms
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<_>>(),
            &kex_init
                .encryption_algorithms_server_to_client
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<_>>(),
            &kex_init
                .mac_algorithms_server_to_client
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<_>>(),
            &kex_init
                .compression_algorithms_server_to_client
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<_>>(),
        )
    }
}

impl std::fmt::Display for HASSHServer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.fingerprint)
    }
}

/// SSH KEX_INIT 消息结构
///
/// 包含 SSH 密钥交换初始化消息的所有算法列表
#[derive(Debug, Clone, Default)]
pub struct SSHKexInit {
    /// 密钥交换算法
    pub kex_algorithms: Vec<String>,
    
    /// 服务器主机密钥算法
    pub server_host_key_algorithms: Vec<String>,
    
    /// 客户端到服务器加密算法
    pub encryption_algorithms_client_to_server: Vec<String>,
    
    /// 服务器到客户端加密算法
    pub encryption_algorithms_server_to_client: Vec<String>,
    
    /// 客户端到服务器 MAC 算法
    pub mac_algorithms_client_to_server: Vec<String>,
    
    /// 服务器到客户端 MAC 算法
    pub mac_algorithms_server_to_client: Vec<String>,
    
    /// 客户端到服务器压缩算法
    pub compression_algorithms_client_to_server: Vec<String>,
    
    /// 服务器到客户端压缩算法
    pub compression_algorithms_server_to_client: Vec<String>,
}

impl SSHKexInit {
    /// 创建新的 KEX_INIT 消息
    pub fn new() -> Self {
        Self::default()
    }

    /// 从原始 SSH 数据包解析（简化版本）
    ///
    /// 注意：这是一个简化实现，完整的 SSH 协议解析需要更复杂的状态机
    pub fn parse(data: &[u8]) -> Result<Self, String> {
        // SSH 协议格式复杂，这里提供基本框架
        // 实际应用中应使用专门的 SSH 协议解析库
        
        if data.len() < 16 {
            return Err("数据包太短".to_string());
        }

        // SSH KEX_INIT 消息类型为 20 (SSH_MSG_KEXINIT)
        if data[0] != 20 {
            return Err("不是 KEX_INIT 消息".to_string());
        }

        // 这里应该解析 name-list 字段
        // 由于 SSH 协议解析复杂，暂时返回空结构
        Ok(Self::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hassh_generation() {
        let hassh = HASSH::generate(
            &[
                "diffie-hellman-group14-sha1",
                "diffie-hellman-group-exchange-sha256",
            ],
            &["aes128-ctr", "aes192-ctr", "aes256-ctr"],
            &["hmac-sha2-256", "hmac-sha2-512"],
            &["none", "zlib@openssh.com"],
        );

        assert!(!hassh.fingerprint.is_empty());
        assert_eq!(hassh.fingerprint.len(), 32); // MD5 哈希长度
        assert!(hassh.kex_algorithms.contains("diffie-hellman"));
        assert!(hassh.encryption_algorithms.contains("aes128-ctr"));
    }

    #[test]
    fn test_hassh_openssh_detection() {
        let hassh = HASSH::generate(
            &["curve25519-sha256", "ecdh-sha2-nistp256"],
            &["chacha20-poly1305@openssh.com", "aes256-ctr"],
            &["hmac-sha2-256"],
            &["none"],
        );

        assert_eq!(hassh.client_type, Some("OpenSSH".to_string()));
    }

    #[test]
    fn test_hassh_putty_detection() {
        let hassh = HASSH::generate(
            &["ecdh-sha2-nistp256", "diffie-hellman-group14-sha1"],
            &["aes256-ctr", "aes192-ctr"],
            &["hmac-sha2-256"],
            &["none"],
        );

        assert_eq!(hassh.client_type, Some("PuTTY".to_string()));
    }

    #[test]
    fn test_hassh_display() {
        let hassh = HASSH::generate(&["test"], &["test"], &["test"], &["none"]);
        let displayed = format!("{}", hassh);
        assert_eq!(displayed, hassh.fingerprint);
    }

    #[test]
    fn test_hassh_server_generation() {
        let hassh_server = HASSHServer::generate(
            &["diffie-hellman-group14-sha256"],
            &["aes256-ctr", "aes128-ctr"],
            &["hmac-sha2-512", "hmac-sha2-256"],
            &["none"],
        );

        assert!(!hassh_server.fingerprint.is_empty());
        assert_eq!(hassh_server.fingerprint.len(), 32);
    }

    #[test]
    fn test_hassh_server_openssh_detection() {
        let hassh_server = HASSHServer::generate(
            &["curve25519-sha256@libssh.org"],
            &["chacha20-poly1305@openssh.com"],
            &["hmac-sha2-256"],
            &["none"],
        );

        assert_eq!(hassh_server.server_type, Some("OpenSSH".to_string()));
    }

    #[test]
    fn test_hassh_empty_algorithms() {
        let hassh = HASSH::generate(&[], &[], &[], &[]);
        assert!(!hassh.fingerprint.is_empty());
        assert_eq!(hassh.hassh_string, ";;;");
    }

    #[test]
    fn test_ssh_kex_init_creation() {
        let kex_init = SSHKexInit::new();
        assert!(kex_init.kex_algorithms.is_empty());
    }

    #[test]
    fn test_ssh_kex_init_parse_invalid() {
        let data = vec![0u8; 10];
        let result = SSHKexInit::parse(&data);
        assert!(result.is_err());
    }

    #[test]
    fn test_hassh_string_format() {
        let hassh = HASSH::generate(&["kex1", "kex2"], &["enc1"], &["mac1"], &["comp1"]);
        assert_eq!(hassh.hassh_string, "kex1;kex2;enc1;mac1;comp1");
    }
}
