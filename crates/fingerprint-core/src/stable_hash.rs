use xxhash_rust::xxh3::Xxh3;

/// Deterministic hash builder that uses a fixed byte encoding.
///
/// This avoids `DefaultHasher` randomization and native-endian encoding
/// differences so the same logical fingerprint yields the same hash across
/// platforms, processes, and Rust versions.
#[derive(Default)]
pub struct StableHashBuilder {
    hasher: Xxh3,
}

impl StableHashBuilder {
    pub fn new() -> Self {
        Self {
            hasher: Xxh3::new(),
        }
    }

    pub fn write_u8(&mut self, value: u8) {
        self.hasher.update(&[value]);
    }

    pub fn write_u16(&mut self, value: u16) {
        self.hasher.update(&value.to_be_bytes());
    }

    pub fn write_u32(&mut self, value: u32) {
        self.hasher.update(&value.to_be_bytes());
    }

    pub fn write_u64(&mut self, value: u64) {
        self.hasher.update(&value.to_be_bytes());
    }

    pub fn write_usize(&mut self, value: usize) {
        self.write_u64(value as u64);
    }

    pub fn write_bool(&mut self, value: bool) {
        self.write_u8(u8::from(value));
    }

    pub fn write_bytes(&mut self, value: &[u8]) {
        self.write_usize(value.len());
        self.hasher.update(value);
    }

    pub fn write_str(&mut self, value: &str) {
        self.write_bytes(value.as_bytes());
    }

    pub fn write_option_u8(&mut self, value: Option<u8>) {
        self.write_bool(value.is_some());
        if let Some(value) = value {
            self.write_u8(value);
        }
    }

    pub fn write_option_u16(&mut self, value: Option<u16>) {
        self.write_bool(value.is_some());
        if let Some(value) = value {
            self.write_u16(value);
        }
    }

    pub fn write_option_str(&mut self, value: Option<&str>) {
        self.write_bool(value.is_some());
        if let Some(value) = value {
            self.write_str(value);
        }
    }

    pub fn write_u8_slice(&mut self, values: &[u8]) {
        self.write_usize(values.len());
        for &value in values {
            self.write_u8(value);
        }
    }

    pub fn write_u16_slice(&mut self, values: &[u16]) {
        self.write_usize(values.len());
        for &value in values {
            self.write_u16(value);
        }
    }

    pub fn finish(self) -> u64 {
        self.hasher.digest()
    }
}

pub fn hash_str(value: &str) -> u64 {
    let mut hasher = StableHashBuilder::new();
    hasher.write_str(value);
    hasher.finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stable_hash_matches_expected_value() {
        let mut hasher = StableHashBuilder::new();
        hasher.write_u16(0x1301);
        hasher.write_str("h2");
        hasher.write_option_u8(Some(7));
        assert_eq!(hasher.finish(), 0x3fbd_a8b9_351d_9627);
    }

    #[test]
    fn stable_hash_distinguishes_lengths() {
        let mut first = StableHashBuilder::new();
        first.write_str("ab");
        first.write_str("c");

        let mut second = StableHashBuilder::new();
        second.write_str("a");
        second.write_str("bc");

        assert_ne!(first.finish(), second.finish());
    }
}
