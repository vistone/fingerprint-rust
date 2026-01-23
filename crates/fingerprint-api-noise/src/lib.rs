//! # fingerprint-api-noise
//! 
//! 浏览器 API 噪声注入模块，用于对抗基于 JavaScript 的指纹识别。
//! 
//! ## 功能
//! 
//! - Canvas 指纹噪声
//! - WebGL 参数噪声
//! - AudioContext 噪声
//! - 字体枚举噪声
//! - 屏幕信息噪声
//! - Navigator API 噪声

pub mod canvas;
pub mod webgl;
pub mod audio;
pub mod fonts;
pub mod screen;
pub mod navigator;

pub use canvas::CanvasNoiseInjector;
pub use webgl::{WebGLNoiseInjector, WebGLParams};
pub use audio::AudioNoiseInjector;
pub use fonts::FontNoiseInjector;

use rand::Rng;

/// API 噪声配置
#[derive(Clone, Debug)]
pub struct NoiseConfig {
    /// 噪声种子（用于可重现的噪声）
    pub seed: u64,
    /// Canvas 噪声等级 (0.0 - 1.0)
    pub canvas_noise_level: f64,
    /// WebGL 噪声启用
    pub enable_webgl_noise: bool,
    /// Audio 噪声启用
    pub enable_audio_noise: bool,
    /// 字体噪声启用
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

/// 统一的 API 噪声注入器
pub struct ApiNoiseInjector {
    #[allow(dead_code)]
    config: NoiseConfig,
    canvas: CanvasNoiseInjector,
    webgl: WebGLNoiseInjector,
    audio: AudioNoiseInjector,
    fonts: FontNoiseInjector,
}

impl ApiNoiseInjector {
    /// 创建新的 API 噪声注入器
    pub fn new(config: NoiseConfig) -> Self {
        Self {
            canvas: CanvasNoiseInjector::new(config.seed, config.canvas_noise_level),
            webgl: WebGLNoiseInjector::new(),
            audio: AudioNoiseInjector::new(config.seed),
            fonts: FontNoiseInjector::new(),
            config,
        }
    }

    /// 使用默认配置创建
    pub fn with_defaults() -> Self {
        Self::new(NoiseConfig::default())
    }

    /// 获取 Canvas 噪声注入器
    pub fn canvas(&self) -> &CanvasNoiseInjector {
        &self.canvas
    }

    /// 获取 WebGL 噪声注入器
    pub fn webgl(&self) -> &WebGLNoiseInjector {
        &self.webgl
    }

    /// 获取 Audio 噪声注入器
    pub fn audio(&self) -> &AudioNoiseInjector {
        &self.audio
    }

    /// 获取字体噪声注入器
    pub fn fonts(&self) -> &FontNoiseInjector {
        &self.fonts
    }
}
