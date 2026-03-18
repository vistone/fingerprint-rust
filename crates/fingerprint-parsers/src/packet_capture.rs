/// Advanced Packet Capture and Analysis Module
///
/// This module provides comprehensive packet capture analysis capabilities
/// for network traffic fingerprinting and identification.
///
/// Supported formats:
/// - PCAP files (not raw_socket for security reasons on Linux)
/// - Ethernet frames
/// - IPv4 and IPv6 packets
/// - TCP and UDP packets
/// - TLS handshake packets
/// - HTTP/2 frames
///
/// Integration with other fingerprinting modules:
/// - TCP handshake analysis (SYN/SYN-ACK/ACK)
/// - TLS fingerprint extraction (JA3/JA4)
/// - HPACK header compression analysis
/// - HTTP headers extraction
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::path::Path;

use pcap_file::pcap::PcapReader;

/// PCAP global header (for PCAP file format)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PcapGlobalHeader {
    /// Magic number (0xa1b2c3d4 for normal byte order)
    pub magic_number: u32,
    /// Major version number
    pub version_major: u16,
    /// Minor version number
    pub version_minor: u16,
    /// Timezone offset (usually 0)
    pub timezone_offset: i32,
    /// Timestamp accuracy (usually 0)
    pub timestamp_accuracy: u32,
    /// Snapshot length (maximum packet size)
    pub snapshot_length: u32,
    /// Data link type (1 for Ethernet)
    pub data_link_type: u32,
}

impl PcapGlobalHeader {
    /// Create PCAP header with standard settings
    pub fn standard() -> Self {
        Self {
            magic_number: 0xa1b2c3d4,
            version_major: 2,
            version_minor: 4,
            timezone_offset: 0,
            timestamp_accuracy: 0,
            snapshot_length: 65535,
            data_link_type: 1, // Ethernet
        }
    }

    /// Check if byte order is correct
    pub fn is_valid(&self) -> bool {
        self.magic_number == 0xa1b2c3d4 || self.magic_number == 0xd4c3b2a1
    }

    /// Check if byte order needs swapping
    pub fn needs_byte_swap(&self) -> bool {
        self.magic_number == 0xd4c3b2a1
    }
}

/// PCAP packet header
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PcapPacketHeader {
    /// Timestamp seconds
    pub timestamp_sec: u32,
    /// Timestamp microseconds
    pub timestamp_usec: u32,
    /// Packet length (captured)
    pub incl_len: u32,
    /// Actual packet length
    pub orig_len: u32,
}

/// Ethernet frame header
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EthernetHeader {
    /// Destination MAC address
    pub dst_mac: [u8; 6],
    /// Source MAC address
    pub src_mac: [u8; 6],
    /// EtherType (0x0800 for IPv4, 0x86dd for IPv6)
    pub ether_type: u16,
}

/// IPv4 header
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ipv4Header {
    /// Version and IHL (version in upper 4 bits, IHL in lower 4 bits)
    pub version_ihl: u8,
    /// DSCP and ECN
    pub dscp_ecn: u8,
    /// Total length
    pub total_length: u16,
    /// Identification
    pub identification: u16,
    /// Flags and fragment offset
    pub flags_fragment_offset: u16,
    /// Time To Live
    pub ttl: u8,
    /// Protocol
    pub protocol: u8,
    /// Header checksum
    pub checksum: u16,
    /// Source IP address
    pub src_ip: [u8; 4],
    /// Destination IP address
    pub dst_ip: [u8; 4],
}

impl Ipv4Header {
    /// Get version (upper 4 bits)
    pub fn version(&self) -> u8 {
        self.version_ihl >> 4
    }

    /// Get IHL (lower 4 bits) - header length in 32-bit words
    pub fn ihl(&self) -> u8 {
        self.version_ihl & 0x0f
    }

    /// Get DF (Don't Fragment) flag
    pub fn df_flag(&self) -> bool {
        (self.flags_fragment_offset & 0x4000) != 0
    }

    /// Get MF (More Fragments) flag
    pub fn mf_flag(&self) -> bool {
        (self.flags_fragment_offset & 0x2000) != 0
    }

    /// Get fragment offset
    pub fn fragment_offset(&self) -> u16 {
        self.flags_fragment_offset & 0x1fff
    }
}

/// IPv6 header
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ipv6Header {
    /// Version and traffic class (high 4 bits version, low 4 bits traffic class high)
    pub version_traffic_class_high: u8,
    /// Traffic class low and flow label high
    pub traffic_class_flow_label_high: u8,
    /// Flow label
    pub flow_label: u16,
    /// Payload length
    pub payload_length: u16,
    /// Next header
    pub next_header: u8,
    /// Hop limit (equivalent to TTL)
    pub hop_limit: u8,
    /// Source IP address (128 bits)
    pub src_ip: [u8; 16],
    /// Destination IP address (128 bits)
    pub dst_ip: [u8; 16],
}

/// TCP header
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TcpHeader {
    /// Source port
    pub src_port: u16,
    /// Destination port
    pub dst_port: u16,
    /// Sequence number
    pub sequence_number: u32,
    /// Acknowledgment number
    pub acknowledgment_number: u32,
    /// Data offset and flags
    pub data_offset_flags: u16,
    /// Window size
    pub window_size: u16,
    /// Checksum
    pub checksum: u16,
    /// Urgent pointer
    pub urgent_pointer: u16,
}

impl TcpHeader {
    /// Get data offset (upper 4 bits) - header length in 32-bit words
    pub fn data_offset(&self) -> u8 {
        ((self.data_offset_flags >> 12) & 0x0f) as u8
    }

    /// Check if SYN flag is set
    pub fn syn(&self) -> bool {
        (self.data_offset_flags & 0x0002) != 0
    }

    /// Check if ACK flag is set
    pub fn ack(&self) -> bool {
        (self.data_offset_flags & 0x0010) != 0
    }

    /// Check if FIN flag is set
    pub fn fin(&self) -> bool {
        (self.data_offset_flags & 0x0001) != 0
    }

    /// Check if RST flag is set
    pub fn rst(&self) -> bool {
        (self.data_offset_flags & 0x0004) != 0
    }

    /// Check if PSH flag is set
    pub fn psh(&self) -> bool {
        (self.data_offset_flags & 0x0008) != 0
    }
}

/// Parsed TCP option kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ParsedTcpOptionKind {
    EndOfList,
    NoOperation,
    MSS,
    WindowScale,
    SackPermitted,
    Timestamp,
    TcpFastOpen,
    Unknown(u8),
}

/// Parsed TCP option extracted from a packet.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParsedTcpOption {
    pub kind: ParsedTcpOptionKind,
    pub length: u8,
    pub data: Vec<u8>,
}

/// UDP header
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UdpHeader {
    /// Source port
    pub src_port: u16,
    /// Destination port
    pub dst_port: u16,
    /// Length of header and data
    pub length: u16,
    /// Checksum
    pub checksum: u16,
}

/// Network layer protocol
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NetworkProtocol {
    /// IPv4
    IPv4,
    /// IPv6
    IPv6,
    /// Unknown
    Unknown(u8),
}

/// Transport layer protocol
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransportProtocol {
    /// TCP
    TCP,
    /// UDP
    UDP,
    /// TLS over TCP (HTTPS)
    TLS,
    /// QUIC (UDP-based)
    QUIC,
    /// Unknown
    Unknown(u8),
}

/// Complete parsed packet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedPacket {
    /// Packet capture time (seconds since epoch)
    pub timestamp_sec: u32,
    /// Packet capture time (microseconds)
    pub timestamp_usec: u32,
    /// Ethernet header (if present)
    pub ethernet: Option<EthernetHeader>,
    /// Network layer
    pub network_protocol: NetworkProtocol,
    /// IPv4 header (if IPv4)
    pub ipv4: Option<Ipv4Header>,
    /// IPv6 header (if IPv6)
    pub ipv6: Option<Ipv6Header>,
    /// Transport layer
    pub transport_protocol: TransportProtocol,
    /// TCP header (if TCP)
    pub tcp: Option<TcpHeader>,
    /// Parsed TCP options (if TCP)
    pub tcp_options: Vec<ParsedTcpOption>,
    /// UDP header (if UDP)
    pub udp: Option<UdpHeader>,
    /// Payload data
    pub payload: Vec<u8>,
    /// Total packet size (bytes)
    pub total_size: usize,
}

/// Streamed PCAP packet view.
pub struct PcapPacketView<'a> {
    pub timestamp_sec: u32,
    pub timestamp_usec: u32,
    pub original_len: u32,
    pub data: &'a [u8],
}

impl ParsedPacket {
    /// Get source IP as string
    pub fn src_ip(&self) -> String {
        match &self.ipv4 {
            Some(hdr) => format!(
                "{}.{}.{}.{}",
                hdr.src_ip[0], hdr.src_ip[1], hdr.src_ip[2], hdr.src_ip[3]
            ),
            None => "unknown".to_string(),
        }
    }

    /// Get destination IP as string
    pub fn dst_ip(&self) -> String {
        match &self.ipv4 {
            Some(hdr) => format!(
                "{}.{}.{}.{}",
                hdr.dst_ip[0], hdr.dst_ip[1], hdr.dst_ip[2], hdr.dst_ip[3]
            ),
            None => "unknown".to_string(),
        }
    }

    /// Get source port
    pub fn src_port(&self) -> Option<u16> {
        self.tcp
            .as_ref()
            .map(|t| t.src_port)
            .or_else(|| self.udp.as_ref().map(|u| u.src_port))
    }

    /// Get destination port
    pub fn dst_port(&self) -> Option<u16> {
        self.tcp
            .as_ref()
            .map(|t| t.dst_port)
            .or_else(|| self.udp.as_ref().map(|u| u.dst_port))
    }

    /// Check if this is a SYN packet
    pub fn is_syn(&self) -> bool {
        self.tcp.as_ref().map(|t| t.syn()).unwrap_or(false)
    }

    /// Check if this is a SYN-ACK packet
    pub fn is_syn_ack(&self) -> bool {
        if let Some(tcp) = &self.tcp {
            tcp.syn() && tcp.ack()
        } else {
            false
        }
    }

    /// Check if this is an ACK packet
    pub fn is_ack(&self) -> bool {
        self.tcp.as_ref().map(|t| t.ack()).unwrap_or(false)
    }

    /// Check if this is a FIN packet
    pub fn is_fin(&self) -> bool {
        self.tcp.as_ref().map(|t| t.fin()).unwrap_or(false)
    }

    /// Check if this is a RST packet
    pub fn is_rst(&self) -> bool {
        self.tcp.as_ref().map(|t| t.rst()).unwrap_or(false)
    }
}

/// TCP connection flow (for tracking handshakes)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TcpFlow {
    /// Source IP
    pub src_ip: String,
    /// Destination IP
    pub dst_ip: String,
    /// Source port
    pub src_port: u16,
    /// Destination port
    pub dst_port: u16,
    /// SYN packet details
    pub syn_packet: Option<ParsedPacket>,
    /// SYN-ACK packet details
    pub syn_ack_packet: Option<ParsedPacket>,
    /// ACK packet details
    pub ack_packet: Option<ParsedPacket>,
    /// All packets in flow
    pub packets: Vec<ParsedPacket>,
    /// Handshake complete?
    pub handshake_complete: bool,
}

impl TcpFlow {
    /// Create new TCP flow
    pub fn new(src_ip: String, dst_ip: String, src_port: u16, dst_port: u16) -> Self {
        Self {
            src_ip,
            dst_ip,
            src_port,
            dst_port,
            syn_packet: None,
            syn_ack_packet: None,
            ack_packet: None,
            packets: Vec::new(),
            handshake_complete: false,
        }
    }

    /// Get flow key (for hashtable lookup)
    pub fn flow_key(&self) -> String {
        format!(
            "{}:{} -> {}:{}",
            self.src_ip, self.src_port, self.dst_ip, self.dst_port
        )
    }

    /// Add packet to flow
    pub fn add_packet(&mut self, packet: ParsedPacket) {
        if packet.is_syn() && !packet.is_ack() {
            self.syn_packet = Some(packet.clone());
        } else if packet.is_syn_ack() {
            self.syn_ack_packet = Some(packet.clone());
        } else if packet.is_ack() && self.syn_ack_packet.is_some() && self.ack_packet.is_none() {
            self.ack_packet = Some(packet.clone());
            self.handshake_complete = true;
        }
        self.packets.push(packet);
    }

    /// Check if handshake is complete
    pub fn has_complete_handshake(&self) -> bool {
        self.syn_packet.is_some() && self.syn_ack_packet.is_some() && self.ack_packet.is_some()
    }
}

/// Packet flow analyzer (aggregates TCP flows)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PacketFlowAnalyzer {
    /// TCP flows mapped by flow key
    pub flows: HashMap<String, TcpFlow>,
    /// Total packets captured
    pub total_packets: u64,
    /// IPv4 packets
    pub ipv4_packets: u64,
    /// IPv6 packets
    pub ipv6_packets: u64,
    /// TCP packets
    pub tcp_packets: u64,
    /// UDP packets
    pub udp_packets: u64,
    /// TLS handshakes detected
    pub tls_handshakes: u64,
}

impl Default for PacketFlowAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl PacketFlowAnalyzer {
    /// Create new analyzer
    pub fn new() -> Self {
        Self {
            flows: HashMap::new(),
            total_packets: 0,
            ipv4_packets: 0,
            ipv6_packets: 0,
            tcp_packets: 0,
            udp_packets: 0,
            tls_handshakes: 0,
        }
    }

    /// Add packet to analyzer
    pub fn add_packet(&mut self, packet: ParsedPacket) {
        self.total_packets += 1;

        match packet.network_protocol {
            NetworkProtocol::IPv4 => self.ipv4_packets += 1,
            NetworkProtocol::IPv6 => self.ipv6_packets += 1,
            _ => {}
        }

        match packet.transport_protocol {
            TransportProtocol::TCP => {
                self.tcp_packets += 1;
                self.add_tcp_packet(packet);
            }
            TransportProtocol::UDP => self.udp_packets += 1,
            TransportProtocol::TLS => self.tls_handshakes += 1,
            _ => {}
        }
    }

    /// Add TCP packet to flows
    fn add_tcp_packet(&mut self, packet: ParsedPacket) {
        if let (Some(src_port), Some(dst_port)) = (packet.src_port(), packet.dst_port()) {
            let flow_key = format!(
                "{}:{} -> {}:{}",
                packet.src_ip(),
                src_port,
                packet.dst_ip(),
                dst_port
            );

            let flow = self.flows.entry(flow_key).or_insert_with(|| {
                TcpFlow::new(packet.src_ip(), packet.dst_ip(), src_port, dst_port)
            });

            flow.add_packet(packet);
        }
    }

    /// Get complete TCP handshakes
    pub fn complete_handshakes(&self) -> Vec<&TcpFlow> {
        self.flows
            .values()
            .filter(|f| f.has_complete_handshake())
            .collect()
    }

    /// Get statistics summary
    pub fn get_summary(&self) -> String {
        format!(
            "Packets captured: {}\n  \
             IPv4: {}, IPv6: {}\n  \
             TCP: {}, UDP: {}\n  \
             TCP Flows: {}\n  \
             Complete handshakes: {}",
            self.total_packets,
            self.ipv4_packets,
            self.ipv6_packets,
            self.tcp_packets,
            self.udp_packets,
            self.flows.len(),
            self.complete_handshakes().len()
        )
    }
}

/// Packet parser for converting raw bytes to ParsedPacket
pub struct PacketParser;

impl PacketParser {
    /// Parse Ethernet frame
    pub fn parse_ethernet(data: &[u8]) -> Option<(EthernetHeader, &[u8])> {
        if data.len() < 14 {
            return None;
        }

        let ethernet = EthernetHeader {
            dst_mac: [data[0], data[1], data[2], data[3], data[4], data[5]],
            src_mac: [data[6], data[7], data[8], data[9], data[10], data[11]],
            ether_type: u16::from_be_bytes([data[12], data[13]]),
        };

        Some((ethernet, &data[14..]))
    }

    /// Parse IPv4 packet
    pub fn parse_ipv4(data: &[u8]) -> Option<(Ipv4Header, &[u8])> {
        if data.len() < 20 {
            return None;
        }

        let ipv4 = Ipv4Header {
            version_ihl: data[0],
            dscp_ecn: data[1],
            total_length: u16::from_be_bytes([data[2], data[3]]),
            identification: u16::from_be_bytes([data[4], data[5]]),
            flags_fragment_offset: u16::from_be_bytes([data[6], data[7]]),
            ttl: data[8],
            protocol: data[9],
            checksum: u16::from_be_bytes([data[10], data[11]]),
            src_ip: [data[12], data[13], data[14], data[15]],
            dst_ip: [data[16], data[17], data[18], data[19]],
        };

        let header_len = ipv4.ihl() as usize * 4;
        if data.len() < header_len {
            return None;
        }

        Some((ipv4, &data[header_len..]))
    }

    /// Parse TCP packet
    pub fn parse_tcp(data: &[u8]) -> Option<(TcpHeader, &[u8])> {
        if data.len() < 20 {
            return None;
        }

        let tcp = TcpHeader {
            src_port: u16::from_be_bytes([data[0], data[1]]),
            dst_port: u16::from_be_bytes([data[2], data[3]]),
            sequence_number: u32::from_be_bytes([data[4], data[5], data[6], data[7]]),
            acknowledgment_number: u32::from_be_bytes([data[8], data[9], data[10], data[11]]),
            data_offset_flags: u16::from_be_bytes([data[12], data[13]]),
            window_size: u16::from_be_bytes([data[14], data[15]]),
            checksum: u16::from_be_bytes([data[16], data[17]]),
            urgent_pointer: u16::from_be_bytes([data[18], data[19]]),
        };

        let header_len = tcp.data_offset() as usize * 4;
        if data.len() < header_len {
            return None;
        }

        Some((tcp, &data[header_len..]))
    }

    /// Parse a TCP segment including its options and payload.
    pub fn parse_tcp_with_options(data: &[u8]) -> Option<(TcpHeader, Vec<ParsedTcpOption>, &[u8])> {
        if data.len() < 20 {
            return None;
        }

        let tcp = TcpHeader {
            src_port: u16::from_be_bytes([data[0], data[1]]),
            dst_port: u16::from_be_bytes([data[2], data[3]]),
            sequence_number: u32::from_be_bytes([data[4], data[5], data[6], data[7]]),
            acknowledgment_number: u32::from_be_bytes([data[8], data[9], data[10], data[11]]),
            data_offset_flags: u16::from_be_bytes([data[12], data[13]]),
            window_size: u16::from_be_bytes([data[14], data[15]]),
            checksum: u16::from_be_bytes([data[16], data[17]]),
            urgent_pointer: u16::from_be_bytes([data[18], data[19]]),
        };

        let header_len = tcp.data_offset() as usize * 4;
        if data.len() < header_len || header_len < 20 {
            return None;
        }

        let options = Self::parse_tcp_options(&data[20..header_len]);
        Some((tcp, options, &data[header_len..]))
    }

    /// Parse UDP packet
    pub fn parse_udp(data: &[u8]) -> Option<(UdpHeader, &[u8])> {
        if data.len() < 8 {
            return None;
        }

        let udp = UdpHeader {
            src_port: u16::from_be_bytes([data[0], data[1]]),
            dst_port: u16::from_be_bytes([data[2], data[3]]),
            length: u16::from_be_bytes([data[4], data[5]]),
            checksum: u16::from_be_bytes([data[6], data[7]]),
        };

        let payload_len = (udp.length as usize).saturating_sub(8);
        if data.len() < payload_len + 8 {
            return None;
        }

        Some((udp, &data[8..8 + payload_len]))
    }

    /// Parse a full packet into a structured representation.
    pub fn parse_packet(
        data: &[u8],
        timestamp_sec: u32,
        timestamp_usec: u32,
    ) -> Option<ParsedPacket> {
        let total_size = data.len();
        let (ethernet, network_payload) = Self::parse_ethernet(data)?;
        match ethernet.ether_type {
            0x0800 => {
                let (ipv4, transport_payload) = Self::parse_ipv4(network_payload)?;
                match ipv4.protocol {
                    6 => {
                        let (tcp, tcp_options, payload) =
                            Self::parse_tcp_with_options(transport_payload)?;
                        Some(ParsedPacket {
                            timestamp_sec,
                            timestamp_usec,
                            ethernet: Some(ethernet),
                            network_protocol: NetworkProtocol::IPv4,
                            ipv4: Some(ipv4),
                            ipv6: None,
                            transport_protocol: TransportProtocol::TCP,
                            tcp: Some(tcp),
                            tcp_options,
                            udp: None,
                            payload: payload.to_vec(),
                            total_size,
                        })
                    }
                    17 => {
                        let (udp, payload) = Self::parse_udp(transport_payload)?;
                        Some(ParsedPacket {
                            timestamp_sec,
                            timestamp_usec,
                            ethernet: Some(ethernet),
                            network_protocol: NetworkProtocol::IPv4,
                            ipv4: Some(ipv4),
                            ipv6: None,
                            transport_protocol: TransportProtocol::UDP,
                            tcp: None,
                            tcp_options: Vec::new(),
                            udp: Some(udp),
                            payload: payload.to_vec(),
                            total_size,
                        })
                    }
                    protocol => Some(ParsedPacket {
                        timestamp_sec,
                        timestamp_usec,
                        ethernet: Some(ethernet),
                        network_protocol: NetworkProtocol::IPv4,
                        ipv4: Some(ipv4),
                        ipv6: None,
                        transport_protocol: TransportProtocol::Unknown(protocol),
                        tcp: None,
                        tcp_options: Vec::new(),
                        udp: None,
                        payload: transport_payload.to_vec(),
                        total_size,
                    }),
                }
            }
            0x86dd => {
                let (ipv6, transport_payload) = Self::parse_ipv6(network_payload)?;
                let next_header = ipv6.next_header;
                Some(ParsedPacket {
                    timestamp_sec,
                    timestamp_usec,
                    ethernet: Some(ethernet),
                    network_protocol: NetworkProtocol::IPv6,
                    ipv4: None,
                    ipv6: Some(ipv6),
                    transport_protocol: TransportProtocol::Unknown(next_header),
                    tcp: None,
                    tcp_options: Vec::new(),
                    udp: None,
                    payload: transport_payload.to_vec(),
                    total_size,
                })
            }
            ether_type => Some(ParsedPacket {
                timestamp_sec,
                timestamp_usec,
                ethernet: Some(ethernet),
                network_protocol: NetworkProtocol::Unknown((ether_type >> 8) as u8),
                ipv4: None,
                ipv6: None,
                transport_protocol: TransportProtocol::Unknown(0),
                tcp: None,
                tcp_options: Vec::new(),
                udp: None,
                payload: network_payload.to_vec(),
                total_size,
            }),
        }
    }

    fn parse_ipv6(data: &[u8]) -> Option<(Ipv6Header, &[u8])> {
        if data.len() < 40 {
            return None;
        }

        let ipv6 = Ipv6Header {
            version_traffic_class_high: data[0],
            traffic_class_flow_label_high: data[1],
            flow_label: u16::from_be_bytes([data[2], data[3]]),
            payload_length: u16::from_be_bytes([data[4], data[5]]),
            next_header: data[6],
            hop_limit: data[7],
            src_ip: data[8..24].try_into().ok()?,
            dst_ip: data[24..40].try_into().ok()?,
        };

        Some((ipv6, &data[40..]))
    }

    fn parse_tcp_options(data: &[u8]) -> Vec<ParsedTcpOption> {
        let mut options = Vec::new();
        let mut idx = 0usize;

        while idx < data.len() {
            let kind = data[idx];
            match kind {
                0 => {
                    options.push(ParsedTcpOption {
                        kind: ParsedTcpOptionKind::EndOfList,
                        length: 1,
                        data: Vec::new(),
                    });
                    break;
                }
                1 => {
                    options.push(ParsedTcpOption {
                        kind: ParsedTcpOptionKind::NoOperation,
                        length: 1,
                        data: Vec::new(),
                    });
                    idx += 1;
                }
                _ => {
                    if idx + 1 >= data.len() {
                        break;
                    }
                    let length = data[idx + 1] as usize;
                    if length < 2 || idx + length > data.len() {
                        break;
                    }

                    let payload = data[idx + 2..idx + length].to_vec();
                    options.push(ParsedTcpOption {
                        kind: match kind {
                            2 => ParsedTcpOptionKind::MSS,
                            3 => ParsedTcpOptionKind::WindowScale,
                            4 => ParsedTcpOptionKind::SackPermitted,
                            8 => ParsedTcpOptionKind::Timestamp,
                            34 => ParsedTcpOptionKind::TcpFastOpen,
                            other => ParsedTcpOptionKind::Unknown(other),
                        },
                        length: length as u8,
                        data: payload,
                    });
                    idx += length;
                }
            }
        }

        options
    }

    /// Stream packets from a PCAP file without loading the entire file into memory.
    pub fn stream_pcap_file<P, F>(path: P, mut on_packet: F) -> Result<(), String>
    where
        P: AsRef<Path>,
        F: FnMut(PcapPacketView<'_>) -> Result<(), String>,
    {
        let file = File::open(path.as_ref()).map_err(|e| {
            format!(
                "Failed to open PCAP file {}: {}",
                path.as_ref().display(),
                e
            )
        })?;
        let mut reader =
            PcapReader::new(file).map_err(|e| format!("Failed to parse PCAP file: {}", e))?;

        while let Some(packet) = reader.next_packet() {
            let packet = packet.map_err(|e| format!("Failed to read PCAP packet: {}", e))?;
            let packet_view = PcapPacketView {
                timestamp_sec: packet.timestamp.as_secs() as u32,
                timestamp_usec: packet.timestamp.subsec_micros(),
                original_len: packet.orig_len,
                data: &packet.data,
            };
            on_packet(packet_view)?;
        }

        Ok(())
    }

    /// Count packets in a PCAP file using streaming IO.
    pub fn count_pcap_packets<P>(path: P) -> Result<usize, String>
    where
        P: AsRef<Path>,
    {
        let mut count = 0usize;
        Self::stream_pcap_file(path, |_| {
            count += 1;
            Ok(())
        })?;
        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pcap_header_creation() {
        let header = PcapGlobalHeader::standard();
        assert!(header.is_valid());
        assert!(!header.needs_byte_swap());
    }

    #[test]
    fn test_ipv4_header_flags() {
        let hdr = Ipv4Header {
            version_ihl: 0x45,
            dscp_ecn: 0,
            total_length: 0,
            identification: 0,
            flags_fragment_offset: 0x4000, // DF flag set
            ttl: 64,
            protocol: 6,
            checksum: 0,
            src_ip: [192, 168, 1, 1],
            dst_ip: [192, 168, 1, 2],
        };

        assert_eq!(hdr.version(), 4);
        assert_eq!(hdr.ihl(), 5);
        assert!(hdr.df_flag());
        assert!(!hdr.mf_flag());
    }

    #[test]
    fn test_tcp_header_flags() {
        let tcp = TcpHeader {
            src_port: 443,
            dst_port: 59432,
            sequence_number: 1000,
            acknowledgment_number: 2000,
            data_offset_flags: 0x3002, // SYN flag + data offset 3
            window_size: 64240,
            checksum: 0,
            urgent_pointer: 0,
        };

        assert_eq!(tcp.data_offset(), 3);
        assert!(tcp.syn());
        assert!(!tcp.ack());
        assert!(!tcp.fin());
    }

    #[test]
    fn test_tcp_flow_creation() {
        let flow = TcpFlow::new(
            "192.168.1.1".to_string(),
            "192.168.1.2".to_string(),
            54321,
            443,
        );

        assert_eq!(flow.src_ip, "192.168.1.1");
        assert_eq!(flow.dst_port, 443);
        assert!(!flow.handshake_complete);
    }

    #[test]
    fn test_packet_flow_analyzer() {
        let analyzer = PacketFlowAnalyzer::new();
        assert_eq!(analyzer.total_packets, 0);
        assert_eq!(analyzer.complete_handshakes().len(), 0);
    }

    #[test]
    fn test_ethernet_format() {
        let data = [
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, // dst MAC
            0x00, 0x11, 0x22, 0x33, 0x44, 0x55, // src MAC
            0x08, 0x00, // IPv4 EtherType
        ];

        let (eth, _) = PacketParser::parse_ethernet(&data).unwrap();
        assert_eq!(eth.ether_type, 0x0800);
    }

    #[test]
    fn test_ipv4_parse() {
        let data = [
            0x45, 0x00, // Version/IHL, DSCP/ECN
            0x00, 0x54, // Total length
            0x00, 0x00, // Identification
            0x40, 0x00, // Flags/Fragment offset (DF)
            0x40, 0x06, // TTL, Protocol (TCP)
            0x00, 0x00, // Checksum
            192, 168, 1, 1, // Source IP
            192, 168, 1, 2, // Destination IP
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ];

        let (ipv4, _) = PacketParser::parse_ipv4(&data).unwrap();
        assert_eq!(ipv4.version(), 4);
        assert_eq!(ipv4.ihl(), 5);
        assert!(ipv4.df_flag());
        assert_eq!(ipv4.protocol, 6); // TCP
    }

    #[test]
    fn test_tcp_parse() {
        let data = [
            0x00, 0x50, // Source port (80)
            0xea, 0xa8, // Destination port (60088)
            0x00, 0x00, 0x00, 0x00, // Sequence
            0x00, 0x00, 0x00, 0x00, // ACK
            0x50, 0x02, // Data offset (5) and flags (SYN)
            0xfa, 0xf0, // Window
            0x00, 0x00, // Checksum
            0x00, 0x00, // Urgent pointer
        ];

        let (tcp, _) = PacketParser::parse_tcp(&data).unwrap();
        assert_eq!(tcp.src_port, 80);
        assert_eq!(tcp.dst_port, 60072); // 0xea << 8 | 0xa8
        assert_eq!(tcp.data_offset(), 5);
        assert!(tcp.syn());
    }

    #[test]
    fn test_parse_tcp_with_options_extracts_mss_and_window_scale() {
        let data = [
            0x00, 0x50, 0xea, 0xa8, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x70, 0x02,
            0xfa, 0xf0, 0x00, 0x00, 0x00, 0x00, 0x02, 0x04, 0x05, 0xb4, 0x03, 0x03, 0x08, 0x01,
        ];

        let (_, options, payload) = PacketParser::parse_tcp_with_options(&data).unwrap();
        assert!(payload.is_empty());
        assert_eq!(options.len(), 3);
        assert_eq!(options[0].kind, ParsedTcpOptionKind::MSS);
        assert_eq!(options[1].kind, ParsedTcpOptionKind::WindowScale);
    }

    #[test]
    fn test_parse_packet_builds_structured_tcp_packet() {
        let packet = [
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x08, 0x00,
            0x45, 0x00, 0x00, 0x30, 0x00, 0x00, 0x40, 0x00, 0x40, 0x06, 0x00, 0x00, 192, 168, 1, 1,
            192, 168, 1, 2, 0x00, 0x50, 0xea, 0xa8, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00,
            0x70, 0x02, 0xfa, 0xf0, 0x00, 0x00, 0x00, 0x00, 0x02, 0x04, 0x05, 0xb4, 0x03, 0x03,
            0x08, 0x01,
        ];

        let parsed = PacketParser::parse_packet(&packet, 1, 2).unwrap();
        assert_eq!(parsed.timestamp_sec, 1);
        assert_eq!(parsed.network_protocol, NetworkProtocol::IPv4);
        assert_eq!(parsed.transport_protocol, TransportProtocol::TCP);
        assert_eq!(parsed.tcp_options.len(), 3);
        assert!(parsed.is_syn());
    }

    #[test]
    fn test_stream_pcap_file() {
        let path = std::env::temp_dir().join("packet_capture_stream_test.pcap");
        let mut generator = crate::pcap_generator::PcapGenerator::new();
        generator.add_chrome_syn();
        generator.write_to_file(&path).unwrap();

        let mut packet_count = 0usize;
        PacketParser::stream_pcap_file(&path, |packet| {
            packet_count += 1;
            assert!(!packet.data.is_empty());
            Ok(())
        })
        .unwrap();

        assert_eq!(packet_count, 1);

        std::fs::remove_file(path).unwrap();
    }
}
