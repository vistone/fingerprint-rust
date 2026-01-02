//! fingerprintcore抽象
//!
//! define统一的fingerprint抽象，support TLS、HTTP、TCP 等多种fingerprinttype。

use crate::metadata::FingerprintMetadata;

/// fingerprinttype
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
    /// convert tostring
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

/// fingerprint抽象 trait
///
/// allfingerprinttype（TLS、HTTP、TCP）都shouldimplement这个 trait
pub trait Fingerprint: Send + Sync {
    /// Getfingerprinttype
    fn fingerprint_type(&self) -> FingerprintType;

    /// Getfingerprint的唯一identifier符（通常是hashvalue）
    fn id(&self) -> String;

    /// Getfingerprint的metadata
    fn metadata(&self) -> &FingerprintMetadata;

    /// Getfingerprint的metadata（可变reference）
    fn metadata_mut(&mut self) -> &mut FingerprintMetadata;

    /// Calculatefingerprint的hashvalue（ for 快速比较）
    fn hash(&self) -> u64;

    /// 比较两个fingerprintwhether相似
    fn similar_to(&self, other: &dyn Fingerprint) -> bool;

    /// Getfingerprint的stringrepresent（ for debug and 日志）
    fn to_string(&self) -> String;
}

/// fingerprint比较result
#[derive(Debug, Clone, PartialEq)]
pub struct FingerprintComparison {
    /// 相似度分count (0.0 - 1.0)
    pub similarity: f64,

    /// whethermatch
    pub matched: bool,

    /// match的field
    pub matched_fields: Vec<String>,

    /// does not match的field
    pub unmatched_fields: Vec<String>,
}

impl FingerprintComparison {
    /// Create a new比较result
    pub fn new(similarity: f64, matched: bool) -> Self {
        Self {
            similarity,
            matched,
            matched_fields: Vec::new(),
            unmatched_fields: Vec::new(),
        }
    }

    /// Create完全match的result
    pub fn perfect_match() -> Self {
        Self {
            similarity: 1.0,
            matched: true,
            matched_fields: Vec::new(),
            unmatched_fields: Vec::new(),
        }
    }

    /// Create完全does not match的result
    pub fn no_match() -> Self {
        Self {
            similarity: 0.0,
            matched: false,
            matched_fields: Vec::new(),
            unmatched_fields: Vec::new(),
        }
    }
}

/// fingerprint比较器
pub struct FingerprintComparator;

impl FingerprintComparator {
    /// 比较两个fingerprint
    pub fn compare(f1: &dyn Fingerprint, f2: &dyn Fingerprint) -> FingerprintComparison {
        // typemustsame
        if f1.fingerprint_type() != f2.fingerprint_type() {
            return FingerprintComparison::no_match();
        }

        // use similar_to method进行比较
        if f1.similar_to(f2) {
            FingerprintComparison::perfect_match()
        } else {
            // Calculate相似度（based onhashvalue）
            let h1 = f1.hash();
            let h2 = f2.hash();

            // 简单的相似度Calculate（based onhashvalue的汉明距离）
            let similarity = if h1 == h2 {
                1.0
            } else {
                // Calculatehashvalue的差异
                let diff = (h1 ^ h2).count_ones() as f64;
                let max_diff = 64.0; // u64 的maximumbit count
                1.0 - (diff / max_diff)
            };

            FingerprintComparison {
                similarity,
                matched: similarity > 0.8, // 相似度阈value
                matched_fields: Vec::new(),
                unmatched_fields: Vec::new(),
            }
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
