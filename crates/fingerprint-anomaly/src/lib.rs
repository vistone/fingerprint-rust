#![allow(clippy::all, dead_code, unused_variables, unused_parens)]

//! # Anomaly Detection Module
//!
//! 异常检测模块
//!
//! 提供指纹异常检测和攻击识别功能

use std::collections::VecDeque;

// / exceptiondetect结果
#[derive(Debug, Clone)]
pub struct AnomalyDetectionResult {
    // / 是否exception
    pub is_anomaly: bool,
    // / exceptionscore (0-1)
    pub anomaly_score: f32,
    // / exceptiontype
    pub anomaly_type: Option<AnomalyType>,
    // / confidence
    pub confidence: f32,
}

// / exceptiontype
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnomalyType {
    // / fingerprint矛盾
    FingerprintContradiction,
    // / 不可能ofconvert
    ImpossibleTransition,
    // / timeexception
    TimingAnomaly,
    // / statisticsexception
    StatisticalAnomaly,
    // / behaviorexception
    BehaviorAnomaly,
}

// / exceptiondetector
pub struct AnomalyDetector {
    history: VecDeque<f32>,
    max_history: usize,
}

impl AnomalyDetector {
    // / createnewdetector
    pub fn new(max_history: usize) -> Self {
        AnomalyDetector {
            history: VecDeque::with_capacity(max_history),
            max_history,
        }
    }

    // / detectexception
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

    // / calculateexceptionscore
    fn calculate_anomaly_score(&self, current: &f32) -> (f32, Option<AnomalyType>) {
        if self.history.len() < 2 {
            return (0.0, None);
        }

        // statisticsexceptiondetect
        let mean: f32 = self.history.iter().sum::<f32>() / self.history.len() as f32;
        let variance: f32 = self.history.iter().map(|x| (x - mean).powi(2)).sum::<f32>()
            / self.history.len() as f32;
        let std_dev = variance.sqrt();

        if std_dev > 0.0 && (*current - mean).abs() > 3.0 * std_dev {
            return (0.8, Some(AnomalyType::StatisticalAnomaly));
        }

        // timeexceptiondetect
        if let Some(&last) = self.history.back() {
            if (current - last).abs() > 0.5 {
                return (0.6, Some(AnomalyType::TimingAnomaly));
            }
        }

        (0.0, None)
    }

    // / 清除历史
    pub fn clear_history(&mut self) {
        self.history.clear();
    }
}

impl Default for AnomalyDetector {
    fn default() -> Self {
        Self::new(100)
    }
}

// / fingerprint矛盾detector
pub struct ContradictionDetector;

impl ContradictionDetector {
    // / detectfingerprint矛盾
    pub fn detect_contradictions(fingerprints: &[(&str, f32)]) -> Vec<(usize, usize, String)> {
        let mut contradictions = Vec::new();

        for i in 0..fingerprints.len() {
            for j in (i + 1)..fingerprints.len() {
                let (name1, score1) = fingerprints[i];
                let (name2, score2) = fingerprints[j];

                // check不匹配ofcomposite
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

    // / check是否矛盾
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
        // ChromeandFirefox同时有高分是矛盾of（不能同时recognitionto两个不同浏览器）
        let fingerprints = vec![("Chrome", 0.9), ("Firefox", 0.85)];
        let contradictions = ContradictionDetector::detect_contradictions(&fingerprints);
        assert_eq!(contradictions.len(), 1); // 应该detect到1个矛盾

        // 如果其中一个分数低，则不矛盾
        let non_contradictory = vec![("Chrome", 0.9), ("Firefox", 0.3)];
        let contradictions2 = ContradictionDetector::detect_contradictions(&non_contradictory);
        assert_eq!(contradictions2.len(), 0); // 低分浏览器不构成矛盾
    }
}
