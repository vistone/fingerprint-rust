//! 系统级别统计信息
//!
//! 定义系统级别防护的统计信息。

use std::time::Instant;

/// 系统级别防护统计信息
///
/// 记录系统级别防护系统的运行统计信息。
#[derive(Debug, Clone)]
pub struct SystemProtectionStats {
    /// 总数据包数
    pub total_packets: u64,

    /// 已分析数据包数
    pub analyzed_packets: u64,

    /// 已阻止数据包数
    pub blocked_packets: u64,

    /// 限速数据包数
    pub rate_limited_packets: u64,

    /// 允许通过数据包数
    pub allowed_packets: u64,

    /// 检测到的威胁数
    pub threat_detected: u64,

    /// 启动时间
    pub start_time: Instant,

    /// 最后更新时间
    pub last_update_time: Instant,
}

impl SystemProtectionStats {
    /// 创建新的统计信息
    pub fn new() -> Self {
        let now = Instant::now();
        Self {
            total_packets: 0,
            analyzed_packets: 0,
            blocked_packets: 0,
            rate_limited_packets: 0,
            allowed_packets: 0,
            threat_detected: 0,
            start_time: now,
            last_update_time: now,
        }
    }

    /// 增加总数据包数
    pub fn increment_total(&mut self) {
        self.total_packets += 1;
        self.last_update_time = Instant::now();
    }

    /// 增加已分析数据包数
    pub fn increment_analyzed(&mut self) {
        self.analyzed_packets += 1;
        self.last_update_time = Instant::now();
    }

    /// 增加已阻止数据包数
    pub fn increment_blocked(&mut self) {
        self.blocked_packets += 1;
        self.last_update_time = Instant::now();
    }

    /// 增加限速数据包数
    pub fn increment_rate_limited(&mut self) {
        self.rate_limited_packets += 1;
        self.last_update_time = Instant::now();
    }

    /// 增加允许通过数据包数
    pub fn increment_allowed(&mut self) {
        self.allowed_packets += 1;
        self.last_update_time = Instant::now();
    }

    /// 增加威胁检测数
    pub fn increment_threat(&mut self) {
        self.threat_detected += 1;
        self.last_update_time = Instant::now();
    }

    /// 获取运行时间（秒）
    pub fn uptime_seconds(&self) -> u64 {
        self.start_time.elapsed().as_secs()
    }

    /// 获取数据包处理速率（包/秒）
    pub fn packets_per_second(&self) -> f64 {
        let uptime = self.uptime_seconds() as f64;
        if uptime > 0.0 {
            self.total_packets as f64 / uptime
        } else {
            0.0
        }
    }

    /// 获取分析率（已分析/总数）
    pub fn analysis_rate(&self) -> f64 {
        if self.total_packets > 0 {
            self.analyzed_packets as f64 / self.total_packets as f64
        } else {
            0.0
        }
    }

    /// 获取阻止率（已阻止/总数）
    pub fn block_rate(&self) -> f64 {
        if self.total_packets > 0 {
            self.blocked_packets as f64 / self.total_packets as f64
        } else {
            0.0
        }
    }

    /// 重置统计信息
    pub fn reset(&mut self) {
        self.total_packets = 0;
        self.analyzed_packets = 0;
        self.blocked_packets = 0;
        self.rate_limited_packets = 0;
        self.allowed_packets = 0;
        self.threat_detected = 0;
        self.start_time = Instant::now();
        self.last_update_time = Instant::now();
    }
}

impl Default for SystemProtectionStats {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stats_new() {
        let stats = SystemProtectionStats::new();
        assert_eq!(stats.total_packets, 0);
        assert_eq!(stats.analyzed_packets, 0);
    }

    #[test]
    fn test_stats_increment() {
        let mut stats = SystemProtectionStats::new();
        stats.increment_total();
        stats.increment_analyzed();
        stats.increment_blocked();

        assert_eq!(stats.total_packets, 1);
        assert_eq!(stats.analyzed_packets, 1);
        assert_eq!(stats.blocked_packets, 1);
    }

    #[test]
    fn test_stats_rates() {
        let mut stats = SystemProtectionStats::new();
        stats.increment_total();
        stats.increment_total();
        stats.increment_analyzed();
        stats.increment_blocked();

        assert_eq!(stats.analysis_rate(), 0.5);
        assert_eq!(stats.block_rate(), 0.5);
    }

    #[test]
    fn test_stats_reset() {
        let mut stats = SystemProtectionStats::new();
        stats.increment_total();
        stats.increment_analyzed();
        stats.reset();

        assert_eq!(stats.total_packets, 0);
        assert_eq!(stats.analyzed_packets, 0);
    }
}
