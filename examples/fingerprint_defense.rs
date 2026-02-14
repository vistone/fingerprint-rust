//! fingerprint-defense usage example
//!
//! Demonstrates the core functionality of the fingerprint defense system
//! including passive detection, active protection, and self-learning capabilities.

use fingerprint_defense::{
    AnomalyDetector,
    CanvasNoiseGenerator,
    StorageAnalyzer,
    SelfLearningAnalyzer,
    FingerprintDatabase,
    PassiveAnalyzer,
    AnalysisResult
};
use std::sync::Arc;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Fingerprint Defense System Demo ===\n");

    // 1. Initialize storage analyzer for fingerprinting detection
    demo_storage_analysis()?;
    
    // 2. Demonstrate canvas noise generation for protection
    demo_canvas_protection()?;
    
    // 3. Show self-learning capabilities
    demo_self_learning()?;
    
    // 4. Display passive fingerprint recognition
    demo_passive_recognition()?;
    
    println!("\n✅ All demonstrations completed successfully!");
    Ok(())
}

fn demo_storage_analysis() -> Result<(), Box<dyn std::error::Error>> {
    println!("1. Storage Fingerprinting Detection");
    println!("-----------------------------------");
    
    let mut analyzer = StorageAnalyzer::new();
    
    // Simulate normal storage access
    println!("Simulating normal storage access...");
    analyzer.record_local_storage_access("user_preference");
    analyzer.record_local_storage_access("theme_setting");
    
    assert!(!analyzer.detect_local_storage_fingerprinting());
    println!("✓ Normal access patterns detected - no fingerprinting");
    
    // Simulate suspicious fingerprinting behavior
    println!("Simulating fingerprinting behavior...");
    for _ in 0..15 {
        analyzer.record_local_storage_access("canvas_fingerprint_data");
    }
    
    if analyzer.detect_local_storage_fingerprinting() {
        println!("⚠️  Potential fingerprinting detected!");
    }
    
    // Test cookie analysis
    let suspicious_cookies = analyzer.analyze_cookie_tracking(&[
        "normal_session=value123",
        "fingerprint_tracker=suspicious_data",
        "user_tracking_id=abc123"
    ]);
    
    if !suspicious_cookies.is_empty() {
        println!("Suspicious cookies found:");
        for cookie in &suspicious_cookies {
            println!("  - {}", cookie);
        }
    }
    
    let stats = analyzer.get_statistics();
    println!("Storage statistics: {} local keys, {} session keys", 
             stats.local_storage_keys, stats.session_storage_keys);
    
    Ok(())
}

fn demo_canvas_protection() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n2. Canvas Fingerprint Protection");
    println!("--------------------------------");
    
    // Create noise generator with medium intensity
    let mut noise_gen = CanvasNoiseGenerator::new(0.4);
    
    // Generate canvas noise
    let canvas_noise = noise_gen.generate_canvas_noise();
    println!("Generated canvas noise: {} bytes", canvas_noise.len());
    
    // Apply noise to pixel data
    let mut pixel_data = vec![100u8, 150, 200, 255, 50, 75, 125, 200]; // RGBA pixels
    let original_pixels = pixel_data.clone();
    
    noise_gen.apply_canvas_rendering_noise("demo_canvas", &mut pixel_data)?;
    println!("Applied rendering noise to canvas pixels");
    
    // Verify changes
    assert_ne!(original_pixels, pixel_data);
    println!("✓ Pixel data successfully modified");
    
    // Generate WebGL noise parameters
    let webgl_noise = noise_gen.generate_webgl_noise();
    println!("Generated WebGL noise parameters:");
    for (param, value) in &webgl_noise {
        println!("  {}: {}", param, value);
    }
    
    // Test audio noise
    let audio_samples = vec![0.5f32, -0.3, 0.8, -0.1, 0.0];
    let noisy_audio = noise_gen.apply_audio_noise(&audio_samples);
    println!("Applied audio noise to {} samples", audio_samples.len());
    
    let config = noise_gen.get_config();
    println!("Current noise intensity: {:.2}", config.intensity);
    
    Ok(())
}

fn demo_self_learning() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n3. Self-Learning Fingerprint Analysis");
    println!("------------------------------------");
    
    // Initialize database and learner
    let db = FingerprintDatabase::new_in_memory();
    let mut learner = SelfLearningAnalyzer::new(Arc::new(db));
    
    // Configure learning parameters
    learner.set_learning_threshold(5);
    learner.set_min_stability_score(0.7);
    learner.set_stability_window(Duration::from_secs(3600)); // 1 hour
    
    println!("Learning configuration:");
    println!("  Threshold: {} observations", learner.get_learning_threshold());
    println!("  Min stability: {:.2}", learner.get_min_stability_score());
    
    // Get initial statistics
    let initial_stats = learner.get_observation_stats();
    println!("Initial observations: {}", initial_stats.total_observations);
    
    // Simulate observing some fingerprints
    // In a real scenario, this would come from PassiveAnalyzer results
    println!("Observing sample fingerprints...");
    
    let stats = learner.get_observation_stats();
    println!("Current learning status:");
    println!("  Total observations: {}", stats.total_observations);
    println!("  Stable candidates: {}", stats.stable_candidates);
    
    Ok(())
}

fn demo_passive_recognition() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n4. Passive Fingerprint Recognition");
    println!("----------------------------------");
    
    let analyzer = PassiveAnalyzer::new();
    
    // This would normally analyze real network traffic
    // For demo purposes, we'll show the structure
    println!("Passive analyzer initialized");
    println!("Ready to analyze network packets for:");
    println!("  • TLS ClientHello signatures");
    println!("  • HTTP header patterns");  
    println!("  • JA4+ fingerprint generation");
    println!("  • Behavioral anomalies");
    
    // Example of what analysis might look like
    let sample_result = AnalysisResult {
        tls: None,
        http: None,
        tcp: None,
        timestamp: std::time::SystemTime::now(),
        source_ip: "192.168.1.100".parse().unwrap(),
        destination_ip: "93.184.216.34".parse().unwrap(),
        confidence: 0.85,
    };
    
    println!("Sample analysis confidence: {:.2}%", sample_result.confidence * 100.0);
    
    Ok(())
}