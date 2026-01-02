#[cfg(feature = "defense")]
use bytes::Bytes;
#[cfg(feature = "defense")]
use fingerprint_core::fingerprint::Fingerprint;
#[cfg(feature = "defense")]
use fingerprint_defense::{
 passive::packet::{Packet, TcpHeader},
 FingerprintDatabase, PassiveAnalyzer, SelfLearningAnalyzer,
};
#[cfg(feature = "defense")]
use std::sync::Arc;

#[cfg(feature = "defense")]
#[tokio::main]
async fn main() {
 println!("🚀 Advanced Fingerprinting & Learner Verification\n");

 #[allow(clippy::arc_with_non_send_sync)] // in Examplescode in use，不needcrossthread
 let db = Arc::new(FingerprintDatabase::open("advanced.db").expect("Failed to open DB"));
 let analyzer = Arc::new(PassiveAnalyzer::new().expect("Failed to create analyzer"));
 let learner = SelfLearningAnalyzer::new(db.clone());

 // 1. simulatean HTTP/2 connectionfront奏 and SETTINGS frame
 println!("1️⃣ simulate HTTP/2 handshaketrait:");
 let h2_preface = b"PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n";
 let h2_settings = vec![
 0x00, 0x00, 0x0c, // Length 12
 0x04, // Type SETTINGS
 0x00, // Flags
 0x00, 0x00, 0x00, 0x00, // Stream 0
 0x00, 0x03, 0x00, 0x00, 0x00, 0x64, // MAX_CONCURRENT_STREAMS = 100
 0x00, 0x04, 0x00, 0x01, 0x00, 0x00, // INITIAL_WINDOW_SIZE = 65536
 ];

 let mut payload = h2_preface.to_vec();
 payload.extend_from_slice(&h2_settings);

 let packet = Packet {
 timestamp: 0,
 src_ip: "1.2.3.4".parse().unwrap(),
 dst_ip: "8.8.8.8".parse().unwrap(),
 src_port: 12345,
 dst_port: 443,
 ip_version: 4,
 ttl: 64,
 ip_flags: 0,
 payload: Bytes::from(payload),
 tcp_header: Some(TcpHeader {
 seq: 1,
 ack: None,
 window: 65535,
 flags: 0x02,
 options: vec![],
 }),
 };

 let result = analyzer.analyze(&packet);
 if let Some(http) = &result.http {
 println!(" ✅ HTTP Parsesuccess (Version: {})", http.id());
 if let Some(settings) = &http.h2_settings {
 println!(" ✅ H2 SETTINGS capture: {:?}", settings);
 }
 }

 // 2. simulate自learnprocess
 println!("\n2️⃣ 自learnmechanismValidate:");
 for i in 1..=12 {
 learner.process_result(&result);
 if i == 5 {
 println!("... observenot知fingerprint 5 次");
 }
 if i == 10 {
 println!("... observenot知fingerprint 10 次 (triggerlearning thresholdvalue)");
 }
 }

 println!("\n✨ highlevelFeaturesValidatecomplete！");
}

#[cfg(not(feature = "defense"))]
fn main() {
 println!("Please enable 'defense' feature.");
}
