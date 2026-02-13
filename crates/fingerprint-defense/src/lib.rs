//! # fingerprint-defense
//!
//! **system-level protectionimplementlayer**, based on `fingerprint-core` system-level abstractionsBuild.
//!
//! ## core positioning
//!
//! `fingerprint-defense` is `fingerprint-core` system-level protection interfaceconcreteimplement.
//!
//! ## itemfrontalreadyimplementFeatures
//!
//! - ✅ **passivefingerprintidentify** (`passive`): TCP/IP (p0f), HTTP, TLS passiveidentify
//! - ✅ **crosslayerconsistencyreviewcalculate** (`consistency`): JA4+ consistencyvalidate
//! - ✅ **fingerprintdatabase** (`database`): SQLite store and analysisfingerprinttrait
//! - ✅ **learnmechanism** (`learner`): automaticdiscover and recordnot知fingerprint
//! - ✅ **countpacketcapture** (`capture`): pure Rust implementactual when network interface and pcap filecapture (nonesystemdepend)
//!
//! ## planinFeatures
//!
//! - **systemanalysiser** (`analyzer`): implement `SystemAnalyzer` trait
//! - **systemprotectioner** (`protector`): implement `SystemProtector` trait
//! - **threat狩猎** (`hunting`): honeypot and behavioranalysis

pub mod capture;
pub mod database;
pub mod learner;
pub mod middleware; // API gateway middleware integration
pub mod passive;

pub use capture::CaptureEngine;
pub use database::FingerprintDatabase;
pub use learner::SelfLearningAnalyzer;

// Re-export maintype
pub use passive::{
    ConsistencyAnalyzer, HttpFingerprint, Packet, PacketParser, PassiveAnalysisResult,
    PassiveAnalyzer, PassiveError, TcpFingerprint, TlsFingerprint,
};

// Re-export middleware
pub use middleware::{
    rate_limiting_integration::{SecurityCheckResult, SecurityMiddleware},
    ConsistencyCheckConfig, ConsistencyCheckMiddleware, ConsistencyCheckResult,
};
