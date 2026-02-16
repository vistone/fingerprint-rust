//! Real Image Analysis Example
//!
//! This example demonstrates how to analyze real image files to detect AI-generated content.
//!
//! Usage:
//! ```bash
//! cargo run --example analyze_real_image <image_path>
//! ```
use fingerprint_ai_models::real_detection::RealImageAnalyzer;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("📖 Real Image Analysis Tool");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!();
        println!("Usage: {} <image_path>", args[0]);
        println!();
        println!("Example:");
        println!("  {} photo.jpg", args[0]);
        println!("  {} ai_generated.png", args[0]);
        println!();
        println!("Supported formats: PNG, JPEG, WebP, BMP, etc.");
        return;
    }

    let image_path = &args[1];

    println!("📊 Real Image Analysis: {}", image_path);
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();

    match RealImageAnalyzer::analyze_file(image_path) {
        Ok(result) => {
            println!("🎨 Image Properties:");
            println!("  • Format: {}", result.format);
            println!(
                "  • Dimensions: {}×{}",
                result.dimensions.0, result.dimensions.1
            );
            println!("  • Size: {:.1} KB", result.file_size as f64 / 1024.0);
            println!();

            println!("🔍 Detection Results:");
            if result.is_likely_ai {
                println!("  AI-Generated: ✓ YES");
            } else {
                println!("  AI-Generated: ✗ NO (likely human/real photo)");
            }
            println!("  Confidence: {:.1}%", result.confidence * 100.0);
            println!();

            println!("📈 Analysis Metrics:");
            print_metric(
                "Noise uniformity",
                result.noise_uniformity,
                "Very AI-like",
                "Somewhat AI-like",
                "Natural variation",
            );
            print_metric(
                "Frequency artifacts",
                result.frequency_artifacts,
                "High (GAN-like)",
                "Medium",
                "Low (natural)",
            );
            print_metric(
                "Color uniformity",
                result.color_uniformity,
                "Over-smooth",
                "Moderate",
                "Natural distribution",
            );
            print_metric(
                "Texture uniformity",
                result.texture_uniformity,
                "Synthetic",
                "Somewhat uniform",
                "Natural variation",
            );
            print_metric(
                "EXIF indicators",
                result.exif_indicators,
                "Missing camera data",
                "Some indicators",
                "Full camera data",
            );
            println!();

            if !result.model_attribution.is_empty() {
                println!("🤖 Model Attribution:");
                let mut attrs: Vec<_> = result.model_attribution.iter().collect();
                attrs.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap());

                for (model, prob) in attrs {
                    let bar_length = (prob * 30.0) as usize;
                    let bar = "█".repeat(bar_length);
                    println!("  • {:20} [{:5.1}%] {}", model, prob * 100.0, bar);
                }
            }
            println!();

            // Summary
            if result.is_likely_ai {
                println!("💡 Summary:");
                println!("   This image shows characteristics typical of AI-generated content.");
                if result.confidence > 0.8 {
                    println!("   High confidence - multiple strong AI indicators detected.");
                } else if result.confidence > 0.6 {
                    println!("   Moderate confidence - several AI indicators present.");
                } else {
                    println!("   Low confidence - only weak AI indicators detected.");
                }
            } else {
                println!("💡 Summary:");
                println!("   This image appears to be a real photo or human-created content.");
                if result.confidence < 0.4 {
                    println!("   Strong indicators of natural photography or human creation.");
                } else {
                    println!("   Some characteristics suggest possible editing or processing.");
                }
            }
        }
        Err(e) => {
            eprintln!("❌ Error analyzing image: {}", e);
            eprintln!();
            eprintln!("Make sure the file exists and is a valid image format.");
            std::process::exit(1);
        }
    }
}

fn print_metric(name: &str, value: f64, high_desc: &str, mid_desc: &str, low_desc: &str) {
    let desc = if value > 0.7 {
        high_desc
    } else if value > 0.4 {
        mid_desc
    } else {
        low_desc
    };

    println!("  • {:22} {:.3} ({})", format!("{}:", name), value, desc);
}
