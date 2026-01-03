//! systemlevelprotectioncoreabstract
//!
//! providesystemlevelprotectioncoreabstract and interface，includesystemupdowntext、networktraffic、protectiondecision etc.。
//!
//! ## Core Concept
//!
//! from **singleserviceprotection**improve to **systemlevelprotection**：
//! - from network interfacelevelintercept、analysis and protectionallnetworktraffic
//! - not onlyonly is HTTP，stillinclude TCP、UDP、ICMP etc.allprotocol
//! - systemleveldecision，canactual施firewallrule、trafficrate limit etc.systemlevelprotectionmeasure
//!
//! ## modulestruct
//!
//! - `context`: systemupdowntext，includingnetworkentitycompleteinfo
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
