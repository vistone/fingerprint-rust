#![allow(clippy::all, dead_code, unused_variables, unused_parens)]

//! # Canvas Fingerprint Module
//!
//! Canvas 指纹识别和混淆模块
//!
//! 提供完整的 HTML5 Canvas 指纹识别功能，包括：
//! - Canvas 2D 指纹识别
//! - Canvas 混淆和保护
//! - 预生成指纹库匹配
//! - 浏览器版本识别

use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

// / Canvas 2D fingerprintinfo
#[derive(Debug, Clone, PartialEq)]
pub struct CanvasFingerprint {
    // / 像素data (Base64)
    pub pixel_data: String,
    // / fingerprinthash值
    pub hash: String,
    // / 复杂度score (0.0-1.0)
    pub complexity: f32,
    // / rendering层级
    pub rendering_level: RenderingLevel,
    // / 硬件加速state
    pub hardware_accelerated: bool,
    // / detect到of浏览器version (如果可能)
    pub detected_browser: Option<String>,
    // / 匹配可信度 (0.0-1.0)
    pub confidence: f32,
}

// / rendering层级
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderingLevel {
    // / 软件rendering
    Software,
    // / 硬件加速
    Hardware,
    // / WebGL 加速
    WebGL,
    // / unknown
    Unknown,
}

// / Canvas fingerprinterrortype
#[derive(Debug)]
pub enum CanvasError {
    // / invalidof Canvas data
    InvalidCanvasData,
    // / fingerprintgeneratefailure
    FingerprintGenerationFailed(String),
    // / libraryqueryfailure
    LibraryQueryFailed(String),
    // / othererror
    Other(String),
}

impl std::fmt::Display for CanvasError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CanvasError::InvalidCanvasData => write!(f, "Invalid Canvas data"),
            CanvasError::FingerprintGenerationFailed(msg) => {
                write!(f, "Fingerprint generation failed: {}", msg)
            }
            CanvasError::LibraryQueryFailed(msg) => write!(f, "Library query failed: {}", msg),
            CanvasError::Other(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl std::error::Error for CanvasError {}

// / Canvas fingerprintrecognition器
pub struct CanvasAnalyzer {
    profile_library: CanvasProfileLibrary,
}

impl CanvasAnalyzer {
    // / createnew Canvas analyzer
    pub fn new() -> Self {
        CanvasAnalyzer {
            profile_library: CanvasProfileLibrary::new(),
        }
    }

    // / analyze Canvas data并generatefingerprint
    pub fn analyze(&self, canvas_data: &str) -> Result<CanvasFingerprint, CanvasError> {
        if canvas_data.is_empty() {
            return Err(CanvasError::InvalidCanvasData);
        }

        let hash = self.compute_hash(canvas_data)?;
        let (detected_browser, confidence) = self.detect_browser(&hash);
        let complexity = self.evaluate_complexity(canvas_data);
        let rendering_level = self.detect_rendering_level(canvas_data);
        let hardware_accelerated = self.detect_hardware_acceleration(canvas_data);

        Ok(CanvasFingerprint {
            pixel_data: canvas_data.to_string(),
            hash,
            complexity,
            rendering_level,
            hardware_accelerated,
            detected_browser,
            confidence,
        })
    }

    // / calculatefingerprinthash
    fn compute_hash(&self, canvas_data: &str) -> Result<String, CanvasError> {
        let mut hasher = DefaultHasher::new();
        canvas_data.hash(&mut hasher);
        let hash_value = hasher.finish();
        Ok(format!("{:x}", hash_value))
    }

    // / 从fingerprintlibrary中detect浏览器version
    fn detect_browser(&self, hash: &str) -> (Option<String>, f32) {
        self.profile_library.find_match(hash)
    }

    // / 评估 Canvas 复杂度
    fn evaluate_complexity(&self, canvas_data: &str) -> f32 {
        let length = canvas_data.len() as f32;
        let max_length = 100000.0;
        let base_complexity = (length / max_length).min(1.0);
        let unique_chars = canvas_data
            .chars()
            .collect::<std::collections::HashSet<_>>()
            .len() as f32;
        let diversity = unique_chars / 64.0;
        (base_complexity * 0.6 + diversity * 0.4).min(1.0)
    }

    // / detectrendering层级
    fn detect_rendering_level(&self, canvas_data: &str) -> RenderingLevel {
        if canvas_data.contains("webgl") || canvas_data.len() > 50000 {
            RenderingLevel::Hardware
        } else if canvas_data.len() > 20000 {
            RenderingLevel::Software
        } else {
            RenderingLevel::Unknown
        }
    }

    // / detect硬件加速state
    fn detect_hardware_acceleration(&self, canvas_data: &str) -> bool {
        canvas_data.len() > 30000
    }
}

impl Default for CanvasAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

// / Canvas fingerprintconfigurefilelibrary
pub struct CanvasProfileLibrary {
    profiles: HashMap<String, CanvasProfile>,
}

// / Canvas configurefile
#[derive(Debug, Clone)]
struct CanvasProfile {
    browser: String,
    version: String,
    hash: String,
    #[allow(dead_code)]
    weight: f32,
}

impl CanvasProfileLibrary {
    // / createnewconfigurefilelibrary
    pub fn new() -> Self {
        let mut profiles = HashMap::new();
        Self::load_builtin_profiles(&mut profiles);
        CanvasProfileLibrary { profiles }
    }

    // / 从fingerprintlibrary中查找匹配
    fn find_match(&self, hash: &str) -> (Option<String>, f32) {
        if let Some(profile) = self.profiles.get(hash) {
            return (
                Some(format!("{} {}", profile.browser, profile.version)),
                1.0,
            );
        }

        let mut best_match = None;
        let mut best_confidence = 0.0;

        for profile in self.profiles.values() {
            let confidence = Self::calculate_similarity(hash, &profile.hash);
            if confidence > best_confidence {
                best_confidence = confidence;
                best_match = Some(format!("{} {}", profile.browser, profile.version));
            }
        }

        if best_confidence > 0.8 {
            (best_match, best_confidence)
        } else {
            (None, 0.0)
        }
    }

    // / calculate两个hash值之间ofsimilarity
    fn calculate_similarity(hash1: &str, hash2: &str) -> f32 {
        if hash1 == hash2 {
            return 1.0;
        }

        let matches = hash1
            .chars()
            .zip(hash2.chars())
            .filter(|(a, b)| a == b)
            .count();

        let max_len = hash1.len().max(hash2.len());
        matches as f32 / max_len as f32
    }

    // / load内置of浏览器configure
    fn load_builtin_profiles(profiles: &mut HashMap<String, CanvasProfile>) {
        profiles.insert(
            "a1b2c3d4e5f6g7h8".to_string(),
            CanvasProfile {
                browser: "Chrome".to_string(),
                version: "120".to_string(),
                hash: "a1b2c3d4e5f6g7h8".to_string(),
                weight: 1.0,
            },
        );

        profiles.insert(
            "b2c3d4e5f6g7h8i9".to_string(),
            CanvasProfile {
                browser: "Firefox".to_string(),
                version: "121".to_string(),
                hash: "b2c3d4e5f6g7h8i9".to_string(),
                weight: 1.0,
            },
        );

        profiles.insert(
            "c3d4e5f6g7h8i9j0".to_string(),
            CanvasProfile {
                browser: "Safari".to_string(),
                version: "17".to_string(),
                hash: "c3d4e5f6g7h8i9j0".to_string(),
                weight: 1.0,
            },
        );

        profiles.insert(
            "d4e5f6g7h8i9j0k1".to_string(),
            CanvasProfile {
                browser: "Edge".to_string(),
                version: "120".to_string(),
                hash: "d4e5f6g7h8i9j0k1".to_string(),
                weight: 1.0,
            },
        );
    }

    // / 添加customconfigurefile
    pub fn add_profile(&mut self, hash: String, browser: String, version: String) {
        self.profiles.insert(
            hash.clone(),
            CanvasProfile {
                browser,
                version,
                hash,
                weight: 1.0,
            },
        );
    }

    // / getconfigurefilecount
    pub fn profile_count(&self) -> usize {
        self.profiles.len()
    }
}

impl Default for CanvasProfileLibrary {
    fn default() -> Self {
        Self::new()
    }
}

// / Canvas 混淆器
pub struct CanvasObfuscator;

impl CanvasObfuscator {
    // / 对 Canvas data进行混淆
    pub fn obfuscate(canvas_data: &str, noise_level: f32) -> String {
        if noise_level <= 0.0 {
            return canvas_data.to_string();
        }

        let noise_ratio = noise_level.min(1.0);
        let mut result = canvas_data.to_string();
        let bytes_to_modify = (result.len() as f32 * noise_ratio) as usize;

        for i in 0..bytes_to_modify {
            let pos = (i * 7) % result.len();
            let chars: Vec<char> = result.chars().collect();
            if pos < chars.len() {
                let new_char = match chars[pos] {
                    'A'..='Z' => ((chars[pos] as u8 - b'A' + 1) % 26 + b'A') as char,
                    'a'..='z' => ((chars[pos] as u8 - b'a' + 1) % 26 + b'a') as char,
                    '0'..='9' => ((chars[pos] as u8 - b'0' + 1) % 10 + b'0') as char,
                    _ => chars[pos],
                };
                result = format!(
                    "{}{}{}",
                    &result[..pos],
                    new_char,
                    &result[pos + new_char.len_utf8()..]
                );
            }
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_canvas_analyzer_creation() {
        let analyzer = CanvasAnalyzer::new();
        assert_eq!(analyzer.profile_library.profile_count(), 4);
    }

    #[test]
    fn test_canvas_fingerprint_analysis() {
        let analyzer = CanvasAnalyzer::new();
        let result = analyzer.analyze("a1b2c3d4e5f6g7h8");
        assert!(result.is_ok());
    }

    #[test]
    fn test_invalid_canvas_data() {
        let analyzer = CanvasAnalyzer::new();
        let result = analyzer.analyze("");
        assert!(result.is_err());
    }
}
