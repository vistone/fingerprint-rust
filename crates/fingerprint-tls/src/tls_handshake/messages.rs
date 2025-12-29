//! TLS ClientHello 消息构建
//!
//! 根据 ClientHelloSpec 生成真实的 TLS ClientHello 消息
//!
//! ClientHello 格式 (RFC 5246):
//! ```text
//! struct {
//!     ProtocolVersion client_version;
//!     Random random;
//!     SessionID session_id;
//!     CipherSuite cipher_suites<2..2^16-2>;
//!     CompressionMethod compression_methods<1..2^8-1>;
//!     Extension extensions<0..2^16-1>;
//! } ClientHello;
//! ```

use crate::tls_config::ClientHelloSpec;
use crate::tls_extensions::TLSExtension;

/// ClientHello 消息
#[derive(Debug, Clone)]
pub struct ClientHelloMessage {
    /// 客户端版本
    pub client_version: u16,
    /// 随机数 (32 bytes)
    pub random: Vec<u8>,
    /// 会话 ID
    pub session_id: Vec<u8>,
    /// 密码套件列表
    pub cipher_suites: Vec<u16>,
    /// 压缩方法
    pub compression_methods: Vec<u8>,
    /// 扩展列表
    pub extensions: Vec<u8>,
}

impl ClientHelloMessage {
    /// 从 ClientHelloSpec 创建 ClientHello 消息
    ///
    /// # 错误
    ///
    /// 如果无法获取加密安全的随机数（在没有 `crypto` feature 时），将返回错误。
    /// 建议在生产环境中启用 `crypto` feature 以确保安全性。
    pub fn from_spec(spec: &ClientHelloSpec, server_name: &str) -> Result<Self, String> {
        // 使用 TLS 1.2 作为客户端版本（为了兼容性）
        let client_version = spec.tls_vers_max.max(0x0303);

        // 生成随机数 (32 bytes)
        let mut random = Vec::with_capacity(32);

        // 前 4 bytes: Unix 时间戳
        // 使用当前时间，如果获取失败则使用 0（虽然不太可能失败）
        // 修复 2038 年溢出问题：明确截断高位，确保 u32 范围内
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| (d.as_secs() & 0xFFFFFFFF) as u32) // 明确截断高位，防止 2038 年溢出
            .unwrap_or(0);
        random.extend_from_slice(&timestamp.to_be_bytes());

        // 后 28 bytes: 随机数
        #[cfg(feature = "crypto")]
        {
            use rand::Rng;
            let mut rng = rand::thread_rng();
            for _ in 0..28 {
                random.push(rng.gen());
            }
        }
        #[cfg(not(feature = "crypto"))]
        {
            // 如果没有 crypto feature，尝试从系统随机数源获取加密安全的随机数
            // 如果无法获取，直接返回错误，不允许使用不安全的降级方案
            use std::io::Read;
            let mut random_bytes = [0u8; 28];

            // 尝试从 /dev/urandom (Unix) 获取随机数
            let mut rng = std::fs::File::open("/dev/urandom")
                .map_err(|e| format!(
                    "无法访问系统随机数源 /dev/urandom: {}. 建议启用 'crypto' feature 以使用加密安全的随机数生成器",
                    e
                ))?;

            rng.read_exact(&mut random_bytes)
                .map_err(|e| format!(
                    "无法从 /dev/urandom 读取随机数: {}. 建议启用 'crypto' feature 以使用加密安全的随机数生成器",
                    e
                ))?;

            random.extend_from_slice(&random_bytes);
        }

        // 空的会话 ID（新会话）
        let session_id = Vec::new();

        // 密码套件
        let cipher_suites = spec.cipher_suites.clone();

        // 压缩方法
        let compression_methods = if spec.compression_methods.is_empty() {
            vec![0] // 无压缩
        } else {
            spec.compression_methods.clone()
        };

        // 序列化扩展
        let extensions = Self::serialize_extensions(&spec.extensions, server_name);

        Ok(Self {
            client_version,
            random,
            session_id,
            cipher_suites,
            compression_methods,
            extensions,
        })
    }

    /// 序列化扩展
    fn serialize_extensions(extensions: &[Box<dyn TLSExtension>], server_name: &str) -> Vec<u8> {
        let mut ext_bytes = Vec::new();
        let mut has_sni = false;

        for ext in extensions {
            let ext_id = ext.extension_id();

            // 如果是 SNI 扩展（ID == 0），我们需要特殊处理
            if ext_id == 0 {
                // 跳过重复的 SNI 扩展
                if has_sni {
                    continue;
                }
                has_sni = true;

                // 动态构建 SNI 扩展数据
                let sni_data = Self::build_sni_extension(server_name);

                // 扩展格式: ID (2 bytes) + Length (2 bytes) + Data
                ext_bytes.extend_from_slice(&ext_id.to_be_bytes());
                ext_bytes.extend_from_slice(&(sni_data.len() as u16).to_be_bytes());
                ext_bytes.extend_from_slice(&sni_data);
                continue;
            }

            // 其他扩展：正常序列化
            let ext_len = ext.len();
            if ext_len == 0 {
                // 空扩展也需要写入 ID 和长度
                ext_bytes.extend_from_slice(&ext_id.to_be_bytes());
                ext_bytes.extend_from_slice(&0u16.to_be_bytes());
                continue;
            }

            // 读取扩展数据（包含 ID 和长度）
            let mut ext_data = vec![0u8; ext_len];
            if ext.read(&mut ext_data).is_ok() {
                ext_bytes.extend_from_slice(&ext_data);
            }
        }

        // 如果没有 SNI 扩展，添加一个
        if !has_sni && !server_name.is_empty() {
            let sni_data = Self::build_sni_extension(server_name);
            ext_bytes.extend_from_slice(&0u16.to_be_bytes()); // SNI extension ID
            ext_bytes.extend_from_slice(&(sni_data.len() as u16).to_be_bytes());
            ext_bytes.extend_from_slice(&sni_data);
        }

        ext_bytes
    }

    /// 构建 SNI 扩展数据（不包含扩展 ID 和长度字段）
    fn build_sni_extension(server_name: &str) -> Vec<u8> {
        let mut data = Vec::new();

        // Server Name List Length (2 bytes)
        let list_len = 3 + server_name.len();
        data.extend_from_slice(&(list_len as u16).to_be_bytes());

        // Server Name Type (1 byte): 0 = host_name
        data.push(0);

        // Server Name Length (2 bytes)
        data.extend_from_slice(&(server_name.len() as u16).to_be_bytes());

        // Server Name
        data.extend_from_slice(server_name.as_bytes());

        data
    }

    /// 序列化为字节流
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        // Client Version (2 bytes)
        bytes.extend_from_slice(&self.client_version.to_be_bytes());

        // Random (32 bytes)
        bytes.extend_from_slice(&self.random);

        // Session ID Length (1 byte) + Session ID
        bytes.push(self.session_id.len() as u8);
        bytes.extend_from_slice(&self.session_id);

        // Cipher Suites Length (2 bytes) + Cipher Suites
        let cs_len = (self.cipher_suites.len() * 2) as u16;
        bytes.extend_from_slice(&cs_len.to_be_bytes());
        for cs in &self.cipher_suites {
            bytes.extend_from_slice(&cs.to_be_bytes());
        }

        // Compression Methods Length (1 byte) + Compression Methods
        bytes.push(self.compression_methods.len() as u8);
        bytes.extend_from_slice(&self.compression_methods);

        // Extensions Length (2 bytes) + Extensions
        bytes.extend_from_slice(&(self.extensions.len() as u16).to_be_bytes());
        bytes.extend_from_slice(&self.extensions);

        bytes
    }

    /// 打印调试信息
    pub fn debug_info(&self) -> String {
        format!(
            "ClientHello:\n\
             - Version: 0x{:04x}\n\
             - Random: {} bytes\n\
             - Session ID: {} bytes\n\
             - Cipher Suites: {} suites\n\
             - Compression: {} methods\n\
             - Extensions: {} bytes",
            self.client_version,
            self.random.len(),
            self.session_id.len(),
            self.cipher_suites.len(),
            self.compression_methods.len(),
            self.extensions.len()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clienthello_basic() {
        // 创建一个简单的 ClientHelloSpec
        let spec = ClientHelloSpec {
            cipher_suites: vec![0xc02f, 0xc030], // 两个密码套件
            compression_methods: vec![0],
            extensions: vec![],
            tls_vers_min: 0x0303,
            tls_vers_max: 0x0303,
            metadata: None,
        };

        let msg = ClientHelloMessage::from_spec(&spec, "example.com").unwrap();

        // 验证基本字段
        assert_eq!(msg.client_version, 0x0303);
        assert_eq!(msg.random.len(), 32);
        assert_eq!(msg.cipher_suites.len(), 2);
        assert_eq!(msg.compression_methods, vec![0]);

        // 序列化
        let bytes = msg.to_bytes();
        println!("ClientHello size: {} bytes", bytes.len());
        println!("{}", msg.debug_info());

        // 验证格式
        assert!(bytes.len() >= 41); // 最小长度
    }

    #[test]
    fn test_sni_extension() {
        let data = ClientHelloMessage::build_sni_extension("example.com");

        // SNI 格式验证
        assert!(data.len() > 5);

        // Server Name List Length
        let list_len = u16::from_be_bytes([data[0], data[1]]) as usize;
        assert_eq!(list_len, data.len() - 2);

        // Server Name Type
        assert_eq!(data[2], 0); // host_name

        // Server Name Length
        let name_len = u16::from_be_bytes([data[3], data[4]]) as usize;
        assert_eq!(name_len, 11); // "example.com".len()
    }
}
