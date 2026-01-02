//! systemlevelprotectioninterface
//!
//! definesystemlevelprotection的interface and decisiontype。

use super::flow::NetworkFlow;
use super::stats::SystemProtectionStats;
use std::time::Duration;

/// systemlevelprotectiondecision
///
/// representsystemlevelprotectionsystempairnetworktraffic做出的decision。
#[derive(Debug, Clone, PartialEq)]
pub enum SystemProtectionDecision {
    /// allowthrough
    Allow,

    /// blocktraffic
    Deny {
        /// block原因
        reason: String,
    },

    /// 限速
    RateLimit {
        /// 每秒maximumcountpacketcount
        max_packets_per_second: u64,

        /// 限速持续 when 间
        duration: Duration,
    },

    /// recordbut不block
    Log {
        /// record原因
        reason: String,
    },

    /// need进一步analysis
    RequiresAnalysis,
}

impl SystemProtectionDecision {
    /// judgewhether为allow
    pub fn is_allow(&self) -> bool {
        matches!(self, Self::Allow)
    }

    /// judgewhether为block
    pub fn is_deny(&self) -> bool {
        matches!(self, Self::Deny { .. })
    }

    /// judgewhether为限速
    pub fn is_rate_limit(&self) -> bool {
        matches!(self, Self::RateLimit { .. })
    }

    /// Getdecision的describe
    pub fn description(&self) -> String {
        match self {
            Self::Allow => "allowthrough".to_string(),
            Self::Deny { reason } => format!("block: {}", reason),
            Self::RateLimit {
                max_packets_per_second,
                duration,
            } => {
                format!(
                    "限速: {} 包/秒, 持续 when 间: {:?}",
                    max_packets_per_second, duration
                )
            }
            Self::Log { reason } => format!("record: {}", reason),
            Self::RequiresAnalysis => "need进一步analysis".to_string(),
        }
    }
}

/// systemlevelprotectionresult
///
/// includingprotectiondecision and 相close的metadata。
#[derive(Debug, Clone)]
pub struct SystemProtectionResult {
    /// protectiondecision
    pub decision: SystemProtectionDecision,

    /// 风险评分 (0.0 - 1.0)
    /// - 0.0: completelysecurity
    /// - 1.0: 极高风险
    pub risk_score: f64,

    /// confidence (0.0 - 1.0)
    /// - 0.0: completely不确信
    /// - 1.0: completely确信
    pub confidence: f64,

    /// decision原因
    pub reason: String,

    /// 建议的back续action
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
            suggested_actions: vec!["Add to 黑名单".to_string(), "recordlog".to_string()],
        }
    }

    /// Create限速decision
    pub fn rate_limit(max_packets_per_second: u64, duration: Duration, risk_score: f64) -> Self {
        Self {
            decision: SystemProtectionDecision::RateLimit {
                max_packets_per_second,
                duration,
            },
            risk_score,
            confidence: 0.8,
            reason: "trafficabnormal，need限速".to_string(),
            suggested_actions: vec!["monitortraffic".to_string()],
        }
    }
}

/// systemlevelprotectioninterface
///
/// allsystemlevelprotection器都shouldimplementthis trait。
///
/// ## Core Concept
///
/// systemlevelprotection from **system角度**做出protectiondecision：
/// - not onlyonly是singleservice的protection，而是整个system的protection
/// - can实施systemlevel的措施（黑名单、限速、防火墙规则等）
/// - need考虑systemwhole的securitystatus
///
/// ## Implementation Example
///
/// ```rust
/// use fingerprint_core::system::{SystemProtector, NetworkFlow, SystemProtectionResult, SystemProtectionStats};
///
/// struct MySystemProtector;
///
/// impl SystemProtector for MySystemProtector {
///     fn protect(&self, flow: &NetworkFlow) -> SystemProtectionResult {
///         // implementprotectionlogic
///         SystemProtectionResult::allow()
///     }
///
///     fn update_state(&mut self, flow: &NetworkFlow, result: &SystemProtectionResult) {
///         // Updatesystemstatus
///     }
///
///     fn get_stats(&self) -> SystemProtectionStats {
///         // returnstatisticsinfo
///         SystemProtectionStats::default()
///     }
/// }
/// ```
pub trait SystemProtector: Send {
    /// analysisnetworktraffic并做出protectiondecision
    ///
    /// # Parameters
    ///
    /// - `flow`: 要analysis的networktraffic
    ///
    /// # Returns
    ///
    /// systemlevelprotectionresult，includingdecision、风险评分、confidence等info
    fn protect(&self, flow: &NetworkFlow) -> SystemProtectionResult;

    /// Updatesystemstatus
    ///
    ///  in 做出protectiondecisionback，canBased onresultUpdatesystemstatus（如Update黑名单、statisticsinfo等）。
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
