#[cfg(feature = "defense")]
fn main() {
    use bytes::Bytes;
    use chrono::Utc;
    use fingerprint_core::system::{NetworkFlow, ProtocolType, SystemContext, TrafficDirection};
    use fingerprint_core::Fingerprint;
    use fingerprint_defense::passive::consistency::ConsistencyAnalyzer;
    use fingerprint_defense::passive::http::HttpAnalyzer;
    use fingerprint_defense::passive::packet::{Packet, TcpHeader, TcpOption};
    use fingerprint_defense::passive::tcp::TcpAnalyzer;

    println!("🛡️  Consistency Analyzer & JA4H Verification\n");

    let analyzer = ConsistencyAnalyzer::new();
    let http_analyzer = HttpAnalyzer::new().expect("Failed to create HttpAnalyzer");
    let tcp_analyzer = TcpAnalyzer::new().expect("Failed to create TcpAnalyzer");

    // 1. simulateannormal Chrome request (Windows)
    println!("1️⃣  simulatenormal Chrome request (Windows):");
    let raw_http = b"GET / HTTP/1.1\r\nUser-Agent: Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/136.0.0.0 Safari/537.36\r\nAccept: text/html\r\n\r\n";

    let packet = Packet {
        timestamp: 0,
        src_ip: "192.168.1.100".parse().unwrap(),
        dst_ip: "10.0.0.1".parse().unwrap(),
        src_port: 54321,
        dst_port: 80, // normal HTTP 80 port
        ip_version: 4,
        ttl: 128, // Windows default TTL
        ip_flags: 0,
        payload: Bytes::copy_from_slice(raw_http),
        tcp_header: Some(TcpHeader {
            seq: 1,
            ack: None,
            window: 64240,
            flags: 0x02, // SYN
            options: vec![
                TcpOption {
                    kind: 2,
                    data: vec![0x05, 0xb4],
                }, // MSS 1460
                TcpOption {
                    kind: 3,
                    data: vec![0x08],
                }, // Window Scale 8
            ],
        }),
    };

    let mut flow = NetworkFlow::new(SystemContext {
        source_ip: packet.src_ip,
        target_ip: packet.dst_ip,
        source_port: Some(packet.src_port),
        target_port: Some(packet.dst_port),
        protocol: ProtocolType::Http,
        timestamp: Utc::now(),
        interface: None,
        packet_size: packet.payload.len(),
        direction: TrafficDirection::Inbound,
    });

    if let Some(h_fp) = http_analyzer.analyze(&packet) {
        println!("   ✅ JA4H: {:?}", h_fp.metadata.get("ja4h"));
        flow.add_fingerprint(Box::new(h_fp));
    }

    if let Some(t_fp) = tcp_analyzer.analyze(&packet) {
        println!("   ✅ TCP identify: {}", Fingerprint::id(&t_fp));
        flow.add_fingerprint(Box::new(t_fp));
    }

    let report = analyzer.analyze_flow(&flow);
    println!("   ✅ consistencyscore: {}", report.score);
    if report.discrepancies.is_empty() {
        println!("   ✅ notdiscoverabnormal，determine为合法traffic");
    } else {
        for d in &report.discrepancies {
            println!("   ❌ discover偏差: {}", d);
        }
    }

    // 2. simulatean伪造fingerprint的机器人 (UA 为 Windows, but TCP trait为 Linux)
    println!("\n2️⃣  simulatefingerprint错bitattack (User-Agent: Windows, TCP: Linux):");
    let packet_bot = Packet {
        timestamp: 0,
        src_ip: "192.168.1.101".parse().unwrap(),
        dst_ip: "10.0.0.1".parse().unwrap(),
        src_port: 54322,
        dst_port: 80,
        ip_version: 4,
        ttl: 64, // Linux default TTL
        ip_flags: 0,
        payload: Bytes::copy_from_slice(raw_http),
        tcp_header: Some(TcpHeader {
            seq: 1,
            ack: None,
            window: 65535,
            flags: 0x02,
            options: vec![
                TcpOption {
                    kind: 2,
                    data: vec![0x05, 0xb4],
                }, // MSS 1460
                TcpOption {
                    kind: 3,
                    data: vec![0x07],
                }, // Window Scale 7
            ],
        }),
    };

    let mut flow_bot = NetworkFlow::new(SystemContext {
        source_ip: packet_bot.src_ip,
        target_ip: packet_bot.dst_ip,
        source_port: Some(packet_bot.src_port),
        target_port: Some(packet_bot.dst_port),
        protocol: ProtocolType::Http,
        timestamp: Utc::now(),
        interface: None,
        packet_size: packet_bot.payload.len(),
        direction: TrafficDirection::Inbound,
    });

    if let Some(h_fp) = http_analyzer.analyze(&packet_bot) {
        flow_bot.add_fingerprint(Box::new(h_fp));
    }

    if let Some(t_fp) = tcp_analyzer.analyze(&packet_bot) {
        println!("   ✅ TCP identify (机器人): {}", Fingerprint::id(&t_fp));
        flow_bot.add_fingerprint(Box::new(t_fp));
    }

    let report_bot = analyzer.analyze_flow(&flow_bot);
    println!("   ⚠️  consistencyscore: {}", report_bot.score);
    for d in &report_bot.discrepancies {
        println!("   ❌ discover严重偏差: {}", d);
    }
    if report_bot.bot_detected {
        println!("   🚨 warning: detect to 机器人/fingerprintsimulate器behavior！");
    }

    println!("\n✨ Validatecomplete！");
}

#[cfg(not(feature = "defense"))]
fn main() {
    println!("Please enable the 'defense' feature to run this example.");
}
