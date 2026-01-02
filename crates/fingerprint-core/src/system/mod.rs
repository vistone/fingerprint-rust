//! systemlevelprotectioncore抽象
//!
//! providesystemlevelprotection的core抽象 and interface，includesystemupdown文、networktraffic、protection决策等。
//!
//! ## Core Concept
//!
//!  from **单一serviceprotection**提升 to **systemlevelprotection**：
//! -  from 网卡level拦截、analysis and protectionallnetworktraffic
//! - not onlyonly是 HTTP，stillinclude TCP、UDP、ICMP 等allprotocol
//! - systemlevel决策，can实施防火墙规则、traffic限速等systemlevelprotection措施
//!
//! ## modulestruct
//!
//! - `context`: systemupdown文，includingnetworkentity的completeinfo
//! - `flow`: networktraffic抽象，representsystemlevel的networktraffic
//! - `protection`: systemlevelprotectioninterface and 决策
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

// Note: NetworkFlow  and SystemAnalysisResult implement了 Clone，
// but由于including Box<dyn Fingerprint>，Clone  when  fingerprints fieldwill被清empty
pub use analysis::{AnalysisDetails, SystemAnalysisResult, SystemAnalyzer, ThreatType};
pub use stats::SystemProtectionStats;
