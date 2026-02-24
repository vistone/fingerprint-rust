//! fingerprint self-learning module
//!
//! automatically learns and updates fingerprint signatures from observed traffic.
//! implements complete fingerprint self-learning mechanism, automatically recognizing and recording unknown stable fingerprint features for combating 0-day bots

use crate::database::FingerprintDatabase;
use crate::passive::PassiveAnalysisResult;
use dashmap::DashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use fingerprint_core::fingerprint::Fingerprint;
use serde::{Deserialize, Serialize};

/// Get current Unix timestamp (seconds)
fn current_unix_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::ZERO)
        .as_secs()
}

/// Calculate timestamp difference (seconds)
fn timestamp_duration(from: u64, to: u64) -> Duration {
    Duration::from_secs(to.saturating_sub(from))
}

/// Unknown fingerprint observation record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnknownFingerprintObservation {
    /// Fingerprint ID
    pub fingerprint_id: String,
    /// Fingerprint type (tls/http/tcp)
    pub fingerprint_type: String,
    /// First seen timestamp (Unix timestamp, seconds)
    pub first_seen: u64,
    /// Last seen timestamp (Unix timestamp, seconds)
    pub last_seen: u64,
    /// Observation count
    pub observation_count: u64,
    /// Stability score (0.0-1.0)
    pub stability_score: f64,
    /// Associated features data
    pub features: serde_json::Value,
}

/// Self-learning analyzer
pub struct SelfLearningAnalyzer {
    #[allow(dead_code)] // will be used to store learned fingerprints
    db: Arc<FingerprintDatabase>,
    /// Unknown fingerprint observation records (fp_id -> observation)
    observations: DashMap<String, UnknownFingerprintObservation>,
    /// Learning threshold (how many observations before entering database)
    learning_threshold: u64,
    /// Stability time window (default 24 hours)
    stability_window: Duration,
    /// Minimum stability score threshold
    min_stability_score: f64,
}

impl SelfLearningAnalyzer {
    /// Create a newlearnanalysiser
    pub fn new(db: Arc<FingerprintDatabase>) -> Self {
        Self {
            db,
            observations: DashMap::new(),
            learning_threshold: 10,
            stability_window: Duration::from_secs(24 * 60 * 60), // 24小时
            min_stability_score: 0.8,
        }
    }

    /// Process analysis result and learn
    pub fn process_result(&self, result: &PassiveAnalysisResult) {
        // 分别process各层fingerprint
        if let Some(tls) = &result.tls {
            // Use TLS observation ID directly (JA4)
            self.observe_unknown_fingerprint(
                tls.id(),
                "tls",
                &serde_json::json!({
                    "cipher_suites_count": tls.cipher_suites_count,
                    "extensions_count": tls.extensions_count,
                    "version": tls.version,
                    "ja4": tls.ja4.clone(),
                }),
            );
        }

        if let Some(http) = &result.http {
            if http.signature.is_none() {
                self.observe_unknown_fingerprint(
                    http.id(),
                    "http",
                    &serde_json::json!({
                        "user_agent": http.user_agent,
                        "browser": http.browser,
                        "h2_settings": http.h2_settings,
                    }),
                );
            }
        }

        if let Some(tcp) = &result.tcp {
            if tcp.signature.is_none() {
                self.observe_unknown_fingerprint(
                    tcp.id(),
                    "tcp",
                    &serde_json::json!({
                        "ttl": tcp.features.ttl,
                        "window": tcp.features.window,
                        "mss": tcp.features.mss,
                        "window_scale": tcp.features.window_scale,
                        "options_str": tcp.features.options_str,
                        "ip_flags": tcp.features.ip_flags,
                    }),
                );
            }
        }
    }

    /// Observe unknown fingerprint and calculate stability
    fn observe_unknown_fingerprint(
        &self,
        fp_id: String,
        fp_type: &str,
        features: &serde_json::Value,
    ) {
        if fp_id == "unknown" || fp_id.is_empty() {
            return;
        }

        let key = format!("{}:{}", fp_type, fp_id);
        let now = current_unix_timestamp();

        // Protection point: limit observation list size to prevent memory explosion (DoS protection)
        const MAX_OBSERVATIONS: usize = 10000;
        if self.observations.len() >= MAX_OBSERVATIONS && !self.observations.contains_key(&key) {
            // If maximum reached and it's a new key, ignore it
            return;
        }

        // Use DashMap's entry API for atomic updates
        self.observations
            .entry(key.clone())
            .and_modify(|entry| {
                // Atomically update the entry
                entry.observation_count += 1;
                entry.last_seen = now;

                // Recalculate stability score based on updated observation count
                let time_span = timestamp_duration(entry.first_seen, entry.last_seen);
                let expected_frequency =
                    entry.observation_count as f64 / (time_span.as_secs_f64() / 3600.0).max(1.0); // observations per hour

                // stability score based on observation frequency consistency
                let stability_bonus = if expected_frequency > 1.0 && expected_frequency < 100.0 {
                    0.3 // normal frequency bonus
                } else if expected_frequency >= 100.0 {
                    0.1 // high frequency but not stable
                } else {
                    0.0 // frequency too low
                };

                entry.stability_score =
                    (entry.observation_count as f64 / self.learning_threshold as f64).min(1.0)
                        * 0.7
                        + stability_bonus;

                // check if learning conditions are met
                if entry.observation_count >= self.learning_threshold
                    && entry.stability_score >= self.min_stability_score
                {
                    // threshold reached, can enter database to create preliminary entry
                    // Store reference for later processing
                    log::info!(
                        "[Learner] 🎯 Ready to learn: {}:{} (count: {}, stability: {:.2})",
                        entry.fingerprint_type,
                        entry.fingerprint_id,
                        entry.observation_count,
                        entry.stability_score
                    );
                }
            })
            .or_insert_with(|| {
                // Create new observation record
                UnknownFingerprintObservation {
                    fingerprint_id: fp_id,
                    fingerprint_type: fp_type.to_string(),
                    first_seen: now,
                    last_seen: now,
                    observation_count: 1,
                    stability_score: 0.0,
                    features: features.clone(),
                }
            });

        // Process learning after the atomic update completes
        if let Some(entry) = self.observations.get(&key) {
            if entry.observation_count >= self.learning_threshold
                && entry.stability_score >= self.min_stability_score
            {
                // Create a clone to avoid holding the read lock during database operation
                let observation = entry.value().clone();
                drop(entry); // Explicitly release the DashMap read lock
                self.learn_new_fingerprint(&observation);
            }
        }
    }

    /// Learn new stable fingerprint
    fn learn_new_fingerprint(&self, observation: &UnknownFingerprintObservation) {
        log::info!(
            "[Learner] 🎯 Detected stable unknown fingerprint: {}:{} (count: {}, stability: {:.2})",
            observation.fingerprint_type,
            observation.fingerprint_id,
            observation.observation_count,
            observation.stability_score
        );

        // Store stable fingerprint in database as a candidate signature pending review
        // Use unwrap_or to handle potential overflow when converting u64 to u32
        let observation_count_u32 = observation.observation_count.try_into().unwrap_or(u32::MAX);
        match self.db.store_candidate_fingerprint(
            &observation.fingerprint_type,
            &observation.fingerprint_id,
            observation_count_u32,
            observation.stability_score,
            Some(&format!(
                "Auto-detected stable fingerprint with count {} and stability {:.2}",
                observation.observation_count, observation.stability_score
            )),
        ) {
            Ok(candidate_id) => {
                log::info!(
                    "[Learner] ✅ Successfully stored candidate fingerprint #{} for review",
                    candidate_id
                );
            }
            Err(e) => {
                log::warn!("[Learner] ⚠️ Failed to store candidate fingerprint: {}", e);
            }
        }
    }

    /// Set learning threshold
    pub fn set_threshold(&mut self, threshold: u64) {
        self.learning_threshold = threshold;
    }

    /// Set stability window
    pub fn set_stability_window(&mut self, duration: Duration) {
        self.stability_window = duration;
    }

    /// Set minimum stability score
    pub fn set_min_stability_score(&mut self, score: f64) {
        self.min_stability_score = score.clamp(0.0, 1.0);
    }

    /// Get current observation statistics
    pub fn get_observation_stats(&self) -> ObservationStats {
        let total_observations = self.observations.len() as u64;
        let stable_candidates = self
            .observations
            .iter()
            .filter(|entry| {
                entry.value().observation_count >= self.learning_threshold
                    && entry.value().stability_score >= self.min_stability_score
            })
            .count() as u64;

        ObservationStats {
            total_observations,
            stable_candidates,
            learning_threshold: self.learning_threshold,
            min_stability_score: self.min_stability_score,
        }
    }

    /// Cleanup expired observation records
    pub fn cleanup_expired_observations(&self) {
        let now = current_unix_timestamp();
        let expired_keys: Vec<String> = self
            .observations
            .iter()
            .filter(|entry| {
                timestamp_duration(entry.value().first_seen, now) > self.stability_window
            })
            .map(|entry| entry.key().clone())
            .collect();

        for key in expired_keys {
            self.observations.remove(&key);
        }
    }
}

/// Observation statistics info
#[derive(Debug, Clone)]
pub struct ObservationStats {
    pub total_observations: u64,
    pub stable_candidates: u64,
    pub learning_threshold: u64,
    pub min_stability_score: f64,
}

/// Fingerprint evaluator for assessing stability and credibility
/// Fingerprint evaluator for assessing stability and credibility
pub struct FingerprintEvaluator {
    stability_threshold: f64,
    observation_threshold: u64,
}

impl FingerprintEvaluator {
    /// Create new fingerprint evaluator
    /// Create new fingerprint evaluator
    pub fn new(stability_threshold: f64, observation_threshold: u64) -> Self {
        FingerprintEvaluator {
            stability_threshold,
            observation_threshold,
        }
    }

    /// Evaluate fingerprint stability
    /// Evaluate fingerprint stability
    pub fn evaluate_stability(&self, observation: &UnknownFingerprintObservation) -> f64 {
        // Simple stability calculation based on observation consistency
        if observation.observation_count < self.observation_threshold {
            0.0
        } else {
            observation.stability_score
        }
    }

    /// Check if fingerprint is ready for learning
    /// Check if fingerprint is ready for learning
    pub fn is_ready_for_learning(&self, observation: &UnknownFingerprintObservation) -> bool {
        observation.observation_count >= self.observation_threshold
            && observation.stability_score >= self.stability_threshold
    }
}

/// Fingerprint observer for collecting unknown fingerprints
/// Fingerprint observer for collecting unknown fingerprints
#[allow(dead_code)]
pub struct FingerprintObserver {
    evaluator: FingerprintEvaluator,
    database: Arc<FingerprintDatabase>,
}

impl FingerprintObserver {
    /// Create new fingerprint observer
    /// Create new fingerprint observer
    pub fn new(database: Arc<FingerprintDatabase>) -> Self {
        FingerprintObserver {
            evaluator: FingerprintEvaluator::new(0.8, 10),
            database,
        }
    }

    /// Observe network traffic and collect fingerprints
    /// Observe network traffic and collect fingerprints
    pub fn observe(&self, _result: &PassiveAnalysisResult) {
        // Implementation would go here
        // 实现会在这里
    }

    /// Get evaluation results
    /// Get evaluation results
    pub fn get_results(&self) -> Vec<UnknownFingerprintObservation> {
        // Return collected observations
        vec![]
    }
}
