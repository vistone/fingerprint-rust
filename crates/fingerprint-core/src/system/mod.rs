//! systemlevelprotectioncoreabstract
//!
//! providesystemlevelprotectioncoreabstract and interface，includesystemupdown文、networktraffic、protectiondecision etc.。
//!
//! ## Core Concept
//!
//! from **单一serviceprotection**提升 to **systemlevelprotection**：
//! - from 网卡level拦截、analysis and protectionallnetworktraffic
//! - not onlyonly is HTTP，stillinclude TCP、UDP、ICMP etc.allprotocol
//! - systemleveldecision，can实施防火墙规则、traffic限速 etc.systemlevelprotection措施
//!
//! ## modulestruct
//!
//! - `context`: systemupdown文，includingnetworkentitycompleteinfo
//! - `flow`: networktrafficabstract，representsystemlevelnetworktraffic
//! - `protection`: systemlevelprotectioninterface and decision
//! - `analysis`: systemlevelanalysisinterface and result
//! - `stats`: systemlevelstatisticsinfo

pub mod analysis;
pub mod context;
pub mod flow;
pub mod protection;
pub mod stats;

// Re-export maintype and interface
pub use context::{ProtocolType, SystemContext, TrafficDirection};
pub use flow::{FlowCharacteristics, NetworkFlow};
pub use protection::{SystemProtectionDecision, SystemProtectionResult, SystemProtector};

// Note: NetworkFlow and SystemAnalysisResult implement了 Clone，
// butdue toincluding Box<dyn Fingerprint>，Clone when fingerprints fieldwill被清empty
pub use analysis::{AnalysisDetails, SystemAnalysisResult, SystemAnalyzer, ThreatType};
pub use stats::SystemProtectionStats;
