//! # fingerprint-api-noise
//!
// ! 浏览器 API 噪声注入module，用于对抗based on JavaScript offingerprintrecognition。
//!
// ! ## functionality
//!
// ! - Canvas fingerprint噪声
// ! - WebGL argument噪声
// ! - AudioContext 噪声
// ! - fontenumeration噪声
// ! - 屏幕info噪声
// ! - Navigator API 噪声

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

// / API 噪声configure
#[derive(Clone, Debug)]
pub struct NoiseConfig {
    // / 噪声种子（用于可重现of噪声）
    pub seed: u64,
    // / Canvas 噪声等级 (0.0 - 1.0)
    pub canvas_noise_level: f64,
    // / WebGL 噪声enable
    pub enable_webgl_noise: bool,
    // / Audio 噪声enable
    pub enable_audio_noise: bool,
    // / font噪声enable
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

// / 统一of API 噪声注入器
pub struct ApiNoiseInjector {
    #[allow(dead_code)]
    config: NoiseConfig,
    canvas: CanvasNoiseInjector,
    webgl: WebGLNoiseInjector,
    audio: AudioNoiseInjector,
    fonts: FontNoiseInjector,
}

impl ApiNoiseInjector {
    // / createnew API 噪声注入器
    pub fn new(config: NoiseConfig) -> Self {
        Self {
            canvas: CanvasNoiseInjector::new(config.seed, config.canvas_noise_level),
            webgl: WebGLNoiseInjector::with_seed(config.seed),
            audio: AudioNoiseInjector::new(config.seed),
            fonts: FontNoiseInjector::new(),
            config,
        }
    }

    // / usedefaultconfigurecreate
    pub fn with_defaults() -> Self {
        Self::new(NoiseConfig::default())
    }

    // / get Canvas 噪声注入器
    pub fn canvas(&self) -> &CanvasNoiseInjector {
        &self.canvas
    }

    // / get WebGL 噪声注入器
    pub fn webgl(&self) -> &WebGLNoiseInjector {
        &self.webgl
    }

    // / get Audio 噪声注入器
    pub fn audio(&self) -> &AudioNoiseInjector {
        &self.audio
    }

    // / getfont噪声注入器
    pub fn fonts(&self) -> &FontNoiseInjector {
        &self.fonts
    }
}
