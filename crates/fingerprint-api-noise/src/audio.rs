// / AudioContext fingerprint噪声
pub struct AudioNoiseInjector {
    seed: u64,
}

impl AudioNoiseInjector {
    pub fn new(seed: u64) -> Self {
        Self { seed }
    }

    // / toaudiodata添加噪声
    pub fn add_audio_noise(&self, samples: &[f32]) -> Vec<f32> {
        use rand::{Rng, SeedableRng};
        use rand_chacha::ChaCha8Rng;

        let mut rng = ChaCha8Rng::seed_from_u64(self.seed);
        let mut result = samples.to_vec();

        // 对每个sample添加微小of噪声
        for sample in result.iter_mut() {
            let noise = rng.gen_range(-0.0001..0.0001);
            *sample += noise;
        }

        result
    }

    // / generate Audio fingerprint（带噪声）
    pub fn audio_fingerprint(&self, samples: &[f32]) -> Vec<u8> {
        let noisy_samples = self.add_audio_noise(samples);

        // convertto字节用于hash
        noisy_samples
            .iter()
            .flat_map(|&f| f.to_le_bytes())
            .collect()
    }
}
