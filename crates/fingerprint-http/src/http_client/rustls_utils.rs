//! rustls 配置工具（供 HTTP/1/2/3 复用）
//!
//! 目标：
//! - 单一入口构建 root store
//! - 单一入口应用 verify_tls（可选禁用校验，仅用于调试/内网）
//! - 单一入口配置 ALPN

#![cfg(any(feature = "rustls-tls", feature = "http2", feature = "http3"))]

#[cfg(feature = "dangerous_configuration")]
use std::sync::Arc;

use fingerprint_profiles::profiles::ClientProfile;

#[cfg(feature = "rustls-client-hello-customizer")]
use super::rustls_client_hello_customizer::ProfileClientHelloCustomizer;

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

/// 若 verify_tls=false，则安装"接受所有证书"的 verifier（危险功能，仅用于调试）
#[allow(unused_variables)]
pub fn apply_verify_tls(cfg: &mut rustls::ClientConfig, verify_tls: bool) {
    if verify_tls {
        return;
    }

    // 注意：rustls 0.21 的 API 可能不同
    // 如果 verify_tls=false，使用 dangerous 配置接受所有证书
    // 这需要 rustls 的 dangerous_configuration feature
    #[cfg(feature = "dangerous_configuration")]
    {
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

    #[cfg(not(feature = "dangerous_configuration"))]
    {
        // 如果没有 dangerous_configuration feature，忽略 verify_tls=false 的设置
        // 始终验证证书（更安全）
        eprintln!("警告: verify_tls=false 需要 dangerous_configuration feature，已忽略");
    }
}

/// 构建 rustls::ClientConfig，并设置 ALPN/verify_tls。
pub fn build_client_config(
    verify_tls: bool,
    alpn_protocols: Vec<Vec<u8>>,
    profile: Option<&ClientProfile>,
) -> rustls::ClientConfig {
    // 当未启用 rustls-client-hello-customizer 时，profile 只是一个预留参数（避免到处改签名）。
    #[cfg(not(feature = "rustls-client-hello-customizer"))]
    let _ = profile;

    let root_store = build_root_store();
    let mut cfg = rustls::ClientConfig::builder()
        .with_safe_defaults()
        .with_root_certificates(root_store)
        .with_no_client_auth();

    cfg.alpn_protocols = alpn_protocols;
    apply_verify_tls(&mut cfg, verify_tls);

    // 可选：在发送 ClientHello 之前按指纹 spec 重排扩展编码顺序（需要配套 rustls fork）。
    #[cfg(feature = "rustls-client-hello-customizer")]
    if let Some(profile) = profile {
        if let Some(customizer) = ProfileClientHelloCustomizer::try_from_profile(profile) {
            cfg = cfg.with_client_hello_customizer(customizer.into_arc());
        }
    }
    cfg
}
