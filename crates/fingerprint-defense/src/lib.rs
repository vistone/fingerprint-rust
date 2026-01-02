//! # fingerprint-defense
//!
//! **systemlevelprotectionimplementlayer**，based on `fingerprint-core` systemlevelabstractBuild。
//!
//! ## core定bit
//!
//! `fingerprint-defense` is `fingerprint-core` systemlevelprotectioninterfaceconcreteimplement。
//!
//! ## 目frontalreadyimplementFeatures
//!
//! - ✅ **passivefingerprintidentify** (`passive`): TCP/IP (p0f), HTTP, TLS passiveidentify
//! - ✅ **crosslayerconsistency审计** (`consistency`): JA4+ consistency校验
//! - ✅ **fingerprintdatabase** (`database`): SQLite store and analysisfingerprinttrait
//! - ✅ **learn机制** (`learner`): automaticdiscover and recordnot知fingerprint
//! - ✅ **countpacketcapture** (`capture`): 纯 Rust implement实 when network interface and pcap filecapture（无systemdepend）
//!
//! ## 计划inFeatures
//!
//! - **systemanalysiser** (`analyzer`): implement `SystemAnalyzer` trait
//! - **systemprotectioner** (`protector`): implement `SystemProtector` trait
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
