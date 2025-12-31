//! 系统级别分析接口
//!
//! 定义系统级别分析的接口和结果类型。

use super::flow::NetworkFlow;
use crate::fingerprint::Fingerprint;

/// 威胁类型
///
/// 表示检测到的威胁类型。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ThreatType {
    /// 未知指纹
    UnknownFingerprint,

    /// 可疑行为
    SuspiciousBehavior,

    /// 已知攻击
    KnownAttack,

    /// 异常流量模式
    AbnormalTrafficPattern,

    /// 恶意 IP
    MaliciousIP,

    /// DDoS 攻击
    DDoS,

    /// 端口扫描
    PortScan,

    /// 暴力破解
    BruteForce,
}

impl ThreatType {
    /// 转换为字符串
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::UnknownFingerprint => "未知指纹",
            Self::SuspiciousBehavior => "可疑行为",
            Self::KnownAttack => "已知攻击",
            Self::AbnormalTrafficPattern => "异常流量模式",
            Self::MaliciousIP => "恶意IP",
            Self::DDoS => "DDoS攻击",
            Self::PortScan => "端口扫描",
            Self::BruteForce => "暴力破解",
        }
    }

    /// 获取威胁严重程度 (0.0 - 1.0)
    pub fn severity(&self) -> f64 {
        match self {
            Self::UnknownFingerprint => 0.3,
            Self::SuspiciousBehavior => 0.5,
            Self::AbnormalTrafficPattern => 0.6,
            Self::PortScan => 0.7,
            Self::BruteForce => 0.8,
            Self::DDoS => 0.9,
            Self::KnownAttack => 0.95,
            Self::MaliciousIP => 1.0,
        }
    }
}

impl std::fmt::Display for ThreatType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// 分析详情
///
/// 包含分析的详细信息和证据。
#[derive(Debug, Clone, Default)]
pub struct AnalysisDetails {
    /// 检测到的指纹类型
    pub fingerprint_types: Vec<crate::fingerprint::FingerprintType>,

    /// 匹配的规则或模式
    pub matched_rules: Vec<String>,

    /// 行为特征
    pub behavior_features: Vec<String>,

    /// 异常指标
    pub anomalies: Vec<String>,

    /// 额外信息
    pub additional_info: std::collections::HashMap<String, String>,
}

impl AnalysisDetails {
    /// 创建新的分析详情
    pub fn new() -> Self {
        Self::default()
    }

    /// 添加匹配的规则
    pub fn add_matched_rule(&mut self, rule: String) {
        self.matched_rules.push(rule);
    }

    /// 添加行为特征
    pub fn add_behavior_feature(&mut self, feature: String) {
        self.behavior_features.push(feature);
    }

    /// 添加异常指标
    pub fn add_anomaly(&mut self, anomaly: String) {
        self.anomalies.push(anomaly);
    }
}

/// 系统级别分析结果
///
/// 包含分析结果、威胁类型、风险评分等信息。
pub struct SystemAnalysisResult {
    /// 检测到的指纹列表
    /// 注意：由于 trait object 的限制，这里不能直接 Clone
    fingerprints: Vec<Box<dyn Fingerprint>>,

    /// 风险评分 (0.0 - 1.0)
    /// - 0.0: 完全安全
    /// - 1.0: 极高风险
    pub risk_score: f64,

    /// 置信度 (0.0 - 1.0)
    /// - 0.0: 完全不确信
    /// - 1.0: 完全确信
    pub confidence: f64,

    /// 威胁类型列表
    pub threat_types: Vec<ThreatType>,

    /// 分析详情
    pub details: AnalysisDetails,
}

impl SystemAnalysisResult {
    /// 创建新的分析结果
    pub fn new() -> Self {
        Self {
            fingerprints: Vec::new(),
            risk_score: 0.0,
            confidence: 0.0,
            threat_types: Vec::new(),
            details: AnalysisDetails::new(),
        }
    }

    /// 创建安全的结果（无威胁）
    pub fn safe() -> Self {
        Self {
            fingerprints: Vec::new(),
            risk_score: 0.0,
            confidence: 1.0,
            threat_types: Vec::new(),
            details: AnalysisDetails::new(),
        }
    }

    /// 添加指纹
    pub fn add_fingerprint(&mut self, fingerprint: Box<dyn Fingerprint>) {
        self.fingerprints.push(fingerprint);
    }

    /// 获取所有指纹的引用
    pub fn fingerprints(&self) -> &[Box<dyn Fingerprint>] {
        &self.fingerprints
    }

    /// 添加威胁类型
    pub fn add_threat_type(&mut self, threat_type: ThreatType) {
        if !self.threat_types.contains(&threat_type) {
            self.threat_types.push(threat_type);
            // 根据威胁类型更新风险评分
            self.update_risk_score();
        }
    }

    /// 更新风险评分（基于威胁类型）
    fn update_risk_score(&mut self) {
        if self.threat_types.is_empty() {
            self.risk_score = 0.0;
        } else {
            // 使用最高严重程度作为风险评分
            self.risk_score = self
                .threat_types
                .iter()
                .map(|t| t.severity())
                .fold(0.0, f64::max);
        }
    }

    /// 判断是否存在威胁
    pub fn has_threats(&self) -> bool {
        !self.threat_types.is_empty() && self.risk_score > 0.0
    }

    /// 判断是否为高风险
    pub fn is_high_risk(&self) -> bool {
        self.risk_score >= 0.7
    }
}

impl Default for SystemAnalysisResult {
    fn default() -> Self {
        Self::new()
    }
}

// 手动实现 Debug，因为 Box<dyn Fingerprint> 不能自动实现 Debug
impl std::fmt::Debug for SystemAnalysisResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SystemAnalysisResult")
            .field("fingerprints_count", &self.fingerprints.len())
            .field("risk_score", &self.risk_score)
            .field("confidence", &self.confidence)
            .field("threat_types", &self.threat_types)
            .field("details", &self.details)
            .finish()
    }
}

// 手动实现 Clone，因为 Box<dyn Fingerprint> 不能自动 Clone
impl Clone for SystemAnalysisResult {
    fn clone(&self) -> Self {
        // 注意：fingerprints 不能 Clone，所以新实例从空列表开始
        Self {
            fingerprints: Vec::new(), // 不能 Clone trait object
            risk_score: self.risk_score,
            confidence: self.confidence,
            threat_types: self.threat_types.clone(),
            details: self.details.clone(),
        }
    }
}

/// 系统级别分析接口
///
/// 所有系统级别分析器都应该实现这个 trait。
///
/// ## 核心思想
///
/// 系统级别分析从**系统角度**分析网络流量：
/// - 不仅仅是单个协议的解析，而是综合分析
/// - 考虑系统整体的行为模式
/// - 检测系统级别的威胁（DDoS、扫描、异常流量等）
///
/// ## 实现示例
///
/// ```rust
/// use fingerprint_core::system::{SystemAnalyzer, NetworkFlow, SystemAnalysisResult};
///
/// struct MySystemAnalyzer;
///
/// impl SystemAnalyzer for MySystemAnalyzer {
///     fn analyze(&self, flow: &NetworkFlow) -> SystemAnalysisResult {
///         // 实现分析逻辑
///         SystemAnalysisResult::safe()
///     }
///
///     fn analyze_batch(&self, flows: &[NetworkFlow]) -> Vec<SystemAnalysisResult> {
///         flows.iter().map(|f| self.analyze(f)).collect()
///     }
/// }
/// ```
pub trait SystemAnalyzer: Send + Sync {
    /// 分析网络流量
    ///
    /// # 参数
    ///
    /// - `flow`: 要分析的网络流量
    ///
    /// # 返回
    ///
    /// 系统级别分析结果，包含指纹、威胁类型、风险评分等信息
    fn analyze(&self, flow: &NetworkFlow) -> SystemAnalysisResult;

    /// 批量分析多个流量
    ///
    /// # 参数
    ///
    /// - `flows`: 要分析的网络流量列表
    ///
    /// # 返回
    ///
    /// 分析结果列表
    fn analyze_batch(&self, flows: &[NetworkFlow]) -> Vec<SystemAnalysisResult> {
        flows.iter().map(|f| self.analyze(f)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_threat_type() {
        assert_eq!(ThreatType::MaliciousIP.severity(), 1.0);
        assert_eq!(ThreatType::UnknownFingerprint.severity(), 0.3);
    }

    #[test]
    fn test_analysis_result() {
        let mut result = SystemAnalysisResult::safe();
        assert!(!result.has_threats());

        result.add_threat_type(ThreatType::DDoS);
        assert!(result.has_threats());
        assert!(result.is_high_risk());
    }
}
