use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

// / Canvas 噪声注入器
pub struct CanvasNoiseInjector {
    seed: u64,
    noise_level: f64, // 0.0 - 1.0
}

impl CanvasNoiseInjector {
    // / createnew噪声注入器
    pub fn new(seed: u64, noise_level: f64) -> Self {
        Self {
            seed,
            noise_level: noise_level.clamp(0.0, 1.0),
        }
    }

    // / to Canvas data添加噪声
    pub fn add_noise(&self, data: &[u8]) -> Vec<u8> {
        let mut rng = ChaCha8Rng::seed_from_u64(self.seed);
        let mut result = data.to_vec();

        // 对每个像素添加微小ofrandom变化
        for pixel in result.chunks_mut(4) {
            if rng.gen::<f64>() < self.noise_level {
                // RGBA 各channel添加 ±1 of噪声
                for channel in pixel.iter_mut() {
                    let noise = if rng.gen::<bool>() { 1 } else { -1 };
                    *channel = channel.saturating_add_signed(noise);
                }
            }
        }

        result
    }

    // / generate Canvas fingerprinthash（带噪声）
    pub fn fingerprint_hash(&self, canvas_data: &[u8]) -> String {
        use sha2::{Digest, Sha256};

        let noisy_data = self.add_noise(canvas_data);
        let mut hasher = Sha256::new();
        hasher.update(&noisy_data);

        format!("{:x}", hasher.finalize())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_canvas_noise() {
        let injector = CanvasNoiseInjector::new(12345, 0.1);
        let data = [255, 128, 64, 255].repeat(100); // 100 像素

        let noisy = injector.add_noise(&data);

        // 噪声应该存在但很小
        assert_eq!(noisy.len(), data.len());
        assert_ne!(noisy, data); // 应该有差异

        // 差异应该很小
        let diff: i32 = data
            .iter()
            .zip(&noisy)
            .map(|(a, b)| (*a as i32 - *b as i32).abs())
            .sum();
        assert!(diff < 200); // 总差异应该很小
    }
}
