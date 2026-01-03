//! # fingerprint-defense
//!
//! **system levelprotectionimplementlayer**，based on `fingerprint-core` system levelabstractBuild。
//!
//! ## core定bit
//!
//! `fingerprint-defense` is `fingerprint-core` system levelprotectioninterfaceconcreteimplement。
//!
//! ## item frontalreadyimplementFeatures
//!
//! - ✅ **passivefingerprint identify** (`passive`): TCP/IP (p0f), HTTP, TLS passive identify
//! - ✅ **crosslayerconsistency审计** (`consistency`): JA4+ consistencyvalidate
//! - ✅ **fingerprintdatabase** (`database`): SQLite store and analysisfingerprinttrait
//! - ✅ **learnmechanism** (`learner`): automaticdis cover and recordnot know fingerprint
//! - ✅ **countpacketcapture** (`capture`): 纯 Rust implement实 when network interface and pcap filecapture (no system depend)
//!
//! ## planinFeatures
//!
//! - **system analysiser** (`analyzer`): implement `SystemAnalyzer` trait
//! - **system protectioner** (`protector`): implement `SystemProtector` trait
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
 ConsistencyAnalyzer, HttpFingerprint, Packet, Packet parsed r, PassiveAnalysisResult,
 PassiveAnalyzer, PassiveError, TcpFingerprint, TlsFingerprint,
};
