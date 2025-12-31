//! 网络流量抽象
//!
//! 定义系统级别的网络流量，包含完整的上下文和指纹信息。

use super::context::SystemContext;
use crate::fingerprint::Fingerprint;
use std::time::Duration;

/// 流量特征
///
/// 描述网络流量的统计特征和行为模式。
#[derive(Debug, Clone, PartialEq)]
pub struct FlowCharacteristics {
    /// 数据包数量
    pub packet_count: u64,

    /// 总字节数
    pub total_bytes: u64,

    /// 持续时间
    pub duration: Duration,

    /// 是否加密
    pub encrypted: bool,

    /// 平均数据包大小
    pub avg_packet_size: f64,

    /// 数据包速率（包/秒）
    pub packet_rate: f64,

    /// 字节速率（字节/秒）
    pub byte_rate: f64,
}

impl FlowCharacteristics {
    /// 创建新的流量特征
    pub fn new() -> Self {
        Self {
            packet_count: 0,
            total_bytes: 0,
            duration: Duration::ZERO,
            encrypted: false,
            avg_packet_size: 0.0,
            packet_rate: 0.0,
            byte_rate: 0.0,
        }
    }

    /// 更新统计数据
    pub fn update(&mut self, packet_size: usize) {
        self.packet_count += 1;
        self.total_bytes += packet_size as u64;
        self.avg_packet_size = self.total_bytes as f64 / self.packet_count as f64;

        // 如果 duration 不为零，计算速率
        if !self.duration.is_zero() {
            let secs = self.duration.as_secs_f64();
            self.packet_rate = self.packet_count as f64 / secs;
            self.byte_rate = self.total_bytes as f64 / secs;
        }
    }

    /// 设置持续时间并更新速率
    pub fn set_duration(&mut self, duration: Duration) {
        self.duration = duration;
        if !duration.is_zero() {
            let secs = duration.as_secs_f64();
            self.packet_rate = self.packet_count as f64 / secs;
            self.byte_rate = self.total_bytes as f64 / secs;
        }
    }
}

impl Default for FlowCharacteristics {
    fn default() -> Self {
        Self::new()
    }
}

/// 网络流量
///
/// 表示系统级别的网络流量，包含完整的上下文、指纹信息和特征。
///
/// ## 核心思想
///
/// 系统级别防护需要从**网络流量**的角度进行分析和防护，而不是仅仅关注单个服务：
/// - 完整的系统上下文（源/目标、协议、方向等）
/// - 检测到的指纹信息（TLS、HTTP、TCP等）
/// - 流量的统计特征和行为模式
///
/// ## 示例
///
/// ```rust
/// use fingerprint_core::system::{NetworkFlow, SystemContext, ProtocolType};
///
/// let ctx = SystemContext::new(
///     "192.168.1.100".parse().unwrap(),
///     "10.0.0.1".parse().unwrap(),
///     ProtocolType::Http,
/// );
///
/// let flow = NetworkFlow::new(ctx);
/// ```
pub struct NetworkFlow {
    /// 系统上下文
    pub context: SystemContext,

    /// 检测到的指纹列表（如果有）
    /// 注意：由于 trait object 的限制，这里不能直接 Clone，需要手动处理
    #[cfg_attr(test, allow(dead_code))]
    fingerprints: Vec<Box<dyn Fingerprint>>,

    /// 流量特征
    pub characteristics: FlowCharacteristics,
}

impl NetworkFlow {
    /// 创建新的网络流量
    pub fn new(context: SystemContext) -> Self {
        Self {
            context,
            fingerprints: Vec::new(),
            characteristics: FlowCharacteristics::new(),
        }
    }

    /// 添加指纹
    pub fn add_fingerprint(&mut self, fingerprint: Box<dyn Fingerprint>) {
        self.fingerprints.push(fingerprint);
    }

    /// 检查是否有指纹
    pub fn has_fingerprints(&self) -> bool {
        !self.fingerprints.is_empty()
    }

    /// 获取所有指纹的引用
    pub fn fingerprints(&self) -> &[Box<dyn Fingerprint>] {
        &self.fingerprints
    }

    /// 获取指定类型的指纹
    pub fn get_fingerprints_by_type(
        &self,
        fingerprint_type: crate::fingerprint::FingerprintType,
    ) -> Vec<&dyn Fingerprint> {
        self.fingerprints
            .iter()
            .filter(|f| f.fingerprint_type() == fingerprint_type)
            .map(|f| f.as_ref())
            .collect()
    }

    /// 更新流量特征
    pub fn update_characteristics(&mut self, packet_size: usize) {
        self.characteristics.update(packet_size);
    }

    /// 获取流量的唯一标识符
    pub fn flow_id(&self) -> String {
        self.context.flow_id()
    }
}

// 手动实现 Debug，因为 Box<dyn Fingerprint> 不能自动实现 Debug
impl std::fmt::Debug for NetworkFlow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NetworkFlow")
            .field("context", &self.context)
            .field("fingerprints_count", &self.fingerprints.len())
            .field("characteristics", &self.characteristics)
            .finish()
    }
}

// 手动实现 Clone，因为 Box<dyn Fingerprint> 不能自动 Clone
impl Clone for NetworkFlow {
    fn clone(&self) -> Self {
        // 注意：fingerprints 不能 Clone，所以新实例从空列表开始
        // 这是合理的，因为指纹通常不应该被复制，而是通过引用共享
        Self {
            context: self.context.clone(),
            fingerprints: Vec::new(), // 不能 Clone trait object
            characteristics: self.characteristics.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flow_characteristics() {
        let mut chars = FlowCharacteristics::new();
        chars.update(1024);
        chars.update(2048);

        assert_eq!(chars.packet_count, 2);
        assert_eq!(chars.total_bytes, 3072);
        assert_eq!(chars.avg_packet_size, 1536.0);
    }

    #[test]
    fn test_network_flow() {
        use crate::system::context::ProtocolType;

        let ctx = SystemContext::new(
            "192.168.1.100".parse().unwrap(),
            "10.0.0.1".parse().unwrap(),
            ProtocolType::Http,
        );

        let flow = NetworkFlow::new(ctx);
        assert!(!flow.has_fingerprints());
        assert_eq!(flow.characteristics.packet_count, 0);
    }
}
