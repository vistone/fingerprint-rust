//! # fingerprint-defense
//!
//! **systemlevel防护的implementlayer**，基于 `fingerprint-core` 的systemlevel抽象Build。
//!
//! ## 核心定bit
//!
//! `fingerprint-defense` 是 `fingerprint-core` systemlevel防护interface的具体implement。
//!
//! ## 目frontalreadyimplement的Features
//!
//! - ✅ **被动fingerprint识别** (`passive`): TCP/IP (p0f), HTTP, TLS 被动识别
//! - ✅ **跨layer一致性审计** (`consistency`): JA4+ 一致性校验
//! - ✅ **fingerprintdatabase** (`database`): SQLite 存储 and analysisfingerprinttrait
//! - ✅ **学习机制** (`learner`): automatic发现 and recordnot知fingerprint
//! - ✅ **count据包捕获** (`capture`): 纯 Rust implement的实 when 网卡 and pcap file捕获（无system依赖）
//!
//! ## 计划中的Features
//!
//! - **systemanalysis器** (`analyzer`): implement `SystemAnalyzer` trait
//! - **system防护器** (`protector`): implement `SystemProtector` trait
//! - **威胁狩猎** (`hunting`): 蜜罐 and 行为analysis

pub mod capture;
pub mod database;
pub mod learner;
pub mod passive;

pub use capture::CaptureEngine;
pub use database::FingerprintDatabase;
pub use learner::SelfLearningAnalyzer;

// Re-export 主要type
pub use passive::{
    ConsistencyAnalyzer, HttpFingerprint, Packet, PacketParser, PassiveAnalysisResult,
    PassiveAnalyzer, PassiveError, TcpFingerprint, TlsFingerprint,
};
