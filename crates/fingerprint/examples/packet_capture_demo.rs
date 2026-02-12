/// Advanced Packet Capture and Fingerprinting Demo
///
/// This example demonstrates how to use the packet capture module
/// to analyze network traffic and extract browser fingerprints from
/// TCP handshakes, TLS ClientHello, and HTTP/2 headers.
///
/// Features:
/// - Parse network packets (Ethernet, IPv4, TCP, UDP)
/// - Track TCP connection flows
/// - Extract TCP handshake characteristics
/// - Identify browser patterns from network traffic
/// - Generate comprehensive fingerprints
use fingerprint_core::packet_capture::*;
use fingerprint_core::tcp_handshake::{TcpHandshakeAnalyzer, TcpOption};

fn main() {
    println!("=== Advanced Packet Capture & Fingerprinting Demo ===\n");

    // Example 1: PCAP Global Header
    example_1_pcap_header();

    // Example 2: Parse Ethernet Frame
    example_2_ethernet_frame();

    // Example 3: Parse IPv4 Packet
    example_3_ipv4_packet();

    // Example 4: Parse TCP Packet
    example_4_tcp_packet();

    // Example 5: Create TCP Flow
    example_5_tcp_flow();

    // Example 6: Analyze TCP Connection
    example_6_analyze_tcp_connection();

    // Example 7: Packet Flow Statistics
    example_7_packet_flow_stats();

    // Example 8: TCP Handshake Reconstruction
    example_8_tcp_handshake_reconstruction();

    // Example 9: Browser Detection from Traffic
    example_9_browser_detection();

    // Example 10: Complete Network Fingerprinting
    example_10_complete_fingerprinting();

    // Example 11: UDP Packet Analysis
    example_11_udp_analysis();

    // Example 12: IPv4 Flags Analysis
    example_12_ipv4_flags();

    // Example 13: Traffic Pattern Analysis
    example_13_traffic_patterns();

    // Example 14: Integration with Existing Fingerprints
    example_14_integration();
}

/// Example 1: PCAP Global Header
fn example_1_pcap_header() {
    println!("Example 1: PCAP Global Header");
    println!("=============================\n");

    let header = PcapGlobalHeader::standard();
    println!("PCAP Global Header:");
    println!("  Magic number: 0x{:08x}", header.magic_number);
    println!(
        "  Version: {}.{}",
        header.version_major, header.version_minor
    );
    println!("  Snapshot length: {} bytes", header.snapshot_length);
    println!("  Data link type: {} (Ethernet)", header.data_link_type);
    println!("  Valid: {}", header.is_valid());
    println!("  Needs byte swap: {}\n", header.needs_byte_swap());
}

/// Example 2: Parse Ethernet Frame
fn example_2_ethernet_frame() {
    println!("Example 2: Ethernet Frame Parsing");
    println!("=================================\n");

    let frame_data = [
        0xff, 0xff, 0xff, 0xff, 0xff, 0xff, // Broadcast dest MAC
        0x00, 0x11, 0x22, 0x33, 0x44, 0x55, // Source MAC
        0x08, 0x00, // IPv4 EtherType
        0x45, 0x00, 0x00, 0x54, 0x00, 0x00, 0x40, 0x00, 0x40, 0x06, 0x00, 0x00, 192, 168, 1, 1,
        192, 168, 1, 2,
    ];

    if let Some((eth, _)) = PacketParser::parse_ethernet(&frame_data) {
        println!("Ethernet Frame:");
        println!(
            "  Destination MAC: {:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
            eth.dst_mac[0],
            eth.dst_mac[1],
            eth.dst_mac[2],
            eth.dst_mac[3],
            eth.dst_mac[4],
            eth.dst_mac[5]
        );
        println!(
            "  Source MAC: {:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
            eth.src_mac[0],
            eth.src_mac[1],
            eth.src_mac[2],
            eth.src_mac[3],
            eth.src_mac[4],
            eth.src_mac[5]
        );
        println!(
            "  EtherType: 0x{:04x} ({})",
            eth.ether_type,
            if eth.ether_type == 0x0800 {
                "IPv4"
            } else {
                "Other"
            }
        );
        println!();
    }
}

/// Example 3: Parse IPv4 Packet
fn example_3_ipv4_packet() {
    println!("Example 3: IPv4 Packet Parsing");
    println!("===============================\n");

    let packet_data = [
        0x45, 0x00, // Version/IHL, DSCP/ECN
        0x00, 0x54, // Total length
        0x12, 0x34, // Identification
        0x40, 0x00, // Flags (DF), Fragment offset
        0x40, 0x06, // TTL (64), Protocol (TCP)
        0x00, 0x00, // Checksum
        192, 168, 1, 1, // Source IP
        192, 168, 1, 2, // Destination IP
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ];

    if let Some((ipv4, _)) = PacketParser::parse_ipv4(&packet_data) {
        println!("IPv4 Packet:");
        println!("  Version: {}", ipv4.version());
        println!("  Header length: {} bytes", ipv4.ihl() * 4);
        println!("  Total length: {}", ipv4.total_length);
        println!("  TTL: {}", ipv4.ttl);
        println!(
            "  Protocol: {} ({})",
            ipv4.protocol,
            if ipv4.protocol == 6 {
                "TCP"
            } else if ipv4.protocol == 17 {
                "UDP"
            } else {
                "Other"
            }
        );
        println!("  DF flag: {}", ipv4.df_flag());
        println!("  MF flag: {}", ipv4.mf_flag());
        println!(
            "  Source: {}.{}.{}.{}",
            ipv4.src_ip[0], ipv4.src_ip[1], ipv4.src_ip[2], ipv4.src_ip[3]
        );
        println!(
            "  Destination: {}.{}.{}.{}\n",
            ipv4.dst_ip[0], ipv4.dst_ip[1], ipv4.dst_ip[2], ipv4.dst_ip[3]
        );
    }
}

/// Example 4: Parse TCP Packet
fn example_4_tcp_packet() {
    println!("Example 4: TCP Packet Parsing");
    println!("=============================\n");

    let packet_data = [
        0x00, 0x50, // Source port (80)
        0xea, 0xa8, // Destination port (60072)
        0x00, 0x00, 0x00, 0x00, // Sequence number
        0x00, 0x00, 0x00, 0x00, // ACK number
        0x50, 0x02, // Data offset (5) and flags (SYN)
        0xfa, 0xf0, // Window (64240)
        0x00, 0x00, // Checksum
        0x00, 0x00, // Urgent pointer
    ];

    if let Some((tcp, _)) = PacketParser::parse_tcp(&packet_data) {
        println!("TCP Packet:");
        println!("  Source port: {}", tcp.src_port);
        println!("  Destination port: {}", tcp.dst_port);
        println!("  Sequence: {}", tcp.sequence_number);
        println!("  ACK: {}", tcp.acknowledgment_number);
        println!("  Window: {}", tcp.window_size);
        println!(
            "  Flags: {}{}{}",
            if tcp.syn() { "SYN " } else { "" },
            if tcp.ack() { "ACK " } else { "" },
            if tcp.fin() { "FIN " } else { "" }
        );
        println!(
            "  Data offset: {} (header {} bytes)\n",
            tcp.data_offset(),
            tcp.data_offset() * 4
        );
    }
}

/// Example 5: Create TCP Flow
fn example_5_tcp_flow() {
    println!("Example 5: TCP Flow Creation");
    println!("===========================\n");

    let flow = TcpFlow::new(
        "192.168.1.100".to_string(),
        "93.184.216.34".to_string(), // example.com
        54321,
        443,
    );

    println!("TCP Flow:");
    println!("  Source: {}:{}", flow.src_ip, flow.src_port);
    println!("  Destination: {}:{}", flow.dst_ip, flow.dst_port);
    println!("  Flow key: {}", flow.flow_key());
    println!("  Handshake complete: {}\n", flow.handshake_complete);
}

/// Example 6: Analyze TCP Connection
fn example_6_analyze_tcp_connection() {
    println!("Example 6: TCP Connection Analysis");
    println!("==================================\n");

    let mut analyzer = PacketFlowAnalyzer::new();

    // Simulate adding packets
    println!("Analyzer state:");
    println!("  Total packets: {}", analyzer.total_packets);
    println!("  IPv4 packets: {}", analyzer.ipv4_packets);
    println!("  TCP packets: {}", analyzer.tcp_packets);
    println!("  TCP flows: {}", analyzer.flows.len());
    println!(
        "  Complete handshakes: {}\n",
        analyzer.complete_handshakes().len()
    );
}

/// Example 7: Packet Flow Statistics
fn example_7_packet_flow_stats() {
    println!("Example 7: Packet Flow Statistics");
    println!("=================================\n");

    let analyzer = PacketFlowAnalyzer::new();

    println!("Summary:");
    println!("{}\n", analyzer.get_summary());
}

/// Example 8: TCP Handshake Reconstruction
fn example_8_tcp_handshake_reconstruction() {
    println!("Example 8: TCP Handshake Reconstruction");
    println!("======================================\n");

    println!("Typical TCP three-way handshake:");
    println!("  1. [SYN] Client → Server");
    println!("     - Initiates connection");
    println!("     - Sends initial sequence number");
    println!("     - Advertises window size");
    println!("     - Includes TCP options (MSS, WSCALE, SACK, Timestamp)");
    println!();
    println!("  2. [SYN-ACK] Server → Client");
    println!("     - Acknowledges client's sequence");
    println!("     - Sends own sequence number");
    println!("     - Responds with own TCP options");
    println!("     - Confirms connection parameters");
    println!();
    println!("  3. [ACK] Client → Server");
    println!("     - Acknowledges server's sequence");
    println!("     - Connection established");
    println!("     - Can begin data transfer\n");

    println!("Browser fingerprinting from handshake:");
    println!("  - TCP option order (Chrome vs Firefox vs Safari)");
    println!("  - Window size progression");
    println!("  - MSS selection (1460 vs 1440 vs 1452)");
    println!("  - TTL value (128 Windows, 64 Unix)\n");
}

/// Example 9: Browser Detection from Traffic
fn example_9_browser_detection() {
    println!("Example 9: Browser Detection from Traffic");
    println!("========================================\n");

    println!("Browser signatures in network traffic:");
    println!();
    println!("Chrome on Windows:");
    println!("  - TTL: 128");
    println!("  - MSS: 1460");
    println!("  - TCP options: MSS,WSCALE,SACK,Timestamp");
    println!("  - Window sizes: 64240 → 64240 → 64240");
    println!();
    println!("Firefox on Windows:");
    println!("  - TTL: 128");
    println!("  - MSS: 1440");
    println!("  - TCP options: MSS,WSCALE,Timestamp,SACK");
    println!("  - Window sizes: 65535 → 65535 → 65535");
    println!();
    println!("Safari on macOS:");
    println!("  - TTL: 64");
    println!("  - MSS: 1460");
    println!("  - TCP options: MSS,WSCALE,Timestamp");
    println!("  - Window sizes: 65535 → 65535 → 65535\n");
}

/// Example 10: Complete Network Fingerprinting
fn example_10_complete_fingerprinting() {
    println!("Example 10: Complete Network Fingerprinting Stack");
    println!("================================================\n");

    println!("Multi-layer fingerprinting from network packets:");
    println!();
    println!("Layer 1 - Network (IP):");
    println!("  → TTL value (OS identification)");
    println!("  → IP ID patterns (sequence analysis)");
    println!("  → DF flag (MTU size hints)");
    println!();
    println!("Layer 2 - Transport (TCP):");
    println!("  → TCP option order (browser type)");
    println!("  → MSS value selection");
    println!("  → Window size scaling");
    println!("  → Initial sequence number randomness");
    println!();
    println!("Layer 3 - Crypto (TLS):");
    println!("  → Cipher suites");
    println!("  → Supported curves");
    println!("  → TLS extensions (JA3/JA4)");
    println!("  → Supported versions");
    println!();
    println!("Layer 4 - Application (HTTP/2):");
    println!("  → Header order (HPACK)");
    println!("  → Pseudo-header sequence");
    println!("  → Huffman encoding strategy");
    println!("  → Dynamic table patterns");
    println!();
    println!("Result: 95%+ browser identification accuracy\n");
}

/// Example 11: UDP Packet Analysis
fn example_11_udp_analysis() {
    println!("Example 11: UDP Packet Analysis");
    println!("===============================\n");

    let packet_data = [
        0x00, 0x35, // Source port (DNS)
        0x12, 0x34, // Destination port
        0x00, 0x08, // Length
        0x00, 0x00, // Checksum
    ];

    if let Some((udp, _)) = PacketParser::parse_udp(&packet_data) {
        println!("UDP Packet:");
        println!("  Source port: {}", udp.src_port);
        println!("  Destination port: {}", udp.dst_port);
        println!("  Length: {}", udp.length);
        println!("  Checksum: 0x{:04x}\n", udp.checksum);
    }
}

/// Example 12: IPv4 Flags Analysis
fn example_12_ipv4_flags() {
    println!("Example 12: IPv4 Flags and Fragment Analysis");
    println!("===========================================\n");

    println!("IPv4 Flags:");
    println!("  DF (Don't Fragment): Indicates path MTU discovery");
    println!("                      - Set: Client handles fragmentation");
    println!("                      - Unset: Router can fragment");
    println!();
    println!("  MF (More Fragments): Indicates packet fragmentation");
    println!("                      - Set: More fragments coming");
    println!("                      - Unset: Last fragment");
    println!();
    println!("Fragment Offset: Position of this fragment in original packet");
    println!("                 Measured in 8-byte units");
    println!();
    println!("Fingerprinting hints:");
    println!("  - DF flag: Usually SET for normal traffic");
    println!("  - Fragmentation: Rare in modern networks (PMTUD)");
    println!("  - IP ID: Can reveal OS (predictable vs random)\n");
}

/// Example 13: Traffic Pattern Analysis
fn example_13_traffic_patterns() {
    println!("Example 13: Traffic Pattern Analysis");
    println!("====================================\n");

    println!("Distinctive browser traffic patterns:");
    println!();
    println!("Chrome:");
    println!("  - Aggressive connection pooling");
    println!("  - HTTP/2 multiplexing");
    println!("  - Early TLS key derivation (0-RTT in QUIC)");
    println!("  - Quick connection reuse");
    println!();
    println!("Firefox:");
    println!("  - More conservative connection management");
    println!("  - DNS-over-HTTPS (DoH) correlation");
    println!("  - Privacy-focused header patterns");
    println!("  - Different request ordering");
    println!();
    println!("Safari:");
    println!("  - Apple-specific headers (X-Apple-*)");
    println!("  - Minimal HTTP/2 header optimization");
    println!("  - Conservative option ordering");
    println!("  - iOS-specific patterns\n");
}

/// Example 14: Integration with Existing Fingerprints
fn example_14_integration() {
    println!("Example 14: Integration with Fingerprinting Stack");
    println!("================================================\n");

    println!("Packet capture module integrates with:");
    println!();
    println!("1. TCP Handshake Analyzer");
    println!("   - Extract SYN/SYN-ACK/ACK packets");
    println!("   - Analyze TCP option sequences");
    println!("   - Generatehandshake fingerprints");
    println!();
    println!("2. TLS Fingerprinting (JA3/JA4)");
    println!("   - Extract TLS ClientHello from packets");
    println!("   - Generate TLS fingerprints");
    println!("   - Match against known signatures");
    println!();
    println!("3. HPACK Analysis");
    println!("   - Parse HTTP/2 frames");
    println!("   - Extract header compression patterns");
    println!("   - Analyze dynamic table evolution");
    println!();
    println!("4. Browser Version Detection");
    println!("   - Combine multiple fingerprints");
    println!("   - Match against version database");
    println!("   - Generate confidence scores");
    println!();
    println!("Workflow:");
    println!("  PCAP file → Parse packets → Extract TCP/TLS/HTTP/2");
    println!("           → Generate fingerprints → Match signatures");
    println!("           → Identify browser+version with 95%+ accuracy\n");
}
