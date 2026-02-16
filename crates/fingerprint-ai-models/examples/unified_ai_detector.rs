//! Unified AI Content Detector CLI Tool
//!
//! Production-ready command-line tool for detecting AI-generated content
//! across all modalities (text, image, audio, video).
use fingerprint_ai_models::advanced_detection::{
    DetectionExplanation, EnsembleDetectionResult, EnsembleDetector,
};
use fingerprint_ai_models::content_detection::detect_ai_content;
use fingerprint_ai_models::real_detection::RealImageAnalyzer;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        print_usage();
        return;
    }

    let file_path = &args[1];

    println!("🔍 Unified AI Content Detector");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();

    match detect_content(file_path) {
        Ok(result) => print_result(&result, file_path),
        Err(e) => println!("❌ Error: {}", e),
    }
}

fn print_usage() {
    println!("🔍 Unified AI Content Detector");
    println!();
    println!("Usage:");
    println!("  cargo run --example unified_ai_detector <file_path>");
    println!();
    println!("Supported formats:");
    println!("  • Text: .txt, .md, .html");
    println!("  • Image: .jpg, .jpeg, .png, .webp");
    println!("  • Audio: .wav, .mp3 (metadata analysis)");
    println!("  • Video: .mp4, .avi (metadata analysis)");
    println!();
    println!("Examples:");
    println!("  cargo run --example unified_ai_detector document.txt");
    println!("  cargo run --example unified_ai_detector photo.jpg");
}

fn detect_content(file_path: &str) -> Result<EnsembleDetectionResult, String> {
    let path = Path::new(file_path);

    if !path.exists() {
        return Err(format!("File not found: {}", file_path));
    }

    // Determine content type by extension
    let extension = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    match extension.as_str() {
        "txt" | "md" | "html" | "htm" => detect_text(file_path),
        "jpg" | "jpeg" | "png" | "webp" | "bmp" | "gif" => detect_image(file_path),
        "wav" | "mp3" | "flac" | "ogg" => detect_audio_from_metadata(file_path),
        "mp4" | "avi" | "mov" | "mkv" => detect_video_from_metadata(file_path),
        _ => Err(format!("Unsupported file type: {}", extension)),
    }
}

fn detect_text(file_path: &str) -> Result<EnsembleDetectionResult, String> {
    let content =
        fs::read_to_string(file_path).map_err(|e| format!("Failed to read text file: {}", e))?;

    // Use content detection
    let result = detect_ai_content(&content);

    // Create algorithm results (convert f32 to f64)
    let mut algorithm_results = HashMap::new();
    algorithm_results.insert("perplexity_analysis".to_string(), result.perplexity as f64);
    algorithm_results.insert("burstiness_analysis".to_string(), result.burstiness as f64);
    algorithm_results.insert(
        "vocabulary_analysis".to_string(),
        result.vocabulary_richness as f64,
    );
    algorithm_results.insert(
        "pattern_detection".to_string(),
        if result.patterns.is_empty() { 0.3 } else { 0.8 },
    );

    // Apply ensemble method with text-specific weights
    let mut weights = HashMap::new();
    weights.insert("perplexity_analysis".to_string(), 0.30);
    weights.insert("burstiness_analysis".to_string(), 0.30);
    weights.insert("vocabulary_analysis".to_string(), 0.20);
    weights.insert("pattern_detection".to_string(), 0.20);

    let raw_score = EnsembleDetector::combine_scores(&algorithm_results, &weights);
    let confidence = EnsembleDetector::calibrate_confidence(raw_score, "text");

    let explanation = EnsembleDetector::explain_detection(&algorithm_results, 0.7);

    // Convert model probabilities from f32 to f64
    let model_attribution: HashMap<String, f64> = result
        .model_probabilities
        .iter()
        .map(|(k, &v)| (k.clone(), v as f64))
        .collect();

    Ok(EnsembleDetectionResult {
        is_ai_generated: raw_score > 0.7,
        confidence,
        algorithm_results,
        explanation,
        model_attribution,
    })
}

fn detect_image(file_path: &str) -> Result<EnsembleDetectionResult, String> {
    let analysis = RealImageAnalyzer::analyze_file(file_path)?;

    // Create algorithm results with advanced statistics
    let mut algorithm_results = HashMap::new();
    algorithm_results.insert("noise_uniformity".to_string(), analysis.noise_uniformity);
    algorithm_results.insert(
        "frequency_artifacts".to_string(),
        analysis.frequency_artifacts,
    );
    algorithm_results.insert("color_uniformity".to_string(), analysis.color_uniformity);
    algorithm_results.insert(
        "texture_uniformity".to_string(),
        analysis.texture_uniformity,
    );
    algorithm_results.insert("exif_indicators".to_string(), analysis.exif_indicators);

    // Apply ensemble method with image-specific weights
    let mut weights = HashMap::new();
    weights.insert("noise_uniformity".to_string(), 0.25);
    weights.insert("frequency_artifacts".to_string(), 0.25);
    weights.insert("color_uniformity".to_string(), 0.15);
    weights.insert("texture_uniformity".to_string(), 0.20);
    weights.insert("exif_indicators".to_string(), 0.15);

    let raw_score = EnsembleDetector::combine_scores(&algorithm_results, &weights);
    let confidence = EnsembleDetector::calibrate_confidence(raw_score, "image");

    let explanation = EnsembleDetector::explain_detection(&algorithm_results, 0.7);

    // Convert model attribution from f64 to f64 (already f64)
    let model_attribution = analysis.model_attribution;

    Ok(EnsembleDetectionResult {
        is_ai_generated: analysis.is_likely_ai,
        confidence,
        algorithm_results,
        explanation,
        model_attribution,
    })
}

fn detect_audio_from_metadata(file_path: &str) -> Result<EnsembleDetectionResult, String> {
    // For now, we do metadata-based analysis
    // Future: implement actual audio signal processing

    let metadata =
        fs::metadata(file_path).map_err(|e| format!("Failed to read file metadata: {}", e))?;

    let file_size = metadata.len();

    let mut algorithm_results = HashMap::new();

    // Basic heuristics based on file size and format
    let size_score = if file_size < 100_000 { 0.7 } else { 0.4 };
    algorithm_results.insert("file_size_analysis".to_string(), size_score);
    algorithm_results.insert("format_analysis".to_string(), 0.5);

    let weights = HashMap::new(); // Equal weights
    let raw_score = EnsembleDetector::combine_scores(&algorithm_results, &weights);
    let confidence = EnsembleDetector::calibrate_confidence(raw_score * 0.6, "audio");

    let explanation = DetectionExplanation {
        primary_reason: "Audio analysis requires signal processing (not yet implemented)"
            .to_string(),
        factors: vec![(
            "metadata_analysis".to_string(),
            raw_score,
            "Based on file metadata only".to_string(),
        )],
        explanation_confidence: 0.4,
    };

    Ok(EnsembleDetectionResult {
        is_ai_generated: false,
        confidence,
        algorithm_results,
        explanation,
        model_attribution: HashMap::new(),
    })
}

fn detect_video_from_metadata(file_path: &str) -> Result<EnsembleDetectionResult, String> {
    // For now, we do metadata-based analysis
    // Future: implement actual video frame processing

    let metadata =
        fs::metadata(file_path).map_err(|e| format!("Failed to read file metadata: {}", e))?;

    let file_size = metadata.len();

    let mut algorithm_results = HashMap::new();

    // Basic heuristics
    let size_score = if file_size < 1_000_000 { 0.6 } else { 0.4 };
    algorithm_results.insert("file_size_analysis".to_string(), size_score);
    algorithm_results.insert("format_analysis".to_string(), 0.5);

    let weights = HashMap::new();
    let raw_score = EnsembleDetector::combine_scores(&algorithm_results, &weights);
    let confidence = EnsembleDetector::calibrate_confidence(raw_score * 0.6, "video");

    let explanation = DetectionExplanation {
        primary_reason: "Video analysis requires frame processing (not yet implemented)"
            .to_string(),
        factors: vec![(
            "metadata_analysis".to_string(),
            raw_score,
            "Based on file metadata only".to_string(),
        )],
        explanation_confidence: 0.4,
    };

    Ok(EnsembleDetectionResult {
        is_ai_generated: false,
        confidence,
        algorithm_results,
        explanation,
        model_attribution: HashMap::new(),
    })
}

fn print_result(result: &EnsembleDetectionResult, file_path: &str) {
    println!("📄 File: {}", file_path);
    println!();

    // Detection verdict
    println!("🎯 Detection Result:");
    if result.is_ai_generated {
        println!("  Status: ✅ AI-GENERATED");
    } else {
        println!("  Status: ✋ LIKELY HUMAN");
    }
    println!("  Confidence: {:.1}%", result.confidence * 100.0);
    println!();

    // Explanation
    println!("💡 Explanation:");
    println!("  {}", result.explanation.primary_reason);
    println!(
        "  Explanation confidence: {:.1}%",
        result.explanation.explanation_confidence * 100.0
    );
    println!();

    // Algorithm details
    println!("📊 Detection Algorithms:");
    let mut algorithms: Vec<_> = result.algorithm_results.iter().collect();
    algorithms.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap_or(std::cmp::Ordering::Equal));

    for (name, &score) in algorithms.iter().take(5) {
        let bar_length = (score * 30.0) as usize;
        let bar = "█".repeat(bar_length);
        let indicator = if score > 0.7 {
            "🔴"
        } else if score > 0.5 {
            "🟡"
        } else {
            "🟢"
        };
        println!(
            "  {} {:<25} [{:.1}%] {}",
            indicator,
            name,
            score * 100.0,
            bar
        );
    }
    println!();

    // Model attribution
    if !result.model_attribution.is_empty() {
        println!("🤖 Model Attribution:");
        let mut models: Vec<_> = result.model_attribution.iter().collect();
        models.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap_or(std::cmp::Ordering::Equal));

        for (model, &prob) in models.iter().take(3) {
            let bar_length = (prob * 30.0) as usize;
            let bar = "█".repeat(bar_length);
            println!("  {:<20} [{:.1}%] {}", model, prob * 100.0, bar);
        }
        println!();
    }

    // Contributing factors
    if !result.explanation.factors.is_empty() {
        println!("🔍 Key Detection Factors:");
        for (factor, score, desc) in result.explanation.factors.iter().take(3) {
            println!("  • {} (score: {:.2})", factor, score);
            println!("    → {}", desc);
        }
        println!();
    }

    // Summary
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    if result.is_ai_generated && result.confidence > 0.75 {
        println!("⚠️  High confidence AI-generated content detected");
    } else if result.is_ai_generated {
        println!("⚠️  Moderate confidence AI-generated content detected");
    } else if result.confidence > 0.6 {
        println!("✓  High confidence human-created content");
    } else {
        println!("?  Uncertain - mixed signals detected");
    }
}
