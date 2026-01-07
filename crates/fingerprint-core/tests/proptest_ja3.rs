//! Property-based tests for JA3 fingerprinting
//!
//! This module uses proptest to verify JA3 implementation properties:
//! - Determinism: Same input always produces same output
//! - Stability: GREASE filtering is consistent
//! - Validity: Generated fingerprints are always valid
//! - Completeness: All input combinations produce valid fingerprints

use fingerprint_core::ja3::{JA3, JA3S};
use proptest::prelude::*;

/// Strategy for generating valid TLS versions
fn tls_version_strategy() -> impl Strategy<Value = u16> {
    prop_oneof![
        Just(768u16),  // SSL 3.0
        Just(769u16),  // TLS 1.0
        Just(770u16),  // TLS 1.1
        Just(771u16),  // TLS 1.2
        Just(772u16),  // TLS 1.3
    ]
}

/// Strategy for generating cipher suites
fn cipher_suites_strategy() -> impl Strategy<Value = Vec<u16>> {
    prop::collection::vec(0u16..=0xFFFF, 0..=50)
}

/// Strategy for generating extensions
fn extensions_strategy() -> impl Strategy<Value = Vec<u16>> {
    prop::collection::vec(0u16..=0xFFFF, 0..=30)
}

/// Strategy for generating elliptic curves
fn elliptic_curves_strategy() -> impl Strategy<Value = Vec<u16>> {
    prop::collection::vec(0u16..=0xFFFF, 0..=20)
}

/// Strategy for generating EC point formats
fn ec_point_formats_strategy() -> impl Strategy<Value = Vec<u8>> {
    prop::collection::vec(0u8..=255, 0..=10)
}

proptest! {
    /// Property: JA3 generation is deterministic
    /// Same inputs should always produce the same fingerprint
    #[test]
    fn test_ja3_determinism(
        version in tls_version_strategy(),
        ciphers in cipher_suites_strategy(),
        extensions in extensions_strategy(),
        curves in elliptic_curves_strategy(),
        formats in ec_point_formats_strategy()
    ) {
        let ja3_1 = JA3::generate(version, &ciphers, &extensions, &curves, &formats);
        let ja3_2 = JA3::generate(version, &ciphers, &extensions, &curves, &formats);
        
        prop_assert_eq!(ja3_1.fingerprint, ja3_2.fingerprint);
        prop_assert_eq!(ja3_1.ja3_string, ja3_2.ja3_string);
    }

    /// Property: JA3 fingerprint is always 32 characters (MD5 hash)
    #[test]
    fn test_ja3_fingerprint_length(
        version in tls_version_strategy(),
        ciphers in cipher_suites_strategy(),
        extensions in extensions_strategy(),
        curves in elliptic_curves_strategy(),
        formats in ec_point_formats_strategy()
    ) {
        let ja3 = JA3::generate(version, &ciphers, &extensions, &curves, &formats);
        prop_assert_eq!(ja3.fingerprint.len(), 32);
        
        // Verify all characters are hexadecimal
        prop_assert!(ja3.fingerprint.chars().all(|c| c.is_ascii_hexdigit()));
    }

    /// Property: GREASE values should be consistently filtered
    #[test]
    fn test_ja3_grease_filtering(
        version in tls_version_strategy(),
    ) {
        // Create a set with known GREASE values and non-GREASE values
        let mut ciphers = vec![0x1301, 0x1302, 0x1303]; // Non-GREASE TLS 1.3 ciphers
        ciphers.push(0x0a0a); // GREASE
        ciphers.push(0x1a1a); // GREASE
        ciphers.push(0x2a2a); // GREASE
        ciphers.push(0x3a3a); // GREASE
        
        let ja3 = JA3::generate(version, &ciphers, &[], &[], &[]);
        
        // Verify GREASE values are not in output
        // These are the decimal representations of the GREASE values above
        let grease_decimals = vec!["2570", "6666", "10794", "14906"];
        let cipher_parts: Vec<&str> = ja3.ciphers.split('-').collect();
        
        for grease_decimal in &grease_decimals {
            prop_assert!(!cipher_parts.contains(grease_decimal), 
                "GREASE value {} should not appear in JA3 cipher list", grease_decimal);
        }
        
        // Verify non-GREASE values are present
        prop_assert!(cipher_parts.contains(&"4865"), "TLS_AES_128_GCM_SHA256 (0x1301) should be present");
    }

    /// Property: Empty inputs should produce valid fingerprints
    #[test]
    fn test_ja3_empty_inputs(version in tls_version_strategy()) {
        let ja3 = JA3::generate(version, &[], &[], &[], &[]);
        
        prop_assert_eq!(ja3.fingerprint.len(), 32);
        prop_assert_eq!(ja3.ciphers, "");
        prop_assert_eq!(ja3.extensions, "");
    }

    /// Property: JA3 string format is correct
    #[test]
    fn test_ja3_string_format(
        version in tls_version_strategy(),
        ciphers in cipher_suites_strategy(),
        extensions in extensions_strategy(),
        curves in elliptic_curves_strategy(),
        formats in ec_point_formats_strategy()
    ) {
        let ja3 = JA3::generate(version, &ciphers, &extensions, &curves, &formats);
        
        // JA3 string should contain exactly 4 commas (5 parts)
        let comma_count = ja3.ja3_string.chars().filter(|&c| c == ',').count();
        prop_assert_eq!(comma_count, 4);
        
        // First part should be the version number
        let parts: Vec<&str> = ja3.ja3_string.split(',').collect();
        prop_assert_eq!(parts.len(), 5);
        prop_assert_eq!(parts[0], version.to_string());
    }

    /// Property: JA3S generation is deterministic
    #[test]
    fn test_ja3s_determinism(
        version in tls_version_strategy(),
        cipher in 0u16..=0xFFFF,
        extensions in extensions_strategy()
    ) {
        let ja3s_1 = JA3S::generate(version, cipher, &extensions);
        let ja3s_2 = JA3S::generate(version, cipher, &extensions);
        
        prop_assert_eq!(ja3s_1.fingerprint, ja3s_2.fingerprint);
        prop_assert_eq!(ja3s_1.ja3s_string, ja3s_2.ja3s_string);
    }

    /// Property: JA3S fingerprint is always 32 characters
    #[test]
    fn test_ja3s_fingerprint_length(
        version in tls_version_strategy(),
        cipher in 0u16..=0xFFFF,
        extensions in extensions_strategy()
    ) {
        let ja3s = JA3S::generate(version, cipher, &extensions);
        prop_assert_eq!(ja3s.fingerprint.len(), 32);
        prop_assert!(ja3s.fingerprint.chars().all(|c| c.is_ascii_hexdigit()));
    }

    /// Property: JA3S string format is correct (3 parts)
    #[test]
    fn test_ja3s_string_format(
        version in tls_version_strategy(),
        cipher in 0u16..=0xFFFF,
        extensions in extensions_strategy()
    ) {
        let ja3s = JA3S::generate(version, cipher, &extensions);
        
        // JA3S string should contain exactly 2 commas (3 parts)
        let comma_count = ja3s.ja3s_string.chars().filter(|&c| c == ',').count();
        prop_assert_eq!(comma_count, 2);
        
        let parts: Vec<&str> = ja3s.ja3s_string.split(',').collect();
        prop_assert_eq!(parts.len(), 3);
        prop_assert_eq!(parts[0], version.to_string());
        prop_assert_eq!(parts[1], cipher.to_string());
    }

    /// Property: Display trait shows fingerprint
    #[test]
    fn test_ja3_display(
        version in tls_version_strategy(),
        ciphers in cipher_suites_strategy()
    ) {
        let ja3 = JA3::generate(version, &ciphers, &[], &[], &[]);
        let displayed = format!("{}", ja3);
        prop_assert_eq!(displayed, ja3.fingerprint);
    }

    /// Property: JA3 generation never panics
    #[test]
    fn test_ja3_no_panic(
        version in 0u16..=0xFFFF,
        ciphers in cipher_suites_strategy(),
        extensions in extensions_strategy(),
        curves in elliptic_curves_strategy(),
        formats in ec_point_formats_strategy()
    ) {
        // Should complete without panicking
        let _ = JA3::generate(version, &ciphers, &extensions, &curves, &formats);
    }

    /// Property: Large inputs are handled correctly
    #[test]
    fn test_ja3_large_inputs(version in tls_version_strategy()) {
        let large_ciphers: Vec<u16> = (0..1000).map(|i| i as u16).collect();
        let large_extensions: Vec<u16> = (0..500).map(|i| i as u16).collect();
        let large_curves: Vec<u16> = (0..100).map(|i| i as u16).collect();
        let large_formats: Vec<u8> = (0..50).map(|i| i as u8).collect();
        
        let ja3 = JA3::generate(version, &large_ciphers, &large_extensions, &large_curves, &large_formats);
        
        // Should still produce valid fingerprint
        prop_assert_eq!(ja3.fingerprint.len(), 32);
        prop_assert!(ja3.fingerprint.chars().all(|c| c.is_ascii_hexdigit()));
    }
}

#[cfg(test)]
mod additional_tests {
    use super::*;
    
    #[test]
    fn test_ja3_consistency_with_known_values() {
        // Test with known Chrome-like values
        let ja3 = JA3::generate(
            771, // TLS 1.2
            &[0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f],
            &[0, 10, 11, 13, 16, 23],
            &[23, 24, 25],
            &[0],
        );
        
        // Should produce consistent output
        assert!(!ja3.fingerprint.is_empty());
        assert_eq!(ja3.ssl_version, 771);
        
        // Re-generate and verify consistency
        let ja3_2 = JA3::generate(
            771,
            &[0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f],
            &[0, 10, 11, 13, 16, 23],
            &[23, 24, 25],
            &[0],
        );
        
        assert_eq!(ja3.fingerprint, ja3_2.fingerprint);
    }
    
    #[test]
    fn test_ja3_ordering_independence() {
        // JA3 does not sort, so order matters
        let ja3_1 = JA3::generate(771, &[0x1301, 0x1302], &[], &[], &[]);
        let ja3_2 = JA3::generate(771, &[0x1302, 0x1301], &[], &[], &[]);
        
        // Different orders should produce different fingerprints
        assert_ne!(ja3_1.fingerprint, ja3_2.fingerprint);
    }
    
    #[test]
    fn test_ja3s_consistency() {
        let ja3s = JA3S::generate(771, 0x1301, &[0, 10, 11]);
        
        // Should be consistent
        assert!(!ja3s.fingerprint.is_empty());
        assert_eq!(ja3s.ssl_version, 771);
        assert_eq!(ja3s.cipher, 0x1301);
    }
}
