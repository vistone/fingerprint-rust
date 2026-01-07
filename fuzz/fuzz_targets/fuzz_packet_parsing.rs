#![no_main]

use libfuzzer_sys::fuzz_target;
use fingerprint_defense::PacketParser;

fuzz_target!(|data: &[u8]| {
    // Attempt to parse as generic packet (IPv4 or IPv6)
    let _ = PacketParser::parse(data);
    
    // Additional targeted parsing attempts for better coverage
    if data.len() >= 20 {
        // Try IPv4 specifically
        let mut ipv4_data = vec![0x45]; // Version 4, IHL 5
        ipv4_data.extend_from_slice(data);
        let _ = PacketParser::parse(&ipv4_data[..20.min(ipv4_data.len())]);
    }
    
    if data.len() >= 40 {
        // Try IPv6 specifically
        let mut ipv6_data = vec![0x60]; // Version 6
        ipv6_data.extend_from_slice(data);
        let _ = PacketParser::parse(&ipv6_data[..40.min(ipv6_data.len())]);
    }
});
