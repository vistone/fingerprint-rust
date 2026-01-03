//! # fingerprint-defense
//!
//! **systemlevelprotectionimplementlayer**，based on `fingerprint-core` systemlevelabstractBuild。
//!
//! ## corefixedbit
//!
//! `fingerprint-defense` is `fingerprint-core` systemlevelprotectioninterfaceconcreteimplement。
//!
//! ## itemfrontalreadyimplementFeatures
//!
//! - ✅ **passivefingerprintidentify** (`passive`): TCP/IP (p0f), HTTP, TLS passiveidentify
//! - ✅ **crosslayerconsistency审calculate** (`consistency`): JA4+ consistencyvalidate
//! - ✅ **fingerprintdatabase** (`database`): SQLite store and analysisfingerprinttrait
//! - ✅ **learnmechanism** (`learner`): automaticdiscover and recordnot知fingerprint
//! - ✅ **countpacketcapture** (`capture`): 纯 Rust implementactual when network interface and pcap filecapture (nonesystemdepend)
//!
//! ## planinFeatures
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
