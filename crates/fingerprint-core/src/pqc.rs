//! Post-Quantum Cryptography (PQC) Detection
//!
//! Provides detection and fingerprinting of post-quantum cryptographic algorithms
//! including Kyber (key encapsulation), Dilithium (digital signatures), and hybrid modes.
//!
//! Based on NIST PQC standardization (2024) and modern TLS 1.3 extensions.

use serde::{Deserialize, Serialize};

/// Post-quantum algorithm types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PQCAlgorithm {
    /// CRYSTALS-Kyber (KEM) - NIST PQC Standard
    Kyber512,
    Kyber768,
    Kyber1024,

    /// CRYSTALS-Dilithium (Signature) - NIST PQC Standard
    Dilithium2,
    Dilithium3,
    Dilithium5,

    /// FALCON (Signature) - NIST PQC Standard
    Falcon512,
    Falcon1024,

    /// SPHINCS+ (Signature) - NIST PQC Standard
    SphincsShake128s,
    SphincsShake128f,
    SphincsShake256s,
    SphincsShake256f,

    /// Hybrid modes (PQC + Classical)
    HybridKyber768X25519,
    HybridKyber1024P384,
    HybridDilithium3RSA,
    HybridDilithium3ECDSA,
}

impl PQCAlgorithm {
    /// Get the TLS extension ID for PQC key share
    pub fn tls_extension_id(&self) -> Option<u16> {
        match self {
            // Proposed TLS extension IDs (subject to IANA assignment)
            Self::Kyber512 => Some(0xFE30),
            Self::Kyber768 => Some(0xFE31),
            Self::Kyber1024 => Some(0xFE32),
            Self::HybridKyber768X25519 => Some(0xFE34),
            Self::HybridKyber1024P384 => Some(0xFE35),
            _ => None,
        }
    }

    /// Get the algorithm name as used in TLS
    pub fn tls_name(&self) -> &'static str {
        match self {
            Self::Kyber512 => "kyber512",
            Self::Kyber768 => "kyber768",
            Self::Kyber1024 => "kyber1024",
            Self::Dilithium2 => "dilithium2",
            Self::Dilithium3 => "dilithium3",
            Self::Dilithium5 => "dilithium5",
            Self::Falcon512 => "falcon512",
            Self::Falcon1024 => "falcon1024",
            Self::SphincsShake128s => "sphincs_shake128s",
            Self::SphincsShake128f => "sphincs_shake128f",
            Self::SphincsShake256s => "sphincs_shake256s",
            Self::SphincsShake256f => "sphincs_shake256f",
            Self::HybridKyber768X25519 => "x25519_kyber768",
            Self::HybridKyber1024P384 => "p384_kyber1024",
            Self::HybridDilithium3RSA => "rsa_dilithium3",
            Self::HybridDilithium3ECDSA => "ecdsa_dilithium3",
        }
    }

    /// Check if this is a hybrid (classical + PQC) algorithm
    pub fn is_hybrid(&self) -> bool {
        matches!(
            self,
            Self::HybridKyber768X25519
                | Self::HybridKyber1024P384
                | Self::HybridDilithium3RSA
                | Self::HybridDilithium3ECDSA
        )
    }

    /// Get the security level in bits
    pub fn security_level(&self) -> u16 {
        match self {
            Self::Kyber512 | Self::Dilithium2 | Self::Falcon512 => 128,
            Self::Kyber768
            | Self::Dilithium3
            | Self::HybridKyber768X25519
            | Self::HybridDilithium3RSA
            | Self::HybridDilithium3ECDSA => 192,
            Self::Kyber1024 | Self::Dilithium5 | Self::Falcon1024 | Self::HybridKyber1024P384 => {
                256
            }
            Self::SphincsShake128s | Self::SphincsShake128f => 128,
            Self::SphincsShake256s | Self::SphincsShake256f => 256,
        }
    }
}

/// PQC capability detection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PQCCapabilities {
    /// Whether PQC is supported
    pub supported: bool,

    /// Detected PQC algorithms in TLS handshake
    pub algorithms: Vec<PQCAlgorithm>,

    /// Whether hybrid mode is preferred
    pub hybrid_mode: bool,

    /// TLS extensions indicating PQC support
    pub tls_extensions: Vec<u16>,

    /// Browser/client identifier (if known)
    pub client_hint: Option<String>,
}

impl PQCCapabilities {
    /// Create new PQC capabilities with no support
    pub fn none() -> Self {
        Self {
            supported: false,
            algorithms: Vec::new(),
            hybrid_mode: false,
            tls_extensions: Vec::new(),
            client_hint: None,
        }
    }

    /// Detect PQC support from TLS extensions
    pub fn from_tls_extensions(extensions: &[u16]) -> Self {
        let mut algorithms = Vec::new();
        let mut tls_extensions = Vec::new();

        // Check for known PQC extension IDs
        for &ext_id in extensions {
            if let Some(algo) = Self::extension_to_algorithm(ext_id) {
                algorithms.push(algo);
                tls_extensions.push(ext_id);
            }
        }

        let hybrid_mode = algorithms.iter().any(|a| a.is_hybrid());

        Self {
            supported: !algorithms.is_empty(),
            algorithms,
            hybrid_mode,
            tls_extensions,
            client_hint: None,
        }
    }

    /// Map TLS extension ID to PQC algorithm
    fn extension_to_algorithm(ext_id: u16) -> Option<PQCAlgorithm> {
        match ext_id {
            0xFE30 => Some(PQCAlgorithm::Kyber512),
            0xFE31 => Some(PQCAlgorithm::Kyber768),
            0xFE32 => Some(PQCAlgorithm::Kyber1024),
            0xFE34 => Some(PQCAlgorithm::HybridKyber768X25519),
            0xFE35 => Some(PQCAlgorithm::HybridKyber1024P384),
            _ => None,
        }
    }

    /// Get the maximum security level supported
    pub fn max_security_level(&self) -> u16 {
        self.algorithms
            .iter()
            .map(|a| a.security_level())
            .max()
            .unwrap_or(0)
    }

    /// Check if client prefers hybrid mode
    pub fn prefers_hybrid(&self) -> bool {
        if self.algorithms.is_empty() {
            return false;
        }

        let hybrid_count = self.algorithms.iter().filter(|a| a.is_hybrid()).count();
        let pure_pqc_count = self.algorithms.len() - hybrid_count;

        hybrid_count > pure_pqc_count
    }

    /// Generate a fingerprint string for PQC capabilities
    /// Format: pqc_{count}_{hybrid}_{max_level}_{algo_hash}
    pub fn fingerprint(&self) -> String {
        if !self.supported {
            return "pqc_none".to_string();
        }

        use sha2::{Digest, Sha256};

        let algo_string = self
            .algorithms
            .iter()
            .map(|a| a.tls_name())
            .collect::<Vec<_>>()
            .join(",");

        let mut hasher = Sha256::new();
        hasher.update(algo_string.as_bytes());
        let hash_result = hasher.finalize();
        let hash_hex = format!("{:x}", hash_result);

        format!(
            "pqc_{:02}_{}_{}_{:.8}",
            self.algorithms.len(),
            if self.hybrid_mode { "h" } else { "p" },
            self.max_security_level(),
            &hash_hex[0..8]
        )
    }
}

/// PQC browser support database (2025-2026)
pub struct PQCBrowserSupport;

impl PQCBrowserSupport {
    /// Check if a browser version likely supports PQC
    pub fn supports_pqc(browser: &str, version: u32) -> bool {
        match browser.to_lowercase().as_str() {
            "chrome" | "chromium" => version >= 116, // Chrome 116+ experimental PQC
            "firefox" => version >= 120,              // Firefox 120+ with flags
            "safari" => version >= 17,                // Safari 17+ limited support
            "edge" => version >= 116,                 // Edge follows Chrome
            _ => false,
        }
    }

    /// Get expected PQC algorithms for a browser
    pub fn expected_algorithms(browser: &str, version: u32) -> Vec<PQCAlgorithm> {
        if !Self::supports_pqc(browser, version) {
            return Vec::new();
        }

        match browser.to_lowercase().as_str() {
            "chrome" | "chromium" | "edge" => {
                vec![
                    PQCAlgorithm::HybridKyber768X25519,
                    PQCAlgorithm::Kyber768,
                ]
            }
            "firefox" => {
                vec![PQCAlgorithm::Kyber768, PQCAlgorithm::Kyber1024]
            }
            "safari" => {
                vec![PQCAlgorithm::HybridKyber768X25519]
            }
            _ => Vec::new(),
        }
    }

    /// Detect anomalies in PQC support
    /// Returns true if the PQC capabilities don't match expected browser behavior
    pub fn detect_anomaly(
        browser: &str,
        version: u32,
        capabilities: &PQCCapabilities,
    ) -> Option<String> {
        let expected = Self::expected_algorithms(browser, version);
        let should_support = Self::supports_pqc(browser, version);

        if should_support && !capabilities.supported {
            return Some(format!(
                "{} v{} should support PQC but none detected",
                browser, version
            ));
        }

        if !should_support && capabilities.supported {
            return Some(format!(
                "{} v{} should not support PQC but algorithms detected",
                browser, version
            ));
        }

        if should_support && capabilities.supported {
            // Check if algorithms match expectations
            let unexpected: Vec<_> = capabilities
                .algorithms
                .iter()
                .filter(|a| !expected.contains(a))
                .collect();

            if !unexpected.is_empty() {
                return Some(format!(
                    "{} v{} has unexpected PQC algorithms",
                    browser, version
                ));
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pqc_algorithm_properties() {
        let kyber768 = PQCAlgorithm::Kyber768;
        assert_eq!(kyber768.security_level(), 192);
        assert!(!kyber768.is_hybrid());
        assert_eq!(kyber768.tls_name(), "kyber768");

        let hybrid = PQCAlgorithm::HybridKyber768X25519;
        assert!(hybrid.is_hybrid());
        assert_eq!(hybrid.security_level(), 192);
    }

    #[test]
    fn test_pqc_capabilities_from_extensions() {
        let extensions = vec![0xFE31, 0xFE34]; // Kyber768, HybridKyber768X25519
        let caps = PQCCapabilities::from_tls_extensions(&extensions);

        assert!(caps.supported);
        assert_eq!(caps.algorithms.len(), 2);
        assert!(caps.hybrid_mode);
        assert_eq!(caps.max_security_level(), 192);
    }

    #[test]
    fn test_pqc_capabilities_none() {
        let caps = PQCCapabilities::none();
        assert!(!caps.supported);
        assert_eq!(caps.algorithms.len(), 0);
        assert!(!caps.hybrid_mode);
    }

    #[test]
    fn test_pqc_fingerprint() {
        let extensions = vec![0xFE31];
        let caps = PQCCapabilities::from_tls_extensions(&extensions);

        let fp = caps.fingerprint();
        assert!(fp.starts_with("pqc_01_"));
        assert!(fp.contains("192"));
    }

    #[test]
    fn test_browser_support() {
        assert!(PQCBrowserSupport::supports_pqc("chrome", 120));
        assert!(!PQCBrowserSupport::supports_pqc("chrome", 100));
        assert!(PQCBrowserSupport::supports_pqc("firefox", 125));
    }

    #[test]
    fn test_expected_algorithms() {
        let chrome_algos = PQCBrowserSupport::expected_algorithms("chrome", 120);
        assert!(!chrome_algos.is_empty());
        assert!(chrome_algos.contains(&PQCAlgorithm::HybridKyber768X25519));
    }

    #[test]
    fn test_anomaly_detection() {
        let caps = PQCCapabilities::none();
        let anomaly = PQCBrowserSupport::detect_anomaly("chrome", 120, &caps);
        assert!(anomaly.is_some());
        assert!(anomaly.unwrap().contains("should support PQC"));
    }

    #[test]
    fn test_prefers_hybrid() {
        let caps = PQCCapabilities {
            supported: true,
            algorithms: vec![
                PQCAlgorithm::HybridKyber768X25519,
                PQCAlgorithm::HybridKyber1024P384,
                PQCAlgorithm::Kyber512,
            ],
            hybrid_mode: true,
            tls_extensions: vec![],
            client_hint: None,
        };

        assert!(caps.prefers_hybrid());
    }
}
