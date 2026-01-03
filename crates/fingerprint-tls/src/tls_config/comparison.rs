//! TLS fingerprintcomparemodule
//!
//! providefingerprintcompare and matchFeatures
//! reference：Huginn Net fingerprintcompareimplement

use crate::tls_config::extract::extract_signature;
use crate::tls_config::signature::ClientHelloSignature;
use crate::tls_config::spec::ClientHelloSpec;

/// fingerprintmatchresult
#[derive(Debug, Clone, PartialEq)]
pub enum FingerprintMatch {
 /// completelymatch (include GREASE value)
 Exact,
 /// similarmatch (ignore GREASE valuebacksame)
 Similar,
 /// does not match
 None,
}

/// compare two ClientHelloSpec similar度
///
/// # Parameters
/// * `spec1` - first ClientHelloSpec
/// * `spec2` - second ClientHelloSpec
///
/// # Returns
/// * `FingerprintMatch` - matchresult
///
/// # Examples
/// ```
/// use fingerprint_tls::tls_config::{ClientHelloSpec, compare_specs};
/// let spec1 = ClientHelloSpec::chrome_133();
/// let spec2 = ClientHelloSpec::chrome_103();
/// let match_result = compare_specs(&spec1, &spec2);
/// ```
pub fn compare_specs(spec1: &ClientHelloSpec, spec2: &ClientHelloSpec) -> FingerprintMatch {
 let sig1 = extract_signature(spec1);
 let sig2 = extract_signature(spec2);

 compare_signatures(&sig1, &sig2)
}

/// compare twosignaturesimilar度
///
/// # Parameters
/// * `sig1` - firstsignature
/// * `sig2` - secondsignature
///
/// # Returns
/// * `FingerprintMatch` - matchresult
pub fn compare_signatures(
 sig1: &ClientHelloSignature,
 sig2: &ClientHelloSignature,
) -> FingerprintMatch {
 // completelymatch
 if sig1 == sig2 {
 return FingerprintMatch::Exact;
 }

 // similarmatch (ignore GREASE)
 if sig1.similar_to(sig2) {
 return FingerprintMatch::Similar;
 }

 FingerprintMatch::None
}

/// find and 给定signature最similarfingerprintconfiguration
///
/// # Parameters
/// * `signature` - 要matchsignature
/// * `specs` - 候选 ClientHelloSpec list
///
/// # Returns
/// * `Option<usize>` - 最similarconfigurationindex， if no找 to 则return None
pub fn find_best_match(
 signature: &ClientHelloSignature,
 specs: &[ClientHelloSpec],
) -> Option<usize> {
 let mut best_index = None;
 let mut best_score = 0;

 for (index, spec) in specs.iter().enumerate() {
 let spec_sig = extract_signature(spec);
 let match_result = compare_signatures(signature, &spec_sig);

 let score = match match_result {
 FingerprintMatch::Exact => 100,
 FingerprintMatch::Similar => 50,
 FingerprintMatch::None => 0,
 };

 if score > best_score {
 best_score = score;
 best_index = Some(index);
 }
 }

 best_index
}

#[cfg(test)]
mod tests {
 use super::*;

 #[test]
 fn test_compare_specs() {
 let spec1 = ClientHelloSpec::chrome_133();
 let spec2 = ClientHelloSpec::chrome_133();
 let result = compare_specs(&spec1, &spec2);
 // due toset成了random GREASE，两次Generate spec in GREASE valueupmaydifferent，
 // thereforeresultshould is Similar (ignore GREASE backsame)
 assert!(matches!(
 result,
 FingerprintMatch::Exact | FingerprintMatch::Similar
 ));
 }

 #[test]
 fn test_find_best_match() {
 let signature = extract_signature(&ClientHelloSpec::chrome_133());
 let specs = vec![
 ClientHelloSpec::chrome_103(),
 ClientHelloSpec::chrome_133(),
 ClientHelloSpec::firefox_133(),
 ];
 let best = find_best_match(&signature, &specs);
 assert_eq!(best, Some(1)); // chrome_133 should is 最match的
 }
}
