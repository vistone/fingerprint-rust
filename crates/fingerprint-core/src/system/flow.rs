//! networktrafficabstract
//!
//! definesystemlevelnetworktraffic，includingcompleteupdown文 and fingerprintinfo。

use super::context::SystemContext;
use crate::fingerprint::Fingerprint;
use std::time::Duration;

/// traffictrait
///
/// describenetworktrafficstatisticstrait and behaviorpattern。
#[derive(Debug, Clone, PartialEq)]
pub struct FlowCharacteristics {
 /// countpacketcount
 pub packet_count: u64,

 /// 总bytescount
 pub total_bytes: u64,

 /// 持续 when between
 pub duration: Duration,

 /// whetherencryption
 pub encrypted: bool,

 /// averagecountpacketsize
 pub avg_packet_size: f64,

 /// countpacketrate（包/seconds）
 pub packet_rate: f64,

 /// bytesrate（bytes/seconds）
 pub byte_rate: f64,
}

impl FlowCharacteristics {
 /// Create a newtraffictrait
 pub fn new() -> Self {
 Self {
 packet_count: 0,
 total_bytes: 0,
 duration: Duration::ZERO,
 encrypted: false,
 avg_packet_size: 0.0,
 packet_rate: 0.0,
 byte_rate: 0.0,
 }
 }

 /// Updatestatisticscountdata
 pub fn update(&mut self, packet_size: usize) {
 self.packet_count += 1;
 self.total_bytes += packet_size as u64;
 self.avg_packet_size = self.total_bytes as f64 / self.packet_count as f64;

 // If duration is not零, Calculaterate
 if !self.duration.is_zero() {
 let secs = self.duration.as_secs_f64();
 self.packet_rate = self.packet_count as f64 / secs;
 self.byte_rate = self.total_bytes as f64 / secs;
 }
 }

 /// settings持续 when between并Updaterate
 pub fn set_duration(&mut self, duration: Duration) {
 self.duration = duration;
 if !duration.is_zero() {
 let secs = duration.as_secs_f64();
 self.packet_rate = self.packet_count as f64 / secs;
 self.byte_rate = self.total_bytes as f64 / secs;
 }
 }
}

impl Default for FlowCharacteristics {
 fn default() -> Self {
 Self::new()
 }
}

/// networktraffic
///
/// representsystemlevelnetworktraffic，includingcompleteupdown文、fingerprintinfo and trait。
///
/// ## Core Concept
///
/// systemlevelprotectionneed from **networktraffic**角度performanalysis and protection，而is notonlyonlyfocussingleservice：
/// - completesystemupdown文（source/target、protocol、direction etc.）
/// - detect to fingerprintinfo（TLS、HTTP、TCP etc.）
/// - trafficstatisticstrait and behaviorpattern
///
/// ## Examples
///
/// ```rust
/// use fingerprint_core::system::{NetworkFlow, SystemContext, ProtocolType};
///
/// let ctx = SystemContext::new(
/// "192.168.1.100".parse().unwrap(),
/// "10.0.0.1".parse().unwrap(),
/// ProtocolType::Http,
/// );
///
/// let flow = NetworkFlow::new(ctx);
/// ```
pub struct NetworkFlow {
 /// systemupdown文
 pub context: SystemContext,

 /// detect to fingerprintlist（ if 有）
 /// Note: due to trait object limit，herecannotdirectly Clone，needmanualprocess
 #[cfg_attr(test, allow(dead_code))]
 fingerprints: Vec<Box<dyn Fingerprint>>,

 /// traffictrait
 pub characteristics: FlowCharacteristics,
}

impl NetworkFlow {
 /// Create a newnetworktraffic
 pub fn new(context: SystemContext) -> Self {
 Self {
 context,
 fingerprints: Vec::new(),
 characteristics: FlowCharacteristics::new(),
 }
 }

 /// Addfingerprint
 pub fn add_fingerprint(&mut self, fingerprint: Box<dyn Fingerprint>) {
 self.fingerprints.push(fingerprint);
 }

 /// Checkwhether有fingerprint
 pub fn has_fingerprints(&self) -> bool {
 !self.fingerprints.is_empty()
 }

 /// Getallfingerprintreference
 pub fn fingerprints(&self) -> &[Box<dyn Fingerprint>] {
 &self.fingerprints
 }

 /// Getspecifiedtypefingerprint
 pub fn get_fingerprints_by_type(
 &self,
 fingerprint_type: crate::fingerprint::FingerprintType,
 ) -> Vec<&dyn Fingerprint> {
 self.fingerprints
.iter()
.filter(|f| f.fingerprint_type() == fingerprint_type)
.map(|f| f.as_ref())
.collect()
 }

 /// Updatetraffictrait
 pub fn update_characteristics(&mut self, packet_size: usize) {
 self.characteristics.update(packet_size);
 }

 /// Gettrafficuniqueidentifier符
 pub fn flow_id(&self) -> String {
 self.context.flow_id()
 }
}

// Manual implementation Debug，because Box<dyn Fingerprint> cannotautomaticimplement Debug
impl std::fmt::Debug for NetworkFlow {
 fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
 f.debug_struct("NetworkFlow")
.field("context", &self.context)
.field("fingerprints_count", &self.fingerprints.len())
.field("characteristics", &self.characteristics)
.finish()
 }
}

// Manual implementation Clone，because Box<dyn Fingerprint> cannotautomatic Clone
impl Clone for NetworkFlow {
 fn clone(&self) -> Self {
 // Note: fingerprints cannot Clone，so新instance from emptyliststart
 // this is合理的，becausefingerprintusually不should被copy，而 is throughreferenceshared
 Self {
 context: self.context.clone(),
 fingerprints: Vec::new(), // cannot Clone trait object
 characteristics: self.characteristics.clone(),
 }
 }
}

#[cfg(test)]
mod tests {
 use super::*;

 #[test]
 fn test_flow_characteristics() {
 let mut chars = FlowCharacteristics::new();
 chars.update(1024);
 chars.update(2048);

 assert_eq!(chars.packet_count, 2);
 assert_eq!(chars.total_bytes, 3072);
 assert_eq!(chars.avg_packet_size, 1536.0);
 }

 #[test]
 fn test_network_flow() {
 use crate::system::context::ProtocolType;

 let ctx = SystemContext::new(
 "192.168.1.100".parse().unwrap(),
 "10.0.0.1".parse().unwrap(),
 ProtocolType::Http,
 );

 let flow = NetworkFlow::new(ctx);
 assert!(!flow.has_fingerprints());
 assert_eq!(flow.characteristics.packet_count, 0);
 }
}
