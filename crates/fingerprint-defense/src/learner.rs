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

/// get当前 Unix time戳（秒）
fn current_unix_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::ZERO)
        .as_secs()
}

/// calculatetime戳差（秒）
fn timestamp_duration(from: u64, to: u64) -> Duration {
    Duration::from_secs(to.saturating_sub(from))
}

/// unknownfingerprint观察record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnknownFingerprintObservation {
    /// fingerprintID
    pub fingerprint_id: String,
    /// fingerprinttype (tls/http/tcp)
    pub fingerprint_type: String,
    /// 首次观察time（Unix time戳，秒）
    pub first_seen: u64,
    /// 最后观察time（Unix time戳，秒）
    pub last_seen: u64,
    /// 观察次数
    pub observation_count: u64,
    /// stable性得分 (0.0-1.0)
    pub stability_score: f64,
    /// 相关featuresdata
    pub features: serde_json::Value,
}

/// 自learnanalysiser
pub struct SelfLearningAnalyzer {
    #[allow(dead_code)] // will be used to store learned fingerprints
    db: Arc<FingerprintDatabase>,
    /// unknownfingerprint观察record (fp_id -> observation)
    observations: DashMap<String, UnknownFingerprintObservation>,
    /// learning threshold (how many observations before entering database)
    learning_threshold: u64,
    /// stability time window (default 24 hours)
    stability_window: Duration,
    /// minimum stability score threshold
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

    /// processanalysisresult并learn
    pub fn process_result(&self, result: &PassiveAnalysisResult) {
        // 分别process各层fingerprint
        if let Some(tls) = &result.tls {
            // TLS直接use观察ID (JA4)
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

    /// 观察unknownfingerprint并calculatestable性
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

        // Update or create observation record
        let mut entry =
            self.observations
                .entry(key.clone())
                .or_insert_with(|| UnknownFingerprintObservation {
                    fingerprint_id: fp_id.clone(),
                    fingerprint_type: fp_type.to_string(),
                    first_seen: now,
                    last_seen: now,
                    observation_count: 0,
                    stability_score: 0.0,
                    features: features.clone(),
                });

        // update观察record
        entry.observation_count += 1;
        entry.last_seen = now;

        // calculate stability score
        let time_span = timestamp_duration(entry.first_seen, entry.last_seen);
        let expected_frequency =
            entry.observation_count as f64 / (time_span.as_secs_f64() / 3600.0).max(1.0); // observation frequency per hour

        // stability score based on observation frequency consistency
        let stability_bonus = if expected_frequency > 1.0 && expected_frequency < 100.0 {
            0.3 // normal frequency bonus
        } else if expected_frequency >= 100.0 {
            0.1 // high frequency but not stable
        } else {
            0.0 // frequency too low
        };

        entry.stability_score =
            (entry.observation_count as f64 / self.learning_threshold as f64).min(1.0) * 0.7
                + stability_bonus;

        // check if learning conditions are met
        if entry.observation_count >= self.learning_threshold
            && entry.stability_score >= self.min_stability_score
        {
            // threshold reached, can enter database to create preliminary entry
            self.learn_new_fingerprint(&entry);
        }
    }

    /// learningnewstablefingerprint
    fn learn_new_fingerprint(&self, observation: &UnknownFingerprintObservation) {
        log::info!(
            "[Learner] 🎯 Detected stable unknown fingerprint: {}:{} (count: {}, stability: {:.2})",
            observation.fingerprint_type,
            observation.fingerprint_id,
            observation.observation_count,
            observation.stability_score
        );

        // 将stablefingerprint存入datalibrary作topending reviewcandidatessignature
        match self.db.store_candidate_fingerprint(
            &observation.fingerprint_type,
            &observation.fingerprint_id,
            observation.observation_count.try_into().unwrap(),
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

    /// setlearning阈值
    pub fn set_threshold(&mut self, threshold: u64) {
        self.learning_threshold = threshold;
    }

    /// setstable性窗口
    pub fn set_stability_window(&mut self, duration: Duration) {
        self.stability_window = duration;
    }

    /// set最小stable性得分
    pub fn set_min_stability_score(&mut self, score: f64) {
        self.min_stability_score = score.clamp(0.0, 1.0);
    }

    /// get current observation statistics
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

    /// cleanup expired observation records
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

/// 观察statisticsinfo
#[derive(Debug, Clone)]
pub struct ObservationStats {
    pub total_observations: u64,
    pub stable_candidates: u64,
    pub learning_threshold: u64,
    pub min_stability_score: f64,
}

/// Fingerprint evaluator for assessing stability and credibility
/// 指纹评估器用于评估稳定性和可信度
pub struct FingerprintEvaluator {
    stability_threshold: f64,
    observation_threshold: u64,
}

impl FingerprintEvaluator {
    /// Create new fingerprint evaluator
    /// 创建新的指纹评估器
    pub fn new(stability_threshold: f64, observation_threshold: u64) -> Self {
        FingerprintEvaluator {
            stability_threshold,
            observation_threshold,
        }
    }

    /// Evaluate fingerprint stability
    /// 评估指纹稳定性
    pub fn evaluate_stability(&self, observation: &UnknownFingerprintObservation) -> f64 {
        // Simple stability calculation based on observation consistency
        if observation.observation_count < self.observation_threshold {
            0.0
        } else {
            observation.stability_score
        }
    }

    /// Check if fingerprint is ready for learning
    /// 检查指纹是否准备好进行学习
    pub fn is_ready_for_learning(&self, observation: &UnknownFingerprintObservation) -> bool {
        observation.observation_count >= self.observation_threshold
            && observation.stability_score >= self.stability_threshold
    }
}

/// Fingerprint observer for collecting unknown fingerprints
/// 指纹观察器用于收集未知指纹
#[allow(dead_code)]
pub struct FingerprintObserver {
    evaluator: FingerprintEvaluator,
    database: Arc<FingerprintDatabase>,
}

impl FingerprintObserver {
    /// Create new fingerprint observer
    /// 创建新的指纹观察器
    pub fn new(database: Arc<FingerprintDatabase>) -> Self {
        FingerprintObserver {
            evaluator: FingerprintEvaluator::new(0.8, 10),
            database,
        }
    }

    /// Observe network traffic and collect fingerprints
    /// 观察网络流量并收集指纹
    pub fn observe(&self, _result: &PassiveAnalysisResult) {
        // Implementation would go here
        // 实现会在这里
    }

    /// Get evaluation results
    /// 获取评估结果
    pub fn get_results(&self) -> Vec<UnknownFingerprintObservation> {
        // Return collected observations
        // 返回收集的观察结果
        vec![]
    }
}
