//! Simple demo of enhanced ML anomaly detection capabilities

use fingerprint_ml::{AdvancedAnomalyDetector, FingerprintVector};

fn main() {
    println!("=== Enhanced ML Anomaly Detection Demo ===\n");

    // Initialize detector
    let detector = AdvancedAnomalyDetector::new();
    println!("✓ Advanced anomaly detector initialized\n");

    // Test normal traffic
    println!("--- Testing Normal Browser Traffic ---");
    let normal_fp = FingerprintVector::new(
        vec![0.12, 0.15, 0.18, 0.14, 0.16, 0.22, 0.25, 0.23],
        Some("Chrome_120_Normal".to_string()),
        0.95,
    );

    let normal_result = detector.detect_anomalies(&normal_fp);
    println!("Normal fingerprint analysis:");
    println!("  Anomaly Score: {:.3}", normal_result.anomaly_score);
    println!("  Classification: {:?}", normal_result.classification);
    println!("  Confidence: {:.1}%", normal_result.confidence * 100.0);
    println!();

    // Test suspicious traffic
    println!("--- Testing Suspicious Traffic ---");
    let suspicious_fp = FingerprintVector::new(
        vec![0.45, 0.48, 0.52, 0.47, 0.50, 0.62, 0.65, 0.63],
        Some("Modified_Browser_Suspicious".to_string()),
        0.75,
    );

    let suspicious_result = detector.detect_anomalies(&suspicious_fp);
    println!("Suspicious fingerprint analysis:");
    println!("  Anomaly Score: {:.3}", suspicious_result.anomaly_score);
    println!("  Classification: {:?}", suspicious_result.classification);
    println!("  Confidence: {:.1}%", suspicious_result.confidence * 100.0);
    println!();

    // Test anomalous traffic
    println!("--- Testing Anomalous Traffic ---");
    let anomalous_fp = FingerprintVector::new(
        vec![0.75, 0.78, 0.82, 0.77, 0.80, 0.82, 0.85, 0.83],
        Some("Automated_Tool_Anomalous".to_string()),
        0.45,
    );

    let anomalous_result = detector.detect_anomalies(&anomalous_fp);
    println!("Anomalous fingerprint analysis:");
    println!("  Anomaly Score: {:.3}", anomalous_result.anomaly_score);
    println!("  Classification: {:?}", anomalous_result.classification);
    println!("  Confidence: {:.1}%", anomalous_result.confidence * 100.0);
    println!();

    println!("\n✓ Demo completed successfully!");
}
