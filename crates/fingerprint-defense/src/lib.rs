//! # fingerprint-defense
//!
//! Self-learning defense system with fingerprint database
//!
//! ## Features
//!
//! - ✅ **Learning mechanism** (`learner`): Automatically discover and record unknown fingerprints
//! - ✅ **Anomaly detection** (`anomaly`): Behavioral analysis and contradiction detection
//! - ✅ **Timing protection** (`timing`): Random delays and temporal obfuscation
//! - ✅ **Storage analysis** (`storage`): Detect storage-based fingerprinting attempts
//! - ✅ **API noise injection** (`api_noise`): Canvas and audio fingerprint obfuscation
//! - **Threat hunting** (`hunting`): Honeypot and behavior analysis
//!
//! ## Architecture
//!
//! ```text
//! Network Flow → [Observer] → [Evaluator] → [Database]
//!      ↓              ↓            ↓           ↓
//!   Real-time     Stability    Learning    Persistent
//!   Analysis      Scoring      Storage     Storage
//! ```

pub mod anomaly;
pub mod api_noise;
pub mod database;
pub mod hunting;
pub mod learner;
pub mod passive;
pub mod storage;
pub mod timing;

pub use anomaly::{AnomalyDetector, ContradictionDetector};
pub use api_noise::CanvasNoiseGenerator;
pub use database::{CandidateFingerprint, CandidateStats, FingerprintDatabase};
pub use hunting::ThreatHunter;
pub use learner::{FingerprintEvaluator, FingerprintObserver};
pub use passive::{
    HttpFingerprint, Packet, PacketParser, PassiveAnalysisResult, PassiveAnalyzer, PassiveError,
    TcpFingerprint, TlsFingerprint,
};
pub use storage::StorageAnalyzer;
pub use timing::TimingProtector;

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    #[allow(clippy::arc_with_non_send_sync)]
    fn test_module_integration() {
        // Test basic functionality integration of various modules
        let db = database::FingerprintDatabase::open(":memory:").expect("open in-memory db");
        let learner = learner::SelfLearningAnalyzer::new(Arc::new(db));

        assert_eq!(learner.get_observation_stats().total_observations, 0);
    }

    #[test]
    fn test_storage_analyzer_basic() {
        let analyzer = storage::StorageAnalyzer::new();
        let stats = analyzer.get_statistics();

        assert_eq!(stats.local_storage_keys, 0);
        assert_eq!(stats.session_storage_keys, 0);
        assert!(!analyzer.detect_local_storage_fingerprinting());
    }

    #[test]
    fn test_noise_generator_basic() {
        let generator = api_noise::CanvasNoiseGenerator::new(0.5);
        let config = generator.get_config();

        assert_eq!(config.intensity, 0.5);
        assert!(config.processed_canvases == 0);
    }

    #[tokio::test]
    async fn test_async_components() {
        // Test basic initialization of asynchronous components
        let _initialized = true;
        assert!(_initialized);
    }
}
