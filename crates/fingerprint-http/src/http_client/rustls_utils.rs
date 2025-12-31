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

// 注意：ProfileClientHelloCustomizer 需要支持 ClientHelloCustomizer 的 rustls fork
// 当前被禁用，因为标准 rustls 不包含 ClientHelloCustomizer API
#[cfg(false)] // 暂时禁用，因为标准 rustls 不支持
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

/// 构建 rustls::ClientConfig，并设置 ALPN/verify_tls，以及根据指纹匹配密码套件。
pub fn build_client_config(
    verify_tls: bool,
    alpn_protocols: Vec<Vec<u8>>,
    #[allow(unused_variables)] profile: Option<&ClientProfile>,
) -> rustls::ClientConfig {
    let root_store = build_root_store();

    // 默认配置（如果无法根据 profile 匹配，则回退到安全默认值）
    let builder = rustls::ClientConfig::builder()
        .with_safe_defaults()
        .with_root_certificates(root_store)
        .with_no_client_auth();

    let mut cfg = builder;

    // 强化指纹：匹配特定的密码套件和 TLS 版本
    // FIXME: s.suite() as u16 fail on rustls 0.21. Restore this when fixed.
    /*
    if let Some(profile) = profile {
        if let Ok(spec) = profile.get_client_hello_spec() {
            // 1. 匹配密码套件
            let mut suites = Vec::new();
            for &suite_id in &spec.cipher_suites {
                if let Some(suite) = rustls::ALL_CIPHER_SUITES
                    .iter()
                    .find(|s| s.suite() as u16 == suite_id) {
                    suites.push(*suite);
                }
            }

            if !suites.is_empty() {
                // 重新构建配置以应用特定套件
                let mut versions = Vec::new();
                if spec.tls_vers_max >= 0x0304 { // TLS 1.3
                    versions.push(&rustls::version::TLS13);
                }
                if spec.tls_vers_min <= 0x0303 { // TLS 1.2
                    versions.push(&rustls::version::TLS12);
                }

                // 注意：rustls 0.21 需要通过 builder 重新设置
                let new_builder = if !versions.is_empty() {
                    rustls::ClientConfig::builder()
                        .with_cipher_suites(&suites)
                        .with_safe_default_kx_groups()
                        .with_protocol_versions(&versions)
                        .unwrap_or_else(|_| rustls::ClientConfig::builder().with_safe_defaults())
                } else {
                    rustls::ClientConfig::builder()
                        .with_cipher_suites(&suites)
                        .with_safe_default_kx_groups()
                        .with_safe_default_protocol_versions()
                        .unwrap()
                };

                cfg = new_builder
                    .with_root_certificates(build_root_store())
                    .with_no_client_auth();
            }
        }
    }
    */

    cfg.alpn_protocols = alpn_protocols;
    apply_verify_tls(&mut cfg, verify_tls);

    // 可选：在发送 ClientHello 之前按指纹 spec 重排扩展编码顺序（需要配套 rustls fork）。
    // 注意：此功能需要支持 ClientHelloCustomizer 的 rustls fork，标准 rustls 不支持。
    // 当前被禁用，因为标准 rustls 不包含 ClientHelloCustomizer API。
    // 如需使用此功能，需要使用支持 ClientHelloCustomizer 的 rustls fork 并启用相应 feature。
    #[cfg(false)] // 暂时禁用，因为标准 rustls 不支持
    if let Some(profile) = profile {
        if let Some(customizer) = ProfileClientHelloCustomizer::try_from_profile(profile) {
            cfg = cfg.with_client_hello_customizer(customizer.into_arc());
        }
    }
    cfg
}
