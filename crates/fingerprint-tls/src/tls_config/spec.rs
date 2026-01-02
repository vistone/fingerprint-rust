//! TLS ClientHelloSpec implement
//!
//! providereal TLS Client Hello configuration，Corresponds to Go version's utls.ClientHelloSpec

use crate::tls_extensions::{
 ALPNExtension, ApplicationSettingsExtensionNew, ExtendedMasterSecretExtension, KeyShare,
 KeyShareExtension, PSKKeyExchangeModesExtension, RenegotiationInfoExtension, SCTExtension,
 SNIExtension, SignatureAlgorithmsExtension, StatusRequestExtension, SupportedCurvesExtension,
 SupportedPointsExtension, SupportedVersionsExtension, TLSExtension, UtlsCompressCertExtension,
 UtlsGREASEExtension, UtlsPaddingExtension,
};
use fingerprint_core::dicttls::{
 cipher_suites::{self as cs, GREASE_PLACEHOLDER as GREASE_CS},
 signature_schemes::{
 self as ss, ECDSA_WITH_P256_AND_SHA256, ECDSA_WITH_P384_AND_SHA384, PKCS1_WITH_SHA256,
 PKCS1_WITH_SHA384, PKCS1_WITH_SHA512, PSS_WITH_SHA256, PSS_WITH_SHA384, PSS_WITH_SHA512,
 },
 supported_groups::{
 CURVE_P256, CURVE_P384, GREASE_PLACEHOLDER as GREASE_SG, SECP521R1, X25519, X25519_MLKEM768,
 },
};

/// TLS versionconstant
pub const VERSION_TLS10: u16 = 0x0301;
pub const VERSION_TLS11: u16 = 0x0302;
pub const VERSION_TLS12: u16 = 0x0303;
pub const VERSION_TLS13: u16 = 0x0304;

/// compressionmethodconstant
pub const COMPRESSION_NONE: u8 = 0x00;

/// 点formatconstant
pub const POINT_FORMAT_UNCOMPRESSED: u8 = 0x00;

/// PSK patternconstant
pub const PSK_MODE_DHE: u8 = 0x01;

/// re协商constant
pub const RENEGOTIATE_ONCE_AS_CLIENT: u8 = 1;

/// certificatecompressionalgorithmconstant
pub const CERT_COMPRESSION_BROTLI: u16 = 0x0002;

/// cipher suite ID
pub type CipherSuiteID = u16;

/// TLS Client Hello configuration
/// Corresponds to Go version's tls.ClientHelloSpec
///
/// Note: due toextension is trait pair象，Clone implementwillCreate a newextensioninstance
#[derive(Debug)]
pub struct ClientHelloSpec {
 /// cipher suitelist
 /// Corresponds to Go version's CipherSuites []uint16
 pub cipher_suites: Vec<CipherSuiteID>,
 /// compressionmethod
 /// Corresponds to Go version's CompressionMethods []uint8
 pub compression_methods: Vec<u8>,
 /// extensionlist
 /// Corresponds to Go version's Extensions []TLSExtension
 pub extensions: Vec<Box<dyn TLSExtension>>,
 /// TLS versionminimumvalue
 /// Corresponds to Go version's TLSVersMin uint16
 pub tls_vers_min: u16,
 /// TLS versionmaximumvalue
 /// Corresponds to Go version's TLSVersMax uint16
 pub tls_vers_max: u16,
 /// extensionmetadata（ for store SNI、ALPN etc.countdata）
 /// reference：Huginn Net Profiler 设计
 pub metadata: Option<crate::tls_config::metadata::SpecMetadata>,
}

impl ClientHelloSpec {
 /// Create a new ClientHelloSpec
 /// Corresponds to Go version's ClientHelloSpec{} 零value
 pub fn new() -> Self {
 Self {
 cipher_suites: Vec::new(),
 compression_methods: Vec::new(),
 extensions: Vec::new(),
 tls_vers_min: 0,
 tls_vers_max: 0,
 metadata: None,
 }
 }

 /// Create Chrome 136 fingerprint ClientHelloSpec
 pub fn chrome_136() -> Self {
 use crate::tls_config::ClientHelloSpecBuilder;
 let (extensions, metadata) = ClientHelloSpecBuilder::chrome_136_extensions();
 let mut spec = ClientHelloSpecBuilder::new()
.cipher_suites(ClientHelloSpecBuilder::chrome_136_cipher_suites())
.compression_methods(vec![COMPRESSION_NONE])
.extensions(extensions)
.build();
 spec.metadata = Some(metadata);
 spec
 }

 /// Create Chrome 133 fingerprint ClientHelloSpec
 /// Corresponds to Go version's Chrome_133 SpecFactory
 ///
 /// use Builder patterncan更灵活地Build：
 /// ```rust,no_run
 /// use fingerprint_tls::tls_config::ClientHelloSpecBuilder;
 /// let (extensions, _metadata) = ClientHelloSpecBuilder::chrome_133_extensions();
 /// let spec = ClientHelloSpecBuilder::new()
 ///.cipher_suites(ClientHelloSpecBuilder::chrome_cipher_suites())
 ///.compression_methods(vec![0])
 ///.extensions(extensions)
 ///.build();
 /// ```
 pub fn chrome_133() -> Self {
 use crate::tls_config::ClientHelloSpecBuilder;
 let (extensions, metadata) = ClientHelloSpecBuilder::chrome_133_extensions();
 let mut spec = ClientHelloSpecBuilder::new()
.cipher_suites(ClientHelloSpecBuilder::chrome_cipher_suites())
.compression_methods(vec![COMPRESSION_NONE])
.extensions(extensions)
.build();
 spec.metadata = Some(metadata);
 spec
 }

 /// Create Chrome 133 fingerprint ClientHelloSpec（旧implement，preserve for compatible）
 #[deprecated(note = "use ClientHelloSpecBuilder 代替")]
 pub fn chrome_133_old() -> Self {
 let mut spec = Self::new();

 // Chrome 133 's cipher suites
 spec.cipher_suites = vec![
 GREASE_CS,
 cs::TLS_AES_128_GCM_SHA256,
 cs::TLS_AES_256_GCM_SHA384,
 cs::TLS_CHACHA20_POLY1305_SHA256,
 cs::TLS_ECDHE_ECDSA_WITH_AES_128_GCM_SHA256,
 cs::TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256,
 cs::TLS_ECDHE_ECDSA_WITH_AES_256_GCM_SHA384,
 cs::TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384,
 cs::TLS_ECDHE_ECDSA_WITH_CHACHA20_POLY1305_SHA256,
 cs::TLS_ECDHE_RSA_WITH_CHACHA20_POLY1305_SHA256,
 cs::TLS_ECDHE_RSA_WITH_AES_128_CBC_SHA,
 cs::TLS_ECDHE_RSA_WITH_AES_256_CBC_SHA,
 cs::TLS_RSA_WITH_AES_128_GCM_SHA256,
 cs::TLS_RSA_WITH_AES_256_GCM_SHA384,
 cs::TLS_RSA_WITH_AES_128_CBC_SHA,
 cs::TLS_RSA_WITH_AES_256_CBC_SHA,
 ];

 // compressionmethod
 spec.compression_methods = vec![COMPRESSION_NONE];

 // Chrome 133 's extensionsorder（Corresponds to Go version's ShuffleChromeTLSExtensions）
 spec.extensions = vec![
 Box::new(UtlsGREASEExtension::new()),
 Box::new(SNIExtension::new(String::new())),
 Box::new(ExtendedMasterSecretExtension),
 Box::new(RenegotiationInfoExtension::new(RENEGOTIATE_ONCE_AS_CLIENT)),
 Box::new(SupportedCurvesExtension::new(vec![
 GREASE_SG,
 X25519_MLKEM768,
 X25519,
 CURVE_P256,
 CURVE_P384,
 ])),
 Box::new(SupportedPointsExtension::new(vec![
 POINT_FORMAT_UNCOMPRESSED,
 ])),
 Box::new(crate::tls_extensions::SessionTicketExtension),
 Box::new(ALPNExtension::new(vec![
 "h2".to_string(),
 "http/1.1".to_string(),
 ])),
 Box::new(StatusRequestExtension),
 Box::new(SignatureAlgorithmsExtension::new(vec![
 ECDSA_WITH_P256_AND_SHA256,
 PSS_WITH_SHA256,
 PKCS1_WITH_SHA256,
 ECDSA_WITH_P384_AND_SHA384,
 PSS_WITH_SHA384,
 PKCS1_WITH_SHA384,
 PSS_WITH_SHA512,
 PKCS1_WITH_SHA512,
 ])),
 Box::new(SCTExtension),
 Box::new(KeyShareExtension::new(vec![
 KeyShare {
 group: GREASE_SG,
 data: vec![0],
 },
 KeyShare {
 group: X25519_MLKEM768,
 data: vec![], // actualneedGeneratekey
 },
 KeyShare {
 group: X25519,
 data: vec![], // actualneedGeneratekey
 },
 ])),
 Box::new(PSKKeyExchangeModesExtension::new(vec![PSK_MODE_DHE])),
 Box::new(SupportedVersionsExtension::new(vec![
 GREASE_SG,
 VERSION_TLS13,
 VERSION_TLS12,
 ])),
 Box::new(UtlsCompressCertExtension::new(vec![
 CERT_COMPRESSION_BROTLI,
 ])),
 Box::new(ApplicationSettingsExtensionNew::new(vec!["h2".to_string()])),
 Box::new(UtlsGREASEExtension::new()),
 Box::new(UtlsPaddingExtension::new()),
 ];

 spec
 }

 /// Create Chrome 103 fingerprint ClientHelloSpec
 /// Corresponds to Go version's Chrome_103 SpecFactory
 pub fn calculate_ja4(&self) -> fingerprint_core::ja4::JA4 {
 let transport = 't'; // Default to TCP
 let version = if self.tls_vers_max >= VERSION_TLS13 {
 "1.3"
 } else {
 "1.2"
 };
 let ciphers = self.cipher_suites.clone();
 let extensions: Vec<u16> = self.extensions.iter().map(|e| e.extension_id()).collect();

 let mut alpn = None;
 let mut sig_algs = Vec::new();
 let mut has_sni = false;

 for ext in &self.extensions {
 let any_ext = ext.as_any();
 if let Some(sni) = any_ext.downcast_ref::<SNIExtension>() {
 if !sni.server_name.is_empty() {
 has_sni = true;
 }
 } else if let Some(alpn_ext) = any_ext.downcast_ref::<ALPNExtension>() {
 if let Some(first) = alpn_ext.alpn_protocols.first() {
 alpn = Some(first.as_str());
 }
 } else if let Some(sig_ext) = any_ext.downcast_ref::<SignatureAlgorithmsExtension>() {
 sig_algs = sig_ext.supported_signature_algorithms.clone();
 }
 }

 fingerprint_core::ja4::JA4::generate(
 transport,
 version,
 has_sni,
 &ciphers,
 &extensions,
 alpn,
 &sig_algs,
 )
 }

 pub fn ja4_string(&self) -> String {
 self.calculate_ja4().to_fingerprint_string()
 }

 /// Create Chrome 103 fingerprint ClientHelloSpec
 /// Corresponds to Go version's Chrome_103 SpecFactory
 pub fn chrome_103() -> Self {
 let mut spec = Self::chrome_133();

 // Chrome 103 's elliptic curves (excluding X25519MLKEM768)
 spec.extensions = vec![
 Box::new(UtlsGREASEExtension::new()),
 Box::new(SNIExtension::new(String::new())),
 Box::new(ExtendedMasterSecretExtension),
 Box::new(RenegotiationInfoExtension::new(RENEGOTIATE_ONCE_AS_CLIENT)),
 Box::new(SupportedCurvesExtension::new(vec![
 GREASE_SG, X25519, CURVE_P256, CURVE_P384,
 ])),
 Box::new(SupportedPointsExtension::new(vec![
 POINT_FORMAT_UNCOMPRESSED,
 ])),
 Box::new(crate::tls_extensions::SessionTicketExtension),
 Box::new(ALPNExtension::new(vec![
 "h2".to_string(),
 "http/1.1".to_string(),
 ])),
 Box::new(StatusRequestExtension),
 Box::new(SignatureAlgorithmsExtension::new(vec![
 ECDSA_WITH_P256_AND_SHA256,
 PSS_WITH_SHA256,
 PKCS1_WITH_SHA256,
 ECDSA_WITH_P384_AND_SHA384,
 PSS_WITH_SHA384,
 PKCS1_WITH_SHA384,
 PSS_WITH_SHA512,
 PKCS1_WITH_SHA512,
 ])),
 Box::new(SCTExtension),
 Box::new(KeyShareExtension::new(vec![
 KeyShare {
 group: GREASE_SG,
 data: vec![0],
 },
 KeyShare {
 group: X25519,
 data: vec![], // actualneedGeneratekey
 },
 ])),
 Box::new(PSKKeyExchangeModesExtension::new(vec![PSK_MODE_DHE])),
 Box::new(SupportedVersionsExtension::new(vec![
 GREASE_SG,
 VERSION_TLS13,
 VERSION_TLS12,
 ])),
 Box::new(UtlsCompressCertExtension::new(vec![
 CERT_COMPRESSION_BROTLI,
 ])),
 Box::new(ApplicationSettingsExtensionNew::new(vec!["h2".to_string()])),
 Box::new(UtlsGREASEExtension::new()),
 Box::new(UtlsPaddingExtension::new()),
 ];

 spec
 }

 /// Create Firefox 133 fingerprint ClientHelloSpec
 /// Corresponds to Go version's Firefox_133 SpecFactory
 pub fn firefox_133() -> Self {
 let mut spec = Self::new();

 // Firefox 133 's cipher suites
 spec.cipher_suites = vec![
 cs::TLS_AES_256_GCM_SHA384,
 cs::TLS_AES_128_GCM_SHA256,
 cs::TLS_CHACHA20_POLY1305_SHA256,
 cs::TLS_ECDHE_ECDSA_WITH_AES_256_GCM_SHA384,
 cs::TLS_ECDHE_ECDSA_WITH_AES_128_GCM_SHA256,
 cs::TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384,
 cs::TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256,
 cs::TLS_ECDHE_ECDSA_WITH_CHACHA20_POLY1305_SHA256,
 cs::TLS_ECDHE_RSA_WITH_CHACHA20_POLY1305_SHA256,
 ];

 // compressionmethod
 spec.compression_methods = vec![COMPRESSION_NONE];

 // Firefox 133 's extensions（simplified version，actualneedview Go versioncompleteimplement）
 spec.extensions = vec![
 Box::new(SupportedCurvesExtension::new(vec![
 CURVE_P256, CURVE_P384, SECP521R1, X25519,
 ])),
 Box::new(SupportedPointsExtension::new(vec![
 POINT_FORMAT_UNCOMPRESSED,
 ])),
 Box::new(SignatureAlgorithmsExtension::new(vec![
 PSS_WITH_SHA256,
 PSS_WITH_SHA384,
 PSS_WITH_SHA512,
 PKCS1_WITH_SHA256,
 PKCS1_WITH_SHA384,
 PKCS1_WITH_SHA512,
 ECDSA_WITH_P256_AND_SHA256,
 ECDSA_WITH_P384_AND_SHA384,
 ss::ECDSA_WITH_P521_AND_SHA512,
 ])),
 Box::new(ALPNExtension::new(vec![
 "h2".to_string(),
 "http/1.1".to_string(),
 ])),
 Box::new(ExtendedMasterSecretExtension),
 Box::new(SupportedVersionsExtension::new(vec![VERSION_TLS13])),
 ];

 spec
 }

 /// Create Safari 16.0 fingerprint ClientHelloSpec
 /// Corresponds to Go version's Safari_16_0 SpecFactory
 pub fn safari_16_0() -> Self {
 let mut spec = Self::new();

 // Safari 16.0 's cipher suites
 spec.cipher_suites = vec![
 cs::TLS_AES_128_GCM_SHA256,
 cs::TLS_AES_256_GCM_SHA384,
 cs::TLS_CHACHA20_POLY1305_SHA256,
 cs::TLS_ECDHE_ECDSA_WITH_AES_128_GCM_SHA256,
 cs::TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256,
 cs::TLS_ECDHE_ECDSA_WITH_AES_256_GCM_SHA384,
 cs::TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384,
 ];

 // compressionmethod
 spec.compression_methods = vec![COMPRESSION_NONE];

 // Safari 16.0 's extensions (simplified version)
 spec.extensions = vec![
 Box::new(SupportedCurvesExtension::new(vec![
 CURVE_P256, CURVE_P384, X25519,
 ])),
 Box::new(SupportedPointsExtension::new(vec![
 POINT_FORMAT_UNCOMPRESSED,
 ])),
 Box::new(SignatureAlgorithmsExtension::new(vec![
 PKCS1_WITH_SHA256,
 PKCS1_WITH_SHA384,
 PKCS1_WITH_SHA512,
 ECDSA_WITH_P256_AND_SHA256,
 ECDSA_WITH_P384_AND_SHA384,
 ])),
 Box::new(ALPNExtension::new(vec![
 "h2".to_string(),
 "http/1.1".to_string(),
 ])),
 Box::new(SupportedVersionsExtension::new(vec![VERSION_TLS13])),
 ];

 spec
 }
}

/// Chrome 103 Spec Factory
/// Corresponds to Go version's Chrome_103 SpecFactory
pub fn chrome_103_spec() -> Result<ClientHelloSpec, String> {
 Ok(ClientHelloSpec::chrome_103())
}

/// Chrome 136 Spec Factory
pub fn chrome_136_spec() -> Result<ClientHelloSpec, String> {
 Ok(ClientHelloSpec::chrome_136())
}

/// Chrome 133 Spec Factory
/// Corresponds to Go version's Chrome_133 SpecFactory
pub fn chrome_133_spec() -> Result<ClientHelloSpec, String> {
 Ok(ClientHelloSpec::chrome_133())
}

/// Firefox 133 Spec Factory
/// Corresponds to Go version's Firefox_133 SpecFactory
pub fn firefox_133_spec() -> Result<ClientHelloSpec, String> {
 Ok(ClientHelloSpec::firefox_133())
}

/// Safari 16.0 Spec Factory
/// Corresponds to Go version's Safari_16_0 SpecFactory
pub fn safari_16_0_spec() -> Result<ClientHelloSpec, String> {
 Ok(ClientHelloSpec::safari_16_0())
}

impl Default for ClientHelloSpec {
 fn default() -> Self {
 Self::chrome_133()
 }
}
