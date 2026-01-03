//! system levelprotectioncoreabstract
//!
//! providesystem levelprotectioncoreabstract and interface，includesystem updown text 、networktraffic、protectiondecision etc.。
//!
//! ## Core Concept
//!
//! from **singleserviceprotection**improve to **system levelprotection**：
//! - from network interfacelevel拦截、analysis and protection all networktraffic
//! - not onlyonly is HTTP，stillinclude TCP、UDP、ICMP etc. all protocol
//! - system leveldecision，can实施防火墙rule、trafficrate limit etc.system levelprotectionmeasure
//!
//! ## modulestruct
//!
//! - `context`: system updown text ，includingnetworkentitycompleteinfo
//! - `flow`: networktrafficabstract，representsystem levelnetworktraffic
//! - `protection`: system levelprotectioninterface and decision
//! - `analysis`: system levelanalysisinterface and result
//! - `stats`: system levelstatisticsinfo

pub mod analysis;
pub mod context;
pub mod flow;
pub mod protection;
pub mod stats;

// Re-export maintype and interface
pub use context::{ProtocolType, SystemContext, TrafficDirection};
pub use flow::{FlowCharacteristics, NetworkFlow};
pub use protection::{SystemProtectionDecision, SystemProtectionResult, SystemProtector};

// Note: NetworkFlow and SystemAnalysisResult implement Clone，
// butdue toincluding Box<dyn Fingerprint>，Clone when fingerprints field will by 清empty
pub use analysis::{AnalysisDetails, SystemAnalysisResult, SystemAnalyzer, ThreatType};
pub use stats::SystemProtectionStats;
