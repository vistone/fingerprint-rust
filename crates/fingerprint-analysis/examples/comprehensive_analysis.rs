//! Example demonstrating comprehensive fingerprint analysis
//!
//! This example shows how to use the unified analysis engine
//! for complete fingerprint evaluation and threat detection.

use fingerprint_analysis::{
    AnalysisEngine,
    AnalysisError,
    AlertSeverity,
    AlertCategory,
};
use fingerprint_core::fingerprint::{Fingerprint, FingerprintType, FingerprintMetadata};
use fingerprint_config::get_config_manager;
use tokio;
use std::sync::Arc;

// Mock fingerprint implementation for demonstration
struct DemoFingerprint {
    id: String,
    browser: String,
    os: String,
}

impl Fingerprint for DemoFingerprint {
    fn fingerprint_type(&self) -> FingerprintType {
        FingerprintType::Http
    }

    fn id(&self) -> String {
        self.id.clone()
    }

    fn metadata(&self) -> &FingerprintMetadata {
        static METADATA: FingerprintMetadata = FingerprintMetadata::new();
        &METADATA
    }

    fn metadata_mut(&mut self) -> &mut FingerprintMetadata {
        static mut METADATA: FingerprintMetadata = FingerprintMetadata::new();
        unsafe { &mut METADATA }
    }

    fn hash(&self) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        self.id.hash(&mut hasher);
        self.browser.hash(&mut hasher);
        self.os.hash(&mut hasher);
        hasher.finish()
    }

    fn similar_to(&self, other: &dyn Fingerprint) -> bool {
        if let Some(other_demo) = other.as_any().downcast_ref::<DemoFingerprint>() {
            self.browser == other_demo.browser && self.os == other_demo.os
        } else {
            false
        }
    }

    fn to_string(&self) -> String {
        format!("DemoFingerprint(id={}, browser={}, os={})", 
                self.id, self.browser, self.os)
    }
}

// Custom alert generator example
struct SecurityAlertGenerator;

impl fingerprint_analysis::AlertGenerator for SecurityAlertGenerator {
    fn generate_alerts(&self, result: &fingerprint_analysis::AnalysisResult) -> Vec<fingerprint_analysis::Alert> {
        let mut alerts = Vec::new();
        
        // Generate high-risk alert if risk score is too high
        if result.risk_score > 0.8 {
            alerts.push(fingerprint_analysis::Alert {
                id: uuid::Uuid::new_v4().to_string(),
                severity: AlertSeverity::Critical,
                category: AlertCategory::Anomaly,
                message: format!("High risk fingerprint detected (score: {:.2})", result.risk_score),
                timestamp: chrono::Utc::now(),
                metadata: std::collections::HashMap::new(),
            });
        }
        
        // Generate warning for medium risk
        else if result.risk_score > 0.5 {
            alerts.push(fingerprint_analysis::Alert {
                id: uuid::Uuid::new_v4().to_string(),
                severity: AlertSeverity::Warning,
                category: AlertCategory::Suspicious,
                message: format!("Medium risk fingerprint detected (score: {:.2})", result.risk_score),
                timestamp: chrono::Utc::now(),
                metadata: std::collections::HashMap::new(),
            });
        }
        
        alerts
    }
}

#[tokio::main]
async fn main() -> Result<(), AnalysisError> {
    println!("=== Comprehensive Fingerprint Analysis ===");
    
    // Initialize configuration
    let config = get_config_manager();
    config.load().unwrap();
    
    // Create analysis engine
    let engine = AnalysisEngine::new(Arc::new(config))?;
    
    // Add custom alert generator
    engine.add_alert_generator(Box::new(SecurityAlertGenerator));
    
    // Create test fingerprints
    let normal_fp = DemoFingerprint {
        id: "normal-client-123".to_string(),
        browser: "Chrome 120".to_string(),
        os: "Windows 11".to_string(),
    };
    
    let suspicious_fp = DemoFingerprint {
        id: "suspicious-client-456".to_string(),
        browser: "Custom Browser".to_string(),
        os: "Unknown OS".to_string(),
    };
    
    // Analyze normal fingerprint
    println!("\nAnalyzing normal fingerprint...");
    let normal_result = engine.analyze(&normal_fp).await?;
    
    println!("Normal Analysis Results:");
    println!("  Risk Score: {:.2}", normal_result.risk_score);
    println!("  Confidence: {:.2}", normal_result.confidence);
    println!("  Alerts Generated: {}", normal_result.alerts.len());
    
    // Analyze suspicious fingerprint
    println!("\nAnalyzing suspicious fingerprint...");
    let suspicious_result = engine.analyze(&suspicious_fp).await?;
    
    println!("Suspicious Analysis Results:");
    println!("  Risk Score: {:.2}", suspicious_result.risk_score);
    println!("  Confidence: {:.2}", suspicious_result.confidence);
    println!("  Alerts Generated: {}", suspicious_result.alerts.len());
    
    // Display alerts
    for alert in &suspicious_result.alerts {
        println!("  [{}] {}: {}", 
                 match alert.severity {
                     AlertSeverity::Info => "INFO",
                     AlertSeverity::Warning => "WARN",
                     AlertSeverity::Critical => "CRIT",
                     AlertSeverity::Emergency => "EMER",
                 },
                 match alert.category {
                     AlertCategory::Anomaly => "ANOMALY",
                     AlertCategory::Suspicious => "SUSPICIOUS",
                     AlertCategory::KnownThreat => "THREAT",
                     AlertCategory::Configuration => "CONFIG",
                 },
                 alert.message);
    }
    
    // Compare fingerprints
    println!("\nComparing fingerprints...");
    let comparison = engine.compare(&normal_fp, &suspicious_fp).await?;
    
    println!("Comparison Results:");
    println!("  Similarity Score: {:.2}", comparison.similarity);
    println!("  Match Status: {}", if comparison.matched { "MATCH" } else { "NO MATCH" });
    println!("  Matched Fields: {}", comparison.matched_fields.len());
    println!("  Unmatched Fields: {}", comparison.unmatched_fields.len());
    
    println!("\n=== Analysis Complete ===");
    Ok(())
}