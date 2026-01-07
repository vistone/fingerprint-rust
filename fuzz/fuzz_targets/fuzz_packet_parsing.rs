#![no_main]

use libfuzzer_sys::fuzz_target;
use fingerprint_defense::PacketParser;

// IPv4 header: Version 4, IHL 5 (20 bytes minimum header)
const IPV4_HEADER_START: u8 = 0x45;
const IPV4_MIN_HEADER_LEN: usize = 20;

// IPv6 header: Version 6, fixed 40 bytes header
const IPV6_HEADER_START: u8 = 0x60;
const IPV6_HEADER_LEN: usize = 40;

fuzz_target!(|data: &[u8]| {
    // Attempt to parse as generic packet (IPv4 or IPv6)
    let _ = PacketParser::parse(data);
    
    // Additional targeted parsing attempts for better coverage
    if data.len() >= IPV4_MIN_HEADER_LEN {
        // Try IPv4 specifically
        let mut ipv4_data = vec![IPV4_HEADER_START];
        ipv4_data.extend_from_slice(data);
        let _ = PacketParser::parse(&ipv4_data[..IPV4_MIN_HEADER_LEN.min(ipv4_data.len())]);
    }
    
    if data.len() >= IPV6_HEADER_LEN {
        // Try IPv6 specifically
        let mut ipv6_data = vec![IPV6_HEADER_START];
        ipv6_data.extend_from_slice(data);
        let _ = PacketParser::parse(&ipv6_data[..IPV6_HEADER_LEN.min(ipv6_data.len())]);
    }
});
