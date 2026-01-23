use std::collections::HashMap;

/// WebGL 参数噪声
pub struct WebGLNoiseInjector {
    #[allow(dead_code)]
    noise_patterns: HashMap<String, NoisePattern>,
}

#[derive(Clone)]
pub struct NoisePattern {
    pub parameter: String,
    pub original_value: String,
    pub noise_range: f64,
}

impl WebGLNoiseInjector {
    pub fn new() -> Self {
        let mut noise_patterns = HashMap::new();
        
        // 常见的 WebGL 指纹参数
        noise_patterns.insert(
            "RENDERER".to_string(),
            NoisePattern {
                parameter: "RENDERER".to_string(),
                original_value: "".to_string(),
                noise_range: 0.01,
            }
        );
        
        Self { noise_patterns }
    }

    /// 为 WebGL 参数添加噪声
    pub fn add_webgl_noise(&self, params: &WebGLParams) -> WebGLParams {
        let mut result = params.clone();
        
        // 对浮点参数添加微小噪声
        if let Some(aliased_line_width_range) = &mut result.aliased_line_width_range {
            // 添加 ±0.01 的噪声
            aliased_line_width_range[0] += self.generate_small_noise();
            aliased_line_width_range[1] += self.generate_small_noise();
        }
        
        result
    }

    fn generate_small_noise(&self) -> f32 {
        use rand::Rng;
        let mut rng = rand::thread_rng();
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
