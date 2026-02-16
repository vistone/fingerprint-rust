#![allow(clippy::all, dead_code, unused_variables, unused_parens)]

//! # fingerprint-ml
//!
//! Machine learning fingerprint matching module
//!
//! Provides advanced fingerprint similarity calculation and classification capabilities

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
