//! # fingerprint-defense
//!
//! **systemlevelprotection的implementlayer**，based on `fingerprint-core` 的systemlevel抽象Build。
//!
//! ## core定bit
//!
//! `fingerprint-defense` 是 `fingerprint-core` systemlevelprotectioninterface的具体implement。
//!
//! ## 目frontalreadyimplement的Features
//!
//! - ✅ **被动fingerprintidentify** (`passive`): TCP/IP (p0f), HTTP, TLS 被动identify
//! - ✅ **跨layer一致性审计** (`consistency`): JA4+ 一致性校验
//! - ✅ **fingerprintdatabase** (`database`): SQLite store and analysisfingerprinttrait
//! - ✅ **学习机制** (`learner`): automatic发现 and recordnot知fingerprint
//! - ✅ **countpacket捕获** (`capture`): 纯 Rust implement的实 when 网卡 and pcap file捕获（无systemdepend）
//!
//! ## 计划中的Features
//!
//! - **systemanalysis器** (`analyzer`): implement `SystemAnalyzer` trait
//! - **systemprotection器** (`protector`): implement `SystemProtector` trait
//! - **威胁狩猎** (`hunting`): 蜜罐 and behavioranalysis

pub mod capture;
pub mod database;
pub mod learner;
pub mod passive;

pub use capture::CaptureEngine;
pub use database::FingerprintDatabase;
pub use learner::SelfLearningAnalyzer;

// Re-export maintype
pub use passive::{
    ConsistencyAnalyzer, HttpFingerprint, Packet, PacketParser, PassiveAnalysisResult,
    PassiveAnalyzer, PassiveError, TcpFingerprint, TlsFingerprint,
};
