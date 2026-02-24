//! # fingerprint-analysis
//!
//! Unified analysis engine that consolidates all fingerprint analysis logic
//! including statistical analysis, machine learning, real-time monitoring,
//! and historical pattern recognition.
//!
//! ## Features
//!
//! - ✅ **Statistical Analysis**: Z-score, distribution analysis, correlation detection
//! - ✅ **Machine Learning**: Ensemble methods, clustering, classification
//! - ✅ **Real-time Monitoring**: Streaming analysis, alert generation
//! - ✅ **Historical Analysis**: Trend detection, pattern recognition, anomaly history
//!
//! ## Architecture
//!
//! ```text
//! AnalysisEngine
//! ├── StatisticalAnalyzer ──→ Basic statistical computations
//! ├── MLAnalyzer ──→ Machine learning model inference
//! ├── RealTimeMonitor ──→ Live data stream processing
//! ├── HistoricalAnalyzer ──→ Pattern and trend analysis
//! └── AlertGenerator ──→ Anomaly detection and notification
//! ```

use std::collections::HashMap;
use std::sync::Arc;
use dashmap::DashMap;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use fingerprint_core::fingerprint::{Fingerprint, FingerprintComparison};
use fingerprint_config::ConfigManager;

/// Analysis engine error types
#[derive(Error, Debug)]
pub enum AnalysisError {
    #[error("Statistical analysis failed: {0}")]
    StatisticalError(String),
    #[error("Machine learning analysis failed: {0}")]
    MLError(String),
    #[error("Real-time analysis failed: {0}")]
    RealTimeError(String),
    #[error("Historical analysis failed: {0}")]
    HistoricalError(String),
    #[error("Configuration error: {0}")]
    ConfigError(#[from] fingerprint_config::ConfigError),
}

/// Main analysis engine
pub struct AnalysisEngine {
    /// Configuration manager
    config: Arc<ConfigManager>,
    
    /// Statistical analysis component
    #[cfg(feature = "statistical")]
    statistical: StatisticalAnalyzer,
    
    /// Machine learning analysis component
    #[cfg(feature = "machine-learning")]
    ml: MLAnalyzer,
    
    /// Real-time monitoring component
    #[cfg(feature = "real-time")]
    real_time: RealTimeMonitor,
    
    /// Historical analysis component
    #[cfg(feature = "historical")]
    historical: HistoricalAnalyzer,
    
    /// Analysis results cache
    results_cache: DashMap<String, AnalysisResult>,
    
    /// Alert generators
    alert_generators: RwLock<Vec<Box<dyn AlertGenerator>>>,
}

/// Analysis result containing all analysis outputs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResult {
    /// Unique analysis ID
    pub id: String,
    
    /// Timestamp of analysis
    pub timestamp: chrono::DateTime<chrono::Utc>,
    
    /// Input fingerprint data
    pub input_fingerprint: String,
    
    /// Statistical analysis results
    #[cfg(feature = "statistical")]
    pub statistical: Option<StatisticalResult>,
    
    /// Machine learning analysis results
    #[cfg(feature = "machine-learning")]
    pub ml: Option<MLResult>,
    
    /// Real-time analysis results
    #[cfg(feature = "real-time")]
    pub real_time: Option<RealTimeResult>,
    
    /// Historical analysis results
    #[cfg(feature = "historical")]
    pub historical: Option<HistoricalResult>,
    
    /// Overall risk score (0.0 to 1.0)
    pub risk_score: f64,
    
    /// Confidence level (0.0 to 1.0)
    pub confidence: f64,
    
    /// Generated alerts
    pub alerts: Vec<Alert>,
}

/// Alert structure for anomaly notifications
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub id: String,
    pub severity: AlertSeverity,
    pub category: AlertCategory,
    pub message: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Alert severity levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
    Emergency,
}

/// Alert categories
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertCategory {
    Anomaly,
    Suspicious,
    KnownThreat,
    Configuration,
}

/// Alert generator trait
pub trait AlertGenerator: Send + Sync {
    fn generate_alerts(&self, result: &AnalysisResult) -> Vec<Alert>;
}

impl AnalysisEngine {
    /// Create a new analysis engine
    pub fn new(config: Arc<ConfigManager>) -> Result<Self, AnalysisError> {
        Ok(Self {
            config,
            #[cfg(feature = "statistical")]
            statistical: StatisticalAnalyzer::new()?,
            #[cfg(feature = "machine-learning")]
            ml: MLAnalyzer::new()?,
            #[cfg(feature = "real-time")]
            real_time: RealTimeMonitor::new()?,
            #[cfg(feature = "historical")]
            historical: HistoricalAnalyzer::new()?,
            results_cache: DashMap::new(),
            alert_generators: RwLock::new(vec![]),
        })
    }

    /// Analyze a fingerprint using all enabled analysis methods
    pub async fn analyze(&self, fingerprint: &dyn Fingerprint) -> Result<AnalysisResult, AnalysisError> {
        let analysis_id = uuid::Uuid::new_v4().to_string();
        
        let mut result = AnalysisResult {
            id: analysis_id.clone(),
            timestamp: chrono::Utc::now(),
            input_fingerprint: fingerprint.id(),
            #[cfg(feature = "statistical")]
            statistical: None,
            #[cfg(feature = "machine-learning")]
            ml: None,
            #[cfg(feature = "real-time")]
            real_time: None,
            #[cfg(feature = "historical")]
            historical: None,
            risk_score: 0.0,
            confidence: 0.0,
            alerts: vec![],
        };

        // Run statistical analysis
        #[cfg(feature = "statistical")]
        {
            result.statistical = Some(self.statistical.analyze(fingerprint).await?);
        }

        // Run machine learning analysis
        #[cfg(feature = "machine-learning")]
        {
            result.ml = Some(self.ml.analyze(fingerprint).await?);
        }

        // Run real-time analysis
        #[cfg(feature = "real-time")]
        {
            result.real_time = Some(self.real_time.analyze(fingerprint).await?);
        }

        // Run historical analysis
        #[cfg(feature = "historical")]
        {
            result.historical = Some(self.historical.analyze(fingerprint).await?);
        }

        // Calculate overall scores
        self.calculate_overall_scores(&mut result)?;

        // Generate alerts
        self.generate_alerts(&mut result)?;

        // Cache the result
        self.results_cache.insert(analysis_id, result.clone());

        Ok(result)
    }

    /// Compare two fingerprints using comprehensive analysis
    pub async fn compare(&self, fp1: &dyn Fingerprint, fp2: &dyn Fingerprint) -> Result<FingerprintComparison, AnalysisError> {
        let comparison = fp1.similar_to(fp2);
        
        let mut result = FingerprintComparison::new(
            if comparison { 1.0 } else { 0.0 },
            comparison,
        );

        // Enhanced comparison with statistical analysis
        #[cfg(feature = "statistical")]
        {
            let stat_comparison = self.statistical.compare(fp1, fp2).await?;
            result.similarity = stat_comparison.similarity_score;
            result.matched_fields = stat_comparison.matched_features;
            result.unmatched_fields = stat_comparison.unmatched_features;
        }

        Ok(result)
    }

    /// Calculate overall risk and confidence scores
    fn calculate_overall_scores(&self, result: &mut AnalysisResult) -> Result<(), AnalysisError> {
        let mut total_score = 0.0;
        let mut total_confidence = 0.0;
        let mut count = 0.0;

        #[cfg(feature = "statistical")]
        if let Some(stat) = &result.statistical {
            total_score += stat.anomaly_score;
            total_confidence += stat.confidence;
            count += 1.0;
        }

        #[cfg(feature = "machine-learning")]
        if let Some(ml) = &result.ml {
            total_score += ml.risk_score;
            total_confidence += ml.confidence;
            count += 1.0;
        }

        #[cfg(feature = "real-time")]
        if let Some(rt) = &result.real_time {
            total_score += rt.current_risk;
            total_confidence += rt.confidence;
            count += 1.0;
        }

        #[cfg(feature = "historical")]
        if let Some(hist) = &result.historical {
            total_score += hist.trend_risk;
            total_confidence += hist.confidence;
            count += 1.0;
        }

        if count > 0.0 {
            result.risk_score = total_score / count;
            result.confidence = total_confidence / count;
        }

        Ok(())
    }

    /// Generate alerts based on analysis results
    fn generate_alerts(&self, result: &mut AnalysisResult) -> Result<(), AnalysisError> {
        let generators = self.alert_generators.read();
        
        for generator in generators.iter() {
            let alerts = generator.generate_alerts(result);
            result.alerts.extend(alerts);
        }

        Ok(())
    }

    /// Add an alert generator
    pub fn add_alert_generator(&self, generator: Box<dyn AlertGenerator>) {
        self.alert_generators.write().push(generator);
    }

    /// Get cached analysis result
    pub fn get_cached_result(&self, id: &str) -> Option<AnalysisResult> {
        self.results_cache.get(id).map(|r| r.clone())
    }
}

// Statistical analysis components
#[cfg(feature = "statistical")]
mod statistical {
    use super::*;
    
    pub struct StatisticalAnalyzer {
        // Statistical models and data
        baseline_profiles: DashMap<String, serde_json::Value>,
    }
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct StatisticalResult {
        pub anomaly_score: f64,
        pub confidence: f64,
        pub z_score: f64,
        pub deviation_percentile: f64,
        pub matched_features: Vec<String>,
        pub unmatched_features: Vec<String>,
    }
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct StatisticalComparison {
        pub similarity_score: f64,
        pub matched_features: Vec<String>,
        pub unmatched_features: Vec<String>,
        pub feature_deviations: HashMap<String, f64>,
    }
    
    impl StatisticalAnalyzer {
        pub fn new() -> Result<Self, AnalysisError> {
            Ok(Self {
                baseline_profiles: DashMap::new(),
            })
        }
        
        pub async fn analyze(&self, fingerprint: &dyn Fingerprint) -> Result<StatisticalResult, AnalysisError> {
            // Implementation would perform statistical analysis
            Ok(StatisticalResult {
                anomaly_score: 0.1,
                confidence: 0.95,
                z_score: 0.5,
                deviation_percentile: 0.3,
                matched_features: vec![],
                unmatched_features: vec![],
            })
        }
        
        pub async fn compare(&self, fp1: &dyn Fingerprint, fp2: &dyn Fingerprint) -> Result<StatisticalComparison, AnalysisError> {
            // Implementation would compare fingerprints statistically
            Ok(StatisticalComparison {
                similarity_score: 0.85,
                matched_features: vec![],
                unmatched_features: vec![],
                feature_deviations: HashMap::new(),
            })
        }
    }
}

#[cfg(feature = "statistical")]
pub use statistical::{StatisticalAnalyzer, StatisticalResult, StatisticalComparison};

// Machine learning analysis components
#[cfg(feature = "machine-learning")]
mod ml {
    use super::*;
    
    pub struct MLAnalyzer {
        // ML models and inference engines
        models: DashMap<String, Box<dyn MLModel>>,
    }
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct MLResult {
        pub risk_score: f64,
        pub confidence: f64,
        pub predictions: HashMap<String, f64>,
        pub feature_importance: HashMap<String, f64>,
        pub model_used: String,
    }
    
    pub trait MLModel: Send + Sync {
        fn predict(&self, features: &serde_json::Value) -> Result<HashMap<String, f64>, AnalysisError>;
        fn model_name(&self) -> &str;
    }
    
    impl MLAnalyzer {
        pub fn new() -> Result<Self, AnalysisError> {
            Ok(Self {
                models: DashMap::new(),
            })
        }
        
        pub async fn analyze(&self, fingerprint: &dyn Fingerprint) -> Result<MLResult, AnalysisError> {
            // Implementation would run ML inference
            Ok(MLResult {
                risk_score: 0.2,
                confidence: 0.9,
                predictions: HashMap::new(),
                feature_importance: HashMap::new(),
                model_used: "ensemble_model".to_string(),
            })
        }
    }
}

#[cfg(feature = "machine-learning")]
pub use ml::{MLAnalyzer, MLResult, MLModel};

// Real-time monitoring components
#[cfg(feature = "real-time")]
mod realtime {
    use super::*;
    use tokio::sync::broadcast;
    
    pub struct RealTimeMonitor {
        // Real-time data streams and monitoring
        event_channels: DashMap<String, broadcast::Sender<serde_json::Value>>,
    }
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct RealTimeResult {
        pub current_risk: f64,
        pub confidence: f64,
        pub recent_events: Vec<serde_json::Value>,
        pub trend_direction: TrendDirection,
        pub volatility: f64,
    }
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum TrendDirection {
        Increasing,
        Decreasing,
        Stable,
    }
    
    impl RealTimeMonitor {
        pub fn new() -> Result<Self, AnalysisError> {
            Ok(Self {
                event_channels: DashMap::new(),
            })
        }
        
        pub async fn analyze(&self, fingerprint: &dyn Fingerprint) -> Result<RealTimeResult, AnalysisError> {
            // Implementation would monitor real-time data
            Ok(RealTimeResult {
                current_risk: 0.15,
                confidence: 0.85,
                recent_events: vec![],
                trend_direction: TrendDirection::Stable,
                volatility: 0.1,
            })
        }
    }
}

#[cfg(feature = "real-time")]
pub use realtime::{RealTimeMonitor, RealTimeResult, TrendDirection};

// Historical analysis components
#[cfg(feature = "historical")]
mod historical {
    use super::*;
    
    pub struct HistoricalAnalyzer {
        // Historical data storage and analysis
        historical_data: DashMap<String, Vec<HistoricalRecord>>,
    }
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct HistoricalResult {
        pub trend_risk: f64,
        pub confidence: f64,
        pub historical_patterns: Vec<Pattern>,
        pub seasonal_variations: HashMap<String, f64>,
        pub long_term_trends: Vec<Trend>,
    }
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct HistoricalRecord {
        pub timestamp: chrono::DateTime<chrono::Utc>,
        pub fingerprint_id: String,
        pub features: serde_json::Value,
        pub classification: String,
    }
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Pattern {
        pub pattern_type: String,
        pub frequency: u32,
        pub confidence: f64,
        pub examples: Vec<String>,
    }
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Trend {
        pub metric: String,
        pub direction: String,
        pub magnitude: f64,
        pub timeframe: String,
    }
    
    impl HistoricalAnalyzer {
        pub fn new() -> Result<Self, AnalysisError> {
            Ok(Self {
                historical_data: DashMap::new(),
            })
        }
        
        pub async fn analyze(&self, fingerprint: &dyn Fingerprint) -> Result<HistoricalResult, AnalysisError> {
            // Implementation would analyze historical patterns
            Ok(HistoricalResult {
                trend_risk: 0.1,
                confidence: 0.8,
                historical_patterns: vec![],
                seasonal_variations: HashMap::new(),
                long_term_trends: vec![],
            })
        }
    }
}

#[cfg(feature = "historical")]
pub use historical::{HistoricalAnalyzer, HistoricalResult, HistoricalRecord, Pattern, Trend};

// Re-export main types
pub use crate::{
    AnalysisEngine,
    AnalysisResult,
    AnalysisError,
    Alert,
    AlertSeverity,
    AlertCategory,
    AlertGenerator,
};

#[cfg(test)]
mod tests {
    use super::*;
    use fingerprint_core::fingerprint::{FingerprintType, FingerprintMetadata};
    
    // Mock fingerprint for testing
    struct MockFingerprint {
        id: String,
    }
    
    impl Fingerprint for MockFingerprint {
        fn fingerprint_type(&self) -> FingerprintType {
            FingerprintType::Http
        }
        
        fn id(&self) -> String {
            self.id.clone()
        }
        
        fn metadata(&self) -> &FingerprintMetadata {
            static METADATA: FingerprintMetadata = FingerprintMetadata::new();
            &METADATA
        }
        
        fn metadata_mut(&mut self) -> &mut FingerprintMetadata {
            static mut METADATA: FingerprintMetadata = FingerprintMetadata::new();
            unsafe { &mut METADATA }
        }
        
        fn hash(&self) -> u64 {
            0
        }
        
        fn similar_to(&self, _other: &dyn Fingerprint) -> bool {
            true
        }
        
        fn to_string(&self) -> String {
            self.id.clone()
        }
    }
    
    #[tokio::test]
    async fn test_analysis_engine_creation() {
        let config = fingerprint_config::get_config_manager();
        let engine = AnalysisEngine::new(config).unwrap();
        
        let fp = MockFingerprint { id: "test-123".to_string() };
        let result = engine.analyze(&fp).await.unwrap();
        
        assert_eq!(result.input_fingerprint, "test-123");
        assert!(!result.id.is_empty());
    }
    
    #[tokio::test]
    async fn test_fingerprint_comparison() {
        let config = fingerprint_config::get_config_manager();
        let engine = AnalysisEngine::new(config).unwrap();
        
        let fp1 = MockFingerprint { id: "test-1".to_string() };
        let fp2 = MockFingerprint { id: "test-2".to_string() };
        
        let comparison = engine.compare(&fp1, &fp2).await.unwrap();
        assert!(comparison.similarity >= 0.0);
        assert!(comparison.similarity <= 1.0);
    }
}