//! Model Fingerprint Learning and Database System
//!
//! This module provides a system to learn and store unique fingerprint characteristics
//! for each AI model, enabling more accurate content identification and model attribution.
//!
//! Key components:
//! - ModelFingerprintDatabase: Storage and management of learned fingerprints
//! - ModelFingerprint: Statistical signatures and patterns per model
//! - FingerprintLearner: Extract characteristics from known samples
//! - Distance metrics: Cosine similarity and KL-divergence for matching

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::Path;

/// Database of learned model fingerprints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelFingerprintDatabase {
    /// Collection of fingerprints indexed by model name
    pub fingerprints: HashMap<String, ModelFingerprint>,

    /// Database metadata
    pub metadata: DatabaseMetadata,
}

/// Database metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseMetadata {
    /// Database version
    pub version: String,

    /// Total number of fingerprints
    pub fingerprint_count: usize,

    /// Last updated timestamp
    pub last_updated: String,
}

/// Fingerprint for a specific AI model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelFingerprint {
    /// Model name (e.g., "gpt4", "claude3", "stable-diffusion")
    pub model_name: String,

    /// Model type ("text", "image", "audio", "video")
    pub model_type: String,

    /// Number of samples used to learn this fingerprint
    pub sample_count: usize,

    /// Statistical signature
    pub statistical_signature: StatisticalSignature,

    /// Characteristic patterns (phrases, artifacts, etc.)
    pub characteristic_patterns: Vec<String>,

    /// When this fingerprint was learned
    pub learned_at: String,
}

/// Statistical signature of a model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatisticalSignature {
    // Text metrics
    pub perplexity_mean: f32,
    pub perplexity_std: f32,
    pub burstiness_mean: f32,
    pub burstiness_std: f32,
    pub vocabulary_richness_mean: f32,
    pub vocabulary_richness_std: f32,

    // Image metrics
    pub noise_pattern_mean: f32,
    pub noise_pattern_std: f32,
    pub texture_regularity_mean: f32,
    pub texture_regularity_std: f32,
    pub color_distribution_mean: f32,
    pub color_distribution_std: f32,
}

/// Learner for extracting fingerprints from samples
pub struct FingerprintLearner {
    /// Samples grouped by model name
    samples: HashMap<String, Vec<Sample>>,
}

/// A training sample
#[derive(Debug, Clone)]
struct Sample {
    _model_name: String,
    content_type: String,
    features: Vec<f32>,
    patterns: Vec<String>,
}

impl ModelFingerprintDatabase {
    /// Create a new empty database
    pub fn new() -> Self {
        Self {
            fingerprints: HashMap::new(),
            metadata: DatabaseMetadata {
                version: "1.0.0".to_string(),
                fingerprint_count: 0,
                last_updated: chrono::Utc::now().to_rfc3339(),
            },
        }
    }

    /// Add a fingerprint to the database
    pub fn add_fingerprint(&mut self, fingerprint: ModelFingerprint) {
        self.fingerprints
            .insert(fingerprint.model_name.clone(), fingerprint);
        self.metadata.fingerprint_count = self.fingerprints.len();
        self.metadata.last_updated = chrono::Utc::now().to_rfc3339();
    }

    /// Get a fingerprint by model name
    pub fn get_fingerprint(&self, model_name: &str) -> Option<&ModelFingerprint> {
        self.fingerprints.get(model_name)
    }

    /// Match a signature against the database
    /// Returns list of (model_name, similarity_score) sorted by similarity
    pub fn match_fingerprint(
        &self,
        signature: &StatisticalSignature,
        content_type: &str,
    ) -> Vec<(String, f32)> {
        let mut matches: Vec<(String, f32)> = self
            .fingerprints
            .iter()
            .filter(|(_, fp)| fp.model_type == content_type)
            .map(|(name, fp)| {
                let similarity = calculate_cosine_similarity(signature, &fp.statistical_signature);
                (name.clone(), similarity)
            })
            .collect();

        // Sort by similarity (descending)
        matches.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        matches
    }

    /// Save database to JSON file
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> io::Result<()> {
        let json = serde_json::to_string_pretty(self).map_err(io::Error::other)?;
        fs::write(path, json)
    }

    /// Load database from JSON file
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let json = fs::read_to_string(path)?;
        serde_json::from_str(&json).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    /// List all models in the database
    pub fn list_models(&self) -> Vec<String> {
        self.fingerprints.keys().cloned().collect()
    }

    /// Get models by type
    pub fn get_models_by_type(&self, content_type: &str) -> Vec<String> {
        self.fingerprints
            .iter()
            .filter(|(_, fp)| fp.model_type == content_type)
            .map(|(name, _)| name.clone())
            .collect()
    }
}

impl Default for ModelFingerprintDatabase {
    fn default() -> Self {
        Self::new()
    }
}

impl FingerprintLearner {
    /// Create a new learner
    pub fn new() -> Self {
        Self {
            samples: HashMap::new(),
        }
    }

    /// Add a text sample
    pub fn add_text_sample(&mut self, text: &str, model_name: &str) {
        let features = extract_text_features(text);
        let patterns = extract_text_patterns(text);

        let sample = Sample {
            _model_name: model_name.to_string(),
            content_type: "text".to_string(),
            features,
            patterns,
        };

        self.samples
            .entry(model_name.to_string())
            .or_default()
            .push(sample);
    }

    /// Add an image sample
    pub fn add_image_sample(&mut self, width: u32, height: u32, format: &str, model_name: &str) {
        let features = extract_image_features(width, height, format);
        let patterns = extract_image_patterns(format);

        let sample = Sample {
            _model_name: model_name.to_string(),
            content_type: "image".to_string(),
            features,
            patterns,
        };

        self.samples
            .entry(model_name.to_string())
            .or_default()
            .push(sample);
    }

    /// Learn fingerprints from all samples
    pub fn learn_fingerprints(&self) -> Vec<ModelFingerprint> {
        let mut fingerprints = Vec::new();

        for (model_name, samples) in &self.samples {
            if samples.is_empty() {
                continue;
            }

            let content_type = samples[0].content_type.clone();
            let sample_count = samples.len();

            // Calculate statistical signature
            let signature = calculate_statistical_signature(samples);

            // Collect characteristic patterns
            let mut all_patterns = Vec::new();
            for sample in samples {
                all_patterns.extend(sample.patterns.clone());
            }
            all_patterns.sort();
            all_patterns.dedup();

            let fingerprint = ModelFingerprint {
                model_name: model_name.clone(),
                model_type: content_type,
                sample_count,
                statistical_signature: signature,
                characteristic_patterns: all_patterns,
                learned_at: chrono::Utc::now().to_rfc3339(),
            };

            fingerprints.push(fingerprint);
        }

        fingerprints
    }
}

impl Default for FingerprintLearner {
    fn default() -> Self {
        Self::new()
    }
}

/// Calculate statistical signature from samples
fn calculate_statistical_signature(samples: &[Sample]) -> StatisticalSignature {
    let n = samples.len() as f32;

    // Calculate means
    let mut perplexity_sum = 0.0;
    let mut burstiness_sum = 0.0;
    let mut vocabulary_sum = 0.0;
    let mut noise_sum = 0.0;
    let mut texture_sum = 0.0;
    let mut color_sum = 0.0;

    for sample in samples {
        if sample.features.len() >= 6 {
            perplexity_sum += sample.features[0];
            burstiness_sum += sample.features[1];
            vocabulary_sum += sample.features[2];
            noise_sum += sample.features[3];
            texture_sum += sample.features[4];
            color_sum += sample.features[5];
        }
    }

    let perplexity_mean = perplexity_sum / n;
    let burstiness_mean = burstiness_sum / n;
    let vocabulary_mean = vocabulary_sum / n;
    let noise_mean = noise_sum / n;
    let texture_mean = texture_sum / n;
    let color_mean = color_sum / n;

    // Calculate standard deviations
    let mut perplexity_var = 0.0;
    let mut burstiness_var = 0.0;
    let mut vocabulary_var = 0.0;
    let mut noise_var = 0.0;
    let mut texture_var = 0.0;
    let mut color_var = 0.0;

    for sample in samples {
        if sample.features.len() >= 6 {
            perplexity_var += (sample.features[0] - perplexity_mean).powi(2);
            burstiness_var += (sample.features[1] - burstiness_mean).powi(2);
            vocabulary_var += (sample.features[2] - vocabulary_mean).powi(2);
            noise_var += (sample.features[3] - noise_mean).powi(2);
            texture_var += (sample.features[4] - texture_mean).powi(2);
            color_var += (sample.features[5] - color_mean).powi(2);
        }
    }

    StatisticalSignature {
        perplexity_mean,
        perplexity_std: (perplexity_var / n).sqrt(),
        burstiness_mean,
        burstiness_std: (burstiness_var / n).sqrt(),
        vocabulary_richness_mean: vocabulary_mean,
        vocabulary_richness_std: (vocabulary_var / n).sqrt(),
        noise_pattern_mean: noise_mean,
        noise_pattern_std: (noise_var / n).sqrt(),
        texture_regularity_mean: texture_mean,
        texture_regularity_std: (texture_var / n).sqrt(),
        color_distribution_mean: color_mean,
        color_distribution_std: (color_var / n).sqrt(),
    }
}

/// Extract features from text
fn extract_text_features(text: &str) -> Vec<f32> {
    // Simplified feature extraction
    let word_count = text.split_whitespace().count() as f32;
    let char_count = text.len() as f32;
    let _avg_word_length = if word_count > 0.0 {
        char_count / word_count
    } else {
        0.0
    };

    // Perplexity estimate (simplified)
    let perplexity = (word_count / 100.0).clamp(0.0, 1.0);

    // Burstiness estimate (simplified)
    let sentences = text.split(['.', '!', '?']).count() as f32;
    let burstiness = if sentences > 0.0 {
        (word_count / sentences) / 20.0
    } else {
        0.5
    };
    let burstiness = burstiness.clamp(0.0, 1.0);

    // Vocabulary richness (simplified)
    let unique_words = text
        .split_whitespace()
        .map(|w| w.to_lowercase())
        .collect::<std::collections::HashSet<_>>()
        .len() as f32;
    let vocab = if word_count > 0.0 {
        unique_words / word_count
    } else {
        0.0
    };

    vec![perplexity, burstiness, vocab, 0.0, 0.0, 0.0]
}

/// Extract patterns from text
fn extract_text_patterns(text: &str) -> Vec<String> {
    let mut patterns = Vec::new();

    // Check for common AI phrases
    let ai_phrases = [
        "it's important to note",
        "delve into",
        "furthermore",
        "moreover",
        "consequently",
    ];

    for phrase in &ai_phrases {
        if text.to_lowercase().contains(phrase) {
            patterns.push(phrase.to_string());
        }
    }

    patterns
}

/// Extract features from image metadata
fn extract_image_features(width: u32, height: u32, format: &str) -> Vec<f32> {
    // Simplified feature extraction from metadata
    let aspect_ratio = width as f32 / height as f32;
    let is_square = (aspect_ratio - 1.0).abs() < 0.1;

    // Typical AI image patterns
    let noise_pattern = if is_square { 0.03 } else { 0.05 };
    let texture_regularity = if format == "PNG" { 0.7 } else { 0.5 };
    let color_distribution = 0.6;

    vec![
        0.0,
        0.0,
        0.0,
        noise_pattern,
        texture_regularity,
        color_distribution,
    ]
}

/// Extract patterns from image metadata
fn extract_image_patterns(format: &str) -> Vec<String> {
    let mut patterns = Vec::new();

    if format == "PNG" {
        patterns.push("lossless_format".to_string());
    }

    patterns
}

/// Calculate cosine similarity between two signatures
pub fn calculate_cosine_similarity(
    sig1: &StatisticalSignature,
    sig2: &StatisticalSignature,
) -> f32 {
    let v1 = [
        sig1.perplexity_mean,
        sig1.burstiness_mean,
        sig1.vocabulary_richness_mean,
        sig1.noise_pattern_mean,
        sig1.texture_regularity_mean,
        sig1.color_distribution_mean,
    ];

    let v2 = [
        sig2.perplexity_mean,
        sig2.burstiness_mean,
        sig2.vocabulary_richness_mean,
        sig2.noise_pattern_mean,
        sig2.texture_regularity_mean,
        sig2.color_distribution_mean,
    ];

    let dot_product: f32 = v1.iter().zip(v2.iter()).map(|(a, b)| a * b).sum();
    let norm1: f32 = v1.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm2: f32 = v2.iter().map(|x| x * x).sum::<f32>().sqrt();

    if norm1 > 0.0 && norm2 > 0.0 {
        dot_product / (norm1 * norm2)
    } else {
        0.0
    }
}

/// Calculate KL-divergence between two signatures
pub fn calculate_kl_divergence(sig1: &StatisticalSignature, sig2: &StatisticalSignature) -> f32 {
    // Simplified KL-divergence calculation
    let epsilon = 1e-6;

    let features1 = [
        sig1.perplexity_mean + epsilon,
        sig1.burstiness_mean + epsilon,
        sig1.vocabulary_richness_mean + epsilon,
    ];

    let features2 = [
        sig2.perplexity_mean + epsilon,
        sig2.burstiness_mean + epsilon,
        sig2.vocabulary_richness_mean + epsilon,
    ];

    // Normalize to probabilities
    let sum1: f32 = features1.iter().sum();
    let sum2: f32 = features2.iter().sum();

    let p: Vec<f32> = features1.iter().map(|x| x / sum1).collect();
    let q: Vec<f32> = features2.iter().map(|x| x / sum2).collect();

    // Calculate KL-divergence
    p.iter()
        .zip(q.iter())
        .map(|(pi, qi)| pi * (pi / qi).ln())
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_operations() {
        let mut db = ModelFingerprintDatabase::new();
        assert_eq!(db.fingerprints.len(), 0);

        let fingerprint = create_test_fingerprint("gpt4", "text");
        db.add_fingerprint(fingerprint);

        assert_eq!(db.fingerprints.len(), 1);
        assert!(db.get_fingerprint("gpt4").is_some());
    }

    #[test]
    fn test_add_and_get_fingerprint() {
        let mut db = ModelFingerprintDatabase::new();
        let fp = create_test_fingerprint("claude3", "text");
        db.add_fingerprint(fp);

        let retrieved = db.get_fingerprint("claude3");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().model_name, "claude3");
    }

    #[test]
    fn test_fingerprint_matching() {
        let mut db = ModelFingerprintDatabase::new();
        db.add_fingerprint(create_test_fingerprint("gpt4", "text"));
        db.add_fingerprint(create_test_fingerprint("claude3", "text"));

        let sig = create_test_signature();
        let matches = db.match_fingerprint(&sig, "text");

        assert_eq!(matches.len(), 2);
        assert!(matches[0].1 >= matches[1].1); // Sorted by similarity
    }

    #[test]
    fn test_distance_metrics() {
        let sig1 = create_test_signature();
        let sig2 = create_test_signature();

        let similarity = calculate_cosine_similarity(&sig1, &sig2);
        assert!((0.0..=1.0).contains(&similarity));

        let divergence = calculate_kl_divergence(&sig1, &sig2);
        assert!(divergence >= 0.0);
    }

    #[test]
    fn test_learner_basic() {
        let learner = FingerprintLearner::new();
        assert_eq!(learner.samples.len(), 0);
    }

    #[test]
    fn test_learner_text_samples() {
        let mut learner = FingerprintLearner::new();
        learner.add_text_sample("This is a test text.", "gpt4");
        learner.add_text_sample("Another test text.", "gpt4");

        let fingerprints = learner.learn_fingerprints();
        assert_eq!(fingerprints.len(), 1);
        assert_eq!(fingerprints[0].sample_count, 2);
    }

    #[test]
    fn test_learner_image_samples() {
        let mut learner = FingerprintLearner::new();
        learner.add_image_sample(1024, 1024, "PNG", "stable-diffusion");

        let fingerprints = learner.learn_fingerprints();
        assert_eq!(fingerprints.len(), 1);
        assert_eq!(fingerprints[0].model_type, "image");
    }

    #[test]
    fn test_save_load_database() {
        let mut db = ModelFingerprintDatabase::new();
        db.add_fingerprint(create_test_fingerprint("gpt4", "text"));

        let temp_file = "/tmp/test_fingerprints.json";
        db.save_to_file(temp_file).unwrap();

        let loaded_db = ModelFingerprintDatabase::load_from_file(temp_file).unwrap();
        assert_eq!(loaded_db.fingerprints.len(), 1);

        let _ = std::fs::remove_file(temp_file);
    }

    #[test]
    fn test_match_best_model() {
        let mut db = ModelFingerprintDatabase::new();
        db.add_fingerprint(create_test_fingerprint("gpt4", "text"));
        db.add_fingerprint(create_test_fingerprint("claude3", "text"));

        let sig = create_test_signature();
        let matches = db.match_fingerprint(&sig, "text");

        assert!(!matches.is_empty());
        let best_match = &matches[0];
        assert!(best_match.1 > 0.0);
    }

    #[test]
    fn test_cosine_similarity() {
        let sig1 = create_test_signature();
        let sig2 = create_test_signature();

        let similarity = calculate_cosine_similarity(&sig1, &sig2);
        assert!((similarity - 1.0).abs() < 0.01); // Same signature should be ~1.0
    }

    #[test]
    fn test_kl_divergence() {
        let sig1 = create_test_signature();
        let sig2 = create_test_signature();

        let divergence = calculate_kl_divergence(&sig1, &sig2);
        assert!(divergence < 0.1); // Same signature should have low divergence
    }

    // Helper functions
    fn create_test_fingerprint(name: &str, model_type: &str) -> ModelFingerprint {
        ModelFingerprint {
            model_name: name.to_string(),
            model_type: model_type.to_string(),
            sample_count: 1,
            statistical_signature: create_test_signature(),
            characteristic_patterns: vec!["pattern1".to_string()],
            learned_at: chrono::Utc::now().to_rfc3339(),
        }
    }

    fn create_test_signature() -> StatisticalSignature {
        StatisticalSignature {
            perplexity_mean: 0.25,
            perplexity_std: 0.05,
            burstiness_mean: 0.20,
            burstiness_std: 0.03,
            vocabulary_richness_mean: 0.70,
            vocabulary_richness_std: 0.10,
            noise_pattern_mean: 0.03,
            noise_pattern_std: 0.01,
            texture_regularity_mean: 0.70,
            texture_regularity_std: 0.05,
            color_distribution_mean: 0.60,
            color_distribution_std: 0.08,
        }
    }
}
