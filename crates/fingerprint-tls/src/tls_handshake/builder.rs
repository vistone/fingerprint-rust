//! TLS handshakeBuilder
//!
//! Based on ClientHelloSpec Buildcomplete TLS ClientHello handshake

use super::{ClientHelloMessage, TLSHandshake, TLSRecord};
use crate::tls_config::ClientHelloSpec;

/// TLS handshakeBuilder
pub struct TLSHandshakeBuilder;

impl TLSHandshakeBuilder {
    /// Based on ClientHelloSpec Build TLS ClientHello record
    ///
    /// returncomplete TLS recordbytesstream, candirectlysend to server
    pub fn build_client_hello(
        spec: &ClientHelloSpec,
        server_name: &str,
    ) -> Result<Vec<u8>, String> {
        // 1. Create ClientHello message
        let client_hello = ClientHelloMessage::from_spec(spec, server_name)?;

        // 2. serialize ClientHello message体
        let body = client_hello.to_bytes();

        // 3. Createhandshakemessage
        let handshake = TLSHandshake::client_hello(body);

        // 4. serializehandshakemessage
        let handshake_bytes = handshake.to_bytes();

        // 5. Create TLS record
        // use TLS 1.0 (0x0301) asrecordversion (in order tocompatibleproperty)
        let record = TLSRecord::handshake(0x0301, handshake_bytes);

        // 6. serialize TLS record
        Ok(record.to_bytes())
    }

    /// 构建并打印调试信息
    pub fn build_with_debug(spec: &ClientHelloSpec, server_name: &str) -> Result<Vec<u8>, String> {
        // 1. 创建 ClientHello 消息
        let client_hello = ClientHelloMessage::from_spec(spec, server_name)?;
        println!("\n╔══════════════════════════════════════════════════════════╗");
        println!("║ 构建 TLS ClientHello (使用自定义指纹) ║");
        println!("╚══════════════════════════════════════════════════════════╝\n");

        println!("📋 ClientHelloSpec info:");
        println!(" - cipher suitecount: {}", spec.cipher_suites.len());
        println!(" - extensioncount: {}", spec.extensions.len());
        println!(
            " - TLS versionrange: 0x{:04x} - 0x{:04x}",
            spec.tls_vers_min, spec.tls_vers_max
        );
        println!(" - compressionmethod: {:?}", spec.compression_methods);

        // print ClientHello debuginfo
        println!("\n{}", client_hello.debug_info());

        let body = client_hello.to_bytes();
        println!("\n📦 ClientHello message体: {} bytes", body.len());

        let handshake = TLSHandshake::client_hello(body);
        let handshake_bytes = handshake.to_bytes();
        println!("📦 handshakemessage: {} bytes", handshake_bytes.len());

        let record = TLSRecord::handshake(0x0301, handshake_bytes);
        let record_bytes = record.to_bytes();
        println!("📦 TLS record: {} bytes", record_bytes.len());

        println!("\n✅ TLS ClientHello Buildcomplete！");
        println!(
            " useweselffingerprint: {} cipher suite, {} extension\n",
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
        // Createansimple ClientHelloSpec
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

        // Validate TLS recordformat
        assert_eq!(bytes[0], 22); // Handshake
        assert_eq!(bytes[1], 0x03); // Version major
        assert_eq!(bytes[2], 0x01); // Version minor (TLS 1.0)

        // Validatelengthfield
        let record_length = u16::from_be_bytes([bytes[3], bytes[4]]) as usize;
        assert_eq!(bytes.len(), 5 + record_length);
    }

    #[test]
    fn test_build_with_real_spec() {
        // usereal Chrome 133 fingerprint
        let spec = ClientHelloSpec::chrome_133();

        let result = TLSHandshakeBuilder::build_with_debug(&spec, "www.google.com");
        assert!(result.is_ok());

        let bytes = result.unwrap();
        println!("\nGenerate Chrome 133 ClientHello: {} bytes", bytes.len());

        // Chrome 133 should有comparemultiple's cipher suites and extension
        assert!(bytes.len() > 200); // Chrome ClientHello usually很大
    }
}
