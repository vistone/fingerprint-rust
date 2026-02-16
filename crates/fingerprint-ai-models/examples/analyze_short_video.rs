//! AI-Generated Video Detection for Short-Form Platforms
//!
//! CLI tool to analyze videos and detect AI-generated content,
//! optimized for short-form platforms like TikTok, YouTube Shorts, etc.
//!
//! Usage:
//!   cargo run --example analyze_short_video
//!
//! Note: This example uses simulated frames. In production:
//! - Use ffmpeg to extract real frames from video files
//! - Process audio track separately
//! - Analyze full video duration

use fingerprint_ai_models::real_video_detection::RealVideoAnalyzer;
use image::{DynamicImage, ImageBuffer, Rgba};

fn main() {
    println!("🎬 AI-Generated Video Detection for Short-Form Platforms");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();

    // Example 1: AI-generated video (too uniform)
    println!("Example 1: AI-Generated Video (Synthetic Content)");
    println!("─────────────────────────────────────────────────────");
    
    let ai_frames = create_ai_generated_video_frames();
    let result1 = RealVideoAnalyzer::quick_analyze_short_form(&ai_frames, 30.0);
    
    display_result(&result1, "TikTok-style-ai-avatar.mp4");
    println!();

    // Example 2: Real video (natural variation)
    println!("Example 2: Real Video (Human-Created Content)");
    println!("─────────────────────────────────────────────────────");
    
    let real_frames = create_real_video_frames();
    let result2 = RealVideoAnalyzer::quick_analyze_short_form(&real_frames, 30.0);
    
    display_result(&result2, "youtube-shorts-real-vlog.mp4");
    println!();

    // Example 3: Deepfake detection
    println!("Example 3: Potential Deepfake Video");
    println!("─────────────────────────────────────────────────────");
    
    let deepfake_frames = create_deepfake_video_frames();
    let result3 = RealVideoAnalyzer::analyze_frames(&deepfake_frames, 30.0);
    
    display_result(&result3, "suspicious-face-swap.mp4");
    println!();

    // Summary
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("📊 Detection Summary");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();
    println!("✅ Successfully analyzed 3 video examples");
    println!();
    println!("🎯 Detection Capabilities:");
    println!("  • Temporal consistency analysis");
    println!("  • Motion pattern detection");
    println!("  • Boundary artifact identification");
    println!("  • Lighting consistency checking");
    println!("  • Face morphing detection");
    println!("  • Model attribution (Sora, Runway, Pika, Deepfake, etc.)");
    println!();
    println!("📱 Platform Optimization:");
    println!("  • Fast analysis for short-form content");
    println!("  • Optimized for TikTok/YouTube Shorts formats");
    println!("  • Batch processing support");
    println!("  • Real-time detection capable");
    println!();
    println!("💡 Production Deployment:");
    println!("  1. Extract frames using ffmpeg:");
    println!("     ffmpeg -i video.mp4 -vf fps=5 frame_%04d.png");
    println!("  2. Load frames and analyze:");
    println!("     let frames = load_frames_from_directory(\"frames/\");");
    println!("     let result = RealVideoAnalyzer::analyze_frames(&frames, 5.0);");
    println!("  3. Integrate with content moderation pipeline");
    println!();
}

fn display_result(result: &fingerprint_ai_models::video_detection::VideoFingerprint, filename: &str) {
    println!("📹 File: {}", filename);
    println!();
    
    // Detection status
    let status_icon = if result.is_ai_generated { "⚠️  AI-GENERATED" } else { "✅ REAL" };
    let status_color = if result.is_ai_generated { "🔴" } else { "🟢" };
    
    println!("{} Detection Result: {}", status_color, status_icon);
    println!("   Confidence: {:.1}%", result.confidence * 100.0);
    println!();

    // Video metadata
    println!("📊 Video Properties:");
    println!("   • Duration: {:.1}s", result.metadata.duration_seconds);
    println!("   • Frames analyzed: {}", result.metadata.frames_analyzed);
    if let Some(fps) = result.metadata.frame_rate {
        println!("   • Frame Rate: {:.0} fps", fps);
    }
    if let Some((w, h)) = result.metadata.resolution {
        let aspect = if h > w { "Vertical (TikTok/Stories)" } else if w > h { "Horizontal" } else { "Square" };
        println!("   • Resolution: {}×{} ({})", w, h, aspect);
    }
    println!();

    // Detection metrics
    println!("🔍 Analysis Metrics:");
    
    let temporal_status = if result.temporal_consistency < 0.5 { "⚠️  Suspicious" } else { "✓ Normal" };
    println!("   • Temporal consistency: {:.3} {}", result.temporal_consistency, temporal_status);
    
    let motion_status = if result.motion_consistency < 0.5 { "⚠️  Unnatural" } else { "✓ Natural" };
    println!("   • Motion consistency: {:.3} {}", result.motion_consistency, motion_status);
    
    let boundary_status = if result.boundary_artifacts > 0.5 { "⚠️  Detected" } else { "✓ None" };
    println!("   • Boundary artifacts: {:.3} {}", result.boundary_artifacts, boundary_status);
    
    if let Some(lip_sync) = result.lip_sync_quality {
        let sync_status = if lip_sync < 0.5 { "⚠️  Poor" } else { "✓ Good" };
        println!("   • Lip-sync quality: {:.3} {}", lip_sync, sync_status);
    }
    println!();

    // Model attribution
    if !result.model_probabilities.is_empty() {
        println!("🤖 Likely AI Model:");
        let mut probs: Vec<_> = result.model_probabilities.iter().collect();
        probs.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap());
        
        for (model, prob) in probs.iter().take(3) {
            let bar_length = (*prob * 30.0) as usize;
            let bar = "█".repeat(bar_length);
            println!("   {:15} [{:3.0}%] {}", model, *prob * 100.0, bar);
        }
        println!();
    }

    // Detected patterns
    if !result.patterns.is_empty() {
        println!("🔎 Detected Patterns ({}):", result.patterns.len());
        for (i, pattern) in result.patterns.iter().enumerate().take(5) {
            println!("   {}. {:?}", i + 1, pattern.pattern_type);
            println!("      → {}", pattern.description);
        }
        if result.patterns.len() > 5 {
            println!("   ... and {} more patterns", result.patterns.len() - 5);
        }
        println!();
    }

    // Recommendation
    if result.is_ai_generated {
        println!("⚠️  Recommendation:");
        println!("   This video shows characteristics of AI-generated content.");
        if result.confidence > 0.8 {
            println!("   High confidence - likely created by AI video generator.");
        } else if result.confidence > 0.6 {
            println!("   Moderate confidence - may contain AI-generated elements.");
        } else {
            println!("   Low confidence - manual review recommended.");
        }
        
        // Platform-specific guidance
        println!();
        println!("📱 For Content Moderation:");
        println!("   • Flag for review if required by platform policy");
        println!("   • Check for disclosure requirements (AI-generated label)");
        println!("   • Consider authenticity verification for sensitive content");
    } else {
        println!("✅ Recommendation:");
        println!("   This video appears to be human-created content.");
        println!("   No significant AI generation indicators detected.");
    }
}

/// Create simulated AI-generated video frames (too uniform)
fn create_ai_generated_video_frames() -> Vec<DynamicImage> {
    // AI-generated videos often have suspiciously uniform frames
    // Simulating an AI avatar video (Synthesia/HeyGen style)
    let mut frames = Vec::new();
    
    for i in 0..30 {
        // Very subtle changes (AI-like)
        let intensity = 128 + (i % 3) as u8;
        let img = ImageBuffer::from_fn(480, 854, |x, _y| {
            // Create a simple gradient with minimal variation
            let value = if x < 240 {
                intensity
            } else {
                intensity + 2
            };
            Rgba([value, value + 10, value + 20, 255])
        });
        frames.push(DynamicImage::ImageRgba8(img));
    }
    
    frames
}

/// Create simulated real video frames (natural variation)
fn create_real_video_frames() -> Vec<DynamicImage> {
    // Real videos have natural camera movement, lighting changes
    let mut frames = Vec::new();
    
    for i in 0..30 {
        // Natural variation in brightness and color
        let base = 100 + (i * 5) % 50;
        let variation = (i * 7) % 30;
        
        let img = ImageBuffer::from_fn(1080, 1920, |x, y| {
            // More complex patterns (real-like)
            let r = ((base + variation + (x % 50)) as u8).wrapping_add(y as u8 % 20);
            let g = ((base + variation + (y % 50)) as u8).wrapping_add(x as u8 % 20);
            let b = (base + variation + ((x + y) % 50)) as u8;
            Rgba([r, g, b, 255])
        });
        frames.push(DynamicImage::ImageRgba8(img));
    }
    
    frames
}

/// Create simulated deepfake video frames (face morphing)
fn create_deepfake_video_frames() -> Vec<DynamicImage> {
    // Deepfakes often have boundary artifacts and morphing
    let mut frames = Vec::new();
    
    for i in 0..25 {
        let img = ImageBuffer::from_fn(720, 1280, |x, y| {
            // Simulate face boundary artifact
            let center_x = 360;
            let center_y = 640;
            let dist = (((x as i32 - center_x).pow(2) + (y as i32 - center_y).pow(2)) as f32).sqrt();
            
            if dist > 200.0 && dist < 220.0 {
                // Sharp boundary (deepfake artifact)
                Rgba([200 + (i * 2) as u8, 150, 150, 255])
            } else if dist < 200.0 {
                // Face area (too smooth)
                Rgba([180, 160, 160, 255])
            } else {
                // Background
                Rgba([100, 100, 100, 255])
            }
        });
        frames.push(DynamicImage::ImageRgba8(img));
    }
    
    frames
}
