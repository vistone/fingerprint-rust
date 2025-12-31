//! 系统级别防护接口
//!
//! 定义系统级别防护的接口和决策类型。

use super::flow::NetworkFlow;
use super::stats::SystemProtectionStats;
use std::time::Duration;

/// 系统级别防护决策
///
/// 表示系统级别防护系统对网络流量做出的决策。
#[derive(Debug, Clone, PartialEq)]
pub enum SystemProtectionDecision {
    /// 允许通过
    Allow,

    /// 阻止流量
    Deny {
        /// 阻止原因
        reason: String,
    },

    /// 限速
    RateLimit {
        /// 每秒最大数据包数
        max_packets_per_second: u64,

        /// 限速持续时间
        duration: Duration,
    },

    /// 记录但不阻止
    Log {
        /// 记录原因
        reason: String,
    },

    /// 需要进一步分析
    RequiresAnalysis,
}

impl SystemProtectionDecision {
    /// 判断是否为允许
    pub fn is_allow(&self) -> bool {
        matches!(self, Self::Allow)
    }

    /// 判断是否为阻止
    pub fn is_deny(&self) -> bool {
        matches!(self, Self::Deny { .. })
    }

    /// 判断是否为限速
    pub fn is_rate_limit(&self) -> bool {
        matches!(self, Self::RateLimit { .. })
    }

    /// 获取决策的描述
    pub fn description(&self) -> String {
        match self {
            Self::Allow => "允许通过".to_string(),
            Self::Deny { reason } => format!("阻止: {}", reason),
            Self::RateLimit {
                max_packets_per_second,
                duration,
            } => {
                format!(
                    "限速: {} 包/秒, 持续时间: {:?}",
                    max_packets_per_second, duration
                )
            }
            Self::Log { reason } => format!("记录: {}", reason),
            Self::RequiresAnalysis => "需要进一步分析".to_string(),
        }
    }
}

/// 系统级别防护结果
///
/// 包含防护决策和相关的元数据。
#[derive(Debug, Clone)]
pub struct SystemProtectionResult {
    /// 防护决策
    pub decision: SystemProtectionDecision,

    /// 风险评分 (0.0 - 1.0)
    /// - 0.0: 完全安全
    /// - 1.0: 极高风险
    pub risk_score: f64,

    /// 置信度 (0.0 - 1.0)
    /// - 0.0: 完全不确信
    /// - 1.0: 完全确信
    pub confidence: f64,

    /// 决策原因
    pub reason: String,

    /// 建议的后续动作
    pub suggested_actions: Vec<String>,
}

impl SystemProtectionResult {
    /// 创建新的防护结果
    pub fn new(decision: SystemProtectionDecision) -> Self {
        Self {
            decision,
            risk_score: 0.0,
            confidence: 1.0,
            reason: String::new(),
            suggested_actions: Vec::new(),
        }
    }

    /// 创建允许决策
    pub fn allow() -> Self {
        Self {
            decision: SystemProtectionDecision::Allow,
            risk_score: 0.0,
            confidence: 1.0,
            reason: "正常流量".to_string(),
            suggested_actions: Vec::new(),
        }
    }

    /// 创建阻止决策
    pub fn deny(reason: String, risk_score: f64) -> Self {
        Self {
            decision: SystemProtectionDecision::Deny {
                reason: reason.clone(),
            },
            risk_score,
            confidence: 1.0,
            reason,
            suggested_actions: vec!["添加到黑名单".to_string(), "记录日志".to_string()],
        }
    }

    /// 创建限速决策
    pub fn rate_limit(max_packets_per_second: u64, duration: Duration, risk_score: f64) -> Self {
        Self {
            decision: SystemProtectionDecision::RateLimit {
                max_packets_per_second,
                duration,
            },
            risk_score,
            confidence: 0.8,
            reason: "流量异常，需要限速".to_string(),
            suggested_actions: vec!["监控流量".to_string()],
        }
    }
}

/// 系统级别防护接口
///
/// 所有系统级别防护器都应该实现这个 trait。
///
/// ## 核心思想
///
/// 系统级别防护从**系统角度**做出防护决策：
/// - 不仅仅是单个服务的防护，而是整个系统的防护
/// - 可以实施系统级别的措施（黑名单、限速、防火墙规则等）
/// - 需要考虑系统整体的安全状态
///
/// ## 实现示例
///
/// ```rust
/// use fingerprint_core::system::{SystemProtector, NetworkFlow, SystemProtectionResult, SystemProtectionStats};
///
/// struct MySystemProtector;
///
/// impl SystemProtector for MySystemProtector {
///     fn protect(&self, flow: &NetworkFlow) -> SystemProtectionResult {
///         // 实现防护逻辑
///         SystemProtectionResult::allow()
///     }
///
///     fn update_state(&mut self, flow: &NetworkFlow, result: &SystemProtectionResult) {
///         // 更新系统状态
///     }
///
///     fn get_stats(&self) -> SystemProtectionStats {
///         // 返回统计信息
///         SystemProtectionStats::default()
///     }
/// }
/// ```
pub trait SystemProtector: Send {
    /// 分析网络流量并做出防护决策
    ///
    /// # 参数
    ///
    /// - `flow`: 要分析的网络流量
    ///
    /// # 返回
    ///
    /// 系统级别防护结果，包含决策、风险评分、置信度等信息
    fn protect(&self, flow: &NetworkFlow) -> SystemProtectionResult;

    /// 更新系统状态
    ///
    /// 在做出防护决策后，可以根据结果更新系统状态（如更新黑名单、统计信息等）。
    ///
    /// # 参数
    ///
    /// - `flow`: 网络流量
    /// - `result`: 防护决策结果
    fn update_state(&mut self, flow: &NetworkFlow, result: &SystemProtectionResult);

    /// 获取系统统计信息
    ///
    /// # 返回
    ///
    /// 系统级别防护统计信息
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

        let deny = SystemProtectionResult::deny("测试".to_string(), 0.9);
        assert!(deny.decision.is_deny());
        assert_eq!(deny.risk_score, 0.9);
    }
}
