#![allow(clippy::all, dead_code, unused_variables, unused_parens)]

//! # fingerprint-ml
//!
//! 机器学习指纹匹配模块
//!
//! 提供高级的指纹相似度计算和分类能力

use std::collections::HashMap;

/// 指纹向量
#[derive(Debug, Clone)]
pub struct FingerprintVector {
    /// 特征向量
    pub features: Vec<f32>,
    /// 标签
    pub label: Option<String>,
    /// 置信度
    pub confidence: f32,
}

/// ML 指纹匹配器
pub struct FingerprintMatcher {
    profiles: HashMap<String, FingerprintVector>,
}

impl FingerprintMatcher {
    /// 创建新的匹配器
    pub fn new() -> Self {
        FingerprintMatcher {
            profiles: HashMap::new(),
        }
    }

    /// 添加参考指纹
    pub fn add_reference(&mut self, id: String, features: Vec<f32>, label: String) {
        self.profiles.insert(
            id,
            FingerprintVector {
                features,
                label: Some(label),
                confidence: 1.0,
            },
        );
    }

    /// 查找最相似的指纹
    pub fn find_best_match(&self, query: &[f32]) -> Option<(String, f32)> {
        let mut best_id = None;
        let mut best_similarity = 0.0;

        for (id, profile) in &self.profiles {
            let similarity = Self::cosine_similarity(query, &profile.features);
            if similarity > best_similarity {
                best_similarity = similarity;
                best_id = Some(id.clone());
            }
        }

        best_id.map(|id| (id, best_similarity))
    }

    /// 计算余弦相似度
    fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
        if a.len() != b.len() || a.is_empty() {
            return 0.0;
        }

        let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let magnitude_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let magnitude_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

        if magnitude_a == 0.0 || magnitude_b == 0.0 {
            return 0.0;
        }

        dot_product / (magnitude_a * magnitude_b)
    }

    /// 获取所有匹配项
    pub fn find_matches(&self, query: &[f32], threshold: f32) -> Vec<(String, f32)> {
        let mut matches = Vec::new();

        for (id, profile) in &self.profiles {
            let similarity = Self::cosine_similarity(query, &profile.features);
            if similarity >= threshold {
                matches.push((id.clone(), similarity));
            }
        }

        matches.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        matches
    }
}

impl Default for FingerprintMatcher {
    fn default() -> Self {
        Self::new()
    }
}

/// 行为分类器
pub struct BehaviorClassifier;

impl BehaviorClassifier {
    /// 分类行为
    pub fn classify(features: &[f32]) -> BehaviorClass {
        if features.is_empty() {
            return BehaviorClass::Unknown;
        }

        let avg: f32 = features.iter().sum::<f32>() / features.len() as f32;

        match avg {
            x if x < 0.3 => BehaviorClass::Bot,
            x if x < 0.6 => BehaviorClass::Suspicious,
            x if x < 0.8 => BehaviorClass::Normal,
            _ => BehaviorClass::Human,
        }
    }

    /// 计算风险评分
    pub fn calculate_risk_score(features: &[f32]) -> f32 {
        if features.is_empty() {
            return 0.5;
        }

        let variance = Self::calculate_variance(features);
        let anomaly_count = features
            .iter()
            .filter(|&&x| !(0.2..=0.9).contains(&x))
            .count();

        (variance + (anomaly_count as f32 * 0.1)).min(1.0)
    }

    /// 计算方差
    fn calculate_variance(features: &[f32]) -> f32 {
        let mean: f32 = features.iter().sum::<f32>() / features.len() as f32;
        let variance: f32 =
            features.iter().map(|x| (x - mean).powi(2)).sum::<f32>() / features.len() as f32;
        variance.sqrt()
    }
}

/// 行为分类
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BehaviorClass {
    /// 人类用户
    Human,
    /// 正常行为
    Normal,
    /// 可疑行为
    Suspicious,
    /// 机器人
    Bot,
    /// 未知
    Unknown,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fingerprint_matcher() {
        let mut matcher = FingerprintMatcher::new();
        matcher.add_reference(
            "chrome_120".to_string(),
            vec![0.95, 0.88, 0.92],
            "Chrome 120".to_string(),
        );

        let query = vec![0.96, 0.89, 0.91];
        let result = matcher.find_best_match(&query);
        assert!(result.is_some());
    }

    #[test]
    fn test_behavior_classification() {
        let human_features = vec![0.95, 0.92, 0.88, 0.91];
        let bot_features = vec![0.1, 0.15, 0.12, 0.18];

        assert_eq!(
            BehaviorClassifier::classify(&human_features),
            BehaviorClass::Human
        );
        assert_eq!(
            BehaviorClassifier::classify(&bot_features),
            BehaviorClass::Bot
        );
    }
}
