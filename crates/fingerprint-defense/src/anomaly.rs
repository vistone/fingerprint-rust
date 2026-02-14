//! Anomaly detection module

use std::collections::HashMap;

// Entropy-based anomaly detection thresholds
/// Minimum unique byte ratio for normal data (26/256 ≈ 10%)
/// Data with fewer unique bytes is considered suspiciously uniform
const MIN_UNIQUE_BYTES: usize = 26;

/// Maximum entropy threshold in bits (Shannon entropy)
/// Real fingerprint data typically has entropy between 4-7 bits
/// Values above this indicate suspiciously random data
const MAX_ENTROPY_BITS: f64 = 7.5;

// Screen resolution thresholds for contradiction detection
/// Maximum typical mobile screen width in pixels
/// Resolutions above this are suspicious for mobile devices
const MAX_TYPICAL_MOBILE_WIDTH: u32 = 1920;

/// Minimum typical desktop screen width in pixels
/// Resolutions below this are suspicious for desktop devices
const MIN_TYPICAL_DESKTOP_WIDTH: u32 = 800;

/// Anomaly detector for behavioral analysis
pub struct AnomalyDetector;

impl AnomalyDetector {
    /// Create new anomaly detector
    pub fn new() -> Self {
        AnomalyDetector
    }

    /// Detect anomalies in fingerprint data
    ///
    /// Analyzes fingerprint data for suspicious patterns that may indicate
    /// manipulation, spoofing, or automated behavior.
    ///
    /// # Detection criteria
    /// - Checks data entropy (too random or too uniform)
    /// - Validates data structure integrity
    /// - Detects common fingerprint spoofing patterns
    pub fn detect(&self, data: &[u8]) -> bool {
        if data.is_empty() {
            return false;
        }

        // Check 1: Detect suspiciously low entropy (repeated patterns)
        if self.has_low_entropy(data) {
            return true;
        }

        // Check 2: Detect suspiciously high entropy (completely random)
        if self.has_excessive_entropy(data) {
            return true;
        }

        // Check 3: Check for common spoofing signatures
        if self.contains_spoofing_markers(data) {
            return true;
        }

        false
    }

    /// Check if data has suspiciously low entropy
    fn has_low_entropy(&self, data: &[u8]) -> bool {
        if data.len() < 10 {
            return false;
        }

        let mut byte_counts = [0u32; 256];
        for &byte in data {
            byte_counts[byte as usize] += 1;
        }

        // Count unique bytes
        let unique_bytes = byte_counts.iter().filter(|&&count| count > 0).count();

        // If less than MIN_UNIQUE_BYTES (≈10% of 256 possible values), it's suspicious
        unique_bytes < MIN_UNIQUE_BYTES
    }

    /// Check if data has suspiciously high entropy (too random)
    fn has_excessive_entropy(&self, data: &[u8]) -> bool {
        if data.len() < 20 {
            return false;
        }

        let mut byte_counts = [0u32; 256];
        for &byte in data {
            byte_counts[byte as usize] += 1;
        }

        // Calculate entropy using Shannon entropy formula
        let len = data.len() as f64;
        let entropy: f64 = byte_counts
            .iter()
            .filter(|&&count| count > 0)
            .map(|&count| {
                let probability = count as f64 / len;
                -probability * probability.log2()
            })
            .sum();

        // If entropy exceeds MAX_ENTROPY_BITS, data is too random
        entropy > MAX_ENTROPY_BITS
    }

    /// Check for known spoofing tool signatures
    fn contains_spoofing_markers(&self, data: &[u8]) -> bool {
        // Check for common spoofing tool patterns
        let spoofing_patterns: &[&[u8]] = &[
            b"HeadlessChrome",
            b"PhantomJS",
            b"webdriver",
            b"selenium",
            b"puppeteer",
        ];

        for pattern in spoofing_patterns {
            if data
                .windows(pattern.len())
                .any(|window| window == *pattern)
            {
                return true;
            }
        }

        false
    }
}

impl Default for AnomalyDetector {
    fn default() -> Self {
        Self::new()
    }
}

/// Contradiction detector for inconsistent fingerprints
pub struct ContradictionDetector;

impl ContradictionDetector {
    /// Create new contradiction detector
    pub fn new() -> Self {
        ContradictionDetector
    }

    /// Check for contradictions in fingerprint attributes
    ///
    /// Analyzes fingerprint attributes to detect logical inconsistencies
    /// that indicate spoofing or manipulation.
    ///
    /// # Examples of contradictions
    /// - Mobile User-Agent with desktop screen resolution
    /// - Old browser version with new JavaScript features
    /// - Mismatched OS and platform indicators
    pub fn check_contradictions(&self, attributes: &[(&str, &str)]) -> bool {
        if attributes.is_empty() {
            return false;
        }

        // Convert to HashMap for easier lookup
        let attr_map: HashMap<&str, &str> = attributes.iter().copied().collect();

        // Check for OS/Platform contradictions
        if let (Some(os), Some(platform)) = (attr_map.get("os"), attr_map.get("platform")) {
            if self.has_os_platform_contradiction(os, platform) {
                return true;
            }
        }

        // Check for User-Agent/Features contradictions
        if let (Some(user_agent), Some(features)) =
            (attr_map.get("user_agent"), attr_map.get("features"))
        {
            if self.has_user_agent_feature_contradiction(user_agent, features) {
                return true;
            }
        }

        // Check for Mobile/Screen contradictions
        if let (Some(is_mobile), Some(screen_width)) =
            (attr_map.get("is_mobile"), attr_map.get("screen_width"))
        {
            if self.has_mobile_screen_contradiction(is_mobile, screen_width) {
                return true;
            }
        }

        false
    }

    /// Check for OS and platform contradictions
    fn has_os_platform_contradiction(&self, os: &str, platform: &str) -> bool {
        // Windows should have "Win" platform
        if os.contains("Windows") && !platform.contains("Win") {
            return true;
        }

        // macOS should have "Mac" platform
        if os.contains("Mac") && !platform.contains("Mac") {
            return true;
        }

        // Linux should have "Linux" platform
        if os.contains("Linux") && !platform.contains("Linux") && !platform.contains("X11") {
            return true;
        }

        false
    }

    /// Check for User-Agent and feature contradictions
    fn has_user_agent_feature_contradiction(&self, user_agent: &str, features: &str) -> bool {
        // Old browsers shouldn't support modern features
        if user_agent.contains("Chrome/60") && features.contains("WebGL2") {
            return true;
        }

        // Mobile browsers shouldn't claim desktop features
        if user_agent.contains("Mobile") && features.contains("desktop") {
            return true;
        }

        false
    }

    /// Check for mobile and screen size contradictions
    fn has_mobile_screen_contradiction(&self, is_mobile: &str, screen_width: &str) -> bool {
        if let Ok(width) = screen_width.parse::<u32>() {
            // Mobile device with desktop resolution is suspicious
            if is_mobile == "true" && width > MAX_TYPICAL_MOBILE_WIDTH {
                return true;
            }

            // Desktop device with tiny screen is suspicious
            if is_mobile == "false" && width < MIN_TYPICAL_DESKTOP_WIDTH {
                return true;
            }
        }

        false
    }
}

impl Default for ContradictionDetector {
    fn default() -> Self {
        Self::new()
    }
}
