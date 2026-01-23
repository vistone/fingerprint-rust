/// AudioContext 指纹噪声
pub struct AudioNoiseInjector {
    seed: u64,
}

impl AudioNoiseInjector {
    pub fn new(seed: u64) -> Self {
        Self { seed }
    }

    /// 为音频数据添加噪声
    pub fn add_audio_noise(&self, samples: &[f32]) -> Vec<f32> {
        use rand::{Rng, SeedableRng};
        use rand_chacha::ChaCha8Rng;

        let mut rng = ChaCha8Rng::seed_from_u64(self.seed);
        let mut result = samples.to_vec();

        // 对每个样本添加微小的噪声
        for sample in result.iter_mut() {
            let noise = rng.gen_range(-0.0001..0.0001);
            *sample += noise;
        }

        result
    }

    /// 生成 Audio 指纹（带噪声）
    pub fn audio_fingerprint(&self, samples: &[f32]) -> Vec<u8> {
        let noisy_samples = self.add_audio_noise(samples);
        
        // 转换为字节用于哈希
        noisy_samples.iter()
            .flat_map(|&f| f.to_le_bytes())
            .collect()
    }
}
