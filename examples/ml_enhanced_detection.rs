//! Enhanced Machine Learning Anomaly Detection Example
//!
//! Demonstrates the advanced anomaly detection capabilities including:
//! - Multi-algorithm ensemble detection
//! - Online learning and adaptation
//! - Comprehensive anomaly classification
//! - Detailed risk assessment and recommendations

use fingerprint_ml::{
    AdvancedAnomalyDetector, AnomalyClassification, AnomalyRecommendation, FingerprintVector,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Enhanced ML Anomaly Detection Demo ===\n");

    // Initialize the advanced anomaly detector
    let mut detector = AdvancedAnomalyDetector::new();
    println!("✓ Advanced anomaly detector initialized\n");

    // Test various fingerprint scenarios
    test_normal_traffic(&mut detector)?;
    test_suspicious_traffic(&mut detector)?;
    test_anomalous_traffic(&mut detector)?;
    test_critical_threat(&mut detector)?;

    // Demonstrate online learning capabilities
    demonstrate_online_learning(&mut detector)?;

    Ok(())
}

fn test_normal_traffic(detector: &mut AdvancedAnomalyDetector) -> Result<(), Box<dyn std::error::Error>> {
    println!("--- Testing Normal Traffic ---");
    
    // Simulate normal browser fingerprint
    let normal_fingerprint = FingerprintVector::new(
        vec![
            0.12, 0.15, 0.18, 0.14, 0.16,  // TLS handshake characteristics
            0.22, 0.25, 0.23, 0.24, 0.21,  // HTTP header patterns
            0.08, 0.11, 0.09, 0.10, 0.12,  // Canvas rendering features
            0.31, 0.33, 0.30, 0.32, 0.29,  // WebGL capabilities
        ],
        Some("Chrome_120_Normal".to_string()),
        0.95,
    );

    let result = detector.detect_anomalies(&normal_fingerprint);
    
    println!("Fingerprint: Chrome 120 (Normal traffic)");
    println!("Anomaly Score: {:.3}", result.anomaly_score);
    println!("Confidence: {:.1}%", result.confidence * 100.0);
    println!("Classification: {:?}", result.classification);
    println!("Explanation: {}", result.explanation);
    println!("Recommendations:");
    for rec in &result.recommendations {
        println!("  • {:?}", rec);
    }
    
    assert_eq!(result.classification, AnomalyClassification::Normal);
    assert!(result.anomaly_score < 0.3);
    println!("✓ Normal traffic correctly identified\n");

    Ok(())
}

fn test_suspicious_traffic(detector: &mut AdvancedAnomalyDetector) -> Result<(), Box<dyn std::error::Error>> {
    println!("--- Testing Suspicious Traffic ---");
    
    // Simulate suspicious fingerprint with mixed characteristics
    let suspicious_fingerprint = FingerprintVector::new(
        vec![
            0.45, 0.48, 0.52, 0.47, 0.50,  // Unusual TLS patterns
            0.62, 0.65, 0.63, 0.64, 0.61,  // Modified HTTP headers
            0.38, 0.41, 0.39, 0.40, 0.42,  // Altered canvas features
            0.71, 0.73, 0.70, 0.72, 0.69,  // Suspicious WebGL usage
        ],
        Some("Modified_Browser_Suspicious".to_string()),
        0.75,
    );

    let result = detector.detect_anomalies(&suspicious_fingerprint);
    
    println!("Fingerprint: Modified Browser (Suspicious)");
    println!("Anomaly Score: {:.3}", result.anomaly_score);
    println!("Confidence: {:.1}%", result.confidence * 100.0);
    println!("Classification: {:?}", result.classification);
    println!("Individual Algorithm Scores:");
    for (algo, score) in &result.algorithm_scores {
        println!("  {}: {:.3}", algo, score);
    }
    println!("Explanation: {}", result.explanation);
    println!("Recommendations:");
    for rec in &result.recommendations {
        println!("  • {:?}", rec);
    }
    
    assert_eq!(result.classification, AnomalyClassification::Suspicious);
    assert!(result.anomaly_score >= 0.3 && result.anomaly_score < 0.6);
    println!("✓ Suspicious traffic correctly identified\n");

    Ok(())
}

fn test_anomalous_traffic(detector: &mut AdvancedAnomalyDetector) -> Result<(), Box<dyn std::error::Error>> {
    println!("--- Testing Anomalous Traffic ---");
    
    // Simulate clearly anomalous fingerprint
    let anomalous_fingerprint = FingerprintVector::new(
        vec![
            0.75, 0.78, 0.82, 0.77, 0.80,  // Highly unusual TLS
            0.82, 0.85, 0.83, 0.84, 0.81,  // Suspicious HTTP patterns
            0.68, 0.71, 0.69, 0.70, 0.72,  // Artificial canvas signatures
            0.91, 0.93, 0.90, 0.92, 0.89,  // Obvious WebGL manipulation
        ],
        Some("Automated_Tool_Anomalous".to_string()),
        0.45,
    );

    let result = detector.detect_anomalies(&anomalous_fingerprint);
    
    println!("Fingerprint: Automated Tool (Anomalous)");
    println!("Anomaly Score: {:.3}", result.anomaly_score);
    println!("Confidence: {:.1}%", result.confidence * 100.0);
    println!("Classification: {:?}", result.classification);
    println!("Explanation: {}", result.explanation);
    println!("Recommendations:");
    for rec in &result.recommendations {
        println!("  • {:?}", rec);
    }
    
    assert_eq!(result.classification, AnomalyClassification::Anomalous);
    assert!(result.anomaly_score >= 0.6 && result.anomaly_score < 0.8);
    println!("✓ Anomalous traffic correctly identified\n");

    Ok(())
}

fn test_critical_threat(detector: &mut AdvancedAnomalyDetector) -> Result<(), Box<dyn std::error::Error>> {
    println!("--- Testing Critical Threat ---");
    
    // Simulate critical security threat
    let critical_fingerprint = FingerprintVector::new(
        vec![
            0.95, 0.98, 0.92, 0.97, 0.90,  // Malicious TLS signatures
            0.92, 0.95, 0.93, 0.94, 0.91,  // Attack-oriented HTTP
            0.88, 0.91, 0.89, 0.90, 0.92,  // Exploitation canvas techniques
            0.91, 0.93, 0.90, 0.92, 0.89,  // Weaponized WebGL exploitation
        ],
        Some("Malware_Critical".to_string()),
        0.25,
    );

    let result = detector.detect_anomalies(&critical_fingerprint);
    
    println!("Fingerprint: Malware Attack (Critical)");
    println!("Anomaly Score: {:.3}", result.anomaly_score);
    println!("Confidence: {:.1}%", result.confidence * 100.0);
    println!("Classification: {:?}", result.classification);
    println!("Explanation: {}", result.explanation);
    println!("Recommendations:");
    for rec in &result.recommendations {
        println!("  • {:?}", rec);
    }
    
    assert_eq!(result.classification, AnomalyClassification::Critical);
    assert!(result.anomaly_score >= 0.8);
    println!("✓ Critical threat correctly identified\n");

    Ok(())
}

fn demonstrate_online_learning(detector: &mut AdvancedAnomalyDetector) -> Result<(), Box<dyn std::error::Error>> {
    println!("--- Demonstrating Online Learning ---");
    
    // Simulate continuous traffic to show adaptation
    let normal_patterns = [
        vec![0.10, 0.13, 0.16, 0.12, 0.14, 0.20, 0.23, 0.21, 0.22, 0.19],
        vec![0.11, 0.14, 0.17, 0.13, 0.15, 0.21, 0.24, 0.22, 0.23, 0.20],
        vec![0.09, 0.12, 0.15, 0.11, 0.13, 0.19, 0.22, 0.20, 0.21, 0.18],
    ];

    let anomalous_patterns = [
        vec![0.85, 0.88, 0.92, 0.87, 0.90, 0.82, 0.85, 0.83, 0.84, 0.81],
        vec![0.87, 0.90, 0.94, 0.89, 0.92, 0.84, 0.87, 0.85, 0.86, 0.83],
        vec![0.83, 0.86, 0.90, 0.85, 0.88, 0.80, 0.83, 0.81, 0.82, 0.79],
    ];

    println!("Training detector with normal patterns...");
    for (i, pattern) in normal_patterns.iter().enumerate() {
        let fingerprint = FingerprintVector::new(
            pattern.clone(),
            Some(format!("Normal_Training_{}", i)),
            0.9,
        );
        let result = detector.detect_anomalies(&fingerprint);
        println!("Normal sample {}: score = {:.3}, classification = {:?}", 
                 i + 1, result.anomaly_score, result.classification);
    }

    println!("\nTraining detector with anomalous patterns...");
    for (i, pattern) in anomalous_patterns.iter().enumerate() {
        let fingerprint = FingerprintVector::new(
            pattern.clone(),
            Some(format!("Anomalous_Training_{}", i)),
            0.4,
        );
        let result = detector.detect_anomalies(&fingerprint);
        println!("Anomalous sample {}: score = {:.3}, classification = {:?}", 
                 i + 1, result.anomaly_score, result.classification);
    }

    // Test adaptation with a borderline case
    println!("\nTesting adaptation with borderline case...");
    let borderline_fingerprint = FingerprintVector::new(
        vec![0.65, 0.68, 0.72, 0.67, 0.70, 0.52, 0.55, 0.53, 0.54, 0.51],
        Some("Borderline_Case".to_string()),
        0.6,
    );
    
    let result = detector.detect_anomalies(&borderline_fingerprint);
    println!("Borderline case score: {:.3}", result.anomaly_score);
    println!("Classification: {:?}", result.classification);
    println!("Confidence: {:.1}%", result.confidence * 100.0);
    
    println!("✓ Online learning demonstrated successfully\n");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_complete_detection_workflow() {
        let mut detector = AdvancedAnomalyDetector::new();
        
        // Test full workflow
        test_normal_traffic(&mut detector).unwrap();
        test_suspicious_traffic(&mut detector).unwrap();
        test_anomalous_traffic(&mut detector).unwrap();
        test_critical_threat(&mut detector).unwrap();
    }

    #[test]
    fn test_algorithm_consistency() {
        let mut detector = AdvancedAnomalyDetector::new();
        
        // Same input should produce consistent results
        let test_fingerprint = FingerprintVector::new(
            vec![0.5, 0.6, 0.55, 0.58, 0.52, 0.61, 0.59, 0.57, 0.60, 0.54],
            Some("Consistency_Test".to_string()),
            0.8,
        );
        
        let result1 = detector.detect_anomalies(&test_fingerprint);
        let result2 = detector.detect_anomalies(&test_fingerprint);
        
        // Results should be similar (allowing for small variations in online learning)
        assert!((result1.anomaly_score - result2.anomaly_score).abs() < 0.1);
        assert_eq!(result1.classification, result2.classification);
    }

    #[test]
    fn test_recommendation_logic() {
        let mut detector = AdvancedAnomalyDetector::new();
        
        // Test that critical anomalies get appropriate recommendations
        let critical_fingerprint = FingerprintVector::new(
            vec![0.95; 20],
            Some("Critical_Test".to_string()),
            0.1,
        );
        
        let result = detector.detect_anomalies(&critical_fingerprint);
        
        // Critical anomalies should trigger security alerts and blocking
        assert!(result.recommendations.contains(&AnomalyRecommendation::SecurityAlert));
        assert!(result.recommendations.contains(&AnomalyRecommendation::TemporaryBlock));
        assert!(result.recommendations.contains(&AnomalyRecommendation::EnhancedAuthRequired));
    }
}