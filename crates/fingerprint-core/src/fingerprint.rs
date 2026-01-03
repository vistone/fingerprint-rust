//! fingerprintcoreabstract
//!
//! defineunifiedfingerprintabstract，support TLS、HTTP、TCP etc.multiplefingerprinttype。

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

/// fingerprintabstract trait
///
/// allfingerprinttype (TLS、HTTP、TCP)都shouldimplementthis trait
pub trait Fingerprint: Send + Sync {
 /// Getfingerprinttype
 fn fingerprint_type(&self) -> FingerprintType;

 /// Getfingerprintuniqueidentifier符 (usually is hashvalue)
 fn id(&self) -> String;

 /// Getfingerprintmetadata
 fn metadata(&self) -> &FingerprintMetadata;

 /// Getfingerprintmetadata (mutablereference)
 fn metadata_mut(&mut self) -> &mut FingerprintMetadata;

 /// Calculatefingerprinthashvalue ( for fastcompare)
 fn hash(&self) -> u64;

 /// compare twofingerprintwhethersimilar
 fn similar_to(&self, other: &dyn Fingerprint) -> bool;

 /// Getfingerprintstringrepresent ( for debug and log)
 fn to_string(&self) -> String;
}

/// fingerprintcompareresult
#[derive(Debug, Clone, PartialEq)]
pub struct FingerprintComparison {
 /// similar度分count (0.0 - 1.0)
 pub similarity: f64,

 /// whethermatch
 pub matched: bool,

 /// matchfield
 pub matched_fields: Vec<String>,

 /// does not matchfield
 pub unmatched_fields: Vec<String>,
}

impl FingerprintComparison {
 /// Create a newcompareresult
 pub fn new(similarity: f64, matched: bool) -> Self {
 Self {
 similarity,
 matched,
 matched_fields: Vec::new(),
 unmatched_fields: Vec::new(),
 }
 }

 /// Createcompletelymatchresult
 pub fn perfect_match() -> Self {
 Self {
 similarity: 1.0,
 matched: true,
 matched_fields: Vec::new(),
 unmatched_fields: Vec::new(),
 }
 }

 /// Createcompletelydoes not matchresult
 pub fn no_match() -> Self {
 Self {
 similarity: 0.0,
 matched: false,
 matched_fields: Vec::new(),
 unmatched_fields: Vec::new(),
 }
 }
}

/// fingerprintcompareer
pub struct FingerprintComparator;

impl FingerprintComparator {
 /// compare twofingerprint
 pub fn compare(f1: &dyn Fingerprint, f2: &dyn Fingerprint) -> FingerprintComparison {
 // typemustsame
 if f1.fingerprint_type() != f2.fingerprint_type() {
 return FingerprintComparison::no_match();
 }

 // use similar_to methodperformcompare
 if f1.similar_to(f2) {
 FingerprintComparison::perfect_match()
 } else {
 // Calculatesimilar度 (based onhashvalue)
 let h1 = f1.hash();
 let h2 = f2.hash();

 // simplesimilar度Calculate (based onhashvalue汉明distance)
 let similarity = if h1 == h2 {
 1.0
 } else {
 // Calculatehashvaluedifference
 let diff = (h1 ^ h2).count_ones() as f64;
 let max_diff = 64.0; // u64 maximumbit count
 1.0 - (diff / max_diff)
 };

 FingerprintComparison {
 similarity,
 matched: similarity > 0.8, // similarthresholdvalue
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
