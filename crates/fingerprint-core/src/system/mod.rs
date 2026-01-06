//! system-level protectioncoreabstract
//!
//! providesystem-level protectioncoreabstract and interface, includesystem context, network traffic, protection decision etc..
//!
//! ## Core Concept
//!
//! from **single service protection**improve to **system-level protection**：
//! - from network interfacelevelintercept, analysis and protectionallnetwork traffic
//! - not onlyonly is HTTP, stillinclude TCP, UDP, ICMP etc.allprotocol
//! - systemleveldecision, can actually implementfirewallrule, trafficrate limit etc.system-level protectionmeasure
//!
//! ## modulestruct
//!
//! - `context`: system context, includingcomplete network entity information
//! - `flow`: network trafficabstract, representsystemlevelnetwork traffic
//! - `protection`: system-level protection interface and decision
//! - `analysis`: systemlevelanalysis interface and result
//! - `stats`: systemlevelstatisticsinfo pub mod analysis;
pub mod context;
pub mod flow;
pub mod protection;
pub mod stats; // Re-export maintype and interface
pub use context::{ProtocolType, SystemContext, TrafficDirection};
pub use flow::{FlowCharacteristics, NetworkFlow};
pub use protection::{SystemProtectionDecision, SystemProtectionResult, SystemProtector}; // Note: NetworkFlow and SystemAnalysisResult implement Clone,
// butdue toincluding Box<dyn Fingerprint>, Clone when fingerprints fieldwill be cleared
pub use analysis::{AnalysisDetails, SystemAnalysisResult, SystemAnalyzer, ThreatType};
pub use stats::SystemProtectionStats;
