#![no_main]

use libfuzzer_sys::fuzz_target;
use fingerprint_core::ja3::JA3;

fuzz_target!(|data: &[u8]| {
    // Ensure we have enough data
    if data.len() < 10 {
        return;
    }

    // Extract version (2 bytes)
    let version = u16::from_be_bytes([data[0], data[1]]);
    
    // Parse cipher suites length and data
    let cipher_len = (data[2] as usize).min((data.len() - 10) / 2);
    let mut ciphers = Vec::new();
    for i in 0..cipher_len {
        let idx = 3 + i * 2;
        if idx + 1 < data.len() {
            ciphers.push(u16::from_be_bytes([data[idx], data[idx + 1]]));
        }
    }
    
    // Parse extensions (similar approach)
    let ext_start = 3 + cipher_len * 2;
    let ext_len = if ext_start < data.len() {
        (data[ext_start] as usize).min((data.len() - ext_start) / 2)
    } else {
        0
    };
    
    let mut extensions = Vec::new();
    for i in 0..ext_len {
        let idx = ext_start + 1 + i * 2;
        if idx + 1 < data.len() {
            extensions.push(u16::from_be_bytes([data[idx], data[idx + 1]]));
        }
    }
    
    // Parse curves
    let curves_start = ext_start + 1 + ext_len * 2;
    let curves_len = if curves_start < data.len() {
        (data[curves_start] as usize).min((data.len() - curves_start) / 2)
    } else {
        0
    };
    
    let mut curves = Vec::new();
    for i in 0..curves_len {
        let idx = curves_start + 1 + i * 2;
        if idx + 1 < data.len() {
            curves.push(u16::from_be_bytes([data[idx], data[idx + 1]]));
        }
    }
    
    // Parse point formats
    let formats_start = curves_start + 1 + curves_len * 2;
    let formats: Vec<u8> = if formats_start < data.len() {
        data[formats_start..].to_vec()
    } else {
        Vec::new()
    };
    
    // Try to generate JA3 - should never panic
    let _ja3 = JA3::generate(version, &ciphers, &extensions, &curves, &formats);
    
    // Verify basic properties
    assert_eq!(_ja3.fingerprint.len(), 32, "JA3 fingerprint must be 32 characters");
    assert!(_ja3.fingerprint.chars().all(|c| c.is_ascii_hexdigit()), "JA3 must be hex");
});
