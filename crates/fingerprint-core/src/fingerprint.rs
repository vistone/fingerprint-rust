//! Fingerprint core abstraction
//!
//! Define unified fingerprint abstractions, support TLS, HTTP, TCP and other multiple fingerprint types.

use crate::metadata::FingerprintMetadata;

/// Fingerprint type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FingerprintType {
    /// TLS fingerprint
    Tls,
    /// HTTP fingerprint
    Http,
    /// TCP fingerprint
    Tcp,
}

impl FingerprintType {
    /// Convert to string
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Tls => "tls",
            Self::Http => "http",
            Self::Tcp => "tcp",
        }
    }
}

impl std::fmt::Display for FingerprintType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Fingerprint abstractions trait
///
/// All fingerprint types (TLS, HTTP, TCP) should implement this trait
pub trait Fingerprint: Send + Sync {
    /// Get fingerprint type
    fn fingerprint_type(&self) -> FingerprintType;

    /// Get fingerprint unique identifier symbol (usually is hash value)
    fn id(&self) -> String;

    /// Get fingerprint metadata
    fn metadata(&self) -> &FingerprintMetadata;

    /// Get fingerprint metadata (mutable reference)
    fn metadata_mut(&mut self) -> &mut FingerprintMetadata;

    /// Calculate fingerprint hash value (for fast compare)
    fn hash(&self) -> u64;

    /// Compare two fingerprints whether similar
    fn similar_to(&self, other: &dyn Fingerprint) -> bool;

    /// Get fingerprint string represent (for debug and log)
    fn to_string(&self) -> String;
}

/// Fingerprint compare result
#[derive(Debug, Clone, PartialEq)]
pub struct FingerprintComparison {
    /// Similar degree minute count (0.0 - 1.0)
    pub similarity: f64,

    /// Whether match
    pub matched: bool,

    /// Match field
    pub matched_fields: Vec<String>,

    /// Does not match field
    pub unmatched_fields: Vec<String>,
}

impl FingerprintComparison {
    /// Create a new compare result
    pub fn new(similarity: f64, matched: bool) -> Self {
        Self {
            similarity,
            matched,
            matched_fields: Vec::new(),
            unmatched_fields: Vec::new(),
        }
    }

    /// Create completely match result
    pub fn perfect_match() -> Self {
        Self {
            similarity: 1.0,
            matched: true,
            matched_fields: Vec::new(),
            unmatched_fields: Vec::new(),
        }
    }

    /// Create completely does not match result
    pub fn no_match() -> Self {
        Self {
            similarity: 0.0,
            matched: false,
            matched_fields: Vec::new(),
            unmatched_fields: Vec::new(),
        }
    }
}

/// Fingerprint compareer
pub struct FingerprintComparator;

impl FingerprintComparator {
    /// compare two fingerprint using field-level similarity instead of unreliable hash
    pub fn compare(f1: &dyn Fingerprint, f2: &dyn Fingerprint) -> FingerprintComparison {
        // type must be same
        if f1.fingerprint_type() != f2.fingerprint_type() {
            return FingerprintComparison::no_match();
        }

        // use similar_to method for exact match
        if f1.similar_to(f2) {
            return FingerprintComparison::perfect_match();
        }

        // Calculate field-level similarity for softer matching
        let meta1 = f1.metadata();
        let meta2 = f2.metadata();

        let mut matched_fields = Vec::new();
        let mut unmatched_fields = Vec::new();
        let mut total_fields = 0;
        let mut matching_count = 0;

        // Browser type field comparison
        total_fields += 1;
        if meta1.browser_type == meta2.browser_type {
            matching_count += 1;
            matched_fields.push("browser_type".to_string());
        } else {
            unmatched_fields.push("browser_type".to_string());
        }

        // Operating system type field comparison
        total_fields += 1;
        if meta1.os_type == meta2.os_type {
            matching_count += 1;
            matched_fields.push("os_type".to_string());
        } else {
            unmatched_fields.push("os_type".to_string());
        }

        // Confidence field comparison (within 0.1 threshold)
        total_fields += 1;
        if (meta1.confidence - meta2.confidence).abs() < 0.1 {
            matching_count += 1;
            matched_fields.push("confidence".to_string());
        } else {
            unmatched_fields.push("confidence".to_string());
        }

        // Tags field comparison (any overlap counts as match)
        total_fields += 1;
        let has_common_tags = meta1.tags.iter().any(|tag| meta2.tags.contains(tag));
        if has_common_tags || (meta1.tags.is_empty() && meta2.tags.is_empty()) {
            matching_count += 1;
            matched_fields.push("tags".to_string());
        } else {
            unmatched_fields.push("tags".to_string());
        }

        // Calculate similarity based on matching fields
        let similarity = matching_count as f64 / total_fields as f64;

        // Use more conservative matching threshold (0.6 instead of 0.8)
        // Require at least 60% field match to be considered similar
        FingerprintComparison {
            similarity,
            matched: similarity >= 0.6,
            matched_fields,
            unmatched_fields,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fingerprint_type() {
        assert_eq!(FingerprintType::Tls.as_str(), "tls");
        assert_eq!(FingerprintType::Http.as_str(), "http");
        assert_eq!(FingerprintType::Tcp.as_str(), "tcp");
    }

    #[test]
    fn test_fingerprint_comparison() {
        let perfect = FingerprintComparison::perfect_match();
        assert_eq!(perfect.similarity, 1.0);
        assert!(perfect.matched);

        let no_match = FingerprintComparison::no_match();
        assert_eq!(no_match.similarity, 0.0);
        assert!(!no_match.matched);
    }
}
