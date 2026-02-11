//! # fingerprint-timing
//!
//! 时序攻击防护模块
//!
//! 提供时间戳一致性检查和时序侧信道防护

use std::time::{SystemTime, Duration};

/// 时序指纹
#[derive(Debug, Clone)]
pub struct TimingFingerprint {
    /// 时间戳
    pub timestamp: u64,
    /// 时间源精度
    pub precision: TimingPrecision,
    /// 时间漂移
    pub drift: i64,
    /// 高分辨率时间
    pub high_resolution_time: f64,
    /// 时间一致性分数
    pub consistency_score: f32,
}

/// 时间精度
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimingPrecision {
    /// 毫秒
    Millisecond,
    /// 微秒
    Microsecond,
    /// 纳秒
    Nanosecond,
    /// 低精度
    Low,
}

/// 时序错误类型
#[derive(Debug)]
pub enum TimingError {
    /// 无效时间
    InvalidTime,
    /// 检测失败
    DetectionFailed(String),
    /// 其他错误
    Other(String),
}

impl std::fmt::Display for TimingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TimingError::InvalidTime => write!(f, "Invalid timing data"),
            TimingError::DetectionFailed(msg) => write!(f, "Detection failed: {}", msg),
            TimingError::Other(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl std::error::Error for TimingError {}

/// 时序分析器
pub struct TimingAnalyzer {
    last_timestamp: u64,
}

impl TimingAnalyzer {
    /// 创建新的分析器
    pub fn new() -> Self {
        TimingAnalyzer {
            last_timestamp: Self::current_timestamp(),
        }
    }

    /// 获取当前时间戳
    fn current_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or(Duration::ZERO)
            .as_millis() as u64
    }

    /// 分析时序信息
    pub fn analyze(&mut self, high_res_time: f64) -> Result<TimingFingerprint, TimingError> {
        let current = Self::current_timestamp();
        let drift = (current as i64) - (self.last_timestamp as i64);
        self.last_timestamp = current;

        if drift < 0 {
            return Err(TimingError::InvalidTime);
        }

        let precision = Self::detect_precision(high_res_time);
        let consistency = Self::calculate_consistency(drift);

        Ok(TimingFingerprint {
            timestamp: current,
            precision,
            drift,
            high_resolution_time: high_res_time,
            consistency_score: consistency,
        })
    }

    /// 检测时间精度
    fn detect_precision(high_res_time: f64) -> TimingPrecision {
        if high_res_time > 1_000_000.0 {
            TimingPrecision::Nanosecond
        } else if high_res_time > 1000.0 {
            TimingPrecision::Microsecond
        } else if high_res_time > 1.0 {
            TimingPrecision::Millisecond
        } else {
            TimingPrecision::Low
        }
    }

    /// 计算时间一致性
    fn calculate_consistency(drift: i64) -> f32 {
        if drift < 0 {
            0.0
        } else if drift < 100 {
            1.0
        } else if drift < 500 {
            0.8
        } else if drift < 1000 {
            0.6
        } else {
            0.4
        }
    }
}

impl Default for TimingAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

/// 时序防护器
pub struct TimingProtection;

impl TimingProtection {
    /// 添加随机延迟
    pub fn add_random_delay(min_ms: u64, max_ms: u64) -> Duration {
        let delay = (min_ms + ((max_ms - min_ms) / 2)) as u64;
        Duration::from_millis(delay)
    }

    /// 隐藏时间分辨率
    pub fn obfuscate_time(timestamp: u64, granularity_ms: u64) -> u64 {
        (timestamp / granularity_ms) * granularity_ms
    }

    /// 检测时间异常
    pub fn detect_anomalies(timings: &[TimingFingerprint]) -> Vec<usize> {
        if timings.len() < 2 {
            return Vec::new();
        }

        let mut anomalies = Vec::new();
        let avg_drift: i64 = timings.iter().map(|t| t.drift).sum::<i64>() / timings.len() as i64;

        for (i, timing) in timings.iter().enumerate() {
            let deviation = (timing.drift - avg_drift).abs();
            if deviation > avg_drift * 2 {
                anomalies.push(i);
            }
        }

        anomalies
    }

    /// 标准化时间戳
    pub fn normalize_timestamps(timings: &mut [TimingFingerprint]) {
        if timings.is_empty() {
            return;
        }

        let min_timestamp = timings.iter().map(|t| t.timestamp).min().unwrap_or(0);

        for timing in timings.iter_mut() {
            timing.timestamp = timing.timestamp.saturating_sub(min_timestamp);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timing_analyzer() {
        let mut analyzer = TimingAnalyzer::new();
        let result = analyzer.analyze(1000.5);
        assert!(result.is_ok());
    }

    #[test]
    fn test_timing_obfuscation() {
        let ts = 123456789u64;
        let obfuscated = TimingProtection::obfuscate_time(ts, 100);
        assert_eq!(obfuscated, 123456700);
    }

    #[test]
    fn test_random_delay() {
        let delay = TimingProtection::add_random_delay(100, 200);
        assert!(delay.as_millis() >= 100 && delay.as_millis() <= 200);
    }
}
