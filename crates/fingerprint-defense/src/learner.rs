//! fingerprint自learnmodule
//!
//! automatic from observe to traffic in learn and Updatefingerprintsignature.
//! 实现完整的指纹自学习机制，自动识别并记录未知稳定指纹特征以对抗0-day bots

use crate::database::FingerprintDatabase;
use crate::passive::PassiveAnalysisResult;
use dashmap::DashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use fingerprint_core::fingerprint::Fingerprint;
use serde::{Deserialize, Serialize};

/// 获取当前 Unix 时间戳（秒）
fn current_unix_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::ZERO)
        .as_secs()
}

/// 计算时间戳差（秒）
fn timestamp_duration(from: u64, to: u64) -> Duration {
    Duration::from_secs(to.saturating_sub(from))
}

/// 未知指纹观察记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnknownFingerprintObservation {
    /// 指纹ID
    pub fingerprint_id: String,
    /// 指纹类型 (tls/http/tcp)
    pub fingerprint_type: String,
    /// 首次观察时间（Unix 时间戳，秒）
    pub first_seen: u64,
    /// 最后观察时间（Unix 时间戳，秒）
    pub last_seen: u64,
    /// 观察次数
    pub observation_count: u64,
    /// 稳定性得分 (0.0-1.0)
    pub stability_score: f64,
    /// 相关特征数据
    pub features: serde_json::Value,
}

/// 自learnanalysiser
pub struct SelfLearningAnalyzer {
    #[allow(dead_code)] // will来will for storelearn to fingerprint
    db: Arc<FingerprintDatabase>,
    /// 未知指纹观察记录 (fp_id -> observation)
    observations: DashMap<String, UnknownFingerprintObservation>,
    /// 学习阈值 (观察多少次后进入数据库)
    learning_threshold: u64,
    /// 稳定性时间窗口 (默认24小时)
    stability_window: Duration,
    /// 最小稳定性得分阈值
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
        // 分别处理各层指纹
        if let Some(tls) = &result.tls {
            // TLS直接使用观察ID (JA4)
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

    /// 观察未知指纹并计算稳定性
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

        // 保护点：限制观察列表大小，防止内存爆增 (DoS防护)
        const MAX_OBSERVATIONS: usize = 10000;
        if self.observations.len() >= MAX_OBSERVATIONS && !self.observations.contains_key(&key) {
            // 如果达到上限且是新键，则忽略
            return;
        }

        // 更新或创建观察记录
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

        // 更新观察记录
        entry.observation_count += 1;
        entry.last_seen = now;

        // 计算稳定性得分
        let time_span = timestamp_duration(entry.first_seen, entry.last_seen);
        let expected_frequency =
            entry.observation_count as f64 / (time_span.as_secs_f64() / 3600.0).max(1.0); // 每小时观察频率

        // 稳定性得分基于观察频率的一致性
        let stability_bonus = if expected_frequency > 1.0 && expected_frequency < 100.0 {
            0.3 // 正常频率加分
        } else if expected_frequency >= 100.0 {
            0.1 // 高频但不稳定
        } else {
            0.0 // 频率太低
        };

        entry.stability_score =
            (entry.observation_count as f64 / self.learning_threshold as f64).min(1.0) * 0.7
                + stability_bonus;

        // 检查是否达到学习条件
        if entry.observation_count >= self.learning_threshold
            && entry.stability_score >= self.min_stability_score
        {
            // 达到阈值，可以进入数据库建立初步条目
            self.learn_new_fingerprint(&entry);
        }
    }

    /// 学习新的稳定指纹
    fn learn_new_fingerprint(&self, observation: &UnknownFingerprintObservation) {
        println!(
            "[Learner] 🎯 Detected stable unknown fingerprint: {}:{} (count: {}, stability: {:.2})",
            observation.fingerprint_type,
            observation.fingerprint_id,
            observation.observation_count,
            observation.stability_score
        );

        // TODO: 将稳定指纹存入数据库作为待审核候选签名
        // 这里应该调用数据库接口存储潜在的新指纹模式
        // 例如：self.db.store_candidate_fingerprint(observation)
    }

    /// 设置学习阈值
    pub fn set_threshold(&mut self, threshold: u64) {
        self.learning_threshold = threshold;
    }

    /// 设置稳定性窗口
    pub fn set_stability_window(&mut self, duration: Duration) {
        self.stability_window = duration;
    }

    /// 设置最小稳定性得分
    pub fn set_min_stability_score(&mut self, score: f64) {
        self.min_stability_score = score.clamp(0.0, 1.0);
    }

    /// 获取当前观察统计
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

    /// 清理过期观察记录
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

/// 观察统计信息
#[derive(Debug, Clone)]
pub struct ObservationStats {
    pub total_observations: u64,
    pub stable_candidates: u64,
    pub learning_threshold: u64,
    pub min_stability_score: f64,
}
