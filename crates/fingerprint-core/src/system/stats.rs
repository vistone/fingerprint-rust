//! systemlevelstatisticsinfo
//!
//! definesystemlevelprotectionstatisticsinfo。

use std::time::Instant;

/// systemlevelprotectionstatisticsinfo
///
/// recordsystemlevelprotectionsystemrunstatisticsinfo。
#[derive(Debug, Clone)]
pub struct SystemProtectionStats {
 /// totalpacketcount
 pub total_packets: u64,

 /// alreadyanalysiscountpacketcount
 pub analyzed_packets: u64,

 /// alreadyblockcountpacketcount
 pub blocked_packets: u64,

 /// rate limitcountpacketcount
 pub rate_limited_packets: u64,

 /// allowthroughcountpacketcount
 pub allowed_packets: u64,

 /// detect to threatcount
 pub threat_detected: u64,

 /// start when between
 pub start_time: Instant,

 /// finallyUpdate when between
 pub last_update_time: Instant,
}

impl SystemProtectionStats {
 /// Create a newstatisticsinfo
 pub fn new() -> Self {
 let now = Instant::now();
 Self {
 total_packets: 0,
 analyzed_packets: 0,
 blocked_packets: 0,
 rate_limited_packets: 0,
 allowed_packets: 0,
 threat_detected: 0,
 start_time: now,
 last_update_time: now,
 }
 }

 /// increasetotalpacketcount
 pub fn increment_total(&mut self) {
 self.total_packets += 1;
 self.last_update_time = Instant::now();
 }

 /// increasealreadyanalysiscountpacketcount
 pub fn increment_analyzed(&mut self) {
 self.analyzed_packets += 1;
 self.last_update_time = Instant::now();
 }

 /// increasealreadyblockcountpacketcount
 pub fn increment_blocked(&mut self) {
 self.blocked_packets += 1;
 self.last_update_time = Instant::now();
 }

 /// increaserate limitcountpacketcount
 pub fn increment_rate_limited(&mut self) {
 self.rate_limited_packets += 1;
 self.last_update_time = Instant::now();
 }

 /// increaseallowthroughcountpacketcount
 pub fn increment_allowed(&mut self) {
 self.allowed_packets += 1;
 self.last_update_time = Instant::now();
 }

 /// increasethreatdetectcount
 pub fn increment_threat(&mut self) {
 self.threat_detected += 1;
 self.last_update_time = Instant::now();
 }

 /// Getrun when between (seconds)
 pub fn uptime_seconds(&self) -> u64 {
 self.start_time.elapsed().as_secs()
 }

 /// Getcountpacketprocessrate (包/seconds)
 pub fn packets_per_second(&self) -> f64 {
 let uptime = self.uptime_seconds() as f64;
 if uptime > 0.0 {
 self.total_packets as f64 / uptime
 } else {
 0.0
 }
 }

 /// Getanalysis率 (alreadyanalysis/total)
 pub fn analysis_rate(&self) -> f64 {
 if self.total_packets > 0 {
 self.analyzed_packets as f64 / self.total_packets as f64
 } else {
 0.0
 }
 }

 /// Getblock率 (alreadyblock/total)
 pub fn block_rate(&self) -> f64 {
 if self.total_packets > 0 {
 self.blocked_packets as f64 / self.total_packets as f64
 } else {
 0.0
 }
 }

 /// resetstatisticsinfo
 pub fn reset(&mut self) {
 self.total_packets = 0;
 self.analyzed_packets = 0;
 self.blocked_packets = 0;
 self.rate_limited_packets = 0;
 self.allowed_packets = 0;
 self.threat_detected = 0;
 self.start_time = Instant::now();
 self.last_update_time = Instant::now();
 }
}

impl Default for SystemProtectionStats {
 fn default() -> Self {
 Self::new()
 }
}

#[cfg(test)]
mod tests {
 use super::*;

 #[test]
 fn test_stats_new() {
 let stats = SystemProtectionStats::new();
 assert_eq!(stats.total_packets, 0);
 assert_eq!(stats.analyzed_packets, 0);
 }

 #[test]
 fn test_stats_increment() {
 let mut stats = SystemProtectionStats::new();
 stats.increment_total();
 stats.increment_analyzed();
 stats.increment_blocked();

 assert_eq!(stats.total_packets, 1);
 assert_eq!(stats.analyzed_packets, 1);
 assert_eq!(stats.blocked_packets, 1);
 }

 #[test]
 fn test_stats_rates() {
 let mut stats = SystemProtectionStats::new();
 stats.increment_total();
 stats.increment_total();
 stats.increment_analyzed();
 stats.increment_blocked();

 assert_eq!(stats.analysis_rate(), 0.5);
 assert_eq!(stats.block_rate(), 0.5);
 }

 #[test]
 fn test_stats_reset() {
 let mut stats = SystemProtectionStats::new();
 stats.increment_total();
 stats.increment_analyzed();
 stats.reset();

 assert_eq!(stats.total_packets, 0);
 assert_eq!(stats.analyzed_packets, 0);
 }
}
