//! TLS 可observepropertymodule
//!
//! provide TLS ClientHello 可observepropertycountdataExtract
//! reference：Huginn Net Profiler TlsClientObserved design

use crate::tls_config::extract::extract_signature;
use crate::tls_config::signature::ClientHelloSignature;
use crate::tls_config::spec::ClientHelloSpec;
use fingerprint_core::dicttls::supported_groups::CurveID;

/// TLS ClientHello 可observecountdata
/// includingallcan from ClientHello in observe to info
/// reference：Huginn Net Profiler TlsClientObserved
#[derive(Debug, Clone, PartialEq)]
pub struct TlsClientObserved {
 /// TLS version (stringrepresent, 如 "13", "12")
 pub version: String,
 /// Server Name Indication
 pub sni: Option<String>,
 /// Application-Layer Protocol Negotiation
 pub alpn: Option<String>,
 /// cipher suitelist
 pub cipher_suites: Vec<u16>,
 /// extensionlist
 pub extensions: Vec<u16>,
 /// signaturealgorithmlist
 pub signature_algorithms: Vec<u16>,
 /// elliptic curvelist
 pub elliptic_curves: Vec<CurveID>,
}

impl TlsClientObserved {
 /// from ClientHelloSpec Create可observecountdata
 pub fn from_spec(spec: &ClientHelloSpec) -> Self {
 let signature = extract_signature(spec);
 Self::from_signature(&signature)
 }

 /// from ClientHelloSignature Create可observecountdata
 pub fn from_signature(signature: &ClientHelloSignature) -> Self {
 Self {
 version: format!("{}", signature.version),
 sni: signature.sni.clone(),
 alpn: signature.alpn.clone(),
 cipher_suites: signature.cipher_suites.clone(),
 extensions: signature.extensions.clone(),
 signature_algorithms: signature.signature_algorithms.clone(),
 elliptic_curves: signature.elliptic_curves.clone(),
 }
 }

 /// Getcipher suitecount
 pub fn cipher_suite_count(&self) -> usize {
 self.cipher_suites.len()
 }

 /// Getextensioncount
 pub fn extension_count(&self) -> usize {
 self.extensions.len()
 }

 /// Getsignaturealgorithmcount
 pub fn signature_algorithm_count(&self) -> usize {
 self.signature_algorithms.len()
 }

 /// Checkwhetherincludingspecificextension
 pub fn has_extension(&self, ext_id: u16) -> bool {
 self.extensions.contains(&ext_id)
 }

 /// Checkwhetherincludingspecificcipher suite
 pub fn has_cipher_suite(&self, suite: u16) -> bool {
 self.cipher_suites.contains(&suite)
 }
}

#[cfg(test)]
mod tests {
 use super::*;
 use crate::tls_config::version::TlsVersion;

 #[test]
 fn test_format_tls_version() {
 assert_eq!(format!("{}", TlsVersion::V1_3), "13");
 assert_eq!(format!("{}", TlsVersion::V1_2), "12");
 assert_eq!(format!("{}", TlsVersion::V1_0), "10");
 }

 #[test]
 fn test_from_spec() {
 let spec = ClientHelloSpec::chrome_133();
 let observed = TlsClientObserved::from_spec(&spec);
 assert!(!observed.cipher_suites.is_empty());
 assert!(!observed.extensions.is_empty());
 }

 #[test]
 fn test_has_extension() {
 let spec = ClientHelloSpec::chrome_133();
 let observed = TlsClientObserved::from_spec(&spec);
 // Checkwhetherincluding SNI extension (0x0000)
 let has_sni = observed.has_extension(0x0000);
 // Chrome 133 shouldincluding SNI
 assert!(has_sni);
 }
}
