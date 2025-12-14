//! rustls 配置工具（供 HTTP/1/2/3 复用）
//!
//! 目标：
//! - 单一入口构建 root store
//! - 单一入口应用 verify_tls（可选禁用校验，仅用于调试/内网）
//! - 单一入口配置 ALPN

#![cfg(any(feature = "rustls-tls", feature = "http2", feature = "http3"))]

use std::sync::Arc;

/// 构建 rustls 根证书存储（Mozilla roots）
pub fn build_root_store() -> rustls::RootCertStore {
    let mut root_store = rustls::RootCertStore::empty();
    root_store.add_trust_anchors(webpki_roots::TLS_SERVER_ROOTS.iter().map(|ta| {
        rustls::OwnedTrustAnchor::from_subject_spki_name_constraints(
            ta.subject,
            ta.spki,
            ta.name_constraints,
        )
    }));
    root_store
}

/// 若 verify_tls=false，则安装“接受所有证书”的 verifier（危险功能，仅用于调试）
pub fn apply_verify_tls(cfg: &mut rustls::ClientConfig, verify_tls: bool) {
    if verify_tls {
        return;
    }

    use rustls::client::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier};
    use rustls::{Certificate, Error as RustlsError, ServerName};
    use std::time::SystemTime;

    #[derive(Debug)]
    struct NoCertificateVerification;

    impl ServerCertVerifier for NoCertificateVerification {
        fn verify_server_cert(
            &self,
            _end_entity: &Certificate,
            _intermediates: &[Certificate],
            _server_name: &ServerName,
            _scts: &mut dyn Iterator<Item = &[u8]>,
            _ocsp_response: &[u8],
            _now: SystemTime,
        ) -> std::result::Result<ServerCertVerified, RustlsError> {
            Ok(ServerCertVerified::assertion())
        }

        fn verify_tls12_signature(
            &self,
            _message: &[u8],
            _cert: &Certificate,
            _dss: &rustls::DigitallySignedStruct,
        ) -> std::result::Result<HandshakeSignatureValid, RustlsError> {
            Ok(HandshakeSignatureValid::assertion())
        }

        fn verify_tls13_signature(
            &self,
            _message: &[u8],
            _cert: &Certificate,
            _dss: &rustls::DigitallySignedStruct,
        ) -> std::result::Result<HandshakeSignatureValid, RustlsError> {
            Ok(HandshakeSignatureValid::assertion())
        }
    }

    cfg.dangerous()
        .set_certificate_verifier(Arc::new(NoCertificateVerification));
}

/// 构建 rustls::ClientConfig，并设置 ALPN/verify_tls。
pub fn build_client_config(verify_tls: bool, alpn_protocols: Vec<Vec<u8>>) -> rustls::ClientConfig {
    let root_store = build_root_store();
    let mut cfg = rustls::ClientConfig::builder()
        .with_safe_defaults()
        .with_root_certificates(root_store)
        .with_no_client_auth();

    cfg.alpn_protocols = alpn_protocols;
    apply_verify_tls(&mut cfg, verify_tls);
    cfg
}
