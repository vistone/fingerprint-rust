//! JARM: Active TLS Server Fingerprinting
//!
//! JARM is an active TLS server fingerprinting technique developed by Salesforce.
//! It sends 10 specific TLS Client Hello packets to a server and analyzes the responses
//! to generate a unique 62-character fingerprint.
//!
//! ## Algorithm Overview
//!
//! 1. Send 10 TLS Client Hello probes with different configurations:
//!    - TLS 1.2/1.3 versions
//!    - Different cipher suites
//!    - Different TLS extensions
//!    - ALPN variations
//! 2. Parse Server Hello responses (or capture errors/timeouts)
//! 3. Extract key features from each response:
//!    - TLS version
//!    - Cipher suite selected
//!    - TLS extensions present
//! 4. Hash and combine features into 62-character fingerprint
//!
//! ## Format
//!
//! JARM fingerprint: 62 hexadecimal characters
//! - First 30 chars: Hashed cipher and version info from first 5 probes
//! - Next 30 chars: Hashed cipher and version info from last 5 probes  
//! - Last 2 chars: Hashed extension info from all probes
//!
//! ## References
//!
//! - Salesforce Engineering Blog: "JARM: TLS Active Transport Fingerprinting"
//! - GitHub: salesforce/jarm

use crate::version::TlsVersion;
use sha2::{Digest, Sha256};
use std::fmt;

/// JARM probe configuration
///
/// Represents one of the 10 TLS Client Hello probes sent to the target server.
#[derive(Debug, Clone)]
pub struct JarmProbe {
    /// TLS version to advertise in ClientHello
    pub tls_version: TlsVersion,
    /// List of cipher suites to include
    pub cipher_suites: Vec<u16>,
    /// TLS extensions to include
    pub extensions: Vec<u16>,
    /// ALPN protocols (if any)
    pub alpn_protocols: Vec<String>,
    /// Use GREASE values
    pub use_grease: bool,
}

impl JarmProbe {
    /// Create a new JARM probe with specified configuration
    pub fn new(
        tls_version: TlsVersion,
        cipher_suites: Vec<u16>,
        extensions: Vec<u16>,
        alpn_protocols: Vec<String>,
        use_grease: bool,
    ) -> Self {
        Self {
            tls_version,
            cipher_suites,
            extensions,
            alpn_protocols,
            use_grease,
        }
    }

    /// Create the 10 standard JARM probes
    pub fn standard_probes() -> Vec<Self> {
        vec![
            // Probe 1: TLS 1.2, ALL ciphers, ALL extensions, ALPN
            Self::new(
                TlsVersion::V1_2,
                Self::all_ciphers_tls12(),
                Self::all_extensions(),
                vec!["h2".to_string(), "http/1.1".to_string()],
                false,
            ),
            // Probe 2: TLS 1.2, ALL ciphers, ALL extensions, No ALPN
            Self::new(
                TlsVersion::V1_2,
                Self::all_ciphers_tls12(),
                Self::all_extensions(),
                vec![],
                false,
            ),
            // Probe 3: TLS 1.2, RARE ciphers, ALL extensions, ALPN
            Self::new(
                TlsVersion::V1_2,
                Self::rare_ciphers(),
                Self::all_extensions(),
                vec!["h2".to_string()],
                false,
            ),
            // Probe 4: TLS 1.2, RARE ciphers, ALL extensions, No ALPN
            Self::new(
                TlsVersion::V1_2,
                Self::rare_ciphers(),
                Self::all_extensions(),
                vec![],
                false,
            ),
            // Probe 5: TLS 1.1, ALL ciphers, No extensions
            Self::new(
                TlsVersion::V1_1,
                Self::all_ciphers_tls11(),
                vec![],
                vec![],
                false,
            ),
            // Probe 6: TLS 1.3, ALL ciphers, ALL extensions, ALPN
            Self::new(
                TlsVersion::V1_3,
                Self::all_ciphers_tls13(),
                Self::all_extensions(),
                vec!["h2".to_string(), "http/1.1".to_string()],
                true,
            ),
            // Probe 7: TLS 1.3, ALL ciphers, ALL extensions, No ALPN
            Self::new(
                TlsVersion::V1_3,
                Self::all_ciphers_tls13(),
                Self::all_extensions(),
                vec![],
                true,
            ),
            // Probe 8: TLS 1.3, RARE ciphers, ALL extensions, ALPN
            Self::new(
                TlsVersion::V1_3,
                Self::rare_ciphers_tls13(),
                Self::all_extensions(),
                vec!["h2".to_string()],
                true,
            ),
            // Probe 9: TLS 1.3, RARE ciphers, ALL extensions, No ALPN
            Self::new(
                TlsVersion::V1_3,
                Self::rare_ciphers_tls13(),
                Self::all_extensions(),
                vec![],
                true,
            ),
            // Probe 10: TLS 1.2, ALL ciphers, ALL extensions, ALPN (1.3 advertised)
            Self::new(
                TlsVersion::V1_2,
                Self::all_ciphers_tls12(),
                Self::all_extensions_with_supported_versions(),
                vec!["h2".to_string()],
                false,
            ),
        ]
    }

    /// Common TLS 1.2 cipher suites
    fn all_ciphers_tls12() -> Vec<u16> {
        vec![
            0x1301, 0x1302, 0x1303, 0x1304, 0x1305, // TLS 1.3 ciphers
            0xc02c, 0xc030, 0x009f, 0xcca9, 0xcca8, // ECDHE-RSA/ECDSA with AEAD
            0xccaa, 0xc02b, 0xc02f, 0x009e, 0xc024, // More ECDHE
            0xc028, 0x006b, 0xc023, 0xc027, 0x0067, // DHE variants
            0xc00a, 0xc014, 0x0039, 0xc009, 0xc013, // Older ECDHE
            0x0033, 0x009d, 0x009c, 0x003d, 0x003c, // Other AES-GCM
        ]
    }

    /// Common TLS 1.1 cipher suites (legacy)
    fn all_ciphers_tls11() -> Vec<u16> {
        vec![
            0xc014, 0xc00a, 0x0039, 0x0038, 0x0035, 0x0016, 0x0013, 0x000a, 0x0033, 0x0032,
            0x002f, 0x000d, 0x0005, 0x0004,
        ]
    }

    /// TLS 1.3 cipher suites
    fn all_ciphers_tls13() -> Vec<u16> {
        vec![
            0x1301, // TLS_AES_128_GCM_SHA256
            0x1302, // TLS_AES_256_GCM_SHA384
            0x1303, // TLS_CHACHA20_POLY1305_SHA256
            0x1304, // TLS_AES_128_CCM_SHA256
            0x1305, // TLS_AES_128_CCM_8_SHA256
        ]
    }

    /// Rare/uncommon cipher suites for TLS 1.2
    fn rare_ciphers() -> Vec<u16> {
        vec![
            0x009f, 0x00a3, 0x00a7, 0xc0ac, 0xc0ad, 0xc0ae, 0xc0af, 0x00be, 0x00c4, 0x0088,
        ]
    }

    /// Rare/uncommon cipher suites for TLS 1.3
    fn rare_ciphers_tls13() -> Vec<u16> {
        vec![
            0x1304, // TLS_AES_128_CCM_SHA256
            0x1305, // TLS_AES_128_CCM_8_SHA256
        ]
    }

    /// Standard TLS extensions
    fn all_extensions() -> Vec<u16> {
        vec![
            0x0000, // server_name
            0x000b, // ec_point_formats
            0x000a, // supported_groups
            0x0023, // session_ticket
            0x0016, // encrypt_then_mac
            0x0017, // extended_master_secret
            0x000d, // signature_algorithms
            0x0005, // status_request
            0x0012, // signed_certificate_timestamp
        ]
    }

    /// Extensions including supported_versions for TLS 1.3 downgrade detection
    fn all_extensions_with_supported_versions() -> Vec<u16> {
        let mut exts = Self::all_extensions();
        exts.push(0x002b); // supported_versions
        exts
    }
}

/// JARM Server Response
///
/// Represents the server's response to a JARM probe.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct JarmResponse {
    /// Whether the server responded (vs timeout/error)
    pub responded: bool,
    /// TLS version selected by server
    pub tls_version: Option<TlsVersion>,
    /// Cipher suite selected by server
    pub cipher_suite: Option<u16>,
    /// Extensions returned by server
    pub extensions: Vec<u16>,
    /// Server used ALPN
    pub has_alpn: bool,
}

impl JarmResponse {
    /// Create a response indicating no response (timeout/error)
    pub fn no_response() -> Self {
        Self {
            responded: false,
            tls_version: None,
            cipher_suite: None,
            extensions: vec![],
            has_alpn: false,
        }
    }

    /// Create a response from server hello data
    pub fn from_server_hello(
        tls_version: TlsVersion,
        cipher_suite: u16,
        extensions: Vec<u16>,
        has_alpn: bool,
    ) -> Self {
        Self {
            responded: true,
            tls_version: Some(tls_version),
            cipher_suite: Some(cipher_suite),
            extensions,
            has_alpn,
        }
    }

    /// Convert response to JARM component string (for hashing)
    fn to_component_string(&self) -> String {
        if !self.responded {
            return "|||".to_string();
        }

        let version_str = self
            .tls_version
            .map(|v| format!("{:04x}", v.to_u16()))
            .unwrap_or_else(|| "0000".to_string());

        let cipher_str = self
            .cipher_suite
            .map(|c| format!("{:04x}", c))
            .unwrap_or_else(|| "0000".to_string());

        let ext_count = format!("{:02x}", self.extensions.len().min(99));

        let alpn_str = if self.has_alpn { "1" } else { "0" };

        format!("{}|{}|{}|{}", version_str, cipher_str, ext_count, alpn_str)
    }
}

/// JARM Fingerprint
///
/// Represents a complete JARM fingerprint generated from 10 probe responses.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Jarm {
    /// The 62-character JARM fingerprint
    pub fingerprint: String,
    /// Individual probe responses (for debugging)
    pub responses: Vec<JarmResponse>,
}

impl Jarm {
    /// Generate JARM fingerprint from probe responses
    ///
    /// # Arguments
    ///
    /// * `responses` - Vector of 10 JarmResponse objects (one per probe)
    ///
    /// # Returns
    ///
    /// JARM fingerprint or None if invalid input
    pub fn from_responses(responses: Vec<JarmResponse>) -> Option<Self> {
        if responses.len() != 10 {
            return None;
        }

        let fingerprint = Self::compute_fingerprint(&responses);

        Some(Self {
            fingerprint,
            responses,
        })
    }

    /// Compute the 62-character JARM fingerprint
    fn compute_fingerprint(responses: &[JarmResponse]) -> String {
        let mut result = String::with_capacity(62);

        // Process first 5 probes (30 characters)
        for i in 0..5 {
            let component = responses[i].to_component_string();
            let hash = Self::hash_component(&component);
            result.push_str(&hash[..6]); // 6 chars per probe
        }

        // Process last 5 probes (30 characters)
        for i in 5..10 {
            let component = responses[i].to_component_string();
            let hash = Self::hash_component(&component);
            result.push_str(&hash[..6]); // 6 chars per probe
        }

        // Process all extensions (2 characters)
        let ext_component = Self::compute_extension_component(responses);
        let ext_hash = Self::hash_component(&ext_component);
        result.push_str(&ext_hash[..2]);

        result
    }

    /// Hash a component string using SHA256
    fn hash_component(component: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(component.as_bytes());
        let result = hasher.finalize();
        hex::encode(result)
    }

    /// Compute extension aggregation component
    fn compute_extension_component(responses: &[JarmResponse]) -> String {
        let mut all_extensions = Vec::new();
        for response in responses {
            if response.responded {
                for ext in &response.extensions {
                    all_extensions.push(format!("{:04x}", ext));
                }
            }
        }
        all_extensions.join(",")
    }

    /// Get the fingerprint string
    pub fn fingerprint_string(&self) -> &str {
        &self.fingerprint
    }

    /// Detect server type based on JARM fingerprint
    pub fn detect_server_type(&self) -> Option<String> {
        // Known JARM fingerprints for common servers
        match self.fingerprint.as_str() {
            // Cloudflare
            s if s.starts_with("27d40d40d29d40d1dc42d43d00041d") => {
                Some("Cloudflare".to_string())
            }
            // AWS ELB
            s if s.starts_with("27d3ed3ed0003ed1dc42d43d00041d") => {
                Some("AWS ELB".to_string())
            }
            // nginx
            s if s.starts_with("27d27d27d29d27d1dc42d43d00041d") => Some("nginx".to_string()),
            // Apache
            s if s.starts_with("27d27d00029d27d1dc41d43d00041d") => Some("Apache".to_string()),
            // IIS
            s if s.starts_with("29d29d00029d29d1dc42d43d00041d") => {
                Some("Microsoft IIS".to_string())
            }
            _ => None,
        }
    }
}

impl fmt::Display for Jarm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.fingerprint)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jarm_probe_creation() {
        let probe = JarmProbe::new(
            TlsVersion::V1_2,
            vec![0xc02c, 0xc030],
            vec![0x0000, 0x000b],
            vec!["h2".to_string()],
            false,
        );

        assert_eq!(probe.tls_version, TlsVersion::V1_2);
        assert_eq!(probe.cipher_suites.len(), 2);
        assert_eq!(probe.extensions.len(), 2);
        assert_eq!(probe.alpn_protocols.len(), 1);
        assert!(!probe.use_grease);
    }

    #[test]
    fn test_jarm_standard_probes() {
        let probes = JarmProbe::standard_probes();
        assert_eq!(probes.len(), 10);

        // Verify probe 1 (TLS 1.2 with ALPN)
        assert_eq!(probes[0].tls_version, TlsVersion::V1_2);
        assert!(!probes[0].alpn_protocols.is_empty());

        // Verify probe 6 (TLS 1.3 with ALPN and GREASE)
        assert_eq!(probes[5].tls_version, TlsVersion::V1_3);
        assert!(probes[5].use_grease);
        assert!(!probes[5].alpn_protocols.is_empty());
    }

    #[test]
    fn test_jarm_response_no_response() {
        let response = JarmResponse::no_response();
        assert!(!response.responded);
        assert_eq!(response.to_component_string(), "|||");
    }

    #[test]
    fn test_jarm_response_from_server_hello() {
        let response = JarmResponse::from_server_hello(
            TlsVersion::V1_3,
            0x1301,
            vec![0x0000, 0x000b, 0x002b],
            true,
        );

        assert!(response.responded);
        assert_eq!(response.tls_version, Some(TlsVersion::V1_3));
        assert_eq!(response.cipher_suite, Some(0x1301));
        assert_eq!(response.extensions.len(), 3);
        assert!(response.has_alpn);

        let component = response.to_component_string();
        assert!(component.contains("0304")); // TLS 1.3 version
        assert!(component.contains("1301")); // Cipher
        assert!(component.contains("03")); // 3 extensions
        assert!(component.contains("1")); // ALPN present
    }

    #[test]
    fn test_jarm_fingerprint_generation() {
        // Create mock responses
        let mut responses = Vec::new();
        for i in 0..10 {
            if i % 2 == 0 {
                responses.push(JarmResponse::from_server_hello(
                    TlsVersion::V1_3,
                    0x1301,
                    vec![0x0000, 0x002b],
                    true,
                ));
            } else {
                responses.push(JarmResponse::no_response());
            }
        }

        let jarm = Jarm::from_responses(responses);
        assert!(jarm.is_some());

        let jarm = jarm.unwrap();
        assert_eq!(jarm.fingerprint.len(), 62);
        assert!(jarm.fingerprint.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_jarm_fingerprint_invalid_length() {
        let responses = vec![JarmResponse::no_response()]; // Only 1 response
        let jarm = Jarm::from_responses(responses);
        assert!(jarm.is_none());
    }

    #[test]
    fn test_jarm_display() {
        let responses = (0..10)
            .map(|_| JarmResponse::no_response())
            .collect();
        let jarm = Jarm::from_responses(responses).unwrap();

        let display = format!("{}", jarm);
        assert_eq!(display, jarm.fingerprint);
    }

    #[test]
    fn test_jarm_server_detection_cloudflare() {
        let mut responses = Vec::new();
        for _ in 0..10 {
            responses.push(JarmResponse::from_server_hello(
                TlsVersion::V1_3,
                0x1301,
                vec![0x0000, 0x002b],
                true,
            ));
        }

        // Create a fingerprint starting with Cloudflare pattern
        let mut jarm = Jarm::from_responses(responses).unwrap();
        jarm.fingerprint = "27d40d40d29d40d1dc42d43d00041d4689ee210389f4f6b4b5b1b93f92252d".to_string();

        let server_type = jarm.detect_server_type();
        assert_eq!(server_type, Some("Cloudflare".to_string()));
    }

    #[test]
    fn test_jarm_server_detection_unknown() {
        let responses = (0..10)
            .map(|_| JarmResponse::no_response())
            .collect();
        let jarm = Jarm::from_responses(responses).unwrap();

        let server_type = jarm.detect_server_type();
        assert!(server_type.is_none());
    }

    #[test]
    fn test_jarm_hash_consistency() {
        let component = "0304|1301|03|1";
        let hash1 = Jarm::hash_component(component);
        let hash2 = Jarm::hash_component(component);
        assert_eq!(hash1, hash2);
        assert_eq!(hash1.len(), 64); // SHA256 hex output
    }
}
