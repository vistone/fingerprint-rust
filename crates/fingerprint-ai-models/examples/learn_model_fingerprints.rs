//! Model Fingerprint Learning Example
//!
//! This example demonstrates how to learn and store model fingerprints from samples.

use fingerprint_ai_models::model_fingerprints::{FingerprintLearner, ModelFingerprintDatabase};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("📚 Model Fingerprint Learning System");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    // Create learner
    let mut learner = FingerprintLearner::new();

    println!("🔬 Processing Samples...");

    // Add text samples for different models

    // GPT-4 sample
    let gpt4_text = "Artificial intelligence has revolutionized numerous sectors. It's important to note that machine learning algorithms can delve into complex patterns. Furthermore, neural networks provide unprecedented capabilities. Moreover, the field continues to advance rapidly.";
    learner.add_text_sample(gpt4_text, "gpt4");
    println!("  ✓ Added text sample for model: gpt4");

    // Claude 3 sample
    let claude_text = "The development of artificial intelligence represents a significant milestone in technological advancement. Modern machine learning systems demonstrate remarkable capabilities across various domains. Consequently, researchers continue to explore novel architectures and methodologies.";
    learner.add_text_sample(claude_text, "claude3");
    println!("  ✓ Added text sample for model: claude3");

    // Gemini sample
    let gemini_text = "AI transforms industries. Machine learning detects patterns efficiently. Neural networks process data rapidly. Technology evolves constantly.";
    learner.add_text_sample(gemini_text, "gemini");
    println!("  ✓ Added text sample for model: gemini");

    // Chinese model - Qwen
    let qwen_text = "人工智能正在改变世界。机器学习算法能够识别复杂的模式。深度学习技术不断进步。这些技术为各行业带来革新。";
    learner.add_text_sample(qwen_text, "qwen");
    println!("  ✓ Added text sample for model: qwen");

    // Image samples
    learner.add_image_sample(1024, 1024, "PNG", "stable-diffusion");
    println!("  ✓ Added image sample for model: stable-diffusion");

    learner.add_image_sample(1024, 1024, "PNG", "midjourney");
    println!("  ✓ Added image sample for model: midjourney");

    learner.add_image_sample(1024, 1024, "PNG", "dall-e");
    println!("  ✓ Added image sample for model: dall-e");

    println!();

    // Learn fingerprints
    let fingerprints = learner.learn_fingerprints();

    println!("📊 Learning Statistics:");
    println!(
        "  Total samples: {}",
        fingerprints.iter().map(|fp| fp.sample_count).sum::<usize>()
    );
    println!("  Models identified: {}", fingerprints.len());
    println!();

    println!("🔍 Learned Fingerprints:\n");

    for fp in &fingerprints {
        let icon = if fp.model_type == "text" {
            "📝"
        } else {
            "🎨"
        };
        println!("  {} Model: {} ({})", icon, fp.model_name, fp.model_type);
        println!("  ├─ Samples: {}", fp.sample_count);
        println!("  ├─ Statistical Signature:");

        if fp.model_type == "text" {
            println!(
                "  │  ├─ Perplexity: {:.3} ± {:.3}",
                fp.statistical_signature.perplexity_mean, fp.statistical_signature.perplexity_std
            );
            println!(
                "  │  ├─ Burstiness: {:.3} ± {:.3}",
                fp.statistical_signature.burstiness_mean, fp.statistical_signature.burstiness_std
            );
            println!(
                "  │  └─ Vocabulary: {:.3} ± {:.3}",
                fp.statistical_signature.vocabulary_richness_mean,
                fp.statistical_signature.vocabulary_richness_std
            );
        } else {
            println!(
                "  │  ├─ Noise patterns: {:.3} ± {:.3}",
                fp.statistical_signature.noise_pattern_mean,
                fp.statistical_signature.noise_pattern_std
            );
            println!(
                "  │  ├─ Texture regularity: {:.3} ± {:.3}",
                fp.statistical_signature.texture_regularity_mean,
                fp.statistical_signature.texture_regularity_std
            );
            println!(
                "  │  └─ Color distribution: {:.3} ± {:.3}",
                fp.statistical_signature.color_distribution_mean,
                fp.statistical_signature.color_distribution_std
            );
        }

        println!(
            "  └─ Characteristic Patterns: {}",
            fp.characteristic_patterns.len()
        );
        println!();
    }

    // Create database and add fingerprints
    let mut db = ModelFingerprintDatabase::new();
    for fp in fingerprints {
        db.add_fingerprint(fp);
    }

    // Save to file
    let filename = "fingerprints.json";
    db.save_to_file(filename)?;

    println!("💾 Database Operations:");
    println!("  ✓ Saved fingerprint database to: {}", filename);
    println!("  ✓ Total fingerprints: {}", db.fingerprints.len());
    println!("  ✓ Models by type:");
    println!(
        "     - text: {} models",
        db.get_models_by_type("text").len()
    );
    println!(
        "     - image: {} models",
        db.get_models_by_type("image").len()
    );
    println!();

    // Demonstrate fingerprint matching
    println!("🎯 Fingerprint Matching Example:");
    println!("  Testing against learned database...");

    // Load the database we just saved
    let loaded_db = ModelFingerprintDatabase::load_from_file(filename)?;

    // Get a fingerprint to test with
    if let Some(test_fp) = loaded_db.get_fingerprint("gpt4") {
        let matches = loaded_db.match_fingerprint(&test_fp.statistical_signature, "text");
        if let Some((best_model, similarity)) = matches.first() {
            println!(
                "  ✓ Best match found: {} (similarity: {:.1}%)",
                best_model,
                similarity * 100.0
            );
        }
    }

    println!();
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("✅ Fingerprint learning complete!");
    println!("   Database ready for enhanced detection.");

    Ok(())
}
