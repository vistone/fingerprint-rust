//! TLS fingerprintstatisticsmodule
//!
//! providefingerprintstatistics and analysisFeatures
//! reference：Huginn Net Profiler statisticsFeatures

use crate::tls_config::signature::ClientHelloSignature;
use crate::tls_config::spec::ClientHelloSpec;
use std::collections::HashMap;

/// TLS fingerprintstatisticsinfo
#[derive(Debug, Clone)]
pub struct FingerprintStats {
 /// 总fingerprintcount
 pub total_fingerprints: usize,
 /// including GREASE fingerprintcount
 pub fingerprints_with_grease: usize,
 /// including SNI fingerprintcount
 pub fingerprints_with_sni: usize,
 /// including ALPN fingerprintcount
 pub fingerprints_with_alpn: usize,
 /// TLS versiondistribution
 pub version_distribution: HashMap<String, usize>,
 /// most common's cipher suites (front 10)
 pub top_cipher_suites: Vec<(u16, usize)>,
 /// most common's extensions (front 10)
 pub top_extensions: Vec<(u16, usize)>,
}

impl FingerprintStats {
 /// from multiple ClientHelloSpec Calculatestatisticsinfo
 pub fn from_specs(specs: &[ClientHelloSpec]) -> Self {
 let mut stats = Self {
 total_fingerprints: specs.len(),
 fingerprints_with_grease: 0,
 fingerprints_with_sni: 0,
 fingerprints_with_alpn: 0,
 version_distribution: HashMap::new(),
 top_cipher_suites: Vec::new(),
 top_extensions: Vec::new(),
 };

 let mut cipher_suite_counts: HashMap<u16, usize> = HashMap::new();
 let mut extension_counts: HashMap<u16, usize> = HashMap::new();

 for spec in specs {
 let signature = crate::tls_config::extract::extract_signature(spec);

 // Check GREASE
 if signature.has_grease() {
 stats.fingerprints_with_grease += 1;
 }

 // Check SNI
 if signature.sni.is_some() {
 stats.fingerprints_with_sni += 1;
 }

 // Check ALPN
 if signature.alpn.is_some() {
 stats.fingerprints_with_alpn += 1;
 }

 // TLS versiondistribution
 let version_str = format!("{}", signature.version);
 *stats.version_distribution.entry(version_str).or_insert(0) += 1;

 // statisticscipher suite
 for suite in &signature.cipher_suites {
 *cipher_suite_counts.entry(*suite).or_insert(0) += 1;
 }

 // statisticsextension
 for ext in &signature.extensions {
 *extension_counts.entry(*ext).or_insert(0) += 1;
 }
 }

 // Getmost common's cipher suites
 let mut cipher_vec: Vec<(u16, usize)> = cipher_suite_counts.into_iter().collect();
 cipher_vec.sort_by_key(|(_, count)| *count);
 cipher_vec.reverse();
 stats.top_cipher_suites = cipher_vec.into_iter().take(10).collect();

 // Getmost common's extensions
 let mut ext_vec: Vec<(u16, usize)> = extension_counts.into_iter().collect();
 ext_vec.sort_by_key(|(_, count)| *count);
 ext_vec.reverse();
 stats.top_extensions = ext_vec.into_iter().take(10).collect();

 stats
 }

 /// from multiplesignatureCalculatestatisticsinfo
 pub fn from_signatures(signatures: &[ClientHelloSignature]) -> Self {
 // directly from signatureCalculatestatisticsinfo
 let mut stats = Self {
 total_fingerprints: signatures.len(),
 fingerprints_with_grease: 0,
 fingerprints_with_sni: 0,
 fingerprints_with_alpn: 0,
 version_distribution: HashMap::new(),
 top_cipher_suites: Vec::new(),
 top_extensions: Vec::new(),
 };

 let mut cipher_suite_counts: HashMap<u16, usize> = HashMap::new();
 let mut extension_counts: HashMap<u16, usize> = HashMap::new();

 for signature in signatures {
 // Check GREASE
 if signature.has_grease() {
 stats.fingerprints_with_grease += 1;
 }

 // Check SNI
 if signature.sni.is_some() {
 stats.fingerprints_with_sni += 1;
 }

 // Check ALPN
 if signature.alpn.is_some() {
 stats.fingerprints_with_alpn += 1;
 }

 // TLS versiondistribution
 let version_str = format!("{}", signature.version);
 *stats.version_distribution.entry(version_str).or_insert(0) += 1;

 // statisticscipher suite
 for suite in &signature.cipher_suites {
 *cipher_suite_counts.entry(*suite).or_insert(0) += 1;
 }

 // statisticsextension
 for ext in &signature.extensions {
 *extension_counts.entry(*ext).or_insert(0) += 1;
 }
 }

 // Getmost common's cipher suites
 let mut cipher_vec: Vec<(u16, usize)> = cipher_suite_counts.into_iter().collect();
 cipher_vec.sort_by_key(|(_, count)| *count);
 cipher_vec.reverse();
 stats.top_cipher_suites = cipher_vec.into_iter().take(10).collect();

 // Getmost common's extensions
 let mut ext_vec: Vec<(u16, usize)> = extension_counts.into_iter().collect();
 ext_vec.sort_by_key(|(_, count)| *count);
 ext_vec.reverse();
 stats.top_extensions = ext_vec.into_iter().take(10).collect();

 stats
 }
}

#[cfg(test)]
mod tests {
 use super::*;

 #[test]
 fn test_stats_from_specs() {
 let specs = vec![
 ClientHelloSpec::chrome_133(),
 ClientHelloSpec::chrome_103(),
 ClientHelloSpec::firefox_133(),
 ];
 let stats = FingerprintStats::from_specs(&specs);
 assert_eq!(stats.total_fingerprints, 3);
 assert!(!stats.version_distribution.is_empty());
 }
}
