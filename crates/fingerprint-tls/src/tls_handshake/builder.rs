//! TLS 握手构建器
//!
//! 根据 ClientHelloSpec 构建完整的 TLS ClientHello 握手

use super::{ClientHelloMessage, TLSHandshake, TLSRecord};
use crate::tls_config::ClientHelloSpec;

/// TLS 握手构建器
pub struct TLSHandshakeBuilder;

impl TLSHandshakeBuilder {
    /// 根据 ClientHelloSpec 构建 TLS ClientHello 记录
    ///
    /// 返回完整的 TLS 记录字节流，可以直接发送到服务器
    pub fn build_client_hello(
        spec: &ClientHelloSpec,
        server_name: &str,
    ) -> Result<Vec<u8>, String> {
        // 1. 创建 ClientHello 消息
        let client_hello = ClientHelloMessage::from_spec(spec, server_name)?;

        // 2. 序列化 ClientHello 消息体
        let body = client_hello.to_bytes();

        // 3. 创建握手消息
        let handshake = TLSHandshake::client_hello(body);

        // 4. 序列化握手消息
        let handshake_bytes = handshake.to_bytes();

        // 5. 创建 TLS 记录
        // 使用 TLS 1.0 (0x0301) 作为记录版本（为了兼容性）
        let record = TLSRecord::handshake(0x0301, handshake_bytes);

        // 6. 序列化 TLS 记录
        Ok(record.to_bytes())
    }

    /// 构建并打印调试信息
    pub fn build_with_debug(spec: &ClientHelloSpec, server_name: &str) -> Result<Vec<u8>, String> {
        // 1. 创建 ClientHello 消息
        let client_hello = ClientHelloMessage::from_spec(spec, server_name)?;
        println!("\n╔══════════════════════════════════════════════════════════╗");
        println!("║          构建 TLS ClientHello（使用自己的指纹）          ║");
        println!("╚══════════════════════════════════════════════════════════╝\n");

        println!("📋 ClientHelloSpec 信息:");
        println!("  - 密码套件数: {}", spec.cipher_suites.len());
        println!("  - 扩展数: {}", spec.extensions.len());
        println!(
            "  - TLS 版本范围: 0x{:04x} - 0x{:04x}",
            spec.tls_vers_min, spec.tls_vers_max
        );
        println!("  - 压缩方法: {:?}", spec.compression_methods);

        // 打印 ClientHello 调试信息
        println!("\n{}", client_hello.debug_info());

        let body = client_hello.to_bytes();
        println!("\n📦 ClientHello 消息体: {} bytes", body.len());

        let handshake = TLSHandshake::client_hello(body);
        let handshake_bytes = handshake.to_bytes();
        println!("📦 握手消息: {} bytes", handshake_bytes.len());

        let record = TLSRecord::handshake(0x0301, handshake_bytes);
        let record_bytes = record.to_bytes();
        println!("📦 TLS 记录: {} bytes", record_bytes.len());

        println!("\n✅ TLS ClientHello 构建完成！");
        println!(
            "   使用我们自己的指纹: {} 密码套件, {} 扩展\n",
            spec.cipher_suites.len(),
            spec.extensions.len()
        );

        Ok(record_bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_client_hello() {
        // 创建一个简单的 ClientHelloSpec
        let spec = ClientHelloSpec {
            cipher_suites: vec![
                0xc02f, // TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256
                0xc030, // TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384
                0x1301, // TLS_AES_128_GCM_SHA256
            ],
            compression_methods: vec![0],
            extensions: vec![],
            tls_vers_min: 0x0303,
            tls_vers_max: 0x0304,
            metadata: None,
        };

        let result = TLSHandshakeBuilder::build_client_hello(&spec, "example.com");
        assert!(result.is_ok());

        let bytes = result.unwrap();
        println!("Generated ClientHello: {} bytes", bytes.len());

        // 验证 TLS 记录格式
        assert_eq!(bytes[0], 22); // Handshake
        assert_eq!(bytes[1], 0x03); // Version major
        assert_eq!(bytes[2], 0x01); // Version minor (TLS 1.0)

        // 验证长度字段
        let record_length = u16::from_be_bytes([bytes[3], bytes[4]]) as usize;
        assert_eq!(bytes.len(), 5 + record_length);
    }

    #[test]
    fn test_build_with_real_spec() {
        // 使用真实的 Chrome 133 指纹
        let spec = ClientHelloSpec::chrome_133();

        let result = TLSHandshakeBuilder::build_with_debug(&spec, "www.google.com");
        assert!(result.is_ok());

        let bytes = result.unwrap();
        println!("\n生成的 Chrome 133 ClientHello: {} bytes", bytes.len());

        // Chrome 133 应该有较多的密码套件和扩展
        assert!(bytes.len() > 200); // Chrome 的 ClientHello 通常很大
    }
}
