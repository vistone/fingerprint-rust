//! # fingerprint-defense
//!
//! **systemlevelprotection的implementlayer**，based on `fingerprint-core` 的systemlevelabstractBuild。
//!
//! ## core定bit
//!
//! `fingerprint-defense` 是 `fingerprint-core` systemlevelprotectioninterface的concreteimplement。
//!
//! ## 目frontalreadyimplement的Features
//!
//! - ✅ **passivefingerprintidentify** (`passive`): TCP/IP (p0f), HTTP, TLS passiveidentify
//! - ✅ **crosslayerconsistency审计** (`consistency`): JA4+ consistency校验
//! - ✅ **fingerprintdatabase** (`database`): SQLite store and analysisfingerprinttrait
//! - ✅ **学习机制** (`learner`): automaticdiscover and recordnot知fingerprint
//! - ✅ **countpacketcapture** (`capture`): 纯 Rust implement的实 when 网卡 and pcap filecapture（无systemdepend）
//!
//! ## 计划inFeatures
//!
//! - **systemanalysis器** (`analyzer`): implement `SystemAnalyzer` trait
//! - **systemprotection器** (`protector`): implement `SystemProtector` trait
//! - **threat狩猎** (`hunting`): 蜜罐 and behavioranalysis

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
