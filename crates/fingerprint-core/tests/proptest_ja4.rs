//! Property-based tests for JA4 fingerprinting
//!
//! This module uses proptest to verify JA4 implementation properties:
//! - Determinism: Same input always produces same output
//! - Stability: GREASE values are consistently filtered and sorted
//! - Validity: Generated fingerprints follow JA4 format
//! - Completeness: All input combinations produce valid fingerprints

use fingerprint_core::ja4::JA4;
use proptest::prelude::*;

/// Strategy for generating transport protocol
fn transport_strategy() -> impl Strategy<Value = char> {
    prop_oneof![
        Just('t'), // TCP
        Just('q'), // QUIC
    ]
}

/// Strategy for generating TLS version strings
fn version_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("1.0".to_string()),
        Just("1.1".to_string()),
        Just("1.2".to_string()),
        Just("1.3".to_string()),
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

/// Strategy for generating signature algorithms
fn signature_algorithms_strategy() -> impl Strategy<Value = Vec<u16>> {
    prop::collection::vec(0u16..=0xFFFF, 0..=20)
}

/// Strategy for generating ALPN values
fn alpn_strategy() -> impl Strategy<Value = Option<String>> {
    prop_oneof![
        Just(None),
        Just(Some("h2".to_string())),
        Just(Some("http/1.1".to_string())),
        Just(Some("h3".to_string())),
    ]
}

proptest! {
    /// Property: JA4 generation is deterministic
    /// Same inputs should always produce the same fingerprint
    #[test]
    fn test_ja4_determinism(
        transport in transport_strategy(),
        version in version_strategy(),
        has_sni in any::<bool>(),
        ciphers in cipher_suites_strategy(),
        extensions in extensions_strategy(),
        alpn in alpn_strategy(),
        signature_algorithms in signature_algorithms_strategy()
    ) {
        let alpn_ref = alpn.as_deref();
        let ja4_1 = JA4::generate(transport, &version, has_sni, &ciphers, &extensions, alpn_ref, &signature_algorithms);
        let ja4_2 = JA4::generate(transport, &version, has_sni, &ciphers, &extensions, alpn_ref, &signature_algorithms);

        prop_assert_eq!(ja4_1.to_fingerprint_string(), ja4_2.to_fingerprint_string());
    }

    /// Property: JA4 fingerprint string format is correct
    #[test]
    fn test_ja4_fingerprint_format(
        transport in transport_strategy(),
        version in version_strategy(),
        has_sni in any::<bool>(),
        ciphers in cipher_suites_strategy(),
        extensions in extensions_strategy(),
        alpn in alpn_strategy(),
        signature_algorithms in signature_algorithms_strategy()
    ) {
        let alpn_ref = alpn.as_deref();
        let ja4 = JA4::generate(transport, &version, has_sni, &ciphers, &extensions, alpn_ref, &signature_algorithms);
        let fingerprint = ja4.to_fingerprint_string();

        // JA4 format: t13d1516h2_8daaf6152771_000a (transport + version + destination + counts + alpn _ hashes)
        // Should contain underscores separating parts
        prop_assert!(fingerprint.contains('_'), "JA4 fingerprint should contain underscores");

        // Transport should be 't' or 'q'
        prop_assert!(ja4.transport == 't' || ja4.transport == 'q');

        // Version should be 2 digits
        prop_assert_eq!(ja4.version.len(), 2);

        // Destination should be 'd' or 'i'
        prop_assert!(ja4.destination == 'd' || ja4.destination == 'i');

        // Counts should be within bounds
        prop_assert!(ja4.cipher_count <= 99);
        prop_assert!(ja4.extension_count <= 99);

        // Hash lengths should be correct
        prop_assert_eq!(ja4.cipher_hash.len(), 12);
        prop_assert_eq!(ja4.extension_hash.len(), 12);
        prop_assert_eq!(ja4.signature_hash.len(), 4);
    }

    /// Property: GREASE values should be consistently filtered
    #[test]
    fn test_ja4_grease_filtering(
        transport in transport_strategy(),
        version in version_strategy(),
        has_sni in any::<bool>()
    ) {
        // Create a set with known GREASE values and non-GREASE values
        let mut ciphers = vec![0x1301, 0x1302, 0x1303]; // Non-GREASE TLS 1.3 ciphers
        ciphers.push(0x0a0a); // GREASE
        ciphers.push(0x1a1a); // GREASE

        let ja4 = JA4::generate(transport, &version, has_sni, &ciphers, &[], None, &[]);

        // Cipher count should exclude GREASE values
        prop_assert_eq!(ja4.cipher_count, 3, "GREASE values should be filtered");
    }

    /// Property: Empty inputs should produce valid fingerprints
    #[test]
    fn test_ja4_empty_inputs(
        transport in transport_strategy(),
        version in version_strategy(),
        has_sni in any::<bool>()
    ) {
        let ja4 = JA4::generate(transport, &version, has_sni, &[], &[], None, &[]);

        prop_assert_eq!(ja4.cipher_count, 0);
        prop_assert_eq!(ja4.extension_count, 0);
        prop_assert_eq!(ja4.cipher_hash.len(), 12);
        prop_assert_eq!(ja4.extension_hash.len(), 12);
    }

    /// Property: JA4 generation never panics
    #[test]
    fn test_ja4_no_panic(
        transport in transport_strategy(),
        version in version_strategy(),
        has_sni in any::<bool>(),
        ciphers in cipher_suites_strategy(),
        extensions in extensions_strategy(),
        alpn in alpn_strategy(),
        signature_algorithms in signature_algorithms_strategy()
    ) {
        let alpn_ref = alpn.as_deref();
        // Should complete without panicking
        let _ = JA4::generate(transport, &version, has_sni, &ciphers, &extensions, alpn_ref, &signature_algorithms);
    }

    /// Property: Large inputs are handled correctly
    #[test]
    fn test_ja4_large_inputs(
        transport in transport_strategy(),
        version in version_strategy(),
        has_sni in any::<bool>()
    ) {
        let large_ciphers: Vec<u16> = (0..200).map(|i| i as u16).collect();
        let large_extensions: Vec<u16> = (0..200).map(|i| i as u16).collect();
        let large_signatures: Vec<u16> = (0..100).map(|i| i as u16).collect();

        let ja4 = JA4::generate(transport, &version, has_sni, &large_ciphers, &large_extensions, None, &large_signatures);

        // Counts should be capped at 99
        prop_assert!(ja4.cipher_count <= 99);
        prop_assert!(ja4.extension_count <= 99);

        // Hashes should still be correct length
        prop_assert_eq!(ja4.cipher_hash.len(), 12);
        prop_assert_eq!(ja4.extension_hash.len(), 12);
        prop_assert_eq!(ja4.signature_hash.len(), 4);
    }

    /// Property: Display trait shows fingerprint string
    #[test]
    fn test_ja4_display(
        transport in transport_strategy(),
        version in version_strategy(),
        has_sni in any::<bool>()
    ) {
        let ja4 = JA4::generate(transport, &version, has_sni, &[0x1301], &[0], None, &[]);
        let displayed = format!("{}", ja4);
        let fingerprint = ja4.to_fingerprint_string();
        prop_assert_eq!(displayed, fingerprint);
    }

    /// Property: SNI flag affects destination character
    #[test]
    fn test_ja4_sni_flag(
        transport in transport_strategy(),
        version in version_strategy()
    ) {
        let ja4_with_sni = JA4::generate(transport, &version, true, &[], &[], None, &[]);
        let ja4_without_sni = JA4::generate(transport, &version, false, &[], &[], None, &[]);

        prop_assert_eq!(ja4_with_sni.destination, 'd');
        prop_assert_eq!(ja4_without_sni.destination, 'i');
    }
}

#[cfg(test)]
mod additional_tests {
    use super::*;

    #[test]
    fn test_ja4_consistency_with_known_values() {
        // Test with known Chrome-like values
        let ja4 = JA4::generate(
            't',
            "1.3",
            true,
            &[0x1301, 0x1302, 0x1303],
            &[0, 10, 11, 13, 16],
            Some("h2"),
            &[0x0403, 0x0804],
        );

        // Should produce consistent output
        assert!(!ja4.to_fingerprint_string().is_empty());
        assert_eq!(ja4.transport, 't');
        assert_eq!(ja4.version, "13");
        assert_eq!(ja4.destination, 'd');

        // Re-generate and verify consistency
        let ja4_2 = JA4::generate(
            't',
            "1.3",
            true,
            &[0x1301, 0x1302, 0x1303],
            &[0, 10, 11, 13, 16],
            Some("h2"),
            &[0x0403, 0x0804],
        );

        assert_eq!(ja4.to_fingerprint_string(), ja4_2.to_fingerprint_string());
    }

    #[test]
    fn test_ja4_sorting() {
        // JA4 sorts ciphers and extensions (unlike JA3)
        let ja4_1 = JA4::generate('t', "1.2", true, &[0x1301, 0x1302], &[], None, &[]);
        let ja4_2 = JA4::generate('t', "1.2", true, &[0x1302, 0x1301], &[], None, &[]);

        // Same ciphers in different order should produce same fingerprint after sorting
        assert_eq!(ja4_1.to_fingerprint_string(), ja4_2.to_fingerprint_string());
    }

    #[test]
    fn test_ja4_alpn_truncation() {
        let ja4_short = JA4::generate('t', "1.3", true, &[], &[], Some("h2"), &[]);
        let ja4_long = JA4::generate('t', "1.3", true, &[], &[], Some("http/1.1"), &[]);

        // Short ALPN should be preserved
        assert_eq!(ja4_short.alpn, "h2");

        // Long ALPN should be truncated to 2 characters
        assert_eq!(ja4_long.alpn, "ht");
    }
}
