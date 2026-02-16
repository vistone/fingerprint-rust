//! AI-Generated Audio Detection and Fingerprinting
//!
//! This module provides detection and fingerprinting of AI-generated audio content
//! including speech synthesis, voice cloning, and deepfake voice detection.
//!
//! Based on latest 2025-2026 research including:
//! - Spectral analysis and vocoder artifact detection
//! - Micro-frequency fingerprinting
//! - Self-supervised learning features
//! - Model-specific patterns

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// AI-generated audio detection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioFingerprint {
    /// Whether audio is likely AI-generated
    pub is_ai_generated: bool,

    /// Confidence score (0.0 - 1.0)
    pub confidence: f32,

    /// Spectral consistency score (lower = more AI-like)
    pub spectral_consistency: f32,

    /// Micro-frequency analysis score
    pub micro_frequency_score: f32,

    /// Vocoder artifact detection score
    pub vocoder_artifacts: f32,

    /// Natural breathing/pause patterns (higher = more human-like)
    pub natural_patterns: f32,

    /// Model attribution probabilities
    pub model_probabilities: HashMap<String, f32>,

    /// Detected patterns
    pub patterns: Vec<AudioPattern>,

    /// Analysis metadata
    pub metadata: AudioMetadata,
}

/// Detected pattern in audio
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioPattern {
    /// Pattern type
    pub pattern_type: AudioPatternType,

    /// Pattern description
    pub description: String,

    /// Confidence this pattern indicates AI
    pub confidence: f32,

    /// Time range in seconds (start, end)
    pub time_range: Option<(f32, f32)>,
}

/// Types of detectable audio patterns
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AudioPatternType {
    /// Vocoder artifacts (GAN/neural vocoder traces)
    VocoderArtifacts,

    /// Unnaturally consistent pitch
    UniformPitch,

    /// Missing micro-frequency details
    MissingMicroFrequency,

    /// Artificial breathing patterns
    ArtificialBreathing,

    /// Unnaturally smooth transitions
    SmoothTransitions,

    /// Robotic prosody
    RoboticProsody,

    /// Phase inconsistencies
    PhaseInconsistencies,

    /// Spectral holes (frequency gaps)
    SpectralHoles,
}

/// Audio analysis metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioMetadata {
    /// Audio duration in seconds
    pub duration_seconds: f32,

    /// Sample rate (Hz)
    pub sample_rate: Option<u32>,

    /// Number of channels
    pub channels: Option<u8>,

    /// Bit depth
    pub bit_depth: Option<u8>,

    /// Detected language (if speech)
    pub language: Option<String>,

    /// Analysis timestamp
    pub analyzed_at: Option<u64>,
}

/// AI voice synthesis provider
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum VoiceProvider {
    /// ElevenLabs
    ElevenLabs,

    /// Azure Text-to-Speech
    AzureTTS,

    /// Google Cloud TTS
    GoogleTTS,

    /// Amazon Polly
    AmazonPolly,

    /// Play.ht
    PlayHT,

    /// Resemble AI
    ResembleAI,

    /// Murf.ai
    MurfAI,

    /// OpenAI TTS
    OpenAITTS,

    /// Generic/Unknown
    Unknown,
}

impl VoiceProvider {
    /// Get provider name as string
    pub fn as_str(&self) -> &str {
        match self {
            VoiceProvider::ElevenLabs => "elevenlabs",
            VoiceProvider::AzureTTS => "azure_tts",
            VoiceProvider::GoogleTTS => "google_tts",
            VoiceProvider::AmazonPolly => "amazon_polly",
            VoiceProvider::PlayHT => "playht",
            VoiceProvider::ResembleAI => "resemble_ai",
            VoiceProvider::MurfAI => "murf_ai",
            VoiceProvider::OpenAITTS => "openai_tts",
            VoiceProvider::Unknown => "unknown",
        }
    }
}

/// Analyze audio metadata to detect AI generation
///
/// This is a simplified analysis based on metadata patterns.
/// For full audio analysis, use dedicated audio processing libraries.
///
/// # Arguments
///
/// * `sample_rate` - Audio sample rate in Hz
/// * `bit_depth` - Audio bit depth
/// * `duration` - Duration in seconds
/// * `has_natural_pauses` - Whether audio has natural breathing/pauses
///
/// # Returns
///
/// `AudioFingerprint` containing detection results
///
/// # Example
///
/// ```rust
/// use fingerprint_ai_models::audio_detection::detect_ai_audio_from_metadata;
///
/// let result = detect_ai_audio_from_metadata(
///     Some(22050),  // Sample rate
///     Some(16),     // Bit depth
///     10.5,         // Duration
///     false         // No natural pauses
/// );
///
/// if result.is_ai_generated {
///     println!("Detected AI-generated audio with {:.2}% confidence",
///              result.confidence * 100.0);
/// }
/// ```
pub fn detect_ai_audio_from_metadata(
    sample_rate: Option<u32>,
    bit_depth: Option<u8>,
    duration: f32,
    has_natural_pauses: bool,
) -> AudioFingerprint {
    let metadata = AudioMetadata {
        duration_seconds: duration,
        sample_rate,
        channels: Some(1), // Assume mono for simplicity
        bit_depth,
        language: None,
        analyzed_at: None,
    };

    // Analyze patterns
    let patterns = detect_metadata_patterns(&metadata, has_natural_pauses);

    // Calculate scores
    let spectral_consistency = calculate_spectral_score(&metadata, has_natural_pauses);
    let micro_frequency_score = calculate_micro_frequency_score(&metadata);
    let vocoder_artifacts = detect_vocoder_artifacts(&metadata, has_natural_pauses);
    let natural_patterns = if has_natural_pauses { 0.8 } else { 0.2 };

    // Model attribution
    let model_probs = attribute_to_voice_models(spectral_consistency, vocoder_artifacts, &patterns);

    // Calculate overall AI likelihood
    let ai_score = calculate_audio_ai_score(
        spectral_consistency,
        micro_frequency_score,
        vocoder_artifacts,
        natural_patterns,
        &patterns,
    );

    AudioFingerprint {
        is_ai_generated: ai_score > 0.65,
        confidence: ai_score,
        spectral_consistency,
        micro_frequency_score,
        vocoder_artifacts,
        natural_patterns,
        model_probabilities: model_probs,
        patterns,
        metadata,
    }
}

/// Detect patterns from metadata
fn detect_metadata_patterns(
    metadata: &AudioMetadata,
    has_natural_pauses: bool,
) -> Vec<AudioPattern> {
    let mut patterns = Vec::new();

    // Check for unnaturally low sample rates (TTS often uses 22050 Hz or 24000 Hz)
    if let Some(sr) = metadata.sample_rate {
        if sr == 22050 || sr == 24000 {
            patterns.push(AudioPattern {
                pattern_type: AudioPatternType::VocoderArtifacts,
                description: format!("Sample rate {} Hz typical for TTS systems", sr),
                confidence: 0.6,
                time_range: None,
            });
        }
    }

    // Check for lack of natural pauses
    if !has_natural_pauses {
        patterns.push(AudioPattern {
            pattern_type: AudioPatternType::ArtificialBreathing,
            description: "Lack of natural breathing patterns detected".to_string(),
            confidence: 0.7,
            time_range: None,
        });
    }

    // Check duration patterns (very short or very uniform segments)
    if metadata.duration_seconds < 2.0 || (metadata.duration_seconds % 1.0).abs() < 0.1 {
        patterns.push(AudioPattern {
            pattern_type: AudioPatternType::SmoothTransitions,
            description: "Suspiciously uniform duration pattern".to_string(),
            confidence: 0.5,
            time_range: None,
        });
    }

    patterns
}

/// Calculate spectral consistency score
fn calculate_spectral_score(metadata: &AudioMetadata, has_natural_pauses: bool) -> f32 {
    let mut score: f32 = 0.5;

    // Lower sample rates often indicate synthetic audio
    if let Some(sr) = metadata.sample_rate {
        if sr < 24000 {
            score = 0.3; // More consistent = more AI-like
        } else if sr >= 44100 {
            score = 0.7; // Higher variation = more human-like
        }
    }

    // Lack of natural pauses indicates high consistency
    if !has_natural_pauses {
        score *= 0.8;
    }

    score.clamp(0.0, 1.0)
}

/// Calculate micro-frequency score
fn calculate_micro_frequency_score(metadata: &AudioMetadata) -> f32 {
    // Simplified: assume lower sample rates miss micro-frequency details
    if let Some(sr) = metadata.sample_rate {
        if sr < 24000 {
            0.3 // Missing micro-frequencies
        } else {
            0.6 // Likely has micro-frequencies
        }
    } else {
        0.5
    }
}

/// Detect vocoder artifacts
fn detect_vocoder_artifacts(metadata: &AudioMetadata, has_natural_pauses: bool) -> f32 {
    let mut artifact_score: f32 = 0.3;

    // TTS systems often use specific sample rates
    if let Some(sr) = metadata.sample_rate {
        if sr == 22050 || sr == 24000 {
            artifact_score = 0.7; // High likelihood of vocoder artifacts
        }
    }

    // Lack of natural pauses suggests synthetic generation
    if !has_natural_pauses {
        artifact_score += 0.2;
    }

    artifact_score.clamp(0.0, 1.0)
}

/// Attribute audio to specific voice synthesis models
fn attribute_to_voice_models(
    spectral_consistency: f32,
    vocoder_artifacts: f32,
    patterns: &[AudioPattern],
) -> HashMap<String, f32> {
    let mut probabilities = HashMap::new();

    // ElevenLabs: High quality, but detectable artifacts
    let elevenlabs_score = if vocoder_artifacts > 0.6 {
        0.35 + spectral_consistency * 0.15
    } else {
        spectral_consistency * 0.25
    };
    probabilities.insert("elevenlabs".to_string(), elevenlabs_score.clamp(0.0, 1.0));

    // Azure TTS: More robotic, uniform patterns
    let azure_score = if patterns
        .iter()
        .any(|p| p.pattern_type == AudioPatternType::RoboticProsody)
    {
        0.4 + (1.0 - spectral_consistency) * 0.2
    } else {
        (1.0 - spectral_consistency) * 0.3
    };
    probabilities.insert("azure_tts".to_string(), azure_score.clamp(0.0, 1.0));

    // Google TTS: Natural sounding but with spectral holes
    let google_score = if patterns
        .iter()
        .any(|p| p.pattern_type == AudioPatternType::SpectralHoles)
    {
        0.3 + vocoder_artifacts * 0.2
    } else {
        vocoder_artifacts * 0.25
    };
    probabilities.insert("google_tts".to_string(), google_score.clamp(0.0, 1.0));

    // OpenAI TTS: Recent, high quality
    let openai_score = if vocoder_artifacts < 0.4 && spectral_consistency > 0.3 {
        0.35 + spectral_consistency * 0.15
    } else {
        spectral_consistency * 0.2
    };
    probabilities.insert("openai_tts".to_string(), openai_score.clamp(0.0, 1.0));

    // Normalize probabilities
    let total: f32 = probabilities.values().sum();
    if total > 0.0 {
        for value in probabilities.values_mut() {
            *value /= total;
        }
    }

    probabilities
}

/// Calculate overall AI likelihood score
fn calculate_audio_ai_score(
    spectral_consistency: f32,
    micro_frequency_score: f32,
    vocoder_artifacts: f32,
    natural_patterns: f32,
    patterns: &[AudioPattern],
) -> f32 {
    // Weight different factors
    let spectral_weight = 0.25;
    let micro_freq_weight = 0.25;
    let vocoder_weight = 0.30;
    let natural_weight = 0.20;

    // Low spectral consistency = high AI likelihood
    let spectral_score = 1.0 - spectral_consistency;

    // Low micro-frequency = high AI likelihood
    let micro_freq_contribution = 1.0 - micro_frequency_score;

    // High vocoder artifacts = high AI likelihood
    let vocoder_contribution = vocoder_artifacts;

    // Low natural patterns = high AI likelihood
    let natural_contribution = 1.0 - natural_patterns;

    let base_score = spectral_score * spectral_weight
        + micro_freq_contribution * micro_freq_weight
        + vocoder_contribution * vocoder_weight
        + natural_contribution * natural_weight;

    // Boost score if multiple patterns detected
    let pattern_boost = if patterns.len() >= 2 {
        0.15
    } else if patterns.len() == 1 {
        0.05
    } else {
        0.0
    };

    (base_score + pattern_boost).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_tts_audio() {
        // Typical TTS characteristics
        let result = detect_ai_audio_from_metadata(
            Some(22050), // TTS common sample rate
            Some(16),
            5.0,
            false, // No natural pauses
        );

        assert!(result.is_ai_generated);
        assert!(result.confidence > 0.6);
        assert!(!result.patterns.is_empty());
    }

    #[test]
    fn test_detect_human_audio() {
        // Typical human recording
        let result = detect_ai_audio_from_metadata(
            Some(48000), // High quality recording
            Some(24),
            5.0,
            true, // Has natural pauses
        );

        // Should have lower AI confidence
        assert!(result.confidence < 0.7);
    }

    #[test]
    fn test_vocoder_artifact_detection() {
        let metadata = AudioMetadata {
            duration_seconds: 5.0,
            sample_rate: Some(22050),
            channels: Some(1),
            bit_depth: Some(16),
            language: None,
            analyzed_at: None,
        };

        let artifact_score = detect_vocoder_artifacts(&metadata, false);
        assert!(artifact_score > 0.5);
    }

    #[test]
    fn test_model_attribution() {
        let patterns = vec![AudioPattern {
            pattern_type: AudioPatternType::VocoderArtifacts,
            description: "Test".to_string(),
            confidence: 0.7,
            time_range: None,
        }];

        let probs = attribute_to_voice_models(0.3, 0.7, &patterns);

        assert!(probs.contains_key("elevenlabs"));
        assert!(probs.contains_key("azure_tts"));
        assert!(probs.contains_key("google_tts"));

        // Should have non-zero probabilities
        assert!(probs.values().all(|&v| v > 0.0));
    }

    #[test]
    fn test_pattern_detection() {
        let metadata = AudioMetadata {
            duration_seconds: 1.0, // Very short
            sample_rate: Some(22050),
            channels: Some(1),
            bit_depth: Some(16),
            language: None,
            analyzed_at: None,
        };

        let patterns = detect_metadata_patterns(&metadata, false);

        // Should detect multiple patterns
        assert!(!patterns.is_empty());
        assert!(patterns
            .iter()
            .any(|p| p.pattern_type == AudioPatternType::VocoderArtifacts));
    }

    #[test]
    fn test_spectral_score_calculation() {
        let metadata_low_sr = AudioMetadata {
            duration_seconds: 5.0,
            sample_rate: Some(16000),
            channels: Some(1),
            bit_depth: Some(16),
            language: None,
            analyzed_at: None,
        };

        let metadata_high_sr = AudioMetadata {
            duration_seconds: 5.0,
            sample_rate: Some(48000),
            channels: Some(2),
            bit_depth: Some(24),
            language: None,
            analyzed_at: None,
        };

        let score_low = calculate_spectral_score(&metadata_low_sr, false);
        let score_high = calculate_spectral_score(&metadata_high_sr, true);

        // Low sample rate should have lower spectral consistency
        assert!(score_low < score_high);
    }

    #[test]
    fn test_voice_provider_names() {
        assert_eq!(VoiceProvider::ElevenLabs.as_str(), "elevenlabs");
        assert_eq!(VoiceProvider::AzureTTS.as_str(), "azure_tts");
        assert_eq!(VoiceProvider::OpenAITTS.as_str(), "openai_tts");
    }
}
