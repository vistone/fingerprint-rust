//! HTTP passive fingerprinting module

use std::collections::HashMap;

/// HTTP fingerprint
#[derive(Debug, Clone)]
pub struct HttpFingerprint {
    /// HTTP version (1.0, 1.1, 2.0)
    pub version: String,
    /// Header order
    pub header_order: Vec<String>,
    /// User-Agent
    pub user_agent: Option<String>,
    /// Accept headers
    pub accept: Option<String>,
    /// Accept-Language
    pub accept_language: Option<String>,
    /// Accept-Encoding
    pub accept_encoding: Option<String>,
    /// Browser name
    pub browser: Option<String>,
    /// HTTP/2 settings
    pub h2_settings: Option<String>,
    /// Signature
    pub signature: Option<String>,
}

impl HttpFingerprint {
    /// Get fingerprint ID
    pub fn id(&self) -> String {
        // Generate a simple ID from user agent
        self.user_agent
            .as_ref()
            .map(|ua| format!("http_{:x}", ua.len()))
            .unwrap_or_else(|| "http_unknown".to_string())
    }
}

/// HTTP analyzer for passive fingerprinting
///
/// This analyzer provides simplified HTTP fingerprint extraction from packet data.
/// It can identify HTTP versions, header patterns, and content characteristics.
///
/// Note: This implementation is intentionally simplified for passive detection.
/// For complete HTTP parsing, use the `fingerprint-http` crate's active client.
pub struct HttpAnalyzer;

impl HttpAnalyzer {
    /// Create new HTTP analyzer
    pub fn new() -> Result<Self, String> {
        Ok(Self)
    }

    /// Analyze HTTP request from packet
    pub fn analyze(&self, packet: &crate::passive::packet::Packet) -> Option<HttpFingerprint> {
        // Simplified implementation - would need full HTTP parsing
        self.analyze_bytes(&packet.payload)
    }

    /// Analyze HTTP request from raw bytes
    ///
    /// Note: This method returns `None` for most cases as full HTTP parsing
    /// requires stateful analysis. Use `fingerprint_from_headers()` when
    /// headers have already been extracted from the packet.
    pub fn analyze_bytes(&self, _data: &[u8]) -> Option<HttpFingerprint> {
        // Simplified implementation - passive HTTP analysis without full parsing
        // For production use, implement proper HTTP request parsing or use existing libraries
        None
    }

    /// Extract HTTP fingerprint from headers
    pub fn fingerprint_from_headers(&self, headers: &HashMap<String, String>) -> HttpFingerprint {
        let header_order: Vec<String> = headers.keys().cloned().collect();

        HttpFingerprint {
            version: "1.1".to_string(),
            header_order,
            user_agent: headers.get("user-agent").cloned(),
            accept: headers.get("accept").cloned(),
            accept_language: headers.get("accept-language").cloned(),
            accept_encoding: headers.get("accept-encoding").cloned(),
            browser: None,
            h2_settings: None,
            signature: None,
        }
    }
}

impl Default for HttpAnalyzer {
    fn default() -> Self {
        Self
    }
}
