//! 系统级别防护核心抽象
//!
//! 提供系统级别防护的核心抽象和接口，包括系统上下文、网络流量、防护决策等。
//!
//! ## 核心思想
//!
//! 从**单一服务防护**提升到**系统级别防护**：
//! - 从网卡级别拦截、分析和防护所有网络流量
//! - 不仅仅是 HTTP，还包括 TCP、UDP、ICMP 等所有协议
//! - 系统级别决策，可以实施防火墙规则、流量限速等系统级防护措施
//!
//! ## 模块结构
//!
//! - `context`: 系统上下文，包含网络实体的完整信息
//! - `flow`: 网络流量抽象，表示系统级别的网络流量
//! - `protection`: 系统级别防护接口和决策
//! - `analysis`: 系统级别分析接口和结果
//! - `stats`: 系统级别统计信息

pub mod analysis;
pub mod context;
pub mod flow;
pub mod protection;
pub mod stats;

// Re-export 主要类型和接口
pub use context::{ProtocolType, SystemContext, TrafficDirection};
pub use flow::{FlowCharacteristics, NetworkFlow};
pub use protection::{SystemProtectionDecision, SystemProtectionResult, SystemProtector};

// 注意：NetworkFlow 和 SystemAnalysisResult 实现了 Clone，
// 但由于包含 Box<dyn Fingerprint>，Clone 时 fingerprints 字段会被清空
pub use analysis::{AnalysisDetails, SystemAnalysisResult, SystemAnalyzer, ThreatType};
pub use stats::SystemProtectionStats;
