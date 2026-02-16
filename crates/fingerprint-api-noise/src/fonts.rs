use rand::seq::SliceRandom;

/// fontenumeration噪声
pub struct FontNoiseInjector {
    base_fonts: Vec<String>,
}

impl FontNoiseInjector {
    pub fn new() -> Self {
        Self {
            base_fonts: Self::get_common_fonts(),
        }
    }

    /// getcommonfontlist
    fn get_common_fonts() -> Vec<String> {
        vec![
            "Arial".to_string(),
            "Verdana".to_string(),
            "Times New Roman".to_string(),
            "Courier New".to_string(),
            "Georgia".to_string(),
            "Palatino".to_string(),
            "Garamond".to_string(),
            "Bookman".to_string(),
            "Comic Sans MS".to_string(),
            "Trebuchet MS".to_string(),
            "Impact".to_string(),
        ]
    }

    /// generate带噪声offontlist
    /// 每次callreturn略有不同offont顺序或count
    pub fn get_fonts_with_noise(&self, seed: u64) -> Vec<String> {
        use rand::{Rng, SeedableRng};
        use rand_chacha::ChaCha8Rng;

        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        let mut fonts = self.base_fonts.clone();

        // random打乱顺序
        fonts.shuffle(&mut rng);

        // random移除 0-2 个font
        let remove_count = rng.gen_range(0..=2);
        for _ in 0..remove_count {
            if !fonts.is_empty() {
                fonts.pop();
            }
        }

        fonts
    }
}

impl Default for FontNoiseInjector {
    fn default() -> Self {
        Self::new()
    }
}
