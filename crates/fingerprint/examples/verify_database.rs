#[cfg(feature = "defense")]
fn main() {
 use bytes::Bytes;
 use chrono::Utc;
 use fingerprint_core::system::{NetworkFlow, ProtocolType, SystemContext, TrafficDirection};
 // use fingerprint_core::Fingerprint; // notuse，alreadycomment
 use fingerprint_defense::passive::consistency::ConsistencyAnalyzer;
 use fingerprint_defense::passive::http::HttpAnalyzer;
 use fingerprint_defense::passive::packet::{Packet, TcpHeader, TcpOption};
 use fingerprint_defense::passive::tcp::TcpAnalyzer;
 use fingerprint_defense::FingerprintDatabase;
 use std::fs;

 println!("🗄️ Fingerprint Database & Persistence Verification\n");

 let db_path = "fingerprints.db";
 let _ = fs::remove_file(db_path); // cleanup旧countdata

 let db = FingerprintDatabase::open(db_path).expect("Failed to open database");
 let analyzer = ConsistencyAnalyzer::new();
 let http_analyzer = HttpAnalyzer::new().expect("Failed to create HttpAnalyzer");
 let tcp_analyzer = TcpAnalyzer::new().expect("Failed to create TcpAnalyzer");

 // 1. simulate并storenormal Chrome traffic
 println!("1️⃣ storenormaltraffic:");
 let raw_http = b"GET / HTTP/1.1\r\nUser-Agent: Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/136.0.0.0 Safari/537.36\r\n\r\n";
 let packet = Packet {
 timestamp: 0,
 src_ip: "192.168.1.100".parse().unwrap(),
 dst_ip: "10.0.0.1".parse().unwrap(),
 src_port: 54321,
 dst_port: 80,
 ip_version: 4,
 ttl: 128,
 ip_flags: 0,
 payload: Bytes::copy_from_slice(raw_http),
 tcp_header: Some(TcpHeader {
 seq: 1,
 ack: None,
 window: 64240,
 flags: 0x02,
 options: vec![TcpOption {
 kind: 2,
 data: vec![0x05, 0xb4],
 }],
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
 flow.add_fingerprint(Box::new(h_fp));
 }
 if let Some(t_fp) = tcp_analyzer.analyze(&packet) {
 flow.add_fingerprint(Box::new(t_fp));
 }

 let report = analyzer.analyze_flow(&flow);
 db.store_flow(&flow, report.score, report.bot_detected)
.expect("Failed to store flow");
 println!(" ✅ trafficalreadystore into SQLite");

 // 2. simulate并store机er人traffic
 println!("\n2️⃣ store机er人traffic:");
 let packet_bot = Packet {
 src_ip: "192.168.1.101".parse().unwrap(),
 ttl: 64, // Bot TTL
..packet.clone()
 };

 let mut flow_bot = NetworkFlow::new(SystemContext {
 source_ip: packet_bot.src_ip,
 timestamp: Utc::now(),
..flow.context.clone()
 });

 if let Some(h_fp) = http_analyzer.analyze(&packet_bot) {
 flow_bot.add_fingerprint(Box::new(h_fp));
 }
 if let Some(t_fp) = tcp_analyzer.analyze(&packet_bot) {
 flow_bot.add_fingerprint(Box::new(t_fp));
 }

 let report_bot = analyzer.analyze_flow(&flow_bot);
 db.store_flow(&flow_bot, report_bot.score, report_bot.bot_detected)
.expect("Failed to store bot flow");
 println!(" ✅ 机er人trafficalreadystore into SQLite");

 // 3. readstatisticsinfo
 println!("\n3️⃣ databasestatistics:");
 let stats = db.get_stats().unwrap();
 println!(" 📊 stats: {}", stats);

 println!("\n✨ databaseValidatecomplete (file: fingerprints.db)");
}

#[cfg(not(feature = "defense"))]
fn main() {
 println!("Please enable the 'defense' feature to run this example.");
}
