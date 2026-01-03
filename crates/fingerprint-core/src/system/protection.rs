//! systemlevelprotectioninterface
//!
//! definesystemlevelprotectioninterface and decisiontype。

use super::flow::NetworkFlow;
use super::stats::SystemProtectionStats;
use std::time::Duration;

/// systemlevelprotectiondecision
///
/// representsystemlevelprotectionsystempairnetworktrafficmakedecision。
#[derive(Debug, Clone, PartialEq)]
pub enum SystemProtectionDecision {
 /// allowthrough
 Allow,

 /// blocktraffic
 Deny {
 /// blockreason
 reason: String,
 },

 /// rate limit
 RateLimit {
 /// 每secondsmaximumcountpacketcount
 max_packets_per_second: u64,

 /// rate limitcontinuous when between
 duration: Duration,
 },

 /// recordbut不block
 Log {
 /// recordreason
 reason: String,
 },

 /// needfurtheranalysis
 RequiresAnalysis,
}

impl SystemProtectionDecision {
 /// judgewhether as allow
 pub fn is_allow(&self) -> bool {
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
 Self::Allow => "allowthrough".to_string(),
 Self::Deny { reason } => format!("block: {}", reason),
 Self::RateLimit {
 max_packets_per_second,
 duration,
 } => {
 format!(
 "rate limit: {} 包/seconds, continuous when between: {:?}",
 max_packets_per_second, duration
 )
 }
 Self::Log { reason } => format!("record: {}", reason),
 Self::RequiresAnalysis => "needfurtheranalysis".to_string(),
 }
 }
}

/// systemlevelprotectionresult
///
/// includingprotectiondecision and mutualclosemetadata。
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

 /// suggestbackcontinueaction
 pub suggested_actions: Vec<String>,
}

impl SystemProtectionResult {
 /// Create a newprotectionresult
 pub fn new(decision: SystemProtectionDecision) -> Self {
 Self {
 decision,
 risk_score: 0.0,
 confidence: 1.0,
 reason: String::new(),
 suggested_actions: Vec::new(),
 }
 }

 /// Createallowdecision
 pub fn allow() -> Self {
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

/// systemlevelprotectioninterface
///
/// allsystemlevelprotectioner都shouldimplementthis trait。
///
/// ## Core Concept
///
/// systemlevelprotection from **systemperspective**makeprotectiondecision：
/// - not onlyonly is singleserviceprotection，而 is wholesystemprotection
/// - canactual施systemlevelmeasure (blacklist、rate limit、firewallrule etc.)
/// - needconsidersystemwholesecuritystatus
///
/// ## Implementation Example
///
/// ```rust
/// use fingerprint_core::system::{SystemProtector, NetworkFlow, SystemProtectionResult, SystemProtectionStats};
///
/// struct MySystemProtector;
///
/// impl SystemProtector for MySystemProtector {
/// fn protect(&self, flow: &NetworkFlow) -> SystemProtectionResult {
/// // implementprotectionlogic
/// SystemProtectionResult::allow()
/// }
///
/// fn update_state(&mut self, flow: &NetworkFlow, result: &SystemProtectionResult) {
/// // Updatesystemstatus
/// }
///
/// fn get_stats(&self) -> SystemProtectionStats {
/// // returnstatisticsinfo
/// SystemProtectionStats::default()
/// }
/// }
/// ```
pub trait SystemProtector: Send {
 /// analysisnetworktraffic并makeprotectiondecision
 ///
 /// # Parameters
 ///
 /// - `flow`: needanalysisnetworktraffic
 ///
 /// # Returns
 ///
 /// systemlevelprotectionresult，includingdecision、risk score、confidence etc.info
 fn protect(&self, flow: &NetworkFlow) -> SystemProtectionResult;

 /// Updatesystemstatus
 ///
 /// in makeprotectiondecisionback，canBased onresultUpdatesystemstatus (如Updateblacklist、statisticsinfo etc.)。
 ///
 /// # Parameters
 ///
 /// - `flow`: networktraffic
 /// - `result`: protectiondecisionresult
 fn update_state(&mut self, flow: &NetworkFlow, result: &SystemProtectionResult);

 /// Getsystemstatisticsinfo
 ///
 /// # Returns
 ///
 /// systemlevelprotectionstatisticsinfo
 fn get_stats(&self) -> SystemProtectionStats;
}

#[cfg(test)]
mod tests {
 use super::*;

 #[test]
 fn test_protection_decision() {
 let allow = SystemProtectionDecision::Allow;
 assert!(allow.is_allow());
 assert!(!allow.is_deny());

 let deny = SystemProtectionDecision::Deny {
 reason: "maliciousIP".to_string(),
 };
 assert!(deny.is_deny());
 assert!(!deny.is_allow());
 }

 #[test]
 fn test_protection_result() {
 let allow = SystemProtectionResult::allow();
 assert!(allow.decision.is_allow());
 assert_eq!(allow.risk_score, 0.0);

 let deny = SystemProtectionResult::deny("test".to_string(), 0.9);
 assert!(deny.decision.is_deny());
 assert_eq!(deny.risk_score, 0.9);
 }
}
