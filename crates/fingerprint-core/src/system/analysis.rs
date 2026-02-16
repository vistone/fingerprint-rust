//! systemlevelanalysis interface
//!
//! definesystemlevelanalysis interface and resulttype.

use super::flow::NetworkFlow;
use crate::fingerprint::Fingerprint;

/// threattype
///
/// representdetect to threattype.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ThreatType {
    /// not知fingerprint
    UnknownFingerprint,

    /// suspiciousbehavior
    SuspiciousBehavior,

    /// already知attack
    KnownAttack,

    /// abnormaltrafficpattern
    AbnormalTrafficPattern,

    /// malicious IP
    MaliciousIP,

    /// DDoS attack
    DDoS,

    /// portscan
    PortScan,

    /// brute force
    BruteForce,
}

impl ThreatType {
    /// convert tostring
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::UnknownFingerprint => "not知fingerprint",
            Self::SuspiciousBehavior => "suspiciousbehavior",
            Self::KnownAttack => "already知attack",
            Self::AbnormalTrafficPattern => "abnormaltrafficpattern",
            Self::MaliciousIP => "maliciousIP",
            Self::DDoS => "DDoSattack",
            Self::PortScan => "portscan",
            Self::BruteForce => "brute force",
        }
    }

    /// Getthreatseverity (0.0 - 1.0)
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

/// analysisdetails
///
/// includinganalysisdetailedinfo and verifydata.
#[derive(Debug, Clone, Default)]
pub struct AnalysisDetails {
    /// detect to fingerprinttype
    pub fingerprint_types: Vec<crate::fingerprint::FingerprintType>,

    /// matchrule or pattern
    pub matched_rules: Vec<String>,

    /// behaviortrait
    pub behavior_features: Vec<String>,

    /// abnormalindicator
    pub anomalies: Vec<String>,

    /// 额outsideinfo
    pub additional_info: std::collections::HashMap<String, String>,
}

impl AnalysisDetails {
    /// Create a newanalysisdetails
    pub fn new() -> Self {
        Self::default()
    }

    /// Addmatchrule
    pub fn add_matched_rule(&mut self, rule: String) {
        self.matched_rules.push(rule);
    }

    /// Addbehaviortrait
    pub fn add_behavior_feature(&mut self, feature: String) {
        self.behavior_features.push(feature);
    }

    /// Addabnormalindicator
    pub fn add_anomaly(&mut self, anomaly: String) {
        self.anomalies.push(anomaly);
    }
}

/// systemlevelanalysisresult
///
/// includinganalysisresult, threattype, risk score etc.info.
pub struct SystemAnalysisResult {
    /// detect to fingerprintlist
    /// Note: due to trait object limit, herecannotdirectly Clone
    fingerprints: Vec<Box<dyn Fingerprint>>,

    /// risk score (0.0 - 1.0)
    /// - 0.0: completelysecurity
    /// - 1.0: 极highrisk
    pub risk_score: f64,

    /// confidence (0.0 - 1.0)
    /// - 0.0: completelynot confident
    /// - 1.0: completelyconfident
    pub confidence: f64,

    /// threattypelist
    pub threat_types: Vec<ThreatType>,

    /// analysisdetails
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

    /// Createsecurityresult (nonethreat)
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

    /// Getallfingerprintreference
    pub fn fingerprints(&self) -> &[Box<dyn Fingerprint>] {
        &self.fingerprints
    }

    /// Addthreattype
    pub fn add_threat_type(&mut self, threat_type: ThreatType) {
        if !self.threat_types.contains(&threat_type) {
            self.threat_types.push(threat_type);
            // Based onthreattypeUpdaterisk score
            self.update_risk_score();
        }
    }

    /// Updaterisk score (based onthreattype)
    fn update_risk_score(&mut self) {
        if self.threat_types.is_empty() {
            self.risk_score = 0.0;
        } else {
            // usemosthighseverityasrisk score
            self.risk_score = self
                .threat_types
                .iter()
                .map(|t| t.severity())
                .fold(0.0, f64::max);
        }
    }

    /// judgewhether existsthreat
    pub fn has_threats(&self) -> bool {
        !self.threat_types.is_empty() && self.risk_score > 0.0
    }

    /// judgewhether as highrisk
    pub fn is_high_risk(&self) -> bool {
        self.risk_score >= 0.7
    }
}

impl Default for SystemAnalysisResult {
    fn default() -> Self {
        Self::new()
    }
}

// Manual implementation Debug, because Box<dyn Fingerprint> cannotautomaticimplement Debug
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

// Manual implementation Clone, because Box<dyn Fingerprint> cannotautomatic Clone
impl Clone for SystemAnalysisResult {
    fn clone(&self) -> Self {
        // Note: fingerprints cannot Clone, sonewinstance from emptyliststart
        Self {
            fingerprints: Vec::new(), // cannot Clone trait object
            risk_score: self.risk_score,
            confidence: self.confidence,
            threat_types: self.threat_types.clone(),
            details: self.details.clone(),
        }
    }
}

/// systemlevelanalysis interface
///
/// allsystemlevelanalysiser都shouldimplementthis trait.
///
/// ## Core Concept
///
/// systemlevelanalysis from **systemperspective**analysisnetwork traffic：
/// - not onlyonly is singleprotocolParse, 而 is comprehensiveanalysis
/// - considersystemwholebehaviorpattern
/// - detectsystemlevelthreat (DDoS, scan, abnormaltraffic etc.)
///
/// ## Implementation Example
///
/// ```rust
/// use fingerprint_core::system::{SystemAnalyzer, NetworkFlow, SystemAnalysisResult};
///
/// struct MySystemAnalyzer;
///
/// impl SystemAnalyzer for MySystemAnalyzer {
/// fn analyze(&self, flow: &NetworkFlow) -> SystemAnalysisResult {
/// // implementanalysislogic
/// SystemAnalysisResult::safe()
/// }
///
/// fn analyze_batch(&self, flows: &[NetworkFlow]) -> Vec<SystemAnalysisResult> {
/// flows.iter().map(|f| self.analyze(f)).collect()
/// }
/// }
/// ```
pub trait SystemAnalyzer: Send + Sync {
    /// analysisnetwork traffic
    ///
    /// # Parameters
    ///
    /// - `flow`: needanalysisnetwork traffic
    ///
    /// # Returns
    ///
    /// systemlevelanalysisresult, includingfingerprint, threattype, risk score etc.info
    fn analyze(&self, flow: &NetworkFlow) -> SystemAnalysisResult;

    /// bulkanalysismultipletraffic
    ///
    /// # Parameters
    ///
    /// - `flows`: needanalysisnetwork trafficlist
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
