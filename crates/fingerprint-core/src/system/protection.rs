//! systemlevelprotectioninterface
//!
//! definesystemlevelprotection的interface and 决策type。

use super::flow::NetworkFlow;
use super::stats::SystemProtectionStats;
use std::time::Duration;

/// systemlevelprotection决策
///
/// representsystemlevelprotectionsystempairnetworktraffic做出的决策。
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

    /// Get决策的describe
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
/// includingprotection决策 and 相关的metadata。
#[derive(Debug, Clone)]
pub struct SystemProtectionResult {
    /// protection决策
    pub decision: SystemProtectionDecision,

    /// 风险评分 (0.0 - 1.0)
    /// - 0.0: 完全security
    /// - 1.0: 极高风险
    pub risk_score: f64,

    /// 置信度 (0.0 - 1.0)
    /// - 0.0: 完全不确信
    /// - 1.0: 完全确信
    pub confidence: f64,

    /// 决策原因
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

    /// Createallow决策
    pub fn allow() -> Self {
        Self {
            decision: SystemProtectionDecision::Allow,
            risk_score: 0.0,
            confidence: 1.0,
            reason: "正常traffic".to_string(),
            suggested_actions: Vec::new(),
        }
    }

    /// Createblock决策
    pub fn deny(reason: String, risk_score: f64) -> Self {
        Self {
            decision: SystemProtectionDecision::Deny {
                reason: reason.clone(),
            },
            risk_score,
            confidence: 1.0,
            reason,
            suggested_actions: vec!["Add to 黑名单".to_string(), "record日志".to_string()],
        }
    }

    /// Create限速决策
    pub fn rate_limit(max_packets_per_second: u64, duration: Duration, risk_score: f64) -> Self {
        Self {
            decision: SystemProtectionDecision::RateLimit {
                max_packets_per_second,
                duration,
            },
            risk_score,
            confidence: 0.8,
            reason: "traffic异常，need限速".to_string(),
            suggested_actions: vec!["monitortraffic".to_string()],
        }
    }
}

/// systemlevelprotectioninterface
///
/// allsystemlevelprotection器都shouldimplement这个 trait。
///
/// ## Core Concept
///
/// systemlevelprotection from **system角度**做出protection决策：
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
///         // implementprotection逻辑
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
    /// analysisnetworktraffic并做出protection决策
    ///
    /// # Parameters
    ///
    /// - `flow`: 要analysis的networktraffic
    ///
    /// # Returns
    ///
    /// systemlevelprotectionresult，including决策、风险评分、置信度等info
    fn protect(&self, flow: &NetworkFlow) -> SystemProtectionResult;

    /// Updatesystemstatus
    ///
    ///  in 做出protection决策back，canBased onresultUpdatesystemstatus（如Update黑名单、statisticsinfo等）。
    ///
    /// # Parameters
    ///
    /// - `flow`: networktraffic
    /// - `result`: protection决策result
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
            reason: "恶意IP".to_string(),
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
