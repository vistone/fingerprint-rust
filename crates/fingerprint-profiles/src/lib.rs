//! # fingerprint-profiles
//!
//! Browser fingerprint profiles module

pub mod profiles;
pub mod version_adapter;
pub mod version_detector;
pub mod version_registry;
pub mod version_update;

pub use profiles::{mapped_tls_clients, BrowserProfile, ProfileMetadata};
pub use version_adapter::VersionAdapter;
pub use version_detector::VersionDetector;
pub use version_registry::{BrowserType, VersionEntry, VersionRegistry};
pub use version_update::VersionUpdateManager;
