//! Advanced AI Content Detection
//!
//! This module provides advanced detection algorithms including statistical tests,
//! ensemble methods, and explainability features.
use std::collections::HashMap;

/// Detection explanation - why content was flagged as AI
#[derive(Debug, Clone)]
pub struct DetectionExplanation {
    /// Primary reason for detection
    pub primary_reason: String,

    /// All contributing factors with scores
    pub factors: Vec<(String, f64, String)>, // (factor_name, score, description)

    /// Confidence in the explanation
    pub explanation_confidence: f64,
}

/// Ensemble detection result combining multiple algorithms
#[derive(Debug, Clone)]
pub struct EnsembleDetectionResult {
    /// Final AI detection verdict
    pub is_ai_generated: bool,

    /// Overall confidence (0.0 to 1.0)
    pub confidence: f64,

    /// Individual algorithm results
    pub algorithm_results: HashMap<String, f64>,

    /// Explanation of detection
    pub explanation: DetectionExplanation,

    /// Model attribution if applicable
    pub model_attribution: HashMap<String, f64>,
}

/// Advanced statistical tests for AI detection
pub struct AdvancedStatistics;

impl AdvancedStatistics {
    /// Benford's Law analysis for pixel/frequency distributions
    ///
    /// Natural images follow Benford's Law for leading digits in measurements.
    /// AI-generated content often violates this.
    pub fn benfords_law_test(values: &[u32]) -> f64 {
        if values.is_empty() {
            return 0.5;
        }

        // Expected frequencies for leading digits (Benford's Law)
        let expected = [
            0.301, // 1
            0.176, // 2
            0.125, // 3
            0.097, // 4
            0.079, // 5
            0.067, // 6
            0.058, // 7
            0.051, // 8
            0.046, // 9
        ];

        // Count leading digits
        let mut counts = [0; 9];
        for &value in values {
            if value > 0 {
                let first_digit = Self::get_leading_digit(value);
                if (1..=9).contains(&first_digit) {
                    counts[first_digit - 1] += 1;
                }
            }
        }

        // Calculate observed frequencies
        let total: usize = counts.iter().sum();
        if total == 0 {
            return 0.5;
        }

        let observed: Vec<f64> = counts.iter().map(|&c| c as f64 / total as f64).collect();

        // Chi-square test
        let chi_square: f64 = expected
            .iter()
            .zip(observed.iter())
            .map(|(&exp, &obs)| {
                let diff = obs - exp;
                (diff * diff) / exp
            })
            .sum();

        // Normalize chi-square to 0-1 score
        // Higher chi-square = more deviation from Benford's Law = more AI-like
        // Chi-square critical value at 0.05 significance for 8 df is ~15.5

        (chi_square / 15.5).min(1.0)
    }

    /// Get leading (first non-zero) digit
    fn get_leading_digit(mut n: u32) -> usize {
        while n >= 10 {
            n /= 10;
        }
        n as usize
    }

    /// Chi-square test for distribution uniformity
    ///
    /// Tests if a distribution is too uniform (AI-like) or natural
    pub fn chi_square_uniformity_test(distribution: &[u32], expected_uniform: bool) -> f64 {
        if distribution.is_empty() {
            return 0.5;
        }

        let total: u32 = distribution.iter().sum();
        if total == 0 {
            return 0.5;
        }

        let expected_per_bin = total as f64 / distribution.len() as f64;

        let chi_square: f64 = distribution
            .iter()
            .map(|&observed| {
                let diff = observed as f64 - expected_per_bin;
                (diff * diff) / expected_per_bin
            })
            .sum();

        // Critical value for typical case (adjusted for bin count)
        let critical_value = distribution.len() as f64 * 2.0;

        if expected_uniform {
            // High chi-square means non-uniform (natural)
            // Low chi-square means uniform (AI-like)
            1.0 - (chi_square / critical_value).min(1.0)
        } else {
            // High chi-square means non-uniform (as expected for natural)
            (chi_square / critical_value).min(1.0)
        }
    }

    /// Kolmogorov-Smirnov test for distribution similarity
    pub fn ks_test_naturalness(values: &[f64]) -> f64 {
        if values.len() < 10 {
            return 0.5;
        }

        let mut sorted = values.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        // Compare against expected Gaussian distribution
        let mean = values.iter().sum::<f64>() / values.len() as f64;
        let variance =
            values.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / values.len() as f64;
        let std_dev = variance.sqrt();

        if std_dev == 0.0 {
            return 1.0; // Perfect uniformity = very AI-like
        }

        // Calculate max distance between empirical and theoretical CDF
        let mut max_distance = 0.0;
        for (i, &value) in sorted.iter().enumerate() {
            let empirical_cdf = (i + 1) as f64 / sorted.len() as f64;

            // Gaussian CDF approximation
            let z = (value - mean) / std_dev;
            let theoretical_cdf = Self::normal_cdf(z);

            let distance = (empirical_cdf - theoretical_cdf).abs();
            if distance > max_distance {
                max_distance = distance;
            }
        }

        // K-S statistic, higher = more different from natural
        // Typical critical value ~0.3 for moderate sample sizes

        (max_distance / 0.3).min(1.0)
    }

    /// Approximate normal CDF using error function approximation
    fn normal_cdf(z: f64) -> f64 {
        0.5 * (1.0 + Self::erf(z / std::f64::consts::SQRT_2))
    }

    /// Error function approximation
    fn erf(x: f64) -> f64 {
        // Abramowitz and Stegun approximation
        let a1 = 0.254829592;
        let a2 = -0.284496736;
        let a3 = 1.421413741;
        let a4 = -1.453152027;
        let a5 = 1.061405429;
        let p = 0.3275911;

        let sign = if x < 0.0 { -1.0 } else { 1.0 };
        let x = x.abs();

        let t = 1.0 / (1.0 + p * x);
        let y = 1.0 - (((((a5 * t + a4) * t) + a3) * t + a2) * t + a1) * t * (-x * x).exp();

        sign * y
    }
}

/// Ensemble detector combining multiple algorithms
pub struct EnsembleDetector;

impl EnsembleDetector {
    /// Combine multiple detection scores using weighted voting
    pub fn combine_scores(scores: &HashMap<String, f64>, weights: &HashMap<String, f64>) -> f64 {
        let mut weighted_sum = 0.0;
        let mut total_weight = 0.0;

        for (algorithm, &score) in scores {
            let weight = weights.get(algorithm).copied().unwrap_or(1.0);
            weighted_sum += score * weight;
            total_weight += weight;
        }

        if total_weight > 0.0 {
            weighted_sum / total_weight
        } else {
            0.5
        }
    }

    /// Generate explanation for detection
    pub fn explain_detection(
        algorithm_results: &HashMap<String, f64>,
        threshold: f64,
    ) -> DetectionExplanation {
        let mut factors = Vec::new();

        // Analyze each algorithm result
        for (algorithm, &score) in algorithm_results {
            let description = if score > threshold {
                format!(
                    "{} indicates strong AI patterns (score: {:.2})",
                    algorithm, score
                )
            } else {
                format!("{} shows natural patterns (score: {:.2})", algorithm, score)
            };

            factors.push((algorithm.clone(), score, description));
        }

        // Sort by score (highest first)
        factors.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Determine primary reason
        let primary_reason = if let Some((algo, score, _)) = factors.first() {
            if *score > threshold {
                format!(
                    "Primary indicator: {} (confidence: {:.1}%)",
                    algo,
                    score * 100.0
                )
            } else {
                "No strong AI indicators detected".to_string()
            }
        } else {
            "Insufficient data for analysis".to_string()
        };

        // Calculate explanation confidence based on agreement between algorithms
        let high_scores = factors
            .iter()
            .filter(|(_, score, _)| *score > threshold)
            .count();
        let low_scores = factors
            .iter()
            .filter(|(_, score, _)| *score <= threshold)
            .count();
        let total = factors.len();

        let agreement = if total > 0 {
            high_scores.max(low_scores) as f64 / total as f64
        } else {
            0.5
        };

        DetectionExplanation {
            primary_reason,
            factors,
            explanation_confidence: agreement,
        }
    }

    /// Calibrate confidence scores to realistic ranges
    pub fn calibrate_confidence(raw_score: f64, modality: &str) -> f64 {
        // Different modalities have different baseline accuracies
        let (min_conf, max_conf) = match modality {
            "image" => (0.6, 0.95),  // Images are most reliable
            "text" => (0.5, 0.85),   // Text is moderately reliable
            "audio" => (0.55, 0.90), // Audio is good
            "video" => (0.6, 0.92),  // Video is quite reliable
            _ => (0.5, 0.9),
        };

        // Map raw score to calibrated range
        min_conf + (max_conf - min_conf) * raw_score
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_benfords_law_natural() {
        // Natural data following Benford's Law
        let values = vec![100, 223, 156, 498, 531, 267, 178, 451, 392, 146];
        let score = AdvancedStatistics::benfords_law_test(&values);
        assert!(score < 0.5, "Natural data should have low AI score");
    }

    #[test]
    fn test_benfords_law_uniform() {
        // Uniform data NOT following Benford's Law (AI-like)
        // Note: This particular sequence has many leading 1s (100, 300, 500, 700, 900)
        // so it may not deviate enough. Use more extreme example.
        let values = vec![500, 510, 520, 530, 540, 550, 560, 570, 580, 590];
        let score = AdvancedStatistics::benfords_law_test(&values);
        // This should have high chi-square due to all values starting with 5
        assert!(
            score > 0.3,
            "Uniform data should have elevated AI score, got {}",
            score
        );
    }

    #[test]
    fn test_chi_square_uniformity() {
        // Perfectly uniform distribution (AI-like)
        let uniform = vec![100, 100, 100, 100, 100];
        let score = AdvancedStatistics::chi_square_uniformity_test(&uniform, true);
        assert!(
            score > 0.8,
            "Uniform distribution should score high for uniformity"
        );

        // Varied distribution (natural)
        let varied = vec![150, 80, 120, 50, 200];
        let score = AdvancedStatistics::chi_square_uniformity_test(&varied, true);
        assert!(
            score < 0.5,
            "Varied distribution should score low for uniformity"
        );
    }

    #[test]
    fn test_ks_test_naturalness() {
        // Gaussian-like natural values
        let natural = vec![0.5, 0.52, 0.48, 0.51, 0.49, 0.53, 0.47, 0.50, 0.52, 0.48];
        let score = AdvancedStatistics::ks_test_naturalness(&natural);
        assert!(score < 0.6, "Natural distribution should have lower score");
    }

    #[test]
    fn test_ensemble_combine_scores() {
        let mut scores = HashMap::new();
        scores.insert("algorithm1".to_string(), 0.8);
        scores.insert("algorithm2".to_string(), 0.6);
        scores.insert("algorithm3".to_string(), 0.7);

        let mut weights = HashMap::new();
        weights.insert("algorithm1".to_string(), 2.0);
        weights.insert("algorithm2".to_string(), 1.0);
        weights.insert("algorithm3".to_string(), 1.0);

        let combined = EnsembleDetector::combine_scores(&scores, &weights);
        assert!(combined > 0.6 && combined < 0.8);
    }

    #[test]
    fn test_explain_detection() {
        let mut results = HashMap::new();
        results.insert("noise_analysis".to_string(), 0.85);
        results.insert("frequency_analysis".to_string(), 0.75);
        results.insert("texture_analysis".to_string(), 0.45);

        let explanation = EnsembleDetector::explain_detection(&results, 0.7);

        assert!(!explanation.primary_reason.is_empty());
        assert_eq!(explanation.factors.len(), 3);
        assert!(explanation.explanation_confidence > 0.0);
    }

    #[test]
    fn test_calibrate_confidence() {
        let raw = 0.8;

        let image_conf = EnsembleDetector::calibrate_confidence(raw, "image");
        let text_conf = EnsembleDetector::calibrate_confidence(raw, "text");

        assert!(
            image_conf > text_conf,
            "Image should have higher confidence range"
        );
        assert!(image_conf <= 0.95);
        assert!(text_conf <= 0.85);
    }
}
