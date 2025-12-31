//! 系统上下文
//!
//! 定义系统级别防护的上下文信息，包括网络实体、时间、协议等。

use chrono::{DateTime, Utc};
use std::net::IpAddr;

/// 流量方向
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TrafficDirection {
    /// 输入流量（进入系统）
    Inbound,

    /// 输出流量（离开系统）
    Outbound,

    /// 内部流量（系统内部）
    Internal,
}

impl TrafficDirection {
    /// 转换为字符串
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Inbound => "inbound",
            Self::Outbound => "outbound",
            Self::Internal => "internal",
        }
    }
}

impl std::fmt::Display for TrafficDirection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// 协议类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProtocolType {
    /// TCP 协议
    Tcp,

    /// UDP 协议
    Udp,

    /// ICMP 协议
    Icmp,

    /// HTTP 协议
    Http,

    /// HTTPS 协议（TLS over TCP）
    Https,

    /// 其他协议
    Other(u8),
}

impl ProtocolType {
    /// 从 IP 协议号创建
    pub fn from_ip_protocol(protocol: u8) -> Self {
        match protocol {
            6 => Self::Tcp,
            17 => Self::Udp,
            1 => Self::Icmp,
            other => Self::Other(other),
        }
    }

    /// 转换为 IP 协议号
    pub fn to_ip_protocol(&self) -> u8 {
        match self {
            Self::Tcp => 6,
            Self::Udp => 17,
            Self::Icmp => 1,
            Self::Http => 6,  // HTTP over TCP
            Self::Https => 6, // HTTPS over TCP
            Self::Other(p) => *p,
        }
    }

    /// 转换为字符串
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Tcp => "TCP",
            Self::Udp => "UDP",
            Self::Icmp => "ICMP",
            Self::Http => "HTTP",
            Self::Https => "HTTPS",
            Self::Other(_) => "Other",
        }
    }
}

impl std::fmt::Display for ProtocolType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Other(p) => write!(f, "Other({})", p),
            _ => write!(f, "{}", self.as_str()),
        }
    }
}

/// 系统上下文
///
/// 表示系统级别防护的上下文信息，包含网络流量的完整元数据。
///
/// ## 核心思想
///
/// 系统级别防护需要考虑完整的系统上下文，而不仅仅是单个服务或端口：
/// - 网络实体的完整信息（源/目标 IP、端口）
/// - 协议类型和方向
/// - 时间戳和网卡接口
/// - 数据包级别的信息
///
/// ## 示例
///
/// ```rust
/// use fingerprint_core::system::{SystemContext, ProtocolType, TrafficDirection};
/// use std::net::IpAddr;
/// use chrono::Utc;
///
/// let ctx = SystemContext {
///     source_ip: "192.168.1.100".parse().unwrap(),
///     target_ip: "10.0.0.1".parse().unwrap(),
///     source_port: Some(54321),
///     target_port: Some(80),
///     protocol: ProtocolType::Http,
///     timestamp: Utc::now(),
///     interface: Some("eth0".to_string()),
///     packet_size: 1024,
///     direction: TrafficDirection::Inbound,
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SystemContext {
    /// 源 IP 地址
    pub source_ip: IpAddr,

    /// 目标 IP 地址
    pub target_ip: IpAddr,

    /// 源端口（对于 UDP/TCP）
    pub source_port: Option<u16>,

    /// 目标端口（对于 UDP/TCP）
    pub target_port: Option<u16>,

    /// 协议类型
    pub protocol: ProtocolType,

    /// 时间戳
    pub timestamp: DateTime<Utc>,

    /// 网卡接口名称
    pub interface: Option<String>,

    /// 数据包大小（字节）
    pub packet_size: usize,

    /// 流量方向（输入/输出）
    pub direction: TrafficDirection,
}

impl SystemContext {
    /// 创建新的系统上下文
    pub fn new(source_ip: IpAddr, target_ip: IpAddr, protocol: ProtocolType) -> Self {
        Self {
            source_ip,
            target_ip,
            source_port: None,
            target_port: None,
            protocol,
            timestamp: Utc::now(),
            interface: None,
            packet_size: 0,
            direction: TrafficDirection::Inbound,
        }
    }

    /// 创建带端口的系统上下文
    pub fn with_ports(
        source_ip: IpAddr,
        target_ip: IpAddr,
        source_port: u16,
        target_port: u16,
        protocol: ProtocolType,
    ) -> Self {
        Self {
            source_ip,
            target_ip,
            source_port: Some(source_port),
            target_port: Some(target_port),
            protocol,
            timestamp: Utc::now(),
            interface: None,
            packet_size: 0,
            direction: TrafficDirection::Inbound,
        }
    }

    /// 判断是否为本地流量（源或目标为本地地址）
    pub fn is_local(&self) -> bool {
        self.is_source_local() || self.is_target_local()
    }

    /// 判断源地址是否为本地地址
    pub fn is_source_local(&self) -> bool {
        match self.source_ip {
            IpAddr::V4(ip) => ip.is_loopback() || ip.is_private() || ip.is_link_local(),
            IpAddr::V6(ip) => ip.is_loopback() || ip.is_unspecified(),
        }
    }

    /// 判断目标地址是否为本地地址
    pub fn is_target_local(&self) -> bool {
        match self.target_ip {
            IpAddr::V4(ip) => ip.is_loopback() || ip.is_private() || ip.is_link_local(),
            IpAddr::V6(ip) => ip.is_loopback() || ip.is_unspecified(),
        }
    }

    /// 获取流量的唯一标识符（用于追踪）
    pub fn flow_id(&self) -> String {
        format!(
            "{}:{}->{}:{}:{}",
            self.source_ip,
            self.source_port.map(|p| p.to_string()).unwrap_or_default(),
            self.target_ip,
            self.target_port.map(|p| p.to_string()).unwrap_or_default(),
            self.protocol.as_str()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_traffic_direction() {
        assert_eq!(TrafficDirection::Inbound.as_str(), "inbound");
        assert_eq!(TrafficDirection::Outbound.as_str(), "outbound");
    }

    #[test]
    fn test_protocol_type() {
        assert_eq!(ProtocolType::from_ip_protocol(6), ProtocolType::Tcp);
        assert_eq!(ProtocolType::from_ip_protocol(17), ProtocolType::Udp);
        assert_eq!(ProtocolType::Tcp.to_ip_protocol(), 6);
    }

    #[test]
    fn test_system_context() {
        let ctx = SystemContext::with_ports(
            "192.168.1.100".parse().unwrap(),
            "10.0.0.1".parse().unwrap(),
            54321,
            80,
            ProtocolType::Http,
        );

        assert_eq!(ctx.source_port, Some(54321));
        assert_eq!(ctx.target_port, Some(80));
        assert!(ctx.flow_id().contains("192.168.1.100"));
    }

    #[test]
    fn test_is_local() {
        let local_ctx = SystemContext::new(
            "127.0.0.1".parse().unwrap(),
            "192.168.1.1".parse().unwrap(),
            ProtocolType::Tcp,
        );
        assert!(local_ctx.is_source_local());
        assert!(local_ctx.is_local());
    }
}
