//! systemlevelanalysisinterface
//!
//! definesystemlevelanalysis的interface and resulttype。

use super::flow::NetworkFlow;
use crate::fingerprint::Fingerprint;

/// 威胁type
///
/// representdetect to 的威胁type。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ThreatType {
    /// not知fingerprint
    UnknownFingerprint,

    /// 可疑behavior
    SuspiciousBehavior,

    /// already知攻击
    KnownAttack,

    /// 异常trafficpattern
    AbnormalTrafficPattern,

    /// 恶意 IP
    MaliciousIP,

    /// DDoS 攻击
    DDoS,

    /// portscan
    PortScan,

    /// 暴力破解
    BruteForce,
}

impl ThreatType {
    /// convert tostring
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::UnknownFingerprint => "not知fingerprint",
            Self::SuspiciousBehavior => "可疑behavior",
            Self::KnownAttack => "already知攻击",
            Self::AbnormalTrafficPattern => "异常trafficpattern",
            Self::MaliciousIP => "恶意IP",
            Self::DDoS => "DDoS攻击",
            Self::PortScan => "portscan",
            Self::BruteForce => "暴力破解",
        }
    }

    /// Get威胁严重程度 (0.0 - 1.0)
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

/// analysis详情
///
/// includinganalysis的详细info and 证据。
#[derive(Debug, Clone, Default)]
pub struct AnalysisDetails {
    /// detect to 的fingerprinttype
    pub fingerprint_types: Vec<crate::fingerprint::FingerprintType>,

    /// match的规则 or pattern
    pub matched_rules: Vec<String>,

    /// behaviortrait
    pub behavior_features: Vec<String>,

    /// 异常指标
    pub anomalies: Vec<String>,

    /// 额outsideinfo
    pub additional_info: std::collections::HashMap<String, String>,
}

impl AnalysisDetails {
    /// Create a newanalysis详情
    pub fn new() -> Self {
        Self::default()
    }

    /// Addmatch的规则
    pub fn add_matched_rule(&mut self, rule: String) {
        self.matched_rules.push(rule);
    }

    /// Addbehaviortrait
    pub fn add_behavior_feature(&mut self, feature: String) {
        self.behavior_features.push(feature);
    }

    /// Add异常指标
    pub fn add_anomaly(&mut self, anomaly: String) {
        self.anomalies.push(anomaly);
    }
}

/// systemlevelanalysisresult
///
/// includinganalysisresult、威胁type、风险评分等info。
pub struct SystemAnalysisResult {
    /// detect to 的fingerprintlist
    /// Note: 由于 trait object 的limit，这里不能directly Clone
    fingerprints: Vec<Box<dyn Fingerprint>>,

    /// 风险评分 (0.0 - 1.0)
    /// - 0.0: 完全security
    /// - 1.0: 极高风险
    pub risk_score: f64,

    /// 置信度 (0.0 - 1.0)
    /// - 0.0: 完全不确信
    /// - 1.0: 完全确信
    pub confidence: f64,

    /// 威胁typelist
    pub threat_types: Vec<ThreatType>,

    /// analysis详情
    pub details: AnalysisDetails,
}

impl SystemAnalysisResult {
    /// Create a newanalysisresult
    pub fn new() -> Self {
        Self {
            fingerprints: Vec::new(),
            risk_score: 0.0,
            confidence: 0.0,
            threat_types: Vec::new(),
            details: AnalysisDetails::new(),
        }
    }

    /// Createsecurity的result（无威胁）
    pub fn safe() -> Self {
        Self {
            fingerprints: Vec::new(),
            risk_score: 0.0,
            confidence: 1.0,
            threat_types: Vec::new(),
            details: AnalysisDetails::new(),
        }
    }

    /// Addfingerprint
    pub fn add_fingerprint(&mut self, fingerprint: Box<dyn Fingerprint>) {
        self.fingerprints.push(fingerprint);
    }

    /// Getallfingerprint的reference
    pub fn fingerprints(&self) -> &[Box<dyn Fingerprint>] {
        &self.fingerprints
    }

    /// Add威胁type
    pub fn add_threat_type(&mut self, threat_type: ThreatType) {
        if !self.threat_types.contains(&threat_type) {
            self.threat_types.push(threat_type);
            // Based on威胁typeUpdate风险评分
            self.update_risk_score();
        }
    }

    /// Update风险评分（based on威胁type）
    fn update_risk_score(&mut self) {
        if self.threat_types.is_empty() {
            self.risk_score = 0.0;
        } else {
            // use最高严重程度as风险评分
            self.risk_score = self
                .threat_types
                .iter()
                .map(|t| t.severity())
                .fold(0.0, f64::max);
        }
    }

    /// judgewhether exists威胁
    pub fn has_threats(&self) -> bool {
        !self.threat_types.is_empty() && self.risk_score > 0.0
    }

    /// judgewhether为高风险
    pub fn is_high_risk(&self) -> bool {
        self.risk_score >= 0.7
    }
}

impl Default for SystemAnalysisResult {
    fn default() -> Self {
        Self::new()
    }
}

// Manual implementation Debug，because Box<dyn Fingerprint> 不能automaticimplement Debug
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

// Manual implementation Clone，because Box<dyn Fingerprint> 不能automatic Clone
impl Clone for SystemAnalysisResult {
    fn clone(&self) -> Self {
        // Note: fingerprints 不能 Clone，so新instance from emptyliststart
        Self {
            fingerprints: Vec::new(), // 不能 Clone trait object
            risk_score: self.risk_score,
            confidence: self.confidence,
            threat_types: self.threat_types.clone(),
            details: self.details.clone(),
        }
    }
}

/// systemlevelanalysisinterface
///
/// allsystemlevelanalysis器都shouldimplement这个 trait。
///
/// ## Core Concept
///
/// systemlevelanalysis from **system角度**analysisnetworktraffic：
/// - not onlyonly是singleprotocol的Parse，而是综合analysis
/// - 考虑systemwhole的behaviorpattern
/// - detectsystemlevel的威胁（DDoS、scan、异常traffic等）
///
/// ## Implementation Example
///
/// ```rust
/// use fingerprint_core::system::{SystemAnalyzer, NetworkFlow, SystemAnalysisResult};
///
/// struct MySystemAnalyzer;
///
/// impl SystemAnalyzer for MySystemAnalyzer {
///     fn analyze(&self, flow: &NetworkFlow) -> SystemAnalysisResult {
///         // implementanalysis逻辑
///         SystemAnalysisResult::safe()
///     }
///
///     fn analyze_batch(&self, flows: &[NetworkFlow]) -> Vec<SystemAnalysisResult> {
///         flows.iter().map(|f| self.analyze(f)).collect()
///     }
/// }
/// ```
pub trait SystemAnalyzer: Send + Sync {
    /// analysisnetworktraffic
    ///
    /// # Parameters
    ///
    /// - `flow`: 要analysis的networktraffic
    ///
    /// # Returns
    ///
    /// systemlevelanalysisresult，includingfingerprint、威胁type、风险评分等info
    fn analyze(&self, flow: &NetworkFlow) -> SystemAnalysisResult;

    /// bulkanalysismultipletraffic
    ///
    /// # Parameters
    ///
    /// - `flows`: 要analysis的networktrafficlist
    ///
    /// # Returns
    ///
    /// analysisresultlist
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
