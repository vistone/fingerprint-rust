//! rustls configurationtool (供 HTTP/1/2/3 reuse)
//!
//! target：
//! - single entryBuild root store
//! - single entryapplication verify_tls (optionaldisabledvalidate，only for debug/inside网)
//! - single entryconfiguration ALPN

#![cfg(any(feature = "rustls-tls", feature = "http2", feature = "http3"))]

#[cfg(feature = "dangerous_configuration")]
use std::sync::Arc;

use fingerprint_profiles::profiles::ClientProfile;

// Note: ProfileClientHelloCustomizer needsupport ClientHelloCustomizer rustls fork
// current被disabled，becausestandard rustls excluding ClientHelloCustomizer API
#[cfg(false)] // 暂 when disabled，becausestandard rustls 不support
use super::rustls_client_hello_customizer::ProfileClientHelloCustomizer;

/// Build rustls rootcertificatestore (Mozilla roots)
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

/// 若 verify_tls=false，则安装"acceptallcertificate" verifier (危险Features，only for debug)
#[allow(unused_variables)]
pub fn apply_verify_tls(cfg: &mut rustls::ClientConfig, verify_tls: bool) {
 if verify_tls {
 return;
 }

 // Note: rustls 0.21 API maydifferent
 // If verify_tls=false, use dangerous configurationacceptallcertificate
 // 这need rustls dangerous_configuration feature
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
 // Ifno dangerous_configuration feature, ignore verify_tls=false settings
 // beginningfinalValidatecertificate (更security)
 eprintln!("warning: verify_tls=false need dangerous_configuration feature，alreadyignore");
 }
}

/// Build rustls::ClientConfig，并settings ALPN/verify_tls，andBased on fingerprintmatchcipher suite。
pub fn build_client_config(
 verify_tls: bool,
 alpn_protocols: Vec<Vec<u8>>,
 #[allow(unused_variables)] profile: Option<&ClientProfile>,
) -> rustls::ClientConfig {
 let root_store = build_root_store();

 // defaultconfiguration ( if unable toBased on profile match，则back to securitydefaultvalue)
 let builder = rustls::ClientConfig::builder()
.with_safe_defaults()
.with_root_certificates(root_store)
.with_no_client_auth();

 let mut cfg = builder;

 // 强化fingerprint：matchspecific's cipher suites and TLS version
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

 // optional： in send ClientHello before by fingerprint spec reorderextensionencodingorder (need配套 rustls fork)。
 // Note: 此Featuresneedsupport ClientHelloCustomizer rustls fork，standard rustls 不support。
 // current被disabled，becausestandard rustls excluding ClientHelloCustomizer API。
 // if neededuse此Features，needusesupport ClientHelloCustomizer rustls fork 并enabledcorresponding feature。
 #[cfg(false)] // 暂 when disabled，becausestandard rustls 不support
 if let Some(profile) = profile {
 if let Some(customizer) = ProfileClientHelloCustomizer::try_from_profile(profile) {
 cfg = cfg.with_client_hello_customizer(customizer.into_arc());
 }
 }
 cfg
}
