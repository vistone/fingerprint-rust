//! systemupdown文
//!
//! definesystemlevelprotectionupdown文info，includenetworkentity、 when between、protocol etc.。

use chrono::{DateTime, Utc};
use std::net::IpAddr;

/// trafficdirection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TrafficDirection {
 /// inputtraffic（entersystem）
 Inbound,

 /// outputtraffic（leavesystem）
 Outbound,

 /// inside部traffic（systeminside部）
 Internal,
}

impl TrafficDirection {
 /// convert tostring
 pub fn as_str(&self) -> &'static str {
 match self {
 Self::Inbound => "inbound",
 Self::Outbound => "outbound",
 Self::Internal => "internal",
 }
 }
}

impl std::fmt::Display for TrafficDirection {
 fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
 write!(f, "{}", self.as_str())
 }
}

/// protocoltype
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProtocolType {
 /// TCP protocol
 Tcp,

 /// UDP protocol
 Udp,

 /// ICMP protocol
 Icmp,

 /// HTTP protocol
 Http,

 /// HTTPS protocol（TLS over TCP）
 Https,

 /// otherprotocol
 Other(u8),
}

impl ProtocolType {
 /// from IP protocol号Create
 pub fn from_ip_protocol(protocol: u8) -> Self {
 match protocol {
 6 => Self::Tcp,
 17 => Self::Udp,
 1 => Self::Icmp,
 other => Self::Other(other),
 }
 }

 /// convert to IP protocol号
 pub fn to_ip_protocol(&self) -> u8 {
 match self {
 Self::Tcp => 6,
 Self::Udp => 17,
 Self::Icmp => 1,
 Self::Http => 6, // HTTP over TCP
 Self::Https => 6, // HTTPS over TCP
 Self::Other(p) => *p,
 }
 }

 /// convert tostring
 pub fn as_str(&self) -> &'static str {
 match self {
 Self::Tcp => "TCP",
 Self::Udp => "UDP",
 Self::Icmp => "ICMP",
 Self::Http => "HTTP",
 Self::Https => "HTTPS",
 Self::Other(_) => "Other",
 }
 }
}

impl std::fmt::Display for ProtocolType {
 fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
 match self {
 Self::Other(p) => write!(f, "Other({})", p),
 _ => write!(f, "{}", self.as_str()),
 }
 }
}

/// systemupdown文
///
/// representsystemlevelprotectionupdown文info，includingnetworktrafficcompletemetadata。
///
/// ## Core Concept
///
/// systemlevelprotectionneedconsidercompletesystemupdown文，而not onlyonly is singleservice or port：
/// - networkentitycompleteinfo（source/target IP、port）
/// - protocoltype and direction
/// - when between戳 and network interfaceinterface
/// - countpacketlevelinfo
///
/// ## Examples
///
/// ```rust
/// use fingerprint_core::system::{SystemContext, ProtocolType, TrafficDirection};
/// use std::net::IpAddr;
/// use chrono::Utc;
///
/// let ctx = SystemContext {
/// source_ip: "192.168.1.100".parse().unwrap(),
/// target_ip: "10.0.0.1".parse().unwrap(),
/// source_port: Some(54321),
/// target_port: Some(80),
/// protocol: ProtocolType::Http,
/// timestamp: Utc::now(),
/// interface: Some("eth0".to_string()),
/// packet_size: 1024,
/// direction: TrafficDirection::Inbound,
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SystemContext {
 /// source IP address
 pub source_ip: IpAddr,

 /// target IP address
 pub target_ip: IpAddr,

 /// sourceport（ for UDP/TCP）
 pub source_port: Option<u16>,

 /// targetport（ for UDP/TCP）
 pub target_port: Option<u16>,

 /// protocoltype
 pub protocol: ProtocolType,

 /// when between戳
 pub timestamp: DateTime<Utc>,

 /// network interfaceinterfacename
 pub interface: Option<String>,

 /// countpacketsize（bytes）
 pub packet_size: usize,

 /// trafficdirection（input/output）
 pub direction: TrafficDirection,
}

impl SystemContext {
 /// Create a newsystemupdown文
 pub fn new(source_ip: IpAddr, target_ip: IpAddr, protocol: ProtocolType) -> Self {
 Self {
 source_ip,
 target_ip,
 source_port: None,
 target_port: None,
 protocol,
 timestamp: Utc::now(),
 interface: None,
 packet_size: 0,
 direction: TrafficDirection::Inbound,
 }
 }

 /// Createbringportsystemupdown文
 pub fn with_ports(
 source_ip: IpAddr,
 target_ip: IpAddr,
 source_port: u16,
 target_port: u16,
 protocol: ProtocolType,
 ) -> Self {
 Self {
 source_ip,
 target_ip,
 source_port: Some(source_port),
 target_port: Some(target_port),
 protocol,
 timestamp: Utc::now(),
 interface: None,
 packet_size: 0,
 direction: TrafficDirection::Inbound,
 }
 }

 /// judgewhether as localtraffic（source or target as localaddress）
 pub fn is_local(&self) -> bool {
 self.is_source_local() || self.is_target_local()
 }

 /// judgesourceaddresswhether as localaddress
 pub fn is_source_local(&self) -> bool {
 match self.source_ip {
 IpAddr::V4(ip) => ip.is_loopback() || ip.is_private() || ip.is_link_local(),
 IpAddr::V6(ip) => ip.is_loopback() || ip.is_unspecified(),
 }
 }

 /// judgetargetaddresswhether as localaddress
 pub fn is_target_local(&self) -> bool {
 match self.target_ip {
 IpAddr::V4(ip) => ip.is_loopback() || ip.is_private() || ip.is_link_local(),
 IpAddr::V6(ip) => ip.is_loopback() || ip.is_unspecified(),
 }
 }

 /// Gettrafficuniqueidentifier符（ for 追踪）
 pub fn flow_id(&self) -> String {
 format!(
 "{}:{}->{}:{}:{}",
 self.source_ip,
 self.source_port.map(|p| p.to_string()).unwrap_or_default(),
 self.target_ip,
 self.target_port.map(|p| p.to_string()).unwrap_or_default(),
 self.protocol.as_str()
 )
 }
}

#[cfg(test)]
mod tests {
 use super::*;

 #[test]
 fn test_traffic_direction() {
 assert_eq!(TrafficDirection::Inbound.as_str(), "inbound");
 assert_eq!(TrafficDirection::Outbound.as_str(), "outbound");
 }

 #[test]
 fn test_protocol_type() {
 assert_eq!(ProtocolType::from_ip_protocol(6), ProtocolType::Tcp);
 assert_eq!(ProtocolType::from_ip_protocol(17), ProtocolType::Udp);
 assert_eq!(ProtocolType::Tcp.to_ip_protocol(), 6);
 }

 #[test]
 fn test_system_context() {
 let ctx = SystemContext::with_ports(
 "192.168.1.100".parse().unwrap(),
 "10.0.0.1".parse().unwrap(),
 54321,
 80,
 ProtocolType::Http,
 );

 assert_eq!(ctx.source_port, Some(54321));
 assert_eq!(ctx.target_port, Some(80));
 assert!(ctx.flow_id().contains("192.168.1.100"));
 }

 #[test]
 fn test_is_local() {
 let local_ctx = SystemContext::new(
 "127.0.0.1".parse().unwrap(),
 "192.168.1.1".parse().unwrap(),
 ProtocolType::Tcp,
 );
 assert!(local_ctx.is_source_local());
 assert!(local_ctx.is_local());
 }
}
