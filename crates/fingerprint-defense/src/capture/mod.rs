//! countpacketcapturemodule
//!
//! use纯 Rust implement from networkinterface or file实 when capturecountpacket（无systemdepend）。

use crate::passive::{PacketParser, PassiveAnalyzer};
use pnet::datalink::{self, Channel, NetworkInterface};
use std::sync::Arc;

/// capture引擎
pub struct CaptureEngine {
 analyzer: Arc<PassiveAnalyzer>,
}

impl CaptureEngine {
 /// Create a newcapture引擎
 pub fn new(analyzer: Arc<PassiveAnalyzer>) -> Self {
 Self { analyzer }
 }

 /// from specified网卡start实 when capture
 pub async fn start_live(&self, device_name: &str) -> Result<(), String> {
 // findspecifiednetworkinterface
 let interface = datalink::interfaces()
.into_iter()
.find(|iface| iface.name == device_name)
.ok_or_else(|| format!("找不 to networkinterface: {}", device_name))?;

 println!("[Capture] Listening on device: {}", device_name);

 let analyzer = self.analyzer.clone();

 // use spawn_blocking because pnet receive is 阻塞的
 tokio::task::spawn_blocking(move || {
 Self::capture_from_interface(interface, analyzer)
 });

 Ok(())
 }

 /// from networkinterfacecapturecountpacket（阻塞式）
 fn capture_from_interface(
 interface: NetworkInterface,
 analyzer: Arc<PassiveAnalyzer>,
 ) -> Result<(), String> {
 // Createcountdata链路通道
 let (_tx, mut rx) = match datalink::channel(&interface, Default::default()) {
 Ok(Channel::Ethernet(tx, rx)) => (tx, rx),
 Ok(_) => return Err("不support通道type".to_string()),
 Err(e) => return Err(format!("Create通道failure: {}", e)),
 };

 // loopreceivecountpacket
 loop {
 match rx.next() {
 Ok(packet) => {
 // securityCheck：limitmaximumcountpacketsize以prevent DoS attack（65535 bytes = maximum IP 包）
 const MAX_PACKET_SIZE: usize = 65535;
 if packet.len() > MAX_PACKET_SIZE {
 eprintln!("[Capture] countpackettoo large，alreadyignore: {} bytes", packet.len());
 continue;
 }
 
 // skip以太网frameheader（14 bytes）
 if packet.len() > 14 {
 let ip_packet = &packet[14..];
 if let Ok(p) = PacketParser::parse(ip_packet) {
 let _ = analyzer.analyze(&p);
 }
 }
 }
 Err(e) => {
 eprintln!("[Capture] receivecountpacketerror: {}", e);
 // continuereceive，不 in 断
 }
 }
 }
 }

 /// from fileload并process
 pub fn process_file(&self, path: &str) -> Result<(), String> {
 use pcap_file::pcap::PcapReader;
 use std::fs::File;

 // open pcap file
 let file = File::open(path).map_err(|e| format!("openfilefailure: {}", e))?;
 let mut pcap_reader =
 PcapReader::new(file).map_err(|e| format!("Parse pcap filefailure: {}", e))?;

 // readallcountpacket
 let mut packet_count = 0;
 const MAX_PACKETS: usize = 1_000_000; // limitmaximumcountpacketcount以preventinsidememory exhausted
 
 while let Some(packet) = pcap_reader.next_packet() {
 // securityCheck：limitprocesscountpacketcount
 packet_count += 1;
 if packet_count > MAX_PACKETS {
 eprintln!("[Capture] already达 to maximumcountpacketprocesslimit: {}", MAX_PACKETS);
 break;
 }
 
 match packet {
 Ok(pkt) => {
 // securityCheck：limitsinglecountpacketsize
 const MAX_PACKET_SIZE: usize = 65535;
 let data = pkt.data;
 if data.len() > MAX_PACKET_SIZE {
 eprintln!("[Capture] countpackettoo large，alreadyignore: {} bytes", data.len());
 continue;
 }
 
 // pcap fileincountdatausuallyincluding以太网frameheader
 if data.len() > 14 {
 // skip以太网frameheader（14 bytes）
 let ip_packet = &data[14..];
 if let Ok(p) = PacketParser::parse(ip_packet) {
 let _ = self.analyzer.analyze(&p);
 }
 }
 }
 Err(e) => {
 eprintln!("[Capture] readcountpacketerror: {}", e);
 // continueprocessnext包
 }
 }
 }

 println!("[Capture] alreadyprocess {} countpacket", packet_count);
 Ok(())
 }
}
