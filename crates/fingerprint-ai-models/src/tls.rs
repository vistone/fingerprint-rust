//! TLS fingerprinting for AI provider detection

use crate::AiProvider;
use once_cell::sync::Lazy;
use std::collections::HashMap;

/// Known JA3 hashes for AI providers
/// Note: These are example hashes and would need to be updated with real data
static KNOWN_JA3_HASHES: Lazy<HashMap<&'static str, AiProvider>> = Lazy::new(|| {
    let mut m = HashMap::new();
    
    // OpenAI (uses Fastly CDN)
    // These are example hashes - real implementation would need actual fingerprints
    m.insert("771,49195-49199-49196-49200-52393-52392-49171-49172-156-157-47-53,0-23-65281-10-11-35-16-5-13-18-51-45-43-27-21,29-23-24-25,0", AiProvider::OpenAI);
    
    // Google (Google Cloud infrastructure)
    m.insert("771,4865-4866-4867-49195-49199-49196-49200-52393-52392-49171-49172-156-157-47-53,0-23-65281-10-11-35-16-5-13-18-51-45-43-27-17513,29-23-24,0", AiProvider::GoogleGemini);
    
    // AWS (for Bedrock)
    m.insert("771,4865-4866-4867-49195-49199-52393-52392-49196-49200-49171-49172-156-157-47-53,0-23-65281-10-11-35-16-5-13-18-51-45-43-27-21,29-23-24-25,0", AiProvider::AwsBedrock);
    
    m
});

/// Detect AI provider from JA3 fingerprint hash
pub fn detect_provider_from_ja3(ja3_hash: &str) -> Option<AiProvider> {
    // Direct hash lookup
    if let Some(provider) = KNOWN_JA3_HASHES.get(ja3_hash) {
        return Some(provider.clone());
    }

    // Partial matching or pattern analysis could be added here
    // For example, analyzing cipher suites or extension patterns
    
    None
}

/// Analyze TLS version from JA3
pub fn extract_tls_version(ja3_hash: &str) -> Option<String> {
    // JA3 format: SSLVersion,Ciphers,Extensions,EllipticCurves,EllipticCurvePointFormats
    let parts: Vec<&str> = ja3_hash.split(',').collect();
    
    if parts.is_empty() {
        return None;
    }

    let version_code = parts[0];
    
    match version_code {
        "771" => Some("TLS 1.2".to_string()),
        "772" => Some("TLS 1.3".to_string()),
        "769" => Some("TLS 1.0".to_string()),
        "770" => Some("TLS 1.1".to_string()),
        _ => Some(format!("Unknown ({})", version_code)),
    }
}

/// Analyze cipher suites from JA3
pub fn analyze_cipher_suites(ja3_hash: &str) -> Vec<u16> {
    // JA3 format: SSLVersion,Ciphers,Extensions,EllipticCurves,EllipticCurvePointFormats
    let parts: Vec<&str> = ja3_hash.split(',').collect();
    
    if parts.len() < 2 {
        return Vec::new();
    }

    parts[1]
        .split('-')
        .filter_map(|s| s.parse::<u16>().ok())
        .collect()
}

/// Check if TLS configuration suggests bot/automation
pub fn is_likely_automated(ja3_hash: &str) -> bool {
    let cipher_suites = analyze_cipher_suites(ja3_hash);
    
    // Very few cipher suites might indicate a simple client
    if cipher_suites.len() < 5 {
        return true;
    }

    // Check for uncommon cipher suite orders
    // Real browsers have specific, well-known cipher suite preferences
    // This is a simplified check
    
    false
}

/// TLS fingerprint characteristics
#[derive(Debug, Clone)]
pub struct TlsCharacteristics {
    /// TLS version
    pub version: Option<String>,
    
    /// Number of cipher suites
    pub cipher_count: usize,
    
    /// Number of extensions
    pub extension_count: usize,
    
    /// Whether this looks like a standard browser
    pub is_standard_browser: bool,
    
    /// Whether this looks automated
    pub is_likely_automated: bool,
}

impl TlsCharacteristics {
    /// Analyze JA3 hash and extract characteristics
    pub fn from_ja3(ja3_hash: &str) -> Self {
        let version = extract_tls_version(ja3_hash);
        let cipher_suites = analyze_cipher_suites(ja3_hash);
        let cipher_count = cipher_suites.len();
        
        // Parse extensions count
        let parts: Vec<&str> = ja3_hash.split(',').collect();
        let extension_count = if parts.len() > 2 {
            parts[2].split('-').count()
        } else {
            0
        };

        // Heuristics for standard browser
        let is_standard_browser = cipher_count >= 10 && cipher_count <= 30 && extension_count >= 8;
        
        let is_likely_automated = is_likely_automated(ja3_hash);

        Self {
            version,
            cipher_count,
            extension_count,
            is_standard_browser,
            is_likely_automated,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_tls_version() {
        let ja3 = "771,49195-49199,0-23-65281,29-23-24,0";
        let version = extract_tls_version(ja3);
        assert_eq!(version, Some("TLS 1.2".to_string()));

        let ja3_13 = "772,4865-4866,0-23,29-23,0";
        let version_13 = extract_tls_version(ja3_13);
        assert_eq!(version_13, Some("TLS 1.3".to_string()));
    }

    #[test]
    fn test_analyze_cipher_suites() {
        let ja3 = "771,49195-49199-49196,0-23-65281,29-23-24,0";
        let ciphers = analyze_cipher_suites(ja3);
        
        assert_eq!(ciphers.len(), 3);
        assert_eq!(ciphers[0], 49195);
        assert_eq!(ciphers[1], 49199);
        assert_eq!(ciphers[2], 49196);
    }

    #[test]
    fn test_tls_characteristics() {
        let ja3 = "771,49195-49199-49196-49200-52393-52392-49171-49172-156-157-47-53,0-23-65281-10-11-35-16-5-13-18-51-45-43-27-21,29-23-24-25,0";
        let chars = TlsCharacteristics::from_ja3(ja3);
        
        assert_eq!(chars.version, Some("TLS 1.2".to_string()));
        assert_eq!(chars.cipher_count, 12);
        assert!(chars.extension_count > 10);
        assert!(chars.is_standard_browser);
    }

    #[test]
    fn test_automated_detection() {
        // Few ciphers = likely automated
        let ja3_simple = "771,49195-49199,0-23,29,0";
        assert!(is_likely_automated(ja3_simple));

        // Many ciphers = likely real browser
        let ja3_complex = "771,49195-49199-49196-49200-52393-52392-49171-49172-156-157-47-53,0-23-65281-10-11-35,29-23-24,0";
        assert!(!is_likely_automated(ja3_complex));
    }
}
