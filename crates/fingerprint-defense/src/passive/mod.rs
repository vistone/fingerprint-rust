//! passivefingerprintidentifymodule
//!
//! implement p0f stylepassivefingerprintidentify，include TCP、HTTP、TLS analysis。

pub mod consistency;
pub mod http;
pub mod p0f;
pub mod p0f_parser;
pub mod packet;
pub mod tcp;
pub mod tls;

pub use consistency::ConsistencyAnalyzer;

pub use http::{HttpAnalyzer, HttpFingerprint};
pub use packet::{Packet, PacketParser};
pub use tcp::{TcpAnalyzer, TcpFeatures, TcpFingerprint};
pub use tls::{TlsAnalyzer, TlsFingerprint};

// use core insystemlevelabstract
use fingerprint_core::system::{NetworkFlow, ProtocolType, SystemContext, TrafficDirection};

/// passiveanalysiser（多protocol）
pub struct PassiveAnalyzer {
 tcp_analyzer: TcpAnalyzer,
 http_analyzer: HttpAnalyzer,
 tls_analyzer: TlsAnalyzer,
}

impl PassiveAnalyzer {
 /// Create a newpassiveanalysiser
 pub fn new() -> Result<Self, PassiveError> {
 Ok(Self {
 tcp_analyzer: TcpAnalyzer::new().map_err(PassiveError::Tcp)?,
 http_analyzer: HttpAnalyzer::new().map_err(PassiveError::Http)?,
 tls_analyzer: TlsAnalyzer::new().map_err(PassiveError::Tls)?,
 })
 }

 /// analysiscountpacket
 pub fn analyze(&self, packet: &Packet) -> AnalysisResult {
 let mut result = AnalysisResult::default();

 // TCP analysis
 if let Some(tcp_result) = self.tcp_analyzer.analyze(packet) {
 result.tcp = Some(tcp_result);
 }

 // HTTP analysis
 if let Some(http_result) = self.http_analyzer.analyze(packet) {
 result.http = Some(http_result);
 }

 // TLS analysis
 if let Some(tls_result) = self.tls_analyzer.analyze(packet) {
 result.tls = Some(tls_result);
 }

 result
 }

 /// analysiscountpacket并return NetworkFlow（新method， for systemlevelprotection）
 pub fn analyze_to_flow(&self, packet: &Packet) -> Result<NetworkFlow, PassiveError> {
 // 1. determineprotocoltype
 let protocol = match (
 packet.tcp_header.is_some(),
 packet.src_port,
 packet.dst_port,
 ) {
 (true, 80, _) | (true, _, 80) => ProtocolType::Http,
 (true, 443, _) | (true, _, 443) => ProtocolType::Https,
 (true, _, _) => ProtocolType::Tcp,
 (false, 53, _) | (false, _, 53) => ProtocolType::Udp, // simple DNS identify
 (false, _, _) if packet.src_port > 0 || packet.dst_port > 0 => ProtocolType::Udp,
 _ => ProtocolType::Icmp,
 };

 // 2. Create SystemContext
 let mut context = SystemContext::with_ports(
 packet.src_ip,
 packet.dst_ip,
 packet.src_port,
 packet.dst_port,
 protocol,
 );

 // settingsotherupdown文info
 context.timestamp = chrono::Utc::now();
 context.packet_size = packet.payload.len();

 // 智能方向identify： if is privateaddress发往公网，usually is Outbound；反之 is Inbound
 // herelogiccanBased on部署environment（gateway vs final端）进一步微调
 let src_is_local = match packet.src_ip {
 std::net::IpAddr::V4(ip) => ip.is_loopback() || ip.is_private(),
 std::net::IpAddr::V6(ip) => ip.is_loopback(),
 };
 context.direction = if src_is_local {
 TrafficDirection::Outbound
 } else {
 TrafficDirection::Inbound
 };

 // 3. calloriginal analyze methodGetfingerprint
 let analysis_result = self.analyze(packet);

 // 4. Create NetworkFlow
 let mut flow = NetworkFlow::new(context);

 // 5. Updatetraffictrait
 flow.update_characteristics(packet.payload.len());

 // 6. paddingfingerprint
 if let Some(tcp) = analysis_result.tcp {
 flow.add_fingerprint(Box::new(tcp));
 }
 if let Some(http) = analysis_result.http {
 flow.add_fingerprint(Box::new(http));
 }
 if let Some(tls) = analysis_result.tls {
 flow.add_fingerprint(Box::new(tls));
 }

 Ok(flow)
 }
}

/// analysisresult
#[derive(Debug, Clone, Default)]
pub struct AnalysisResult {
 pub tcp: Option<TcpFingerprint>,
 pub http: Option<HttpFingerprint>,
 pub tls: Option<TlsFingerprint>,
}

// exportalias
pub use AnalysisResult as PassiveAnalysisResult;

/// passiveanalysiserror
#[derive(Debug, thiserror::Error)]
pub enum PassiveError {
 #[error("TCP analysiserror: {0}")]
 Tcp(String),

 #[error("HTTP analysiserror: {0}")]
 Http(String),

 #[error("TLS analysiserror: {0}")]
 Tls(String),

 #[error("countpacketParseerror: {0}")]
 Packet(#[from] crate::passive::packet::PacketError),
}

impl Default for PassiveAnalyzer {
 fn default() -> Self {
 Self::new().expect("Failed to create PassiveAnalyzer")
 }
}
