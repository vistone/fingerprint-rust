/// Navigator API 噪声注入器
pub struct NavigatorNoiseInjector {
    seed: u64,
}

impl NavigatorNoiseInjector {
    pub fn new(seed: u64) -> Self {
        Self { seed }
    }

    /// 为硬件并发数添加噪声
    pub fn add_hardware_concurrency_noise(&self, cores: u32) -> u32 {
        use rand::{Rng, SeedableRng};
        use rand_chacha::ChaCha8Rng;

        let mut rng = ChaCha8Rng::seed_from_u64(self.seed);

        // 有 20% 的概率修改核心数 ±1
        if rng.gen::<f64>() < 0.2 {
            let noise = if rng.gen::<bool>() { 1 } else { -1 };
            (cores as i32 + noise).max(1) as u32
        } else {
            cores
        }
    }

    /// 为设备内存添加噪声
    pub fn add_device_memory_noise(&self, memory_gb: u32) -> u32 {
        use rand::{Rng, SeedableRng};
        use rand_chacha::ChaCha8Rng;

        let mut rng = ChaCha8Rng::seed_from_u64(self.seed);

        // 有 15% 的概率修改内存 ±1 GB
        if rng.gen::<f64>() < 0.15 {
            let noise = if rng.gen::<bool>() { 1 } else { -1 };
            (memory_gb as i32 + noise).max(1) as u32
        } else {
            memory_gb
        }
    }
}
