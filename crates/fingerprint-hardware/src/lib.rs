#![allow(clippy::all, dead_code, unused_variables, unused_parens)]

//! # fingerprint-hardware
//!
//! 硬件指纹识别模块
//!
//! 提供 GPU、CPU、内存等硬件识别能力

/// 硬件指纹
#[derive(Debug, Clone)]
pub struct HardwareFingerprint {
    /// CPU 型号
    pub cpu_model: String,
    /// CPU 核心数
    pub cpu_cores: u32,
    /// GPU 型号
    pub gpu_model: String,
    /// GPU 内存
    pub gpu_memory_gb: u32,
    /// 系统内存 (GB)
    pub system_memory_gb: u64,
    /// 屏幕 DPI
    pub screen_dpi: f32,
    /// 屏幕分辨率
    pub screen_resolution: (u32, u32),
    /// 设备类型
    pub device_type: DeviceType,
}

/// 设备类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceType {
    /// 桌面
    Desktop,
    /// 笔记本
    Laptop,
    /// 平板
    Tablet,
    /// 手机
    Phone,
    /// 未知
    Unknown,
}

/// 硬件错误类型
#[derive(Debug)]
pub enum HardwareError {
    /// 无效数据
    InvalidData,
    /// 检测失败
    DetectionFailed(String),
    /// 其他错误
    Other(String),
}

impl std::fmt::Display for HardwareError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HardwareError::InvalidData => write!(f, "Invalid hardware data"),
            HardwareError::DetectionFailed(msg) => write!(f, "Detection failed: {}", msg),
            HardwareError::Other(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl std::error::Error for HardwareError {}

/// 硬件检测器
pub struct HardwareDetector;

impl HardwareDetector {
    /// 检测硬件信息
    pub fn detect(
        cpu_cores: u32,
        gpu_model: &str,
        system_memory_gb: u64,
        screen_dpi: f32,
        screen_width: u32,
        screen_height: u32,
    ) -> Result<HardwareFingerprint, HardwareError> {
        if system_memory_gb == 0 || cpu_cores == 0 {
            return Err(HardwareError::InvalidData);
        }

        let cpu_model = Self::identify_cpu(cpu_cores);
        let device_type = Self::identify_device_type(screen_width, screen_height, system_memory_gb);
        let gpu_memory = Self::estimate_gpu_memory(gpu_model);

        Ok(HardwareFingerprint {
            cpu_model,
            cpu_cores,
            gpu_model: gpu_model.to_string(),
            gpu_memory_gb: gpu_memory,
            system_memory_gb,
            screen_dpi,
            screen_resolution: (screen_width, screen_height),
            device_type,
        })
    }

    /// 识别 CPU 型号
    fn identify_cpu(cores: u32) -> String {
        match cores {
            1..=2 => "Intel Core i5".to_string(),
            3..=4 => "Intel Core i7".to_string(),
            5..=8 => "Intel Core i9".to_string(),
            _ => format!("CPU ({} cores)", cores),
        }
    }

    /// 识别设备类型
    fn identify_device_type(width: u32, height: u32, memory: u64) -> DeviceType {
        match (width, height, memory) {
            (w, h, _) if (w <= 768 && h <= 1024) || (w <= 1024 && h <= 768) => DeviceType::Tablet,
            (w, h, _) if (w <= 480 && h <= 960) || (w <= 540 && h <= 960) => DeviceType::Phone,
            (_, _, m) if m > 16 => DeviceType::Desktop,
            _ => DeviceType::Laptop,
        }
    }

    /// 估计 GPU 内存
    fn estimate_gpu_memory(gpu_model: &str) -> u32 {
        match gpu_model {
            gpu if gpu.contains("RTX 4090") => 24,
            gpu if gpu.contains("RTX 4080") => 16,
            gpu if gpu.contains("RTX 4070") => 12,
            gpu if gpu.contains("M1") || gpu.contains("M2") => 8,
            gpu if gpu.contains("Intel") => 2,
            _ => 4,
        }
    }
}

/// 硬件指纹匹配器
pub struct HardwareProfileMatcher;

impl HardwareProfileMatcher {
    /// 匹配硬件配置
    pub fn match_profile(hardware: &HardwareFingerprint) -> Option<String> {
        match (hardware.cpu_cores, hardware.device_type) {
            (4, DeviceType::Laptop) => Some("MacBook Pro".to_string()),
            (8, DeviceType::Desktop) => Some("Gaming PC".to_string()),
            (2, DeviceType::Tablet) => Some("iPad".to_string()),
            _ => None,
        }
    }

    /// 计算硬件相似度
    pub fn calculate_similarity(hw1: &HardwareFingerprint, hw2: &HardwareFingerprint) -> f32 {
        let mut score = 0.0;
        if hw1.cpu_cores == hw2.cpu_cores {
            score += 0.2;
        }
        if hw1.gpu_model == hw2.gpu_model {
            score += 0.2;
        }
        if hw1.system_memory_gb == hw2.system_memory_gb {
            score += 0.2;
        }
        if hw1.device_type == hw2.device_type {
            score += 0.2;
        }
        if (hw1.screen_dpi - hw2.screen_dpi).abs() < 10.0 {
            score += 0.2;
        }
        score
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hardware_detection() {
        let result = HardwareDetector::detect(4, "RTX 4070", 16, 96.0, 1920, 1080);
        assert!(result.is_ok());
        let hw = result.unwrap();
        assert_eq!(hw.cpu_cores, 4);
        assert_eq!(hw.system_memory_gb, 16);
    }

    #[test]
    fn test_device_type_identification() {
        let result = HardwareDetector::detect(2, "Intel UHD", 8, 72.0, 480, 854);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().device_type, DeviceType::Phone);
    }

    #[test]
    fn test_gpu_memory_estimation() {
        assert_eq!(HardwareDetector::estimate_gpu_memory("RTX 4090"), 24);
        assert_eq!(HardwareDetector::estimate_gpu_memory("M1"), 8);
    }
}
