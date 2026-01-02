//! TLS ClientHello Signature module
//!
//! provide TLS ClientHello signatureExtract and compareFeatures
//! reference：Huginn Net Signature structdesign

use crate::dicttls::supported_groups::CurveID;
use crate::fingerprint::{Fingerprint, FingerprintType};
use crate::grease::{filter_grease_values, is_grease_value};
use crate::metadata::FingerprintMetadata;
use crate::version::TlsVersion;
use sha2::{Digest, Sha256};

/// TLS ClientHello signature
/// including from ClientHello message in Extractallclosekey information
#[derive(Debug, Clone, PartialEq)]
pub struct ClientHelloSignature {
 /// fingerprint ID（based on JA4 hash or signaturetraithash）
 pub id: String,

 /// TLS version
 pub version: TlsVersion,
 /// cipher suitelist (including GREASE)
 pub cipher_suites: Vec<u16>,
 /// extensionlist (including GREASE)
 pub extensions: Vec<u16>,
 /// elliptic curvelist
 pub elliptic_curves: Vec<CurveID>,
 /// elliptic curve点format
 pub elliptic_curve_point_formats: Vec<u8>,
 /// signaturealgorithmlist
 pub signature_algorithms: Vec<u16>,
 /// Server Name Indication
 pub sni: Option<String>,
 /// Application-Layer Protocol Negotiation
 pub alpn: Option<String>,

 /// metadata
 pub metadata: FingerprintMetadata,
}

impl ClientHelloSignature {
 /// Create a newsignature
 pub fn new() -> Self {
 let mut sig = Self {
 id: String::new(),
 version: TlsVersion::V1_2, // default TLS 1.2
 cipher_suites: Vec::new(),
 extensions: Vec::new(),
 elliptic_curves: Vec::new(),
 elliptic_curve_point_formats: Vec::new(),
 signature_algorithms: Vec::new(),
 sni: None,
 alpn: None,
 metadata: FingerprintMetadata::new(),
 };
 sig.id = sig.calculate_id();
 sig
 }

 /// Calculatefingerprint ID（based onsignaturetrait）
 pub fn calculate_id(&self) -> String {
 let mut hasher = Sha256::new();
 hasher.update(self.version.to_u16().to_be_bytes());
 hasher.update(self.cipher_suites_without_grease().len().to_be_bytes());
 for &cs in &self.cipher_suites_without_grease() {
 hasher.update(cs.to_be_bytes());
 }
 hasher.update(self.extensions_without_grease().len().to_be_bytes());
 for &ext in &self.extensions_without_grease() {
 hasher.update(ext.to_be_bytes());
 }
 for &curve in &self.elliptic_curves {
 hasher.update(curve.to_be_bytes());
 }
 if let Some(ref sni) = self.sni {
 hasher.update(sni.as_bytes());
 }
 if let Some(ref alpn) = self.alpn {
 hasher.update(alpn.as_bytes());
 }
 format!("{:x}", hasher.finalize())
 }

 /// Getfilter GREASE back's cipher suites
 pub fn cipher_suites_without_grease(&self) -> Vec<u16> {
 filter_grease_values(&self.cipher_suites)
 }

 /// Getfilter GREASE back's extensions
 pub fn extensions_without_grease(&self) -> Vec<u16> {
 filter_grease_values(&self.extensions)
 }

 /// Getfilter GREASE backsignaturealgorithm
 pub fn signature_algorithms_without_grease(&self) -> Vec<u16> {
 filter_grease_values(&self.signature_algorithms)
 }

 /// Checkwhetherincluding GREASE value
 pub fn has_grease(&self) -> bool {
 self.cipher_suites.iter().any(|&v| is_grease_value(v))
 || self.extensions.iter().any(|&v| is_grease_value(v))
 || self
.signature_algorithms
.iter()
.any(|&v| is_grease_value(v))
 }

 /// compare twosignaturewhethersimilar（ignore GREASE value）
 ///
 /// # Parameters
 /// * `other` - 要compare另ansignature
 ///
 /// # Returns
 /// * `true` if signaturesimilar（ignore GREASE backsame），`false` otherwise
 pub fn similar_to(&self, other: &Self) -> bool {
 self.version == other.version
 && self.cipher_suites_without_grease() == other.cipher_suites_without_grease()
 && self.extensions_without_grease() == other.extensions_without_grease()
 && self.signature_algorithms_without_grease()
 == other.signature_algorithms_without_grease()
 && self.elliptic_curves == other.elliptic_curves
 && self.elliptic_curve_point_formats == other.elliptic_curve_point_formats
 && self.sni == other.sni
 && self.alpn == other.alpn
 }

 /// Calculatesignaturehashvalue（ for fastcompare）
 /// usefilter GREASE backvalue
 pub fn hash(&self) -> u64 {
 use std::collections::hash_map::DefaultHasher;
 use std::hash::{Hash, Hasher};

 let mut hasher = DefaultHasher::new();
 self.version.to_u16().hash(&mut hasher);
 self.cipher_suites_without_grease().hash(&mut hasher);
 self.extensions_without_grease().hash(&mut hasher);
 self.signature_algorithms_without_grease().hash(&mut hasher);
 self.elliptic_curves.hash(&mut hasher);
 self.elliptic_curve_point_formats.hash(&mut hasher);
 self.sni.hash(&mut hasher);
 self.alpn.hash(&mut hasher);
 hasher.finish()
 }
}

impl Fingerprint for ClientHelloSignature {
 fn fingerprint_type(&self) -> FingerprintType {
 FingerprintType::Tls
 }

 fn id(&self) -> String {
 self.id.clone()
 }

 fn metadata(&self) -> &FingerprintMetadata {
 &self.metadata
 }

 fn metadata_mut(&mut self) -> &mut FingerprintMetadata {
 &mut self.metadata
 }

 fn hash(&self) -> u64 {
 self.hash()
 }

 fn similar_to(&self, other: &dyn Fingerprint) -> bool {
 if other.fingerprint_type() != FingerprintType::Tls {
 return false;
 }

 // tryconvert to ClientHelloSignature
 // due to trait limit，wecan onlycomparehashvalue
 // actualuse in ，shouldthroughtypeConvert来compare
 self.hash() == other.hash()
 }

 fn to_string(&self) -> String {
 format!(
 "ClientHelloSignature(id={}, version={:?}, cipher_suites={}, extensions={})",
 self.id,
 self.version,
 self.cipher_suites_without_grease().len(),
 self.extensions_without_grease().len()
 )
 }
}

impl Default for ClientHelloSignature {
 fn default() -> Self {
 Self::new()
 }
}

#[cfg(test)]
mod tests {
 use super::*;

 #[test]
 fn test_similar_to() {
 let mut sig1 = ClientHelloSignature::new();
 sig1.version = TlsVersion::V1_2;
 sig1.cipher_suites = vec![0x0a0a, 0x0017, 0x1a1a]; // including GREASE
 sig1.extensions = vec![0x0000, 0x0010];

 let mut sig2 = ClientHelloSignature::new();
 sig2.version = TlsVersion::V1_2;
 sig2.cipher_suites = vec![0x0017, 0x2a2a]; // different GREASE，butfilterbacksame
 sig2.extensions = vec![0x0000, 0x0010];

 // filter GREASE backshouldsame
 assert_eq!(
 sig1.cipher_suites_without_grease(),
 sig2.cipher_suites_without_grease()
 );
 assert!(sig1.similar_to(&sig2));
 }

 #[test]
 fn test_has_grease() {
 let mut sig = ClientHelloSignature::new();
 assert!(!sig.has_grease());

 sig.cipher_suites = vec![0x0a0a, 0x0017];
 assert!(sig.has_grease());

 sig.cipher_suites = vec![0x0017];
 sig.extensions = vec![0x1a1a];
 assert!(sig.has_grease());
 }
}
