//! # fingerprint-api-noise
//!
//! Browser API noise injection module for anti-fingerprint detection based on JavaScript.
//!
//! ## Features
//!
//! - Canvas fingerprint noise injection
//! - WebGL parameter noise injection
//! - AudioContext noise injection
//! - Font enumeration noise injection
//! - Screen information noise injection
//! - Navigator API noise injection

pub mod audio;
pub mod canvas;
pub mod fonts;
pub mod navigator;
pub mod screen;
pub mod webgl;

pub use audio::AudioNoiseInjector;
pub use canvas::CanvasNoiseInjector;
pub use fonts::FontNoiseInjector;
pub use webgl::{WebGLNoiseInjector, WebGLParams};

use rand::Rng;

/// API noise configuration
#[derive(Clone, Debug)]
pub struct NoiseConfig {
    /// Noise seed (for reproducible noise generation)
    pub seed: u64,
    /// Canvas noise level (0.0 - 1.0)
    pub canvas_noise_level: f64,
    /// WebGL noise enabled
    pub enable_webgl_noise: bool,
    /// Audio noise enabled
    pub enable_audio_noise: bool,
    /// Font noise enabled
    pub enable_font_noise: bool,
}

impl Default for NoiseConfig {
    fn default() -> Self {
        Self {
            seed: rand::thread_rng().gen(),
            canvas_noise_level: 0.1,
            enable_webgl_noise: true,
            enable_audio_noise: true,
            enable_font_noise: true,
        }
    }
}

/// Unified API noise injector
pub struct ApiNoiseInjector {
    #[allow(dead_code)]
    config: NoiseConfig,
    canvas: CanvasNoiseInjector,
    webgl: WebGLNoiseInjector,
    audio: AudioNoiseInjector,
    fonts: FontNoiseInjector,
}

impl ApiNoiseInjector {
    /// Create a new API noise injector
    pub fn new(config: NoiseConfig) -> Self {
        Self {
            canvas: CanvasNoiseInjector::new(config.seed, config.canvas_noise_level),
            webgl: WebGLNoiseInjector::with_seed(config.seed),
            audio: AudioNoiseInjector::new(config.seed),
            fonts: FontNoiseInjector::new(),
            config,
        }
    }

    /// Create with default configuration
    pub fn with_defaults() -> Self {
        Self::new(NoiseConfig::default())
    }

    /// Get Canvas noise injector
    pub fn canvas(&self) -> &CanvasNoiseInjector {
        &self.canvas
    }

    /// Get WebGL noise injector
    pub fn webgl(&self) -> &WebGLNoiseInjector {
        &self.webgl
    }

    /// Get Audio noise injector
    pub fn audio(&self) -> &AudioNoiseInjector {
        &self.audio
    }

    /// Get Font noise injector
    pub fn fonts(&self) -> &FontNoiseInjector {
        &self.fonts
    }
}
