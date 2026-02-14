// Example: TLS 1.3 PSK and 0-RTT (Early Data) Fingerprinting
//!
//! Demonstrates how to use PSK, 0-RTT, and combined PSK+0-RTT extensions
//! for TLS 1.3 session resumption and fast connections.

fn main() {
    println!("=== TLS 1.3 PSK and 0-RTT (Zero Round Trip Time) Demo ===\n");

    // Example 1: Standard TLS 1.3 (Initial connection)
    println!("1. Standard TLS 1.3 ClientHello:");
    println!("   - No session resumption");
    println!("   - Full 1-RTT handshake");
    println!("   - Uses: SNI, Key Share, Supported Groups, Signature Algorithms");
    println!("   - Fingerprint: chrome_133\n");

    // Example 2: TLS 1.3 with PSK (Session Resumption)
    println!("2. TLS 1.3 with PSK (Pre-Shared Key):");
    println!("   - Resumes previous session using session ticket");
    println!("   - Significantly faster than full handshake");
    println!("   - Uses: PSK extension + PSK Key Exchange Modes");
    println!("   - Extensions added:");
    println!("     * PSKKeyExchangeModesExtension::new(vec![PSK_MODE_DHE])");
    println!("     * PreSharedKeyExtension::for_session_resumption(session_id, binder)");
    println!("   - Fingerprint: chrome_133_PSK\n");

    // Example 3: TLS 1.3 with 0-RTT (Early Data)
    println!("3. TLS 1.3 with 0-RTT (Early Data):");
    println!("   - Client sends application data in first packet (ClientHello)");
    println!("   - Server can respond immediately without waiting");
    println!("   - Zero round-trip latency for application data");
    println!("   - Uses: EarlyDataExtension");
    println!("   - Extensions added:");
    println!("     * EarlyDataExtension::standard() // 16KB default");
    println!("   - Security: Only for idempotent operations (GET requests)");
    println!("   - Fingerprint: chrome_133_0RTT\n");

    // Example 4: Combined PSK + 0-RTT (Fastest)
    println!("4. TLS 1.3 with PSK + 0-RTT (Combined):");
    println!("   - Uses session ticket (PSK) AND early data simultaneously");
    println!("   - Fastest possible TLS 1.3 connection");
    println!("   - Client: Send PSK + EarlyData in ClientHello");
    println!("   - Server: Decrypt with PSK, process early data");
    println!("   - Extensions added:");
    println!("     * EarlyDataExtension::standard()");
    println!("     * PreSharedKeyExtension::for_session_resumption(...)");
    println!("     * PSKKeyExchangeModesExtension::new(vec![PSK_MODE_DHE])");
    println!("   - Fingerprint: chrome_133_PSK_0RTT\n");

    // Timeline comparison
    println!("=== Connection Timeline Comparison ===\n");

    println!("Standard 1-RTT:");
    println!("  Client                          Server");
    println!("  |");
    println!("  |---- ClientHello ---------->|");
    println!("  |                         ServerHello");
    println!("  |<--- ServerHello + Cert <--|");
    println!("  |");
    println!("  |---- Finished ------------>|");
    println!("  |                    Finished");
    println!("  |<---- Finished ------------|");
    println!("  |");
    println!("  | Application Data Ready");
    println!("  Latency: 1 Round Trip\n");

    println!("PSK Session Resumption (<1-RTT):");
    println!("  Client                          Server");
    println!("  |");
    println!("  |---- ClientHello (with PSK)-->|");
    println!("  |<----- Finished ------------|");
    println!("  |");
    println!("  | Application Data Ready");
    println!("  Latency: <1 Round Trip\n");

    println!("0-RTT Early Data:");
    println!("  Client                          Server");
    println!("  |");
    println!("  |---- ClientHello + EarlyData-->|");
    println!("  |<----- ServerHello ------------|");
    println!("  |");
    println!("  | Application Data Ready (already sent)");
    println!("  Latency: 0 Round Trips (for early data)\n");

    println!("PSK + 0-RTT (Fastest):");
    println!("  Client                          Server");
    println!("  |");
    println!("  |---- ClientHello + PSK + EarlyData-->|");
    println!("  |<----- 1-RTT Response ----------|");
    println!("  |");
    println!("  | Application Data Ready");
    println!("  Latency: 0 RTT for resumption\n");

    // Client characteristics
    println!("=== Browser Client Characteristics ===\n");

    println!("Chrome 133 Modern Features:");
    println!("  ✓ Supports ECH (RFC 9180) - Encrypted Client Hello");
    println!("  ✓ Supports PSK resumption");
    println!("  ✓ Supports 0-RTT (Early Data)");
    println!("  ✓ Supports X25519Kyber768 (hybrid post-quantum)");
    println!("  ✓ QUIC/HTTP3 support");
    println!("  ✓ Compression: Brotli");
    println!("  ✓ Connection ID: 8 bytes");
    println!("  ✓ ALPN protocols: h3, h2, http/1.1\n");

    // Fingerprint variations
    println!("=== Chrome 133 Fingerprint Variations ===\n");

    println!("| Version       | Session Type    | Extensions                           |");
    println!("|---------------|-----------------|--------------------------------------|");
    println!("| chrome_133    | Initial (1-RTT) | Standard (no PSK/Early Data)        |");
    println!("| chrome_133_PSK| Resumption      | With: PSK, PSKKeyExchangeModes      |");
    println!("| chrome_133_   | Early Data (0RT)| With: EarlyData                     |");
    println!("| 0RTT          |                 |                                      |");
    println!("| chrome_133_   | Both            | With: PSK, PSKKeyExchangeModes,     |");
    println!("| PSK_0RTT      |                 |       EarlyData                    |");

    println!("\n=== Implementation Notes ===\n");

    println!("PSK Object Structure:");
    println!("  PreSharedKeyExtension {{");
    println!("    identities: Vec<Vec<u8>>,  // Session tickets");
    println!("    binders: Vec<Vec<u8>>,      // HMAC signatures");
    println!("  }}\n");

    println!("Early Data Object:");
    println!("  EarlyDataExtension {{");
    println!("    max_size: u32,  // Maximum bytes, typically 16KB");
    println!("  }}\n");

    println!("Key Exchange Modes:");
    println!("  PSK_MODE_DHE = 0x01  // PSK with (EC)DHE");
    println!("  PSK_MODE_KEM = 0x02  // PSK with KEM (Post-Quantum)\n");

    println!("=== Security Considerations ===\n");

    println!("PSK Advantages:");
    println!("  ✓ Faster connection resumption");
    println!("  ✓ Reduced latency");
    println!("  ✓ Smaller handshake size");
    println!("  ⚠ Known plaintext attacks if keys are compromised\n");

    println!("0-RTT Advantages:");
    println!("  ✓ Zero round-trip latency for application data");
    println!("  ✓ Improved user experience");
    println!("  ⚠ Vulnerable to replay attacks");
    println!("  ⚠ Should only be used for idempotent operations (GET, not POST)\n");

    println!("=== Real-World Usage ===\n");

    println!("When Chrome uses PSK:");
    println!("  1. Client connects normally -> Server sends session ticket");
    println!("  2. Client stores ticket locally");
    println!("  3. Client reconnects to same server within ticket lifetime");
    println!("  4. Client sends PSK in ClientHello");
    println!("  5. Server validates binder, derives session key");
    println!("  6. Connection established in <1-RTT\n");

    println!("When Chrome uses 0-RTT:");
    println!("  1. Client has PSK from previous connection");
    println!("  2. Client sends request + PSK + EarlyData in same packet");
    println!("  3. Server receives and immediately processes early data");
    println!("  4. 0 additional round trips needed for GET requests\n");

    println!("Fingerprinting Benefits:");
    println!("  ✓ Identify session reuse patterns");
    println!("  ✓ Detect fast 0-RTT clients");
    println!("  ✓ Understand connection optimization strategies");
    println!("  ✓ Analyze TLS usage lifecycle");
}
