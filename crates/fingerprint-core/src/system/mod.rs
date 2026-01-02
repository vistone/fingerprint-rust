//! systemlevel防护核心抽象
//!
//! providesystemlevel防护的核心抽象 and interface，包括systemupdown文、networktraffic、防护决策等。
//!
//! ## Core Concept
//!
//!  from **单一服务防护**提升 to **systemlevel防护**：
//! -  from 网卡level拦截、analysis and 防护allnetworktraffic
//! - not onlyonly是 HTTP，still包括 TCP、UDP、ICMP 等allprotocol
//! - systemlevel决策，can实施防火墙规则、traffic限速等systemlevel防护措施
//!
//! ## modulestruct
//!
//! - `context`: systemupdown文，includingnetwork实体的completeinfo
//! - `flow`: networktraffic抽象，表示systemlevel的networktraffic
//! - `protection`: systemlevel防护interface and 决策
//! - `analysis`: systemlevelanalysisinterface and result
//! - `stats`: systemlevelstatisticsinfo

pub mod analysis;
pub mod context;
pub mod flow;
pub mod protection;
pub mod stats;

// Re-export 主要type and interface
pub use context::{ProtocolType, SystemContext, TrafficDirection};
pub use flow::{FlowCharacteristics, NetworkFlow};
pub use protection::{SystemProtectionDecision, SystemProtectionResult, SystemProtector};

// Note: NetworkFlow  and SystemAnalysisResult implement了 Clone，
// but由于including Box<dyn Fingerprint>，Clone  when  fingerprints fieldwill被清empty
pub use analysis::{AnalysisDetails, SystemAnalysisResult, SystemAnalyzer, ThreatType};
pub use stats::SystemProtectionStats;
