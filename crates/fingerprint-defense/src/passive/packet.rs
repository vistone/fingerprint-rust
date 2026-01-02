//! 数据包解析模块
//!
//! 提供底层数据包解析功能。

use bytes::Bytes;
use std::net::IpAddr;

/// 数据包
#[derive(Debug, Clone)]
pub struct Packet {
    /// 时间戳
    pub timestamp: u64,

    /// 源 IP
    pub src_ip: IpAddr,

    /// 目标 IP
    pub dst_ip: IpAddr,

    /// 源端口
    pub src_port: u16,

    /// 目标端口
    pub dst_port: u16,

    /// IP 协议版本
    pub ip_version: u8,

    /// TTL
    pub ttl: u8,

    /// IP 标志
    pub ip_flags: u8,

    /// 数据包负载
    pub payload: Bytes,

    /// TCP 头部信息（如果有）
    pub tcp_header: Option<TcpHeader>,
}

/// TCP 头部信息
#[derive(Debug, Clone)]
pub struct TcpHeader {
    /// 序列号
    pub seq: u32,

    /// 确认号
    pub ack: Option<u32>,

    /// 窗口大小
    pub window: u16,

    /// TCP 标志
    pub flags: u8,

    /// TCP 选项
    pub options: Vec<TcpOption>,
}

/// TCP 选项
#[derive(Debug, Clone)]
pub struct TcpOption {
    pub kind: u8,
    pub data: Vec<u8>,
}

/// 数据包解析器
pub struct PacketParser;

impl PacketParser {
    /// 从原始数据包解析
    pub fn parse(raw_packet: &[u8]) -> Result<Packet, PacketError> {
        // 解析 IP 头部
        if raw_packet.len() < 20 {
            return Err(PacketError::TooShort);
        }

        let version = (raw_packet[0] >> 4) & 0x0F;

        match version {
            4 => Self::parse_ipv4(raw_packet),
            6 => Self::parse_ipv6(raw_packet),
            _ => Err(PacketError::InvalidVersion),
        }
    }

    /// 解析 IPv4 数据包
    fn parse_ipv4(raw_packet: &[u8]) -> Result<Packet, PacketError> {
        if raw_packet.len() < 20 {
            return Err(PacketError::TooShort);
        }

        let ihl = (raw_packet[0] & 0x0F) as usize;
        
        // 安全检查：IHL 必须至少为 5（20 字节），最多为 15（60 字节）
        if ihl < 5 || ihl > 15 {
            return Err(PacketError::Other("无效的 IHL 值".to_string()));
        }
        
        let header_len = ihl * 4;
        
        // 安全检查：确保数据包足够长
        if raw_packet.len() < header_len {
            return Err(PacketError::TooShort);
        }
        
        let _total_length = u16::from_be_bytes([raw_packet[2], raw_packet[3]]) as usize;
        let ttl = raw_packet[8];
        let protocol = raw_packet[9];
        let ip_flags = (raw_packet[6] >> 5) & 0x07;

        let src_ip = IpAddr::from([
            raw_packet[12],
            raw_packet[13],
            raw_packet[14],
            raw_packet[15],
        ]);

        let dst_ip = IpAddr::from([
            raw_packet[16],
            raw_packet[17],
            raw_packet[18],
            raw_packet[19],
        ]);

        let payload = Bytes::copy_from_slice(&raw_packet[header_len..]);

        // 根据协议类型解析
        let (src_port, dst_port, tcp_header) = match protocol {
            6 => {
                // TCP
                Self::parse_tcp(&payload)?
            }
            17 => {
                // UDP
                Self::parse_udp(&payload)?
            }
            1 => {
                // ICMP
                Self::parse_icmp(&payload)?
            }
            _ => {
                // 其他协议
                (0, 0, None)
            }
        };

        Ok(Packet {
            timestamp: 0, // TODO: 从 pcap 获取
            src_ip,
            dst_ip,
            src_port,
            dst_port,
            ip_version: 4,
            ttl,
            ip_flags,
            payload,
            tcp_header,
        })
    }

    /// 解析 IPv6 数据包
    fn parse_ipv6(raw_packet: &[u8]) -> Result<Packet, PacketError> {
        if raw_packet.len() < 40 {
            return Err(PacketError::TooShort);
        }

        // IPv6 头部固定 40 字节
        // 版本(4位) + 流量类别(8位) + 流标签(20位) = 前 4 字节
        let version = (raw_packet[0] >> 4) & 0x0F;
        if version != 6 {
            return Err(PacketError::InvalidVersion);
        }

        // 负载长度（16位，字节 4-5）
        let payload_length = u16::from_be_bytes([raw_packet[4], raw_packet[5]]) as usize;

        // 下一个头部（协议类型，字节 6）
        let next_header = raw_packet[6];

        // 跳数限制（TTL，字节 7）
        let hop_limit = raw_packet[7];

        // 源地址（128位，字节 8-23）
        let src_ip = IpAddr::from([
            raw_packet[8],
            raw_packet[9],
            raw_packet[10],
            raw_packet[11],
            raw_packet[12],
            raw_packet[13],
            raw_packet[14],
            raw_packet[15],
            raw_packet[16],
            raw_packet[17],
            raw_packet[18],
            raw_packet[19],
            raw_packet[20],
            raw_packet[21],
            raw_packet[22],
            raw_packet[23],
        ]);

        // 目标地址（128位，字节 24-39）
        let dst_ip = IpAddr::from([
            raw_packet[24],
            raw_packet[25],
            raw_packet[26],
            raw_packet[27],
            raw_packet[28],
            raw_packet[29],
            raw_packet[30],
            raw_packet[31],
            raw_packet[32],
            raw_packet[33],
            raw_packet[34],
            raw_packet[35],
            raw_packet[36],
            raw_packet[37],
            raw_packet[38],
            raw_packet[39],
        ]);

        // 负载数据（从字节 40 开始）
        let payload_start = 40;
        let payload_end = (payload_start + payload_length).min(raw_packet.len());
        let payload = Bytes::copy_from_slice(&raw_packet[payload_start..payload_end]);

        // 根据下一个头部协议解析
        let (src_port, dst_port, tcp_header) = match next_header {
            6 => {
                // TCP over IPv6
                Self::parse_tcp(&payload)?
            }
            17 => {
                // UDP over IPv6
                Self::parse_udp(&payload)?
            }
            58 => {
                // ICMPv6
                Self::parse_icmp(&payload)?
            }
            _ => {
                // 其他协议
                (0, 0, None)
            }
        };

        Ok(Packet {
            timestamp: 0,
            src_ip,
            dst_ip,
            src_port,
            dst_port,
            ip_version: 6,
            ttl: hop_limit,
            ip_flags: 0,
            payload,
            tcp_header,
        })
    }

    /// 解析 UDP 头部
    fn parse_udp(data: &[u8]) -> Result<(u16, u16, Option<TcpHeader>), PacketError> {
        if data.len() < 8 {
            return Err(PacketError::TooShort);
        }

        let src_port = u16::from_be_bytes([data[0], data[1]]);
        let dst_port = u16::from_be_bytes([data[2], data[3]]);
        let _length = u16::from_be_bytes([data[4], data[5]]);
        let _checksum = u16::from_be_bytes([data[6], data[7]]);

        // UDP 没有 TCP 头部结构，返回 None
        Ok((src_port, dst_port, None))
    }

    /// 解析 ICMP 头部
    fn parse_icmp(_data: &[u8]) -> Result<(u16, u16, Option<TcpHeader>), PacketError> {
        // ICMP 没有端口概念，返回 0
        // ICMP 类型和代码在 data[0] 和 data[1]
        Ok((0, 0, None))
    }

    /// 解析 TCP 头部
    fn parse_tcp(data: &[u8]) -> Result<(u16, u16, Option<TcpHeader>), PacketError> {
        if data.len() < 20 {
            return Err(PacketError::TooShort);
        }

        let src_port = u16::from_be_bytes([data[0], data[1]]);
        let dst_port = u16::from_be_bytes([data[2], data[3]]);
        let seq = u32::from_be_bytes([data[4], data[5], data[6], data[7]]);
        let ack = if data[13] & 0x10 != 0 {
            Some(u32::from_be_bytes([data[8], data[9], data[10], data[11]]))
        } else {
            None
        };
        let data_offset = ((data[12] >> 4) & 0x0F) as usize;
        
        // 安全检查：data_offset 必须至少为 5（20 字节），最多为 15（60 字节）
        if data_offset < 5 || data_offset > 15 {
            return Err(PacketError::Other("无效的 TCP data offset".to_string()));
        }
        
        let flags = data[13];
        let window = u16::from_be_bytes([data[14], data[15]]);

        // 解析 TCP 选项
        let mut options = Vec::new();
        let header_len = data_offset * 4;
        
        // 安全检查：确保不会越界访问
        if header_len > data.len() {
            return Err(PacketError::TooShort);
        }
        
        if header_len > 20 {
            let mut offset = 20;
            while offset < header_len {
                if offset >= data.len() {
                    break;
                }
                let kind = data[offset];
                if kind == 0 {
                    // End of options
                    break;
                } else if kind == 1 {
                    // NOP
                    offset += 1;
                    continue;
                } else {
                    if offset + 1 >= data.len() {
                        break;
                    }
                    let length = data[offset + 1] as usize;
                    
                    // 安全检查：length 必须至少为 2，且不能导致越界
                    if length < 2 || offset + length > data.len() || offset + length > header_len {
                        break;
                    }
                    let option_data = data[offset + 2..offset + length].to_vec();
                    options.push(TcpOption {
                        kind,
                        data: option_data,
                    });
                    offset += length;
                }
            }
        }

        let tcp_header = TcpHeader {
            seq,
            ack,
            window,
            flags,
            options,
        };

        Ok((src_port, dst_port, Some(tcp_header)))
    }
}

/// 数据包解析错误
#[derive(Debug, thiserror::Error)]
pub enum PacketError {
    #[error("数据包太短")]
    TooShort,

    #[error("无效的 IP 版本")]
    InvalidVersion,

    #[error("未实现: {0}")]
    NotImplemented(&'static str),

    #[error("其他错误: {0}")]
    Other(String),
}

impl From<PacketError> for String {
    fn from(err: PacketError) -> Self {
        err.to_string()
    }
}

#[cfg(test)]
mod security_tests {
    use super::*;

    #[test]
    fn test_invalid_ihl_zero() {
        // IHL = 0 (invalid, must be at least 5)
        let mut packet = vec![0x00; 20];
        packet[0] = 0x40; // Version 4, IHL 0
        packet[12..16].copy_from_slice(&[192, 168, 1, 1]); // src IP
        packet[16..20].copy_from_slice(&[192, 168, 1, 2]); // dst IP
        
        let result = PacketParser::parse(&packet);
        assert!(result.is_err(), "应该拒绝 IHL = 0 的数据包");
    }

    #[test]
    fn test_invalid_ihl_small() {
        // IHL = 4 (invalid, must be at least 5)
        let mut packet = vec![0x00; 20];
        packet[0] = 0x44; // Version 4, IHL 4
        packet[12..16].copy_from_slice(&[192, 168, 1, 1]); // src IP
        packet[16..20].copy_from_slice(&[192, 168, 1, 2]); // dst IP
        
        let result = PacketParser::parse(&packet);
        assert!(result.is_err(), "应该拒绝 IHL < 5 的数据包");
    }

    #[test]
    fn test_ihl_causing_overflow() {
        // Test case where IHL * 4 would access beyond packet boundary
        let mut packet = vec![0x00; 20];
        packet[0] = 0x4F; // Version 4, IHL 15 (would need 60 bytes)
        packet[12..16].copy_from_slice(&[192, 168, 1, 1]); // src IP
        packet[16..20].copy_from_slice(&[192, 168, 1, 2]); // dst IP
        
        let result = PacketParser::parse(&packet);
        assert!(result.is_err(), "应该拒绝 header_len 超过 packet 长度的数据包");
    }

    #[test]
    fn test_valid_minimal_ipv4_packet() {
        // Test that a valid minimal packet still works
        let mut packet = vec![0x00; 20];
        packet[0] = 0x45; // Version 4, IHL 5
        packet[2] = 0x00; // Total length high byte
        packet[3] = 0x14; // Total length low byte (20 bytes)
        packet[12..16].copy_from_slice(&[192, 168, 1, 1]); // src IP
        packet[16..20].copy_from_slice(&[192, 168, 1, 2]); // dst IP
        
        let result = PacketParser::parse(&packet);
        assert!(result.is_ok(), "有效的最小 IPv4 数据包应该解析成功");
    }

    #[test]
    fn test_invalid_tcp_data_offset_zero() {
        // Create a valid IPv4 packet with TCP
        let mut packet = vec![0x00; 40];
        packet[0] = 0x45; // Version 4, IHL 5
        packet[2] = 0x00; // Total length high byte
        packet[3] = 0x28; // Total length low byte (40 bytes)
        packet[9] = 6; // Protocol: TCP
        packet[12..16].copy_from_slice(&[192, 168, 1, 1]); // src IP
        packet[16..20].copy_from_slice(&[192, 168, 1, 2]); // dst IP
        
        // TCP header - set data offset to 0 (invalid)
        packet[20..22].copy_from_slice(&[0x00, 0x50]); // src port 80
        packet[22..24].copy_from_slice(&[0x00, 0x50]); // dst port 80
        packet[32] = 0x00; // Data offset = 0 (invalid)
        
        let result = PacketParser::parse(&packet);
        assert!(result.is_err(), "应该拒绝 data_offset = 0 的 TCP 数据包");
    }

    #[test]
    fn test_invalid_tcp_data_offset_small() {
        // Create a valid IPv4 packet with TCP
        let mut packet = vec![0x00; 40];
        packet[0] = 0x45; // Version 4, IHL 5
        packet[9] = 6; // Protocol: TCP
        packet[12..16].copy_from_slice(&[192, 168, 1, 1]); // src IP
        packet[16..20].copy_from_slice(&[192, 168, 1, 2]); // dst IP
        
        // TCP header - set data offset to 4 (invalid, must be at least 5)
        packet[32] = 0x40; // Data offset = 4
        
        let result = PacketParser::parse(&packet);
        assert!(result.is_err(), "应该拒绝 data_offset < 5 的 TCP 数据包");
    }

    #[test]
    fn test_valid_tcp_packet() {
        // Test a valid IPv4 packet with TCP
        let mut packet = vec![0x00; 40];
        packet[0] = 0x45; // Version 4, IHL 5
        packet[9] = 6; // Protocol: TCP
        packet[12..16].copy_from_slice(&[192, 168, 1, 1]); // src IP
        packet[16..20].copy_from_slice(&[192, 168, 1, 2]); // dst IP
        
        // TCP header
        packet[20..22].copy_from_slice(&[0x00, 0x50]); // src port 80
        packet[22..24].copy_from_slice(&[0x00, 0x50]); // dst port 80
        packet[32] = 0x50; // Data offset = 5 (20 bytes)
        packet[33] = 0x02; // SYN flag
        
        let result = PacketParser::parse(&packet);
        assert!(result.is_ok(), "有效的 TCP 数据包应该解析成功");
        
        let p = result.unwrap();
        assert_eq!(p.src_port, 80);
        assert_eq!(p.dst_port, 80);
        assert!(p.tcp_header.is_some());
    }

    #[test]
    fn test_packet_too_short() {
        // Packet shorter than minimum IPv4 header
        let packet = vec![0x45; 10];
        
        let result = PacketParser::parse(&packet);
        assert!(result.is_err(), "太短的数据包应该被拒绝");
    }

    #[test]
    fn test_invalid_ip_version() {
        // IP version 3 (invalid, should be 4 or 6)
        let mut packet = vec![0x00; 20];
        packet[0] = 0x35; // Version 3, IHL 5
        
        let result = PacketParser::parse(&packet);
        assert!(result.is_err(), "无效的 IP 版本应该被拒绝");
    }
}
