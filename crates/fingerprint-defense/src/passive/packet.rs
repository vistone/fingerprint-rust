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

        let header_len = (ihl * 4) as usize;
        if raw_packet.len() < header_len {
            return Err(PacketError::TooShort);
        }

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
            ip_flags: ip_flags as u8,
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
        let flags = data[13];
        let window = u16::from_be_bytes([data[14], data[15]]);

        // 解析 TCP 选项
        let mut options = Vec::new();
        let header_len = data_offset * 4;
        if header_len > 20 && data.len() >= header_len {
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
                    if length < 2 || offset + length > data.len() {
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
