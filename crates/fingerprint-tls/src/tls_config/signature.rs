//! TLS ClientHello Signature module
//!
//! provide TLS ClientHello 的signatureExtract and 比较Features
//! 参考：Huginn Net  Signature struct设计

use crate::tls_config::grease::{filter_grease_values, is_grease_value};
use crate::tls_config::version::TlsVersion;
use fingerprint_core::dicttls::supported_groups::CurveID;

/// TLS ClientHello signature
/// including from  ClientHello message中Extract的all关keyinfo
#[derive(Debug, Clone, PartialEq)]
pub struct ClientHelloSignature {
    /// TLS version
    pub version: TlsVersion,
    /// cipher suitelist (including GREASE)
    pub cipher_suites: Vec<u16>,
    /// extensionlist (including GREASE)
    pub extensions: Vec<u16>,
    /// 椭圆曲线list
    pub elliptic_curves: Vec<CurveID>,
    /// 椭圆曲线点format
    pub elliptic_curve_point_formats: Vec<u8>,
    /// signaturealgorithmlist
    pub signature_algorithms: Vec<u16>,
    /// Server Name Indication
    pub sni: Option<String>,
    /// Application-Layer Protocol Negotiation
    pub alpn: Option<String>,
}

impl ClientHelloSignature {
    /// Create a newsignature
    pub fn new() -> Self {
        Self {
            version: TlsVersion::V1_2, // default TLS 1.2
            cipher_suites: Vec::new(),
            extensions: Vec::new(),
            elliptic_curves: Vec::new(),
            elliptic_curve_point_formats: Vec::new(),
            signature_algorithms: Vec::new(),
            sni: None,
            alpn: None,
        }
    }

    /// Get过滤 GREASE back's cipher suites
    pub fn cipher_suites_without_grease(&self) -> Vec<u16> {
        filter_grease_values(&self.cipher_suites)
    }

    /// Get过滤 GREASE back's extensions
    pub fn extensions_without_grease(&self) -> Vec<u16> {
        filter_grease_values(&self.extensions)
    }

    /// Get过滤 GREASE back的signaturealgorithm
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

    /// 比较两个signaturewhether相似（忽略 GREASE value）
    ///
    /// # Parameters
    /// * `other` - 要比较的另ansignature
    ///
    /// # Returns
    /// * `true`  if signature相似（忽略 GREASE back相同），`false` otherwise
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

    /// Calculatesignature的hashvalue（ for 快速比较）
    /// use过滤 GREASE back的value
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
        sig2.cipher_suites = vec![0x0017, 0x2a2a]; // 不同 GREASE，but过滤back相同
        sig2.extensions = vec![0x0000, 0x0010];

        // 过滤 GREASE backshould相同
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
