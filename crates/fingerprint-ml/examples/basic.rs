//! Simple example demonstrating the enhanced ML anomaly detection

use fingerprint_ml::{AdvancedAnomalyDetector, FingerprintVector};

fn main() {
    println!("=== Enhanced ML Anomaly Detection Demo ===\n");

    let detector = AdvancedAnomalyDetector::new();

    // Test normal fingerprint
    let normal_fp = FingerprintVector::new(
        vec![0.12, 0.15, 0.13, 0.17, 0.14],
        Some("Chrome_Normal".to_string()),
        0.95,
    );

    let normal_result = detector.detect_anomalies(&normal_fp);
    println!("Normal fingerprint:");
    println!("  Score: {:.3}", normal_result.anomaly_score);
    println!("  Classification: {:?}", normal_result.classification);

    // Test anomalous fingerprint
    let anomalous_fp = FingerprintVector::new(
        vec![0.82, 0.85, 0.83, 0.87, 0.84],
        Some("Suspicious_Tool".to_string()),
        0.45,
    );

    let anomalous_result = detector.detect_anomalies(&anomalous_fp);
    println!("\nAnomalous fingerprint:");
    println!("  Score: {:.3}", anomalous_result.anomaly_score);
    println!("  Classification: {:?}", anomalous_result.classification);

    println!("\n✓ Demo completed successfully!");
}
