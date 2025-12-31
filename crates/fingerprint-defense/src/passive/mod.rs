//! 被动指纹识别模块
//!
//! 实现 p0f 风格的被动指纹识别，包括 TCP、HTTP、TLS 分析。

pub mod consistency;
pub mod http;
pub mod p0f;
pub mod p0f_parser;
pub mod packet;
pub mod tcp;
pub mod tls;

pub use consistency::ConsistencyAnalyzer;

pub use http::{HttpAnalyzer, HttpFingerprint};
pub use packet::{Packet, PacketParser};
pub use tcp::{TcpAnalyzer, TcpFeatures, TcpFingerprint};
pub use tls::{TlsAnalyzer, TlsFingerprint};

// 使用 core 中的系统级别抽象
use fingerprint_core::system::{NetworkFlow, ProtocolType, SystemContext, TrafficDirection};

/// 被动分析器（多协议）
pub struct PassiveAnalyzer {
    tcp_analyzer: TcpAnalyzer,
    http_analyzer: HttpAnalyzer,
    tls_analyzer: TlsAnalyzer,
}

impl PassiveAnalyzer {
    /// 创建新的被动分析器
    pub fn new() -> Result<Self, PassiveError> {
        Ok(Self {
            tcp_analyzer: TcpAnalyzer::new().map_err(PassiveError::Tcp)?,
            http_analyzer: HttpAnalyzer::new().map_err(PassiveError::Http)?,
            tls_analyzer: TlsAnalyzer::new().map_err(PassiveError::Tls)?,
        })
    }

    /// 分析数据包
    pub fn analyze(&self, packet: &Packet) -> AnalysisResult {
        let mut result = AnalysisResult::default();

        // TCP 分析
        if let Some(tcp_result) = self.tcp_analyzer.analyze(packet) {
            result.tcp = Some(tcp_result);
        }

        // HTTP 分析
        if let Some(http_result) = self.http_analyzer.analyze(packet) {
            result.http = Some(http_result);
        }

        // TLS 分析
        if let Some(tls_result) = self.tls_analyzer.analyze(packet) {
            result.tls = Some(tls_result);
        }

        result
    }

    /// 分析数据包并返回 NetworkFlow（新方法，用于系统级别防护）
    pub fn analyze_to_flow(&self, packet: &Packet) -> Result<NetworkFlow, PassiveError> {
        // 1. 确定协议类型
        let protocol = match (
            packet.tcp_header.is_some(),
            packet.src_port,
            packet.dst_port,
        ) {
            (true, 80, _) | (true, _, 80) => ProtocolType::Http,
            (true, 443, _) | (true, _, 443) => ProtocolType::Https,
            (true, _, _) => ProtocolType::Tcp,
            (false, 53, _) | (false, _, 53) => ProtocolType::Udp, // 简单 DNS 识别
            (false, _, _) if packet.src_port > 0 || packet.dst_port > 0 => ProtocolType::Udp,
            _ => ProtocolType::Icmp,
        };

        // 2. 创建 SystemContext
        let mut context = SystemContext::with_ports(
            packet.src_ip,
            packet.dst_ip,
            packet.src_port,
            packet.dst_port,
            protocol,
        );

        // 设置其他上下文信息
        context.timestamp = chrono::Utc::now();
        context.packet_size = packet.payload.len();

        // 智能方向识别：如果是私有地址发往公网，通常是 Outbound；反之是 Inbound
        // 这里的逻辑可以根据部署环境（网关 vs 终端）进一步微调
        let src_is_local = match packet.src_ip {
            std::net::IpAddr::V4(ip) => ip.is_loopback() || ip.is_private(),
            std::net::IpAddr::V6(ip) => ip.is_loopback(),
        };
        context.direction = if src_is_local {
            TrafficDirection::Outbound
        } else {
            TrafficDirection::Inbound
        };

        // 3. 调用原有的 analyze 方法获取指纹
        let analysis_result = self.analyze(packet);

        // 4. 创建 NetworkFlow
        let mut flow = NetworkFlow::new(context);

        // 5. 更新流量特征
        flow.update_characteristics(packet.payload.len());

        // 6. 填充指纹
        if let Some(tcp) = analysis_result.tcp {
            flow.add_fingerprint(Box::new(tcp));
        }
        if let Some(http) = analysis_result.http {
            flow.add_fingerprint(Box::new(http));
        }
        if let Some(tls) = analysis_result.tls {
            flow.add_fingerprint(Box::new(tls));
        }

        Ok(flow)
    }
}

/// 分析结果
#[derive(Debug, Clone, Default)]
pub struct AnalysisResult {
    pub tcp: Option<TcpFingerprint>,
    pub http: Option<HttpFingerprint>,
    pub tls: Option<TlsFingerprint>,
}

// 导出别名
pub use AnalysisResult as PassiveAnalysisResult;

/// 被动分析错误
#[derive(Debug, thiserror::Error)]
pub enum PassiveError {
    #[error("TCP 分析错误: {0}")]
    Tcp(String),

    #[error("HTTP 分析错误: {0}")]
    Http(String),

    #[error("TLS 分析错误: {0}")]
    Tls(String),

    #[error("数据包解析错误: {0}")]
    Packet(#[from] crate::passive::packet::PacketError),
}

impl Default for PassiveAnalyzer {
    fn default() -> Self {
        Self::new().expect("Failed to create PassiveAnalyzer")
    }
}
