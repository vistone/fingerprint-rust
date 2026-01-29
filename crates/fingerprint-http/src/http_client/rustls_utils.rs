//! rustls configuration utilities (provides HTTP/1/2/3 reuse)
//!
//! This module provides utility functions for building rustls configurations.
//!
//! ## Features
//!
//! - `build_root_store()`: Build root certificate store using Mozilla roots
//! - `apply_verify_tls()`: Configure TLS certificate verification
//! - `build_client_config()`: Build complete rustls ClientConfig with ALPN and verification
//!
//! ## Security Warning
//!
//! **The `dangerous_configuration` feature allows disabling TLS certificate verification.**
//!
//! **DO NOT USE IN PRODUCTION!** This feature is intended only for:
//! - Local development and testing
//! - Internal network environments where certificates cannot be properly issued
//! - Debugging TLS handshake issues
//!
//! Disabling certificate verification exposes your application to:
//! - Man-in-the-middle (MITM) attacks
//! - Credential theft
//! - Data interception
//!
//! If you need to work with self-signed certificates in production, consider:
//! - Using certificate pinning instead
//! - Adding custom root certificates to the trust store
//! - Using proper PKI infrastructure

#![cfg(any(feature = "rustls-tls", feature = "http2", feature = "http3"))]

#[cfg(feature = "dangerous_configuration")]
use std::sync::Arc;

use fingerprint_profiles::profiles::ClientProfile;
use std::sync::Once;

// Note: ProfileClientHelloCustomizer needsupport ClientHelloCustomizer rustls fork
// current被disabled, becausestandard rustls excluding ClientHelloCustomizer API
#[cfg(false)] // 暂 when disabled，becausestandard rustls 不support
use super::rustls_client_hello_customizer::ProfileClientHelloCustomizer;

/// Ensure the crypto provider is installed (ring)
static INIT_CRYPTO_PROVIDER: Once = Once::new();

/// Initialize the rustls crypto provider (ring) if not already done.
/// This must be called before any TLS operations.
fn ensure_crypto_provider() {
    INIT_CRYPTO_PROVIDER.call_once(|| {
        let _ = rustls::crypto::ring::default_provider().install_default();
    });
}

/// Build rustls rootcertificatestore (Mozilla roots)
pub fn build_root_store() -> rustls::RootCertStore {
    ensure_crypto_provider();
    let mut root_store = rustls::RootCertStore::empty();
    root_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
    root_store
}

/// If verify_tls=false, install an "accept all certificates" verifier (dangerous feature, only for debug)
///
/// # Security Warning
///
/// **This function can completely disable TLS certificate verification when the
/// `dangerous_configuration` feature is enabled and `verify_tls` is set to `false`.**
///
/// This makes your application vulnerable to:
/// - Man-in-the-middle (MITM) attacks
/// - Credential theft and data interception
/// - Impersonation attacks
///
/// # Arguments
///
/// * `cfg` - The rustls ClientConfig to modify
/// * `verify_tls` - If true, certificates are verified normally. If false and
///   the `dangerous_configuration` feature is enabled, all certificates are accepted.
///
/// # Example
///
/// ```ignore
/// // DO NOT USE IN PRODUCTION
/// apply_verify_tls(&mut config, false);
/// ```
#[allow(unused_variables)]
pub fn apply_verify_tls(cfg: &mut rustls::ClientConfig, verify_tls: bool) {
    if verify_tls {
        return;
    }

    // Note: rustls 0.23 API changed - dangerous features now under danger module
    // If verify_tls=false, use dangerous configuration to accept all certificates.
    // This needs the rustls dangerous_configuration feature.
    #[cfg(feature = "dangerous_configuration")]
    {
        use rustls::client::danger::{
            HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier,
        };
        use rustls::pki_types::{CertificateDer, ServerName, UnixTime};
        use rustls::{DigitallySignedStruct, Error as RustlsError, SignatureScheme};

        #[derive(Debug)]
        struct NoCertificateVerification;

        impl ServerCertVerifier for NoCertificateVerification {
            fn verify_server_cert(
                &self,
                _end_entity: &CertificateDer,
                _intermediates: &[CertificateDer],
                _server_name: &ServerName,
                _ocsp_response: &[u8],
                _now: UnixTime,
            ) -> std::result::Result<ServerCertVerified, RustlsError> {
                Ok(ServerCertVerified::assertion())
            }

            fn verify_tls12_signature(
                &self,
                _message: &[u8],
                _cert: &CertificateDer,
                _dss: &DigitallySignedStruct,
            ) -> std::result::Result<HandshakeSignatureValid, RustlsError> {
                Ok(HandshakeSignatureValid::assertion())
            }

            fn verify_tls13_signature(
                &self,
                _message: &[u8],
                _cert: &CertificateDer,
                _dss: &DigitallySignedStruct,
            ) -> std::result::Result<HandshakeSignatureValid, RustlsError> {
                Ok(HandshakeSignatureValid::assertion())
            }

            fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
                vec![
                    SignatureScheme::RSA_PKCS1_SHA1,
                    SignatureScheme::ECDSA_SHA1_Legacy,
                    SignatureScheme::RSA_PKCS1_SHA256,
                    SignatureScheme::ECDSA_NISTP256_SHA256,
                    SignatureScheme::RSA_PKCS1_SHA384,
                    SignatureScheme::ECDSA_NISTP384_SHA384,
                    SignatureScheme::RSA_PKCS1_SHA512,
                    SignatureScheme::ECDSA_NISTP521_SHA512,
                    SignatureScheme::RSA_PSS_SHA256,
                    SignatureScheme::RSA_PSS_SHA384,
                    SignatureScheme::RSA_PSS_SHA512,
                    SignatureScheme::ED25519,
                    SignatureScheme::ED448,
                ]
            }
        }

        cfg.dangerous()
            .set_certificate_verifier(Arc::new(NoCertificateVerification));
    }

    #[cfg(not(feature = "dangerous_configuration"))]
    {
        // If dangerous_configuration feature is not enabled, ignore verify_tls=false setting
        // and always validate certificates (more secure)
        eprintln!("warning: verify_tls=false requires dangerous_configuration feature, ignoring");
    }
}

/// Build rustls::ClientConfig with ALPN/verify_tls settings, and match cipher suites based on fingerprint profile.
pub fn build_client_config(
    verify_tls: bool,
    alpn_protocols: Vec<Vec<u8>>,
    #[allow(unused_variables)] profile: Option<&ClientProfile>,
) -> rustls::ClientConfig {
    let root_store = build_root_store();

    // defaultconfiguration ( if unable toBased on profile match, thenback to securitydefaultvalue)
    let builder = rustls::ClientConfig::builder()
        .with_root_certificates(root_store)
        .with_no_client_auth();

    let mut cfg = builder;

    // strong化fingerprint：matchspecific's cipher suites and TLS version
    // FIXME: s.suite() as u16 fail on rustls 0.21. Restore this when fixed.
    /*
     if let Some(profile) = profile {
     if let Ok(spec) = profile.get_client_hello_spec() {
     // 1. matchcipher suite
     let mut suites = Vec::new();
     for &suite_id in &spec.cipher_suites {
     if let Some(suite) = rustls::ALL_CIPHER_SUITES
    .iter()
    .find(|s| s.suite() as u16 == suite_id) {
     suites.push(*suite);
     }
     }

     if !suites.is_empty() {
     // reBuildconfiguration以applicationspecificsuite
     let mut versions = Vec::new();
     if spec.tls_vers_max >= 0x0304 { // TLS 1.3
     versions.push(&rustls::version::TLS13);
     }
     if spec.tls_vers_min <= 0x0303 { // TLS 1.2
     versions.push(&rustls::version::TLS12);
     }

     // Note: rustls 0.21 needthrough builder resettings
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

    // optional： in send ClientHello before by fingerprint spec reorderextensionencodingorder (needmatch套 rustls fork).
    // Note: 此Featuresneedsupport ClientHelloCustomizer rustls fork, standard rustls 不support.
    // current被disabled, becausestandard rustls excluding ClientHelloCustomizer API.
    // if neededuse此Features, needusesupport ClientHelloCustomizer rustls fork 并enabledcorresponding feature.
    #[cfg(false)] // 暂 when disabled，becausestandard rustls 不support
    if let Some(profile) = profile {
        if let Some(customizer) = ProfileClientHelloCustomizer::try_from_profile(profile) {
            cfg = cfg.with_client_hello_customizer(customizer.into_arc());
        }
    }
    cfg
}
