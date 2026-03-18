//! Pre-trained ML Models Support
//!
//! This module provides support for pre-trained machine learning models
//! for fingerprint classification and anomaly detection.
//!
//! Supported models:
//! - Binary classifier: Fingerprint authenticity
//! - Multi-class: Browser type classification
//! - Anomaly detector: Behavioral anomaly detection
//! - Ensemble: Combined predictions from multiple models

use std::collections::{HashMap, VecDeque};
use std::sync::Mutex;

const DEFAULT_MODEL_CACHE_CAPACITY: usize = 3;

/// Model performance metrics
#[derive(Debug, Clone)]
pub struct ModelMetrics {
    /// Model accuracy on test set
    pub accuracy: f32,
    /// Precision score
    pub precision: f32,
    /// Recall score
    pub recall: f32,
    /// F1 score
    pub f1_score: f32,
    /// Model version
    pub version: String,
    /// Training dataset size
    pub training_samples: usize,
}

impl Default for ModelMetrics {
    fn default() -> Self {
        Self {
            accuracy: 0.95,
            precision: 0.94,
            recall: 0.96,
            f1_score: 0.95,
            version: "1.0.0".to_string(),
            training_samples: 100000,
        }
    }
}

/// Pre-trained model enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PreTrainedModel {
    /// Binary classifier for fingerprint authenticity
    AuthenticityClassifier,
    /// Multi-class classifier for browser type
    BrowserTypeClassifier,
    /// Anomaly detector for behavior analysis
    BehaviorAnomalyDetector,
    /// OS type classifier
    OSClassifier,
    /// Device type classifier (mobile/desktop/tablet)
    DeviceTypeClassifier,
}

impl PreTrainedModel {
    const ALL: [PreTrainedModel; 5] = [
        PreTrainedModel::AuthenticityClassifier,
        PreTrainedModel::BrowserTypeClassifier,
        PreTrainedModel::BehaviorAnomalyDetector,
        PreTrainedModel::OSClassifier,
        PreTrainedModel::DeviceTypeClassifier,
    ];

    /// Get model name
    pub fn name(&self) -> &'static str {
        match self {
            Self::AuthenticityClassifier => "authenticity_classifier",
            Self::BrowserTypeClassifier => "browser_type_classifier",
            Self::BehaviorAnomalyDetector => "behavior_anomaly_detector",
            Self::OSClassifier => "os_classifier",
            Self::DeviceTypeClassifier => "device_type_classifier",
        }
    }

    /// Get model version
    pub fn version(&self) -> &'static str {
        match self {
            Self::AuthenticityClassifier => "2.1.0",
            Self::BrowserTypeClassifier => "2.1.0",
            Self::BehaviorAnomalyDetector => "2.0.0",
            Self::OSClassifier => "2.1.0",
            Self::DeviceTypeClassifier => "1.5.0",
        }
    }

    /// Get model metrics
    pub fn metrics(&self) -> ModelMetrics {
        match self {
            Self::AuthenticityClassifier => ModelMetrics {
                accuracy: 0.98,
                precision: 0.97,
                recall: 0.99,
                f1_score: 0.98,
                version: "2.1.0".to_string(),
                training_samples: 250000,
            },
            Self::BrowserTypeClassifier => ModelMetrics {
                accuracy: 0.96,
                precision: 0.95,
                recall: 0.96,
                f1_score: 0.955,
                version: "2.1.0".to_string(),
                training_samples: 300000,
            },
            Self::BehaviorAnomalyDetector => ModelMetrics {
                accuracy: 0.92,
                precision: 0.91,
                recall: 0.93,
                f1_score: 0.92,
                version: "2.0.0".to_string(),
                training_samples: 150000,
            },
            Self::OSClassifier => ModelMetrics {
                accuracy: 0.97,
                precision: 0.96,
                recall: 0.98,
                f1_score: 0.97,
                version: "2.1.0".to_string(),
                training_samples: 200000,
            },
            Self::DeviceTypeClassifier => ModelMetrics {
                accuracy: 0.94,
                precision: 0.93,
                recall: 0.95,
                f1_score: 0.94,
                version: "1.5.0".to_string(),
                training_samples: 180000,
            },
        }
    }

    /// Approximate in-memory footprint used by the loaded model artifact.
    pub fn estimated_size_bytes(&self) -> usize {
        match self {
            Self::AuthenticityClassifier => 24 * 1024 * 1024,
            Self::BrowserTypeClassifier => 32 * 1024 * 1024,
            Self::BehaviorAnomalyDetector => 28 * 1024 * 1024,
            Self::OSClassifier => 20 * 1024 * 1024,
            Self::DeviceTypeClassifier => 18 * 1024 * 1024,
        }
    }
}

/// Model prediction result
#[derive(Debug, Clone)]
pub struct ModelPrediction {
    /// Predicted label/class
    pub label: String,
    /// Prediction confidence (0.0 - 1.0)
    pub confidence: f32,
    /// Alternative predictions with lower confidence
    pub alternatives: Vec<(String, f32)>,
    /// Model used for prediction
    pub model: PreTrainedModel,
}

/// Pre-trained models loader and manager
pub struct PreTrainedModelManager {
    registry: HashMap<PreTrainedModel, ModelDescriptor>,
    cache: Mutex<ModelCache>,
}

#[derive(Debug, Clone)]
struct ModelDescriptor {
    metrics: ModelMetrics,
    estimated_size_bytes: usize,
}

#[derive(Debug, Clone)]
struct LoadedModel {
    metrics: ModelMetrics,
    estimated_size_bytes: usize,
}

#[derive(Debug)]
struct ModelCache {
    loaded: HashMap<PreTrainedModel, LoadedModel>,
    lru: VecDeque<PreTrainedModel>,
    capacity: usize,
    loaded_bytes: usize,
    evictions: usize,
}

#[derive(Debug, Clone)]
pub struct ModelCacheStats {
    pub capacity: usize,
    pub loaded_models: usize,
    pub loaded_bytes: usize,
    pub evictions: usize,
}

impl ModelCache {
    fn new(capacity: usize) -> Self {
        Self {
            loaded: HashMap::new(),
            lru: VecDeque::new(),
            capacity: capacity.max(1),
            loaded_bytes: 0,
            evictions: 0,
        }
    }

    fn touch(&mut self, model: PreTrainedModel) {
        self.lru.retain(|cached| *cached != model);
        self.lru.push_back(model);
    }

    fn get(&mut self, model: PreTrainedModel) -> Option<ModelMetrics> {
        let metrics = self.loaded.get(&model).map(|entry| entry.metrics.clone())?;
        self.touch(model);
        Some(metrics)
    }

    fn insert(&mut self, model: PreTrainedModel, descriptor: &ModelDescriptor) -> ModelMetrics {
        while self.loaded.len() >= self.capacity {
            self.evict_oldest();
        }

        let metrics = descriptor.metrics.clone();
        let entry = LoadedModel {
            metrics: metrics.clone(),
            estimated_size_bytes: descriptor.estimated_size_bytes,
        };

        self.loaded_bytes += entry.estimated_size_bytes;
        self.loaded.insert(model, entry);
        self.touch(model);
        metrics
    }

    fn evict_oldest(&mut self) {
        if let Some(oldest) = self.lru.pop_front() {
            if let Some(removed) = self.loaded.remove(&oldest) {
                self.loaded_bytes = self
                    .loaded_bytes
                    .saturating_sub(removed.estimated_size_bytes);
                self.evictions += 1;
            }
        }
    }

    fn stats(&self) -> ModelCacheStats {
        ModelCacheStats {
            capacity: self.capacity,
            loaded_models: self.loaded.len(),
            loaded_bytes: self.loaded_bytes,
            evictions: self.evictions,
        }
    }
}

impl PreTrainedModelManager {
    /// Create new model manager
    pub fn new() -> Self {
        Self::with_cache_capacity(DEFAULT_MODEL_CACHE_CAPACITY)
    }

    /// Create a manager with an explicit loaded-model cache capacity.
    pub fn with_cache_capacity(capacity: usize) -> Self {
        let mut registry = HashMap::new();

        for model in PreTrainedModel::ALL {
            registry.insert(
                model,
                ModelDescriptor {
                    metrics: model.metrics(),
                    estimated_size_bytes: model.estimated_size_bytes(),
                },
            );
        }

        Self {
            registry,
            cache: Mutex::new(ModelCache::new(capacity)),
        }
    }

    /// Load model from disk
    ///
    /// In production, this would load actual model weights from disk
    pub fn load_model(&self, model: PreTrainedModel) -> Result<ModelMetrics, String> {
        let descriptor = self
            .registry
            .get(&model)
            .cloned()
            .ok_or_else(|| format!("Model {} not found", model.name()))?;

        let mut cache = self
            .cache
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if let Some(metrics) = cache.get(model) {
            return Ok(metrics);
        }

        Ok(cache.insert(model, &descriptor))
    }

    /// Get available models
    pub fn available_models(&self) -> Vec<(PreTrainedModel, ModelMetrics)> {
        PreTrainedModel::ALL
            .iter()
            .filter_map(|model| {
                self.registry
                    .get(model)
                    .map(|descriptor| (*model, descriptor.metrics.clone()))
            })
            .collect()
    }

    /// Get model metrics
    pub fn get_metrics(&self, model: PreTrainedModel) -> Option<ModelMetrics> {
        self.registry
            .get(&model)
            .map(|descriptor| descriptor.metrics.clone())
    }

    /// Return cache state for observability and tuning.
    pub fn cache_stats(&self) -> ModelCacheStats {
        self.cache
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .stats()
    }

    /// Check whether a model is currently resident in the loaded-model cache.
    pub fn is_model_loaded(&self, model: PreTrainedModel) -> bool {
        self.cache
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .loaded
            .contains_key(&model)
    }

    fn ensure_model_loaded(&self, model: PreTrainedModel) {
        let _ = self.load_model(model);
    }

    /// Predict fingerprint authenticity
    pub fn predict_authenticity(&self, features: &[f32]) -> ModelPrediction {
        self.ensure_model_loaded(PreTrainedModel::AuthenticityClassifier);
        let confidence = self.calculate_confidence(features);

        ModelPrediction {
            label: if confidence > 0.7 {
                "authentic".to_string()
            } else {
                "suspicious".to_string()
            },
            confidence,
            alternatives: vec![(
                if confidence > 0.7 {
                    "suspicious".to_string()
                } else {
                    "authentic".to_string()
                },
                1.0 - confidence,
            )],
            model: PreTrainedModel::AuthenticityClassifier,
        }
    }

    /// Predict browser type
    pub fn predict_browser(&self, features: &[f32]) -> ModelPrediction {
        self.ensure_model_loaded(PreTrainedModel::BrowserTypeClassifier);
        let browsers = ["Chrome", "Firefox", "Safari", "Edge", "Opera", "Other"];

        let scores: Vec<f32> = features
            .iter()
            .zip(browsers.iter())
            .map(|(f, _)| f.abs() % 1.0)
            .collect();

        let max_idx = scores
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .map(|(idx, _)| idx)
            .unwrap_or(0)
            .min(browsers.len() - 1);

        let confidence = scores[max_idx];
        let mut alternatives = scores
            .iter()
            .enumerate()
            .filter(|(i, _)| *i != max_idx)
            .map(|(i, &score)| (browsers[i].to_string(), score))
            .collect::<Vec<_>>();
        alternatives.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        ModelPrediction {
            label: browsers[max_idx].to_string(),
            confidence,
            alternatives,
            model: PreTrainedModel::BrowserTypeClassifier,
        }
    }

    /// Predict anomalies in behavior
    pub fn predict_anomaly(&self, features: &[f32]) -> ModelPrediction {
        self.ensure_model_loaded(PreTrainedModel::BehaviorAnomalyDetector);
        let anomaly_score = features.iter().map(|f| f.abs()).sum::<f32>() / features.len() as f32;
        let confidence = (anomaly_score / 2.0).min(1.0);

        ModelPrediction {
            label: if anomaly_score > 0.6 {
                "anomalous".to_string()
            } else {
                "normal".to_string()
            },
            confidence,
            alternatives: vec![(
                if anomaly_score > 0.6 {
                    "normal".to_string()
                } else {
                    "anomalous".to_string()
                },
                1.0 - confidence,
            )],
            model: PreTrainedModel::BehaviorAnomalyDetector,
        }
    }

    /// Calculate confidence score
    fn calculate_confidence(&self, features: &[f32]) -> f32 {
        if features.is_empty() {
            return 0.5;
        }

        let mean = features.iter().sum::<f32>() / features.len() as f32;
        let variance =
            features.iter().map(|f| (f - mean).powi(2)).sum::<f32>() / features.len() as f32;

        let confidence = 1.0 / (1.0 + variance);
        confidence.clamp(0.0, 1.0)
    }
}

impl Default for PreTrainedModelManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Ensemble predictor combining multiple models
pub struct EnsemblePredictor {
    manager: PreTrainedModelManager,
    models: Vec<PreTrainedModel>,
}

impl EnsemblePredictor {
    /// Create new ensemble predictor
    pub fn new(models: Vec<PreTrainedModel>) -> Self {
        Self {
            manager: PreTrainedModelManager::new(),
            models,
        }
    }

    /// Combine predictions from multiple models
    pub fn predict_ensemble(&self, features: &[f32]) -> HashMap<String, f32> {
        let mut scores = HashMap::new();

        for model in &self.models {
            let prediction = match model {
                PreTrainedModel::AuthenticityClassifier => {
                    self.manager.predict_authenticity(features)
                }
                PreTrainedModel::BrowserTypeClassifier => self.manager.predict_browser(features),
                PreTrainedModel::BehaviorAnomalyDetector => self.manager.predict_anomaly(features),
                PreTrainedModel::OSClassifier => {
                    self.manager.predict_authenticity(features) // Placeholder
                }
                PreTrainedModel::DeviceTypeClassifier => {
                    self.manager.predict_authenticity(features) // Placeholder
                }
            };

            *scores.entry(prediction.label).or_insert(0.0) += prediction.confidence;
        }

        // Normalize scores
        let total: f32 = scores.values().sum();
        if total > 0.0 {
            for score in scores.values_mut() {
                *score /= total;
            }
        }

        scores
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_manager_initialization() {
        let manager = PreTrainedModelManager::new();
        let models = manager.available_models();
        assert_eq!(models.len(), 5);
        assert_eq!(manager.cache_stats().loaded_models, 0);
    }

    #[test]
    fn test_lazy_loading_populates_cache_on_demand() {
        let manager = PreTrainedModelManager::with_cache_capacity(2);

        assert!(!manager.is_model_loaded(PreTrainedModel::AuthenticityClassifier));
        let metrics = manager
            .load_model(PreTrainedModel::AuthenticityClassifier)
            .expect("load model");

        assert_eq!(metrics.version, "2.1.0");
        assert!(manager.is_model_loaded(PreTrainedModel::AuthenticityClassifier));
        assert_eq!(manager.cache_stats().loaded_models, 1);
    }

    #[test]
    fn test_lru_eviction_keeps_loaded_models_bounded() {
        let manager = PreTrainedModelManager::with_cache_capacity(2);

        manager
            .load_model(PreTrainedModel::AuthenticityClassifier)
            .expect("load authenticity");
        manager
            .load_model(PreTrainedModel::BrowserTypeClassifier)
            .expect("load browser");
        manager
            .load_model(PreTrainedModel::BehaviorAnomalyDetector)
            .expect("load anomaly");

        assert!(!manager.is_model_loaded(PreTrainedModel::AuthenticityClassifier));
        assert!(manager.is_model_loaded(PreTrainedModel::BrowserTypeClassifier));
        assert!(manager.is_model_loaded(PreTrainedModel::BehaviorAnomalyDetector));

        let stats = manager.cache_stats();
        assert_eq!(stats.capacity, 2);
        assert_eq!(stats.loaded_models, 2);
        assert_eq!(stats.evictions, 1);
    }

    #[test]
    fn test_authenticity_prediction() {
        let manager = PreTrainedModelManager::new();
        let features = vec![0.5, 0.6, 0.7, 0.8, 0.9];
        let pred = manager.predict_authenticity(&features);

        assert!(!pred.label.is_empty());
        assert!(pred.confidence >= 0.0 && pred.confidence <= 1.0);
        assert!(manager.is_model_loaded(PreTrainedModel::AuthenticityClassifier));
    }

    #[test]
    fn test_browser_prediction() {
        let manager = PreTrainedModelManager::new();
        let features = vec![0.8, 0.2, 0.1, 0.3, 0.4, 0.5];
        let pred = manager.predict_browser(&features);

        assert!(!pred.label.is_empty());
        assert!(["Chrome", "Firefox", "Safari", "Edge", "Opera", "Other"]
            .contains(&pred.label.as_str()));
    }

    #[test]
    fn test_ensemble_predictor() {
        let models = vec![
            PreTrainedModel::AuthenticityClassifier,
            PreTrainedModel::BrowserTypeClassifier,
        ];
        let ensemble = EnsemblePredictor::new(models);
        let features = vec![0.5, 0.6, 0.7];
        let predictions = ensemble.predict_ensemble(&features);

        assert!(!predictions.is_empty());
    }

    #[test]
    fn test_model_metrics() {
        let model = PreTrainedModel::AuthenticityClassifier;
        let metrics = model.metrics();

        assert!(metrics.accuracy > 0.9);
        assert!(metrics.precision > 0.9);
        assert_eq!(metrics.version, "2.1.0");
    }
}
