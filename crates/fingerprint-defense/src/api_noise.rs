use rand::Rng;
use std::collections::HashMap;

/// Canvas noise generator for preventing fingerprinting
/// 画布噪声生成器用于防止指纹识别
pub struct CanvasNoiseGenerator {
    /// 噪声强度 (0.0 - 1.0)
    intensity: f64,
    /// 随机种子
    seed: u64,
    /// 已应用噪声的画布标识
    canvas_cache: HashMap<String, bool>,
}

impl CanvasNoiseGenerator {
    /// Create a new canvas noise generator
    /// 创建新的画布噪声生成器
    pub fn new(intensity: f64) -> Self {
        Self {
            intensity: intensity.clamp(0.0, 1.0),
            seed: rand::random(),
            canvas_cache: HashMap::new(),
        }
    }

    /// Generate noise for canvas fingerprint prevention
    /// 生成噪声用于防止画布指纹识别
    pub fn generate_canvas_noise(&self) -> Vec<u8> {
        let mut rng = rand::thread_rng();
        let noise_size = (1024.0 * self.intensity) as usize;
        let mut noise_data = Vec::with_capacity(noise_size);

        // 生成随机噪声数据
        for _ in 0..noise_size {
            noise_data.push(rng.gen::<u8>());
        }

        noise_data
    }

    /// Apply noise to audio context
    /// 对音频上下文应用噪声
    pub fn apply_audio_noise(&self, audio_data: &[f32]) -> Vec<f32> {
        let mut rng = rand::thread_rng();
        let mut noisy_audio = Vec::with_capacity(audio_data.len());

        for &sample in audio_data {
            // 添加轻微的白噪声
            let noise_level = (self.intensity * 0.01) as f32; // 1%的噪声强度
            let noise = (rng.gen::<f32>() - 0.5) * noise_level;
            noisy_audio.push(sample + noise);
        }

        noisy_audio
    }

    /// Apply canvas rendering noise
    /// 应用画布渲染噪声
    pub fn apply_canvas_rendering_noise(
        &mut self, // 改为可变引用
        canvas_id: &str,
        pixel_data: &mut [u8],
    ) -> Result<(), String> {
        if self.canvas_cache.contains_key(canvas_id) {
            return Err("Canvas already processed".to_string());
        }

        let mut rng = rand::thread_rng();

        // 对像素数据应用微小扰动
        for pixel_chunk in pixel_data.chunks_mut(4) {
            if pixel_chunk.len() == 4 {
                // RGBA格式：对RGB通道应用噪声，保持Alpha通道不变
                for channel in pixel_chunk.iter_mut().take(3) {
                    let noise = ((rng.gen::<f64>() - 0.5) * self.intensity * 10.0) as i8;
                    let new_value = *channel as i16 + noise as i16;
                    *channel = new_value.clamp(0, 255) as u8;
                }
            }
        }

        self.canvas_cache.insert(canvas_id.to_string(), true);
        Ok(())
    }

    /// Generate WebGL context noise
    /// 生成WebGL上下文噪声
    pub fn generate_webgl_noise(&self) -> HashMap<String, String> {
        let mut rng = rand::thread_rng();
        let mut noise_params = HashMap::new();

        // 生成随机的WebGL参数扰动
        noise_params.insert(
            "MAX_TEXTURE_SIZE".to_string(),
            (rng.gen_range(2048..=16384)).to_string(),
        );
        noise_params.insert(
            "MAX_VERTEX_UNIFORMS".to_string(),
            (rng.gen_range(1024..=4096)).to_string(),
        );
        noise_params.insert(
            "ALIASED_LINE_WIDTH_RANGE".to_string(),
            format!("[{}, {}]", rng.gen_range(1..=5), rng.gen_range(1..=5)),
        );

        noise_params
    }

    /// Get current noise configuration
    /// 获取当前噪声配置
    pub fn get_config(&self) -> NoiseConfiguration {
        NoiseConfiguration {
            intensity: self.intensity,
            seed: self.seed,
            processed_canvases: self.canvas_cache.len(),
        }
    }

    /// Set noise intensity
    /// 设置噪声强度
    pub fn set_intensity(&mut self, intensity: f64) {
        self.intensity = intensity.clamp(0.0, 1.0);
    }
}

/// Noise configuration information
/// 噪声配置信息
#[derive(Debug, Clone)]
pub struct NoiseConfiguration {
    pub intensity: f64,
    pub seed: u64,
    pub processed_canvases: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_canvas_noise_generation() {
        let generator = CanvasNoiseGenerator::new(0.5);
        let noise = generator.generate_canvas_noise();

        assert!(!noise.is_empty());
        assert!(noise.len() <= 512); // 1024 * 0.5

        // 测试不同强度
        let low_generator = CanvasNoiseGenerator::new(0.1);
        let low_noise = low_generator.generate_canvas_noise();
        assert!(low_noise.len() <= noise.len());
    }

    #[test]
    fn test_audio_noise_application() {
        let generator = CanvasNoiseGenerator::new(0.3);
        let original_audio = vec![0.5f32, -0.3, 0.8, -0.1];
        let noisy_audio = generator.apply_audio_noise(&original_audio);

        assert_eq!(original_audio.len(), noisy_audio.len());

        // 噪声应该改变原始值，但变化很小
        for (orig, noisy) in original_audio.iter().zip(noisy_audio.iter()) {
            let diff = (orig - noisy).abs();
            assert!(diff > 0.0 && diff < 0.1);
        }
    }

    #[test]
    fn test_canvas_rendering_noise() {
        let mut generator = CanvasNoiseGenerator::new(0.2);
        let mut pixel_data = vec![100u8, 150, 200, 255, 50, 75, 125, 200];

        let result = generator.apply_canvas_rendering_noise("test_canvas", &mut pixel_data);
        assert!(result.is_ok());

        // Alpha通道应该保持不变（位置3和7）
        assert_eq!(pixel_data[3], 255);
        assert_eq!(pixel_data[7], 200);

        // 再次处理同一个画布应该失败
        let result2 = generator.apply_canvas_rendering_noise("test_canvas", &mut pixel_data);
        assert!(result2.is_err());
    }

    #[test]
    fn test_webgl_noise_generation() {
        let generator = CanvasNoiseGenerator::new(0.4);
        let webgl_noise = generator.generate_webgl_noise();

        assert!(webgl_noise.contains_key("MAX_TEXTURE_SIZE"));
        assert!(webgl_noise.contains_key("MAX_VERTEX_UNIFORMS"));
        assert!(webgl_noise.contains_key("ALIASED_LINE_WIDTH_RANGE"));

        // 值应该在合理范围内
        let max_texture: i32 = webgl_noise["MAX_TEXTURE_SIZE"].parse().unwrap();
        assert!((2048..=16384).contains(&max_texture));
    }
}
