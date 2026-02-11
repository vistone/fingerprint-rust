#![allow(clippy::all, dead_code, unused_variables, unused_parens)]

//! # fingerprint-anomaly
//!
//! 异常检测模块
//!
//! 提供指纹异常检测和攻击识别能力

use std::collections::VecDeque;

/// 异常检测结果
#[derive(Debug, Clone)]
pub struct AnomalyDetectionResult {
    /// 是否异常
    pub is_anomaly: bool,
    /// 异常评分 (0-1)
    pub anomaly_score: f32,
    /// 异常类型
    pub anomaly_type: Option<AnomalyType>,
    /// 置信度
    pub confidence: f32,
}

/// 异常类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnomalyType {
    /// 指纹矛盾
    FingerprintContradiction,
    /// 不可能的转换
    ImpossibleTransition,
    /// 时间异常
    TimingAnomaly,
    /// 统计异常
    StatisticalAnomaly,
    /// 行为异常
    BehaviorAnomaly,
}

/// 异常检测器
pub struct AnomalyDetector {
    history: VecDeque<f32>,
    max_history: usize,
}

impl AnomalyDetector {
    /// 创建新的检测器
    pub fn new(max_history: usize) -> Self {
        AnomalyDetector {
            history: VecDeque::with_capacity(max_history),
            max_history,
        }
    }

    /// 检测异常
    pub fn detect(&mut self, features: &[f32]) -> AnomalyDetectionResult {
        if features.is_empty() {
            return AnomalyDetectionResult {
                is_anomaly: false,
                anomaly_score: 0.0,
                anomaly_type: None,
                confidence: 0.0,
            };
        }

        let current_score: f32 = features.iter().sum::<f32>() / features.len() as f32;
        self.history.push_back(current_score);

        if self.history.len() > self.max_history {
            self.history.pop_front();
        }

        let (anomaly_score, anomaly_type) = self.calculate_anomaly_score(&current_score);

        AnomalyDetectionResult {
            is_anomaly: anomaly_score > 0.7,
            anomaly_score,
            anomaly_type,
            confidence: (anomaly_score * 100.0).min(100.0) / 100.0,
        }
    }

    /// 计算异常评分
    fn calculate_anomaly_score(&self, current: &f32) -> (f32, Option<AnomalyType>) {
        if self.history.len() < 2 {
            return (0.0, None);
        }

        // 统计异常检测
        let mean: f32 = self.history.iter().sum::<f32>() / self.history.len() as f32;
        let variance: f32 = self.history.iter().map(|x| (x - mean).powi(2)).sum::<f32>()
            / self.history.len() as f32;
        let std_dev = variance.sqrt();

        if std_dev > 0.0 && (*current - mean).abs() > 3.0 * std_dev {
            return (0.8, Some(AnomalyType::StatisticalAnomaly));
        }

        // 时间异常检测
        if let Some(&last) = self.history.back() {
            if (current - last).abs() > 0.5 {
                return (0.6, Some(AnomalyType::TimingAnomaly));
            }
        }

        (0.0, None)
    }

    /// 清除历史
    pub fn clear_history(&mut self) {
        self.history.clear();
    }
}

impl Default for AnomalyDetector {
    fn default() -> Self {
        Self::new(100)
    }
}

/// 指纹矛盾检测器
pub struct ContradictionDetector;

impl ContradictionDetector {
    /// 检测指纹矛盾
    pub fn detect_contradictions(fingerprints: &[(&str, f32)]) -> Vec<(usize, usize, String)> {
        let mut contradictions = Vec::new();

        for i in 0..fingerprints.len() {
            for j in (i + 1)..fingerprints.len() {
                let (name1, score1) = fingerprints[i];
                let (name2, score2) = fingerprints[j];

                // 检查不匹配的组合
                if Self::is_contradictory(name1, name2, score1, score2) {
                    contradictions.push((
                        i,
                        j,
                        format!("Contradictory fingerprints: {} vs {}", name1, name2),
                    ));
                }
            }
        }

        contradictions
    }

    /// 检查是否矛盾
    fn is_contradictory(name1: &str, name2: &str, score1: f32, score2: f32) -> bool {
        // 例如: Chrome 不能同时是 Firefox
        if (name1.contains("Chrome") && name2.contains("Firefox"))
            || (name1.contains("Safari") && name2.contains("Windows"))
        {
            return score1 > 0.5 && score2 > 0.5;
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_anomaly_detector() {
        let mut detector = AnomalyDetector::new(10);
        let normal_features = vec![0.5, 0.5, 0.5];
        let result = detector.detect(&normal_features);
        assert!(!result.is_anomaly);
    }

    #[test]
    fn test_contradiction_detection() {
        let fingerprints = vec![("Chrome", 0.9), ("Firefox", 0.85)];
        let contradictions = ContradictionDetector::detect_contradictions(&fingerprints);
        assert_eq!(contradictions.len(), 0); // 这两个不矛盾
    }
}
