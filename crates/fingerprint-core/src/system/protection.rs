//! system levelprotectioninterface
//!
//! definesystem levelprotectioninterface and decisiontype。

use super::flow::NetworkFlow;
use super::stats::SystemProtectionStats;
use std::time::Duration;

/// system levelprotectiondecision
///
/// representsystem levelprotectionsystem pairnetworktrafficmakedecision。
#[derive(Debug, Clone, PartialEq)]
pub enum SystemProtectionDecision {
 /// all ow through
 Allow,

 /// blocktraffic
 Deny {
 /// blockreason
 reason: String,
 },

 /// rate limit
 RateLimit {
 /// each secondsmaximumcountpacketcount
 max_packets_per_second: u64,

 /// rate limit continuous when between
 duration: Duration,
 },

 /// recordbut not block
 Log {
 /// recordreason
 reason: String,
 },

 /// needfurtheranalysis
 RequiresAnalysis,
}

impl SystemProtectionDecision {
 /// judgewhether as all ow 
 pub fn is_ all ow (&self) -> bool {
 matches!(self, Self::Allow)
 }

 /// judgewhether as block
 pub fn is_deny(&self) -> bool {
 matches!(self, Self::Deny {.. })
 }

 /// judgewhether as rate limit
 pub fn is_rate_limit(&self) -> bool {
 matches!(self, Self::RateLimit {.. })
 }

 /// Getdecisiondescribe
 pub fn description(&self) -> String {
 match self {
 Self::Allow => " all ow through".to_string(),
 Self::Deny { reason } => format!("block: {}", reason),
 Self::RateLimit {
 max_packets_per_second,
 duration,
 } => {
 format!(
 "rate limit: {} package /seconds, continuous when between: {:?}",
 max_packets_per_second, duration
)
 }
 Self::Log { reason } => format!("record: {}", reason),
 Self::RequiresAnalysis => "needfurtheranalysis".to_string(),
 }
 }
}

/// system levelprotectionresult
///
/// includingprotectiondecision and phase closemetadata。
#[derive(Debug, Clone)]
pub struct SystemProtectionResult {
 /// protectiondecision
 pub decision: SystemProtectionDecision,

 /// risk score (0.0 - 1.0)
 /// - 0.0: completelysecurity
 /// - 1.0: 极highrisk
 pub risk_score: f64,

 /// confidence (0.0 - 1.0)
 /// - 0.0: completelynot confident
 /// - 1.0: completelyconfident
 pub confidence: f64,

 /// decisionreason
 pub reason: String,

 /// suggestback续action
 pub suggested_actions: Vec<String>,
}

impl SystemProtectionResult {
 /// create a new protectionresult
 pub fn new(decision: SystemProtectionDecision) -> Self {
 Self {
 decision,
 risk_score: 0.0,
 confidence: 1.0,
 reason: String::new(),
 suggested_actions: Vec::new(),
 }
 }

 /// Create all ow decision
 pub fn all ow () -> Self {
 Self {
 decision: SystemProtectionDecision::Allow,
 risk_score: 0.0,
 confidence: 1.0,
 reason: "normaltraffic".to_string(),
 suggested_actions: Vec::new(),
 }
 }

 /// Createblockdecision
 pub fn deny(reason: String, risk_score: f64) -> Self {
 Self {
 decision: SystemProtectionDecision::Deny {
 reason: reason.clone(),
 },
 risk_score,
 confidence: 1.0,
 reason,
 suggested_actions: vec!["Add to blacklist".to_string(), "recordlog".to_string()],
 }
 }

 /// Createrate limitdecision
 pub fn rate_limit(max_packets_per_second: u64, duration: Duration, risk_score: f64) -> Self {
 Self {
 decision: SystemProtectionDecision::RateLimit {
 max_packets_per_second,
 duration,
 },
 risk_score,
 confidence: 0.8,
 reason: "trafficabnormal，needrate limit".to_string(),
 suggested_actions: vec!["monitortraffic".to_string()],
 }
 }
}

/// system levelprotectioninterface
///
/// all system levelprotectioner all shouldimplementthis trait。
///
/// ## Core Concept
///
/// system levelprotection from **system perspective**makeprotectiondecision：
/// - not onlyonly is singleserviceprotection， and is 整system protection
/// - can实施system levelmeasure (blacklist、rate limit、防火墙rule etc.)
/// - needconsidersystem wholesecuritystatus
///
/// ## Implementation Example
///
/// ```rust
/// use fingerprint_core::system ::{SystemProtector, NetworkFlow, SystemProtectionResult, SystemProtectionStats};
///
/// struct MySystemProtector;
///
/// impl SystemProtector for MySystemProtector {
/// fn protect(&self, flow: &NetworkFlow) -> SystemProtectionResult {
/// // implementprotectionlogic
/// SystemProtectionResult:: all ow ()
/// }
///
/// fn update_state(&mut self, flow: &NetworkFlow, result: &SystemProtectionResult) {
/// // Updatesystem status
/// }
///
/// fn get_stats(&self) -> SystemProtectionStats {
/// // returnstatisticsinfo
/// SystemProtectionStats::default()
/// }
/// }
/// ```
pub trait SystemProtector: Send {
 /// analysisnetworktraffic and makeprotectiondecision
 ///
 /// # Parameters
 ///
 /// - `flow`: need analysisnetworktraffic
 ///
 /// # Returns
 ///
 /// system levelprotectionresult，includingdecision、risk score、confidence etc.info
 fn protect(&self, flow: &NetworkFlow) -> SystemProtectionResult;

 /// Updatesystem status
 ///
 /// in makeprotectiondecisionback，canBased onresultUpdatesystem status (such as Updateblacklist、statisticsinfo etc.)。
 ///
 /// # Parameters
 ///
 /// - `flow`: networktraffic
 /// - `result`: protectiondecisionresult
 fn update_state(&mut self, flow: &NetworkFlow, result: &SystemProtectionResult);

 /// Getsystem statisticsinfo
 ///
 /// # Returns
 ///
 /// system levelprotectionstatisticsinfo
 fn get_stats(&self) -> SystemProtectionStats;
}

#[cfg(test)]
mod tests {
 use super::*;

 #[test]
 fn test_protection_decision() {
 let all ow = SystemProtectionDecision::Allow;
 assert!(all ow.is_ all ow ());
 assert!(! all ow.is_deny());

 let deny = SystemProtectionDecision::Deny {
 reason: "maliciousIP".to_string(),
 };
 assert!(deny.is_deny());
 assert!(!deny.is_ all ow ());
 }

 #[test]
 fn test_protection_result() {
 let all ow = SystemProtectionResult:: all ow ();
 assert!(all ow.decision.is_ all ow ());
 assert_eq!(all ow.risk_score, 0.0);

 let deny = SystemProtectionResult::deny("test".to_string(), 0.9);
 assert!(deny.decision.is_deny());
 assert_eq!(deny.risk_score, 0.9);
 }
}
