// Example: QUIC Initial Packet Fingerprinting
//!
//! This example demonstrates how to use the QUIC fingerprinting module
//! to analyze QUIC Initial packets and create fingerprints.

use fingerprint_http::quic_fingerprint::QuicInitialPacket;

fn main() {
    println!("=== QUIC Initial Packet Fingerprinting Example ===\n");

    // Example 1: Parse a QUIC v1 Initial packet (Chrome-like)
    // This is a minimal example of what a QUIC Initial packet might look like
    let quic_v1_initial = vec![
        0xc0, // Header form (1), Fixed bit (1), Spin (0), Reserved (00), Type (0x00=Initial)
        0x00, 0x00, 0x00, 0x01, // Version 1
        0x08, // DCID length: 8
        0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, // DCID
        0x08, // SCID length: 8
        0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f, 0x10, // SCID
        0x00, // Token length: 0
        0x4a, 0x20, // Length: 1024 (variable length encoded)
        0x00, 0x00, 0x00, 0x01, // Packet number (sample)
    ];

    match QuicInitialPacket::parse(&quic_v1_initial) {
        Ok(packet) => {
            println!("✅ Successfully parsed QUIC v1 Initial packet");
            println!("{}\n", packet.analyze());
        }
        Err(e) => {
            println!("❌ Failed to parse packet: {}\n", e);
        }
    }

    // Example 2: Parse a QUIC v2 Initial packet (future use)
    let quic_v2_initial = vec![
        0xc0, // Header form (1), Fixed bit (1), Spin (0), Reserved (00), Type (0x00=Initial)
        0x6b, 0x33, 0x43, 0xcf, // Version 2
        0x04, // DCID length: 4
        0xaa, 0xbb, 0xcc, 0xdd, // DCID
        0x04, // SCID length: 4
        0x11, 0x22, 0x33, 0x44, // SCID
        0x00, // Token length: 0
        0x40, 0x10, // Length: 16 (variable length encoded)
    ];

    match QuicInitialPacket::parse(&quic_v2_initial) {
        Ok(packet) => {
            println!("✅ Successfully parsed QUIC v2 Initial packet");
            println!("{}\n", packet.analyze());
        }
        Err(e) => {
            println!("❌ Failed to parse packet: {}\n", e);
        }
    }

    // Example 3: Compare fingerprints from different clients
    println!("=== Comparing QUIC Client Fingerprints ===\n");

    // Chrome-like fingerprint
    let chrome_fp_data = vec![
        0xc0, 0x00, 0x00, 0x00, 0x01, // Chrome QUIC v1
        0x08, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, // 8-byte DCID
        0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f, 0x10, // 8-byte SCID
        0x00, 0x4a, 0x20,
    ];

    // Firefox-like fingerprint
    let firefox_fp_data = vec![
        0xc0, 0x00, 0x00, 0x00, 0x01, // Firefox QUIC v1
        0x04, 0x11, 0x22, 0x33, 0x44, // 4-byte DCID
        0x04, 0x55, 0x66, 0x77, 0x88, // 4-byte SCID
        0x00, 0x40, 0x10,
    ];

    // Safari-like fingerprint
    let safari_fp_data = vec![
        0xc0, 0x00, 0x00, 0x00, 0x01, // Safari QUIC v1
        0x14, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa,
        0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, // 20-byte DCID (longer)
        0x00, 0x4a, 0x20,
    ];

    if let Ok(chrome) = QuicInitialPacket::parse(&chrome_fp_data) {
        println!("Chrome fingerprint: {}", chrome.fingerprint());
    }

    if let Ok(firefox) = QuicInitialPacket::parse(&firefox_fp_data) {
        println!("Firefox fingerprint: {}", firefox.fingerprint());
    }

    if let Ok(safari) = QuicInitialPacket::parse(&safari_fp_data) {
        println!("Safari fingerprint: {}", safari.fingerprint());
    }

    println!("\n=== QUIC Fingerprinting Benefits ===");
    println!("✓ Identifies QUIC client characteristics");
    println!("✓ Connection ID length patterns differ by client");
    println!("✓ Reserved bits indicate implementation choices");
    println!("✓ Packet number length hints at MTU assumptions");
    println!("✓ Supports JA4 QUIC variant ('q' protocol marker)");
}
