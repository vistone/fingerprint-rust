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

use crate::dicttls::supported_groups::{CURVE_P256, CURVE_P384, X25519};
use crate::tls_config::ClientHelloSpec;
use crate::tls_extensions::{KeyShareExtension, TLSExtension, UtlsPaddingExtension};
use ring::agreement;
use ring::rand::{self as ring_rand, SecureRandom};

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
    pub fn from_spec(spec: &ClientHelloSpec, server_name: &str) -> Self {
        // 使用 TLS 1.2 作为客户端版本（为了兼容性）
        let client_version = spec.tls_vers_max.max(0x0303);
        let rng = ring_rand::SystemRandom::new();

        // 生成随机数 (32 bytes)
        let mut random = vec![0u8; 32];
        rng.fill(&mut random).unwrap();

        // TLS 1.3 兼容模式：必须发送非空的 Session ID (32 bytes)
        let mut session_id = vec![0u8; 32];
        rng.fill(&mut session_id).unwrap();

        // 密码套件
        let cipher_suites = spec.cipher_suites.clone();

        // 压缩方法
        let compression_methods = if spec.compression_methods.is_empty() {
            vec![0] // 无压缩
        } else {
            spec.compression_methods.clone()
        };

        // 序列化扩展
        // 计算 Base Length (不包含 Extension Length 字段本身 2 bytes)
        let base_len = 2
            + 32
            + 1
            + session_id.len()
            + 2
            + cipher_suites.len() * 2
            + 1
            + compression_methods.len();

        let extensions = Self::serialize_extensions(&spec.extensions, server_name, base_len);

        Self {
            client_version,
            random,
            session_id,
            cipher_suites,
            compression_methods,
            extensions,
        }
    }

    /// 序列化扩展
    fn serialize_extensions(
        extensions: &[Box<dyn TLSExtension>],
        server_name: &str,
        base_len: usize,
    ) -> Vec<u8> {
        let mut ext_bytes = Vec::new();
        let mut has_sni = false;
        let rng = ring_rand::SystemRandom::new();

        // 检查是否存在 PreSharedKey 扩展 (ID 41)
        let has_psk = extensions.iter().any(|e| e.extension_id() == 41);

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

            // 如果是 PSK Key Exchange Modes (ID 45)，但没有 PreSharedKey，则必须跳过
            if ext_id == 45 && !has_psk {
                continue;
            }

            // 过滤 ECH (ID 0xfe0d = 65037)
            if ext_id == 65037 {
                continue;
            }

            // 特殊处理 Padding 扩展 (ID 21)
            if ext_id == 21 {
                // 计算当前总长度 (Handshake Header 4 bytes + ClientHello Base + Extensions Length 2 bytes + Current Extensions)
                // 注意：Record Header 5 bytes 不包含在握手消息长度中，但 Padding 通常是为了对齐 Record Payload 或 Handshake Payload
                // BoringPaddingStyle 似乎是针对 ClientHello 消息总长度（不含 Record Header）
                // 或者是包含 Record Header？
                // Go uTLS: prefixLen = 4 (Handshake header) + len(hello.marshal()) - len(paddingExt)
                // 这里我们计算的是 Handshake Payload 的一部分。
                // 假设我们希望 Handshake Message 总长度对齐。

                let current_len = 4 + base_len + 2 + ext_bytes.len() + 4; // +4 for Padding Header (ID+Len)
                let (padding_len, will_pad) =
                    UtlsPaddingExtension::boring_padding_style(current_len);

                if will_pad {
                    ext_bytes.extend_from_slice(&ext_id.to_be_bytes());
                    ext_bytes.extend_from_slice(&(padding_len as u16).to_be_bytes());
                    ext_bytes.extend(std::iter::repeat(0).take(padding_len));
                }
                continue;
            }

            // 特殊处理 KeyShare 扩展 (ID == 51, 0x0033)
            // 需要为真实的曲线生成公钥
            if ext_id == 51 {
                if let Some(ks_ext) = ext.as_any().downcast_ref::<KeyShareExtension>() {
                    // 生成带有真实公钥的 KeyShare 扩展
                    let real_ks_ext = Self::generate_real_keyshare_extension(ks_ext, &rng);

                    // 序列化
                    let ext_len = real_ks_ext.len();
                    let mut ext_data = vec![0u8; ext_len];
                    if real_ks_ext.read(&mut ext_data).is_ok() {
                        ext_bytes.extend_from_slice(&ext_data);
                    }
                    continue;
                }
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

    /// 生成真实的 KeyShare 扩展（填充公钥）
    fn generate_real_keyshare_extension(
        original: &KeyShareExtension,
        rng: &ring_rand::SystemRandom,
    ) -> KeyShareExtension {
        let mut new_shares = Vec::new();

        for share in &original.key_shares {
            let mut new_share = share.clone();

            // 如果不是 GREASE 且数据为空，则生成密钥
            // GREASE check: (val & 0x0f0f) == 0x0a0a
            let is_grease = (share.group & 0x0f0f) == 0x0a0a;

            if !is_grease && share.data.is_empty() {
                // 生成密钥
                if share.group == X25519 {
                    if let Ok(my_private_key) =
                        agreement::EphemeralPrivateKey::generate(&agreement::X25519, rng)
                    {
                        if let Ok(my_public_key) = my_private_key.compute_public_key() {
                            new_share.data = my_public_key.as_ref().to_vec();
                        }
                    }
                } else if share.group == CURVE_P256 {
                    if let Ok(my_private_key) =
                        agreement::EphemeralPrivateKey::generate(&agreement::ECDH_P256, rng)
                    {
                        if let Ok(my_public_key) = my_private_key.compute_public_key() {
                            new_share.data = my_public_key.as_ref().to_vec();
                        }
                    }
                } else if share.group == CURVE_P384 {
                    if let Ok(my_private_key) =
                        agreement::EphemeralPrivateKey::generate(&agreement::ECDH_P384, rng)
                    {
                        if let Ok(my_public_key) = my_private_key.compute_public_key() {
                            new_share.data = my_public_key.as_ref().to_vec();
                        }
                    }
                }
            }

            new_shares.push(new_share);
        }

        KeyShareExtension::new(new_shares)
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

        let msg = ClientHelloMessage::from_spec(&spec, "example.com");

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
