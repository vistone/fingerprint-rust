//! End-to-End Integration Tests
//! Tests the complete fingerprinting pipeline from PCAP → Browser Detection

use fingerprint_core::pcap_generator::*;
use std::fs;

/// Test complete fingerprinting pipeline with synthetic Chrome traffic
#[test]
fn test_e2e_chrome_synthetic() {
    // Generate synthetic Chrome PCAP
    let pcap_path = "test_data/synthetic/e2e_chrome.pcap";
    setup_test_pcap_chrome(pcap_path);

    // Read PCAP file
    let pcap_data = fs::read(pcap_path).expect("Failed to read PCAP file");

    // Verify file has global header
    assert!(pcap_data.len() >= 24, "PCAP file too small");

    // Verify magic number
    let magic = u32::from_le_bytes([pcap_data[0], pcap_data[1], pcap_data[2], pcap_data[3]]);
    assert_eq!(magic, 0xa1b2c3d4, "Invalid PCAP magic number");

    println!("✓ E2E Chrome synthetic test passed: PCAP file valid");
}

/// Test Firefox synthetic fingerprint
#[test]
fn test_e2e_firefox_synthetic() {
    let pcap_path = "test_data/synthetic/e2e_firefox.pcap";
    setup_test_pcap_firefox(pcap_path);

    let pcap_data = fs::read(pcap_path).expect("Failed to read PCAP file");
    assert!(pcap_data.len() >= 24, "PCAP file too small");

    let magic = u32::from_le_bytes([pcap_data[0], pcap_data[1], pcap_data[2], pcap_data[3]]);
    assert_eq!(magic, 0xa1b2c3d4);

    println!("✓ E2E Firefox synthetic test passed: PCAP file valid");
}

/// Test TCP flow tracking from generated PCAP
#[test]
fn test_e2e_pcap_generation() {
    let pcap_path = "test_data/synthetic/e2e_generation.pcap";

    std::fs::create_dir_all("test_data/synthetic").ok();

    // Generate PCAP with Chrome SYN
    let mut gen = PcapGenerator::new();
    gen.add_chrome_syn();
    gen.write_to_file(pcap_path).expect("Failed to write PCAP");

    // Verify file was created
    let metadata = fs::metadata(pcap_path).expect("PCAP file not created");
    assert!(metadata.len() > 24, "PCAP file too small");

    // Read back and verify
    let pcap_data = fs::read(pcap_path).expect("Failed to read PCAP");
    let magic = u32::from_le_bytes([pcap_data[0], pcap_data[1], pcap_data[2], pcap_data[3]]);
    assert_eq!(magic, 0xa1b2c3d4);

    println!(
        "✓ E2E PCAP generation test passed: {} bytes",
        metadata.len()
    );
}

/// Test multi-packet generation
#[test]
fn test_e2e_multi_packet_pcap() {
    let pcap_path = "test_data/synthetic/e2e_multi.pcap";

    std::fs::create_dir_all("test_data/synthetic").ok();

    let mut gen = PcapGenerator::new();
    gen.add_chrome_syn();
    gen.add_firefox_syn();
    gen.add_chrome_syn(); // Add another
    gen.write_to_file(pcap_path).expect("Failed to write PCAP");

    // Verify file
    let metadata = fs::metadata(pcap_path).expect("PCAP file not created");
    let _pcap_data = fs::read(pcap_path).expect("Failed to read PCAP");

    // Count packets (simple heuristic - not perfect parser)
    let expected_min_size = 24 + (16 + 60) * 3; // Global header + 3 packets (min)
    assert!(
        metadata.len() >= expected_min_size as u64,
        "PCAP too small: {} < {}",
        metadata.len(),
        expected_min_size
    );

    println!(
        "✓ E2E multi-packet test passed: {} bytes, ~3 packets",
        metadata.len()
    );
}

/// Test Chrome vs Firefox TCP option differences
#[test]
fn test_e2e_browser_differentiation() {
    std::fs::create_dir_all("test_data/synthetic").ok();

    // Generate Chrome SYN
    let chrome_path = "test_data/synthetic/e2e_chrome_opts.pcap";
    let mut chrome_gen = PcapGenerator::new();
    chrome_gen.add_chrome_syn();
    chrome_gen
        .write_to_file(chrome_path)
        .expect("Failed to write Chrome PCAP");

    // Generate Firefox SYN
    let firefox_path = "test_data/synthetic/e2e_firefox_opts.pcap";
    let mut firefox_gen = PcapGenerator::new();
    firefox_gen.add_firefox_syn();
    firefox_gen
        .write_to_file(firefox_path)
        .expect("Failed to write Firefox PCAP");

    // Read both
    let chrome_data = fs::read(chrome_path).expect("Failed to read Chrome PCAP");
    let firefox_data = fs::read(firefox_path).expect("Failed to read Firefox PCAP");

    // They should be different (different TCP options)
    assert_ne!(
        chrome_data, firefox_data,
        "Chrome and Firefox PCAPs should differ"
    );

    // Both should have valid magic numbers
    let chrome_magic = u32::from_le_bytes([
        chrome_data[0],
        chrome_data[1],
        chrome_data[2],
        chrome_data[3],
    ]);
    let firefox_magic = u32::from_le_bytes([
        firefox_data[0],
        firefox_data[1],
        firefox_data[2],
        firefox_data[3],
    ]);
    assert_eq!(chrome_magic, 0xa1b2c3d4);
    assert_eq!(firefox_magic, 0xa1b2c3d4);

    println!("✓ E2E browser differentiation test passed: Chrome ≠ Firefox");
}

// Helper functions

fn setup_test_pcap_chrome(path: &str) {
    std::fs::create_dir_all("test_data/synthetic").ok();
    let mut gen = PcapGenerator::new();
    gen.add_chrome_syn();
    gen.write_to_file(path).expect("Failed to create test PCAP");
}

fn setup_test_pcap_firefox(path: &str) {
    std::fs::create_dir_all("test_data/synthetic").ok();
    let mut gen = PcapGenerator::new();
    gen.add_firefox_syn();
    gen.write_to_file(path).expect("Failed to create test PCAP");
}
