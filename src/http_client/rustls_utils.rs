//! rustls 配置工具（供 HTTP/1/2/3 复用）
//!
//! 目标：
//! - 单一入口构建 root store
//! - 单一入口应用 verify_tls（可选禁用校验，仅用于调试/内网）
//! - 单一入口配置 ALPN

#![cfg(any(feature = "rustls-tls", feature = "http2", feature = "http3"))]

#[cfg(feature = "dangerous_configuration")]
use std::sync::Arc;

use crate::ClientProfile;

#[cfg(feature = "rustls-client-hello-customizer")]
use super::rustls_client_hello_customizer::ProfileClientHelloParams;

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
    let mut cfg = {
        // 默认：安全默认值（兼容性最好）。
        let fallback = || {
            let root_store = build_root_store();
            rustls::ClientConfig::builder()
                .with_safe_defaults()
                .with_root_certificates(root_store)
                .with_no_client_auth()
        };

        // 若启用“路线 A”能力且提供了 profile，则尽量把 spec 映射到 rustls builder。
        #[cfg(feature = "rustls-client-hello-customizer")]
        if let Some(profile) = profile {
            if let Some(params) = ProfileClientHelloParams::try_from_profile(profile) {
                if let Some(cfg) = try_build_config_from_params(params) {
                    cfg
                } else {
                    fallback()
                }
            } else {
                fallback()
            }
        } else {
            fallback()
        }

        #[cfg(not(feature = "rustls-client-hello-customizer"))]
        {
            let _ = profile;
            fallback()
        }
    };

    cfg.alpn_protocols = alpn_protocols;
    apply_verify_tls(&mut cfg, verify_tls);

    cfg
}

#[cfg(feature = "rustls-client-hello-customizer")]
fn try_build_config_from_params(params: ProfileClientHelloParams) -> Option<rustls::ClientConfig> {
    use crate::tls_config::{VERSION_TLS12, VERSION_TLS13};
    use crate::{dicttls::supported_groups, tls_config::is_grease_value};

    // 重新建 root_store（避免借用/移动问题，且成本很低）
    let root_store = build_root_store();

    // --- cipher suites（按 spec 顺序取 rustls 支持子集） ---
    fn cipher_suite_id(s: rustls::CipherSuite) -> u16 {
        s.get_u16()
    }

    let mut cipher_suites: Vec<rustls::SupportedCipherSuite> = Vec::new();
    for id in params.cipher_suite_ids.iter().copied().filter(|id| !is_grease_value(*id)) {
        if let Some(cs) = rustls::ALL_CIPHER_SUITES
            .iter()
            .copied()
            .find(|cs| cipher_suite_id(cs.suite()) == id)
        {
            cipher_suites.push(cs);
        }
    }
    if cipher_suites.is_empty() {
        return None;
    }

    // --- kx groups（按 spec 顺序取 rustls 支持子集） ---
    let mut kx_groups: Vec<&'static rustls::SupportedKxGroup> = Vec::new();
    let mut seen_kx_group_ids: Vec<u16> = Vec::new();
    for id in &params.kx_group_ids {
        let g = match *id {
            supported_groups::X25519 => Some(&rustls::kx_group::X25519),
            supported_groups::CURVE_P256 => Some(&rustls::kx_group::SECP256R1),
            supported_groups::CURVE_P384 => Some(&rustls::kx_group::SECP384R1),
            _ => None, // rustls 0.21 不支持的 group（例如 X25519MLKEM768/SECP521R1）直接跳过
        };
        if let Some(g) = g {
            if !seen_kx_group_ids.contains(id) {
                kx_groups.push(g);
                seen_kx_group_ids.push(*id);
            }
        }
    }
    if kx_groups.is_empty() {
        return None;
    }

    // --- versions（按 spec 顺序） ---
    let mut versions: Vec<&'static rustls::SupportedProtocolVersion> = Vec::new();
    for v in &params.versions {
        let vv = match *v {
            VERSION_TLS13 => Some(&rustls::version::TLS13),
            VERSION_TLS12 => Some(&rustls::version::TLS12),
            _ => None,
        };
        if let Some(vv) = vv {
            if !versions.contains(&vv) {
                versions.push(vv);
            }
        }
    }
    // 若 spec 只给了 1.3 或没给，至少要有一个版本
    if versions.is_empty() {
        versions.push(&rustls::version::TLS13);
        versions.push(&rustls::version::TLS12);
    }

    // builder 链路：如果版本组合不合法会返回 Err（例如只有 TLS1.0），此处直接放弃
    let builder = rustls::ClientConfig::builder()
        .with_cipher_suites(&cipher_suites)
        .with_kx_groups(&kx_groups)
        .with_protocol_versions(&versions)
        .ok()?;

    Some(
        builder
            .with_root_certificates(root_store)
            .with_no_client_auth(),
    )
}
