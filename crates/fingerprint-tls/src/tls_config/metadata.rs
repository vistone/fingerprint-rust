//! TLS metadatastoremodule
//!
//! in Build ClientHelloSpec when saveextensionmetadata (SNI、ALPN etc.)
//! this waycan in Extractsignature when Getcompleteinfo

use std::collections::HashMap;

/// TLS extensionmetadata
/// storeextensioninside部countdata， for back续Extract
#[derive(Debug, Clone, Default)]
pub struct ExtensionMetadata {
 /// SNI value ( if exists)
 pub sni: Option<String>,
 /// ALPN protocollist ( if exists)
 pub alpn: Option<Vec<String>>,
 /// elliptic curvelist ( if exists)
 pub elliptic_curves: Option<Vec<u16>>,
 /// elliptic curve点format ( if exists)
 pub elliptic_curve_point_formats: Option<Vec<u8>>,
 /// signaturealgorithmlist ( if exists)
 pub signature_algorithms: Option<Vec<u16>>,
 /// supportversion ( if exists)
 pub supported_versions: Option<Vec<u16>>,
}

/// ClientHelloSpec metadata
/// for storeextensioninside部countdata
#[derive(Debug, Clone, Default)]
pub struct SpecMetadata {
 /// extensionmetadatamap (extension ID -> metadata)
 pub extension_metadata: HashMap<u16, ExtensionMetadata>,
}

impl SpecMetadata {
 /// Create a newmetadata
 pub fn new() -> Self {
 Self::default()
 }

 /// settings SNI
 pub fn set_sni(&mut self, sni: String) {
 let metadata = self
.extension_metadata
.entry(fingerprint_core::dicttls::extensions::EXT_TYPE_SERVER_NAME)
.or_default();
 metadata.sni = Some(sni);
 }

 /// settings ALPN
 pub fn set_alpn(&mut self, alpn: Vec<String>) {
 let metadata = self
.extension_metadata
.entry(fingerprint_core::dicttls::extensions::EXT_TYPE_APPLICATION_LAYER_PROTOCOL_NEGOTIATION)
.or_default();
 metadata.alpn = Some(alpn);
 }

 /// settingselliptic curve
 pub fn set_elliptic_curves(&mut self, curves: Vec<u16>) {
 let metadata = self
.extension_metadata
.entry(fingerprint_core::dicttls::extensions::EXT_TYPE_SUPPORTED_GROUPS)
.or_default();
 metadata.elliptic_curves = Some(curves);
 }

 /// settingselliptic curve点format
 pub fn set_elliptic_curve_point_formats(&mut self, formats: Vec<u8>) {
 let metadata = self
.extension_metadata
.entry(fingerprint_core::dicttls::extensions::EXT_TYPE_EC_POINT_FORMATS)
.or_default();
 metadata.elliptic_curve_point_formats = Some(formats);
 }

 /// settingssignaturealgorithm
 pub fn set_signature_algorithms(&mut self, algorithms: Vec<u16>) {
 let metadata = self
.extension_metadata
.entry(fingerprint_core::dicttls::extensions::EXT_TYPE_SIGNATURE_ALGORITHMS)
.or_default();
 metadata.signature_algorithms = Some(algorithms);
 }

 /// settingssupportversion
 pub fn set_supported_versions(&mut self, versions: Vec<u16>) {
 let metadata = self
.extension_metadata
.entry(fingerprint_core::dicttls::extensions::EXT_TYPE_SUPPORTED_VERSIONS)
.or_default();
 metadata.supported_versions = Some(versions);
 }

 /// Get SNI
 pub fn get_sni(&self) -> Option<&String> {
 self.extension_metadata
.get(&fingerprint_core::dicttls::extensions::EXT_TYPE_SERVER_NAME)
.and_then(|m| m.sni.as_ref())
 }

 /// Get ALPN
 pub fn get_alpn(&self) -> Option<&Vec<String>> {
 self.extension_metadata
.get(&fingerprint_core::dicttls::extensions::EXT_TYPE_APPLICATION_LAYER_PROTOCOL_NEGOTIATION)
.and_then(|m| m.alpn.as_ref())
 }

 /// Getfirst ALPN protocol ( for signature)
 pub fn get_first_alpn(&self) -> Option<String> {
 self.get_alpn().and_then(|alpn| alpn.first().cloned())
 }
}
