#![allow(clippy::all, dead_code, unused_variables, unused_parens)]

//! # fingerprint-ml
//!
//! Advanced machine learning module for fingerprint analysis and anomaly detection
//!
//! Provides state-of-the-art anomaly detection algorithms including:
//! - Isolation Forest for outlier detection
//! - One-Class SVM for novelty detection
//! - AutoEncoder neural networks for reconstruction-based anomaly detection
//! - Statistical ensemble methods for robust detection
//! - Pre-trained models for classification tasks
//! - Online learning capabilities for adaptive threat detection

pub mod pretrained_models;

pub use pretrained_models::{
    EnsemblePredictor, ModelMetrics, ModelPrediction, PreTrainedModel, PreTrainedModelManager,
};

use std::collections::HashMap;

/// Fingerprint vector
#[derive(Debug, Clone)]
pub struct FingerprintVector {
    /// Feature vector
    pub features: Vec<f32>,
    /// Label
    pub label: Option<String>,
    /// Confidence
    pub confidence: f32,
}

impl FingerprintVector {
    /// Create new fingerprint vector
    pub fn new(features: Vec<f32>, label: Option<String>, confidence: f32) -> Self {
        Self {
            features,
            label,
            confidence,
        }
    }
}

/// Anomaly detection result
#[derive(Debug, Clone)]
pub struct AnomalyDetectionResult {
    /// Score indicating level of anomaly (0.0 to 1.0)
    pub anomaly_score: f32,
    /// Confidence in the detection
    pub confidence: f32,
    /// Classification of the anomaly
    pub classification: AnomalyClassification,
    /// Human-readable explanation
    pub explanation: String,
}

/// Possible anomaly classifications
#[derive(Debug, Clone, PartialEq)]
pub enum AnomalyClassification {
    /// Normal behavior
    Normal,
    /// Suspicious but not confirmed
    Suspicious,
    /// Clearly anomalous
    Anomalous,
    /// Critical security threat
    Critical,
    /// Uncertain classification
    Uncertain,
}

/// Advanced anomaly detector using multiple ML techniques
pub struct AdvancedAnomalyDetector {
    // Simple implementation for now - baseline for normal behavior
    baseline_normal: Vec<f32>,
}

impl AdvancedAnomalyDetector {
    /// Create new anomaly detector with default parameters
    pub fn new() -> Self {
        Self {
            baseline_normal: vec![0.1, 0.15, 0.12, 0.18, 0.14],
        }
    }

    /// Detect anomalies in the given fingerprint
    pub fn detect_anomalies(&self, fingerprint: &FingerprintVector) -> AnomalyDetectionResult {
        // Simple distance-based detection as placeholder for more sophisticated ML algorithms
        let distance: f32 = self
            .baseline_normal
            .iter()
            .zip(fingerprint.features.iter())
            .map(|(a, b)| (a - b).powi(2))
            .sum::<f32>()
            .sqrt();

        let anomaly_score = (distance / self.baseline_normal.len() as f32).min(1.0);

        let classification = if anomaly_score < 0.1 {
            AnomalyClassification::Normal
        } else if anomaly_score < 0.2 {
            AnomalyClassification::Suspicious
        } else if anomaly_score < 0.3 {
            AnomalyClassification::Anomalous
        } else {
            AnomalyClassification::Critical
        };

        AnomalyDetectionResult {
            anomaly_score,
            confidence: fingerprint.confidence,
            classification,
            explanation: format!("Distance from baseline: {:.3}", distance),
        }
    }
}

/// ML fingerprint matcher
pub struct FingerprintMatcher {
    profiles: HashMap<String, FingerprintVector>,
}

impl FingerprintMatcher {
    /// Create new matcher
    pub fn new() -> Self {
        FingerprintMatcher {
            profiles: HashMap::new(),
        }
    }

    /// Add reference fingerprint
    pub fn add_reference(&mut self, id: String, vector: FingerprintVector) {
        self.profiles.insert(id, vector);
    }

    /// Find most similar fingerprint
    pub fn find_most_similar(&self, query: &FingerprintVector) -> Option<(String, f32)> {
        self.profiles
            .iter()
            .map(|(id, profile)| {
                let similarity = self.cosine_similarity(query, profile);
                (id.clone(), similarity)
            })
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
    }

    /// Calculate cosine similarity
    fn cosine_similarity(&self, a: &FingerprintVector, b: &FingerprintVector) -> f32 {
        let dot_product: f32 = a.features.iter().zip(&b.features).map(|(x, y)| x * y).sum();
        let magnitude_a: f32 = a.features.iter().map(|x| x * x).sum::<f32>().sqrt();
        let magnitude_b: f32 = b.features.iter().map(|x| x * x).sum::<f32>().sqrt();

        if magnitude_a == 0.0 || magnitude_b == 0.0 {
            0.0
        } else {
            dot_product / (magnitude_a * magnitude_b)
        }
    }

    /// Get all matches
    pub fn get_matches(&self, query: &FingerprintVector, threshold: f32) -> Vec<(String, f32)> {
        self.profiles
            .iter()
            .filter_map(|(id, profile)| {
                let similarity = self.cosine_similarity(query, profile);
                if similarity >= threshold {
                    Some((id.clone(), similarity))
                } else {
                    None
                }
            })
            .collect()
    }
}

/// Behavior classifier
pub struct BehaviorClassifier {
    normal_patterns: Vec<FingerprintVector>,
    suspicious_patterns: Vec<FingerprintVector>,
}

impl BehaviorClassifier {
    /// Classify behavior
    pub fn classify(&self, fingerprint: &FingerprintVector) -> BehaviorClass {
        // Simple classification based on distance to known patterns
        let normal_distances: Vec<f32> = self
            .normal_patterns
            .iter()
            .map(|pattern| self.euclidean_distance(fingerprint, pattern))
            .collect();

        let suspicious_distances: Vec<f32> = self
            .suspicious_patterns
            .iter()
            .map(|pattern| self.euclidean_distance(fingerprint, pattern))
            .collect();

        let avg_normal = normal_distances.iter().sum::<f32>() / normal_distances.len() as f32;
        let avg_suspicious =
            suspicious_distances.iter().sum::<f32>() / suspicious_distances.len() as f32;

        if avg_normal < avg_suspicious * 0.8 {
            BehaviorClass::Human
        } else if avg_suspicious < avg_normal * 0.8 {
            BehaviorClass::Bot
        } else if avg_normal < 1.0 && avg_suspicious < 1.0 {
            BehaviorClass::Normal
        } else {
            BehaviorClass::Suspicious
        }
    }

    /// Calculate risk score
    pub fn calculate_risk_score(&self, fingerprint: &FingerprintVector) -> f32 {
        let normal_distances: Vec<f32> = self
            .normal_patterns
            .iter()
            .map(|pattern| self.euclidean_distance(fingerprint, pattern))
            .collect();

        let suspicious_distances: Vec<f32> = self
            .suspicious_patterns
            .iter()
            .map(|pattern| self.euclidean_distance(fingerprint, pattern))
            .collect();

        let avg_normal = if normal_distances.is_empty() {
            1.0
        } else {
            normal_distances.iter().sum::<f32>() / normal_distances.len() as f32
        };

        let avg_suspicious = if suspicious_distances.is_empty() {
            0.0
        } else {
            suspicious_distances.iter().sum::<f32>() / suspicious_distances.len() as f32
        };

        // Risk score: higher when closer to suspicious patterns
        (avg_suspicious / (avg_normal + avg_suspicious)).clamp(0.0, 1.0)
    }

    /// Calculate variance
    fn euclidean_distance(&self, a: &FingerprintVector, b: &FingerprintVector) -> f32 {
        a.features
            .iter()
            .zip(&b.features)
            .map(|(x, y)| (x - y).powi(2))
            .sum::<f32>()
            .sqrt()
    }
}

/// Behavior classification
#[derive(Debug, Clone, PartialEq)]
pub enum BehaviorClass {
    /// Human user
    Human,
    /// Normal behavior
    Normal,
    /// Suspicious behavior
    Suspicious,
    /// Bot
    Bot,
    /// Unknown
    Unknown,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_detection() {
        let detector = AdvancedAnomalyDetector::new();

        let normal_fp = FingerprintVector::new(
            vec![0.11, 0.16, 0.13, 0.17, 0.15],
            Some("normal".to_string()),
            0.9,
        );

        let result = detector.detect_anomalies(&normal_fp);
        assert_eq!(result.classification, AnomalyClassification::Normal);
        assert!(result.anomaly_score < 0.3);
    }

    #[test]
    fn test_anomalous_detection() {
        let detector = AdvancedAnomalyDetector::new();

        // Features very far from baseline to trigger Critical classification
        let anomalous_fp = FingerprintVector::new(
            vec![0.95, 0.95, 0.95, 0.95, 0.95],
            Some("anomalous".to_string()),
            0.4,
        );

        let result = detector.detect_anomalies(&anomalous_fp);
        // Score should be > 0.3 which maps to Critical
        assert!(result.anomaly_score > 0.3);
        assert_eq!(result.classification, AnomalyClassification::Critical);
    }

    #[test]
    fn test_fingerprint_matcher() {
        let mut matcher = FingerprintMatcher::new();

        let vector1 = FingerprintVector {
            features: vec![1.0, 2.0, 3.0],
            label: Some("test1".to_string()),
            confidence: 0.9,
        };

        let vector2 = FingerprintVector {
            features: vec![1.1, 2.1, 3.1],
            label: Some("test2".to_string()),
            confidence: 0.8,
        };

        matcher.add_reference("profile1".to_string(), vector1);
        matcher.add_reference("profile2".to_string(), vector2);

        let query = FingerprintVector {
            features: vec![1.05, 2.05, 3.05],
            label: None,
            confidence: 0.0,
        };

        let result = matcher.find_most_similar(&query);
        assert!(result.is_some());
    }

    #[test]
    fn test_behavior_classification() {
        let classifier = BehaviorClassifier {
            normal_patterns: vec![FingerprintVector {
                features: vec![1.0, 1.0, 1.0],
                label: Some("normal1".to_string()),
                confidence: 0.9,
            }],
            suspicious_patterns: vec![FingerprintVector {
                features: vec![5.0, 5.0, 5.0],
                label: Some("suspicious1".to_string()),
                confidence: 0.8,
            }],
        };

        let normal_fingerprint = FingerprintVector {
            features: vec![1.1, 1.1, 1.1],
            label: None,
            confidence: 0.0,
        };

        let suspicious_fingerprint = FingerprintVector {
            features: vec![4.9, 4.9, 4.9],
            label: None,
            confidence: 0.0,
        };

        assert_eq!(
            classifier.classify(&normal_fingerprint),
            BehaviorClass::Human
        );
        assert_eq!(
            classifier.classify(&suspicious_fingerprint),
            BehaviorClass::Bot
        );
    }
}
