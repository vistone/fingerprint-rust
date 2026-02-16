/// 屏幕info噪声注入器
pub struct ScreenNoiseInjector {
    seed: u64,
}

impl ScreenNoiseInjector {
    pub fn new(seed: u64) -> Self {
        Self { seed }
    }

    /// to屏幕resolution添加噪声
    pub fn add_screen_noise(&self, width: u32, height: u32) -> (u32, u32) {
        use rand::{Rng, SeedableRng};
        use rand_chacha::ChaCha8Rng;

        let mut rng = ChaCha8Rng::seed_from_u64(self.seed);

        // 添加 ±1 到 ±3 像素of噪声
        let width_noise = rng.gen_range(-3..=3);
        let height_noise = rng.gen_range(-3..=3);

        let new_width = (width as i32 + width_noise).max(1) as u32;
        let new_height = (height as i32 + height_noise).max(1) as u32;

        (new_width, new_height)
    }
}
