/// WebGL 参数噪声
pub struct WebGLNoiseInjector {
    seed: u64,
}

#[derive(Clone)]
pub struct NoisePattern {
    pub parameter: String,
    pub original_value: String,
    pub noise_range: f64,
}

impl WebGLNoiseInjector {
    pub fn new() -> Self {
        Self::with_seed(0)
    }

    /// 使用指定种子创建 WebGL 噪声注入器
    pub fn with_seed(seed: u64) -> Self {
        Self { seed }
    }

    /// 为 WebGL 参数添加噪声
    pub fn add_webgl_noise(&self, params: &WebGLParams) -> WebGLParams {
        let mut result = params.clone();
        
        // 对浮点参数添加微小噪声
        if let Some(aliased_line_width_range) = &mut result.aliased_line_width_range {
            // 添加 ±0.01 的噪声
            aliased_line_width_range[0] += self.generate_small_noise(0);
            aliased_line_width_range[1] += self.generate_small_noise(1);
        }
        
        result
    }

    fn generate_small_noise(&self, index: u64) -> f32 {
        use rand::{Rng, SeedableRng};
        use rand_chacha::ChaCha8Rng;
        
        let mut rng = ChaCha8Rng::seed_from_u64(self.seed + index);
        rng.gen_range(-0.01..0.01)
    }
}

impl Default for WebGLNoiseInjector {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone)]
pub struct WebGLParams {
    pub renderer: String,
    pub vendor: String,
    pub aliased_line_width_range: Option<[f32; 2]>,
    pub aliased_point_size_range: Option<[f32; 2]>,
    pub max_texture_size: Option<u32>,
    pub max_viewport_dims: Option<[u32; 2]>,
}
