//! # fingerprint-defense
//!
//! **系统级别防护的实现层**，基于 `fingerprint-core` 的系统级别抽象构建。
//!
//! ## 核心定位
//!
//! `fingerprint-defense` 是 `fingerprint-core` 系统级别防护接口的具体实现。
//!
//! ## 目前已实现的功能
//!
//! - ✅ **被动指纹识别** (`passive`): TCP/IP (p0f), HTTP, TLS 被动识别
//! - ✅ **跨层一致性审计** (`consistency`): JA4+ 一致性校验
//! - ✅ **指纹数据库** (`database`): SQLite 存储和分析指纹特征
//! - ✅ **学习机制** (`learner`): 自动发现和记录未知指纹
//! - ✅ **数据包捕获** (`capture`): 纯 Rust 实现的实时网卡和 pcap 文件捕获（无系统依赖）
//!
//! ## 计划中的功能
//!
//! - **系统分析器** (`analyzer`): 实现 `SystemAnalyzer` trait
//! - **系统防护器** (`protector`): 实现 `SystemProtector` trait
//! - **威胁狩猎** (`hunting`): 蜜罐和行为分析

pub mod capture;
pub mod database;
pub mod learner;
pub mod passive;

pub use capture::CaptureEngine;
pub use database::FingerprintDatabase;
pub use learner::SelfLearningAnalyzer;

// Re-export 主要类型
pub use passive::{
    ConsistencyAnalyzer, HttpFingerprint, Packet, PacketParser, PassiveAnalysisResult,
    PassiveAnalyzer, PassiveError, TcpFingerprint, TlsFingerprint,
};
