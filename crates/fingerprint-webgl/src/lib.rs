//! # fingerprint-webgl
//!
//! WebGL 指纹识别模块
//!
//! 提供 GPU 和 WebGL 指纹识别能力

use std::collections::HashMap;

/// WebGL 指纹信息
#[derive(Debug, Clone)]
pub struct WebGLFingerprint {
    pub gpu_vendor: String,
    pub gpu_renderer: String,
    pub extensions: Vec<String>,
    pub shader_fingerprint: String,
    pub precision_high: bool,
    pub max_texture_size: u32,
}

/// WebGL 分析器
pub struct WebGLAnalyzer {
    profiles: HashMap<String, WebGLProfile>,
}

#[derive(Debug, Clone)]
struct WebGLProfile {
    gpu_vendor: String,
    gpu_renderer: String,
}

impl WebGLAnalyzer {
    /// 创建新分析器
    pub fn new() -> Self {
        let mut profiles = HashMap::new();

        // 预加载常见 GPU 配置
        profiles.insert("ANGLE".to_string(), WebGLProfile {
            gpu_vendor: "Google".to_string(),
            gpu_renderer: "Angle".to_string(),
        });

        profiles.insert("Apple".to_string(), WebGLProfile {
            gpu_vendor: "Apple".to_string(),
            gpu_renderer: "Metal".to_string(),
        });

        WebGLAnalyzer { profiles }
    }

    /// 分析 WebGL 数据
    pub fn analyze(&self, vendor: &str, renderer: &str, extensions: &[&str]) -> WebGLFingerprint {
        let shader_fp = self.compute_shader_fingerprint(extensions);

        WebGLFingerprint {
            gpu_vendor: vendor.to_string(),
            gpu_renderer: renderer.to_string(),
            extensions: extensions.iter().map(|s| s.to_string()).collect(),
            shader_fingerprint: shader_fp,
            precision_high: true,
            max_texture_size: 16384,
        }
    }

    fn compute_shader_fingerprint(&self, extensions: &[&str]) -> String {
        let mut hash = String::new();
        for ext in extensions {
            hash.push_str(ext);
        }
        format!("{:x}", hash.len())
    }
}

impl Default for WebGLAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_webgl_analyzer() {
        let analyzer = WebGLAnalyzer::new();
        let fp = analyzer.analyze("NVIDIA", "GeForce", &["OES_texture_float"]);
        assert_eq!(fp.gpu_vendor, "NVIDIA");
    }
}
