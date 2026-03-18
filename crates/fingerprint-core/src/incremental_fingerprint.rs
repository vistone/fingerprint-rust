use crate::tcp::TcpFingerprint;
use crate::tcp_handshake::{
    AckCharacteristics, IpCharacteristics, SynAckCharacteristics, SynCharacteristics, TcpFlags,
    TcpHandshakeAnalyzer, TcpHandshakeFingerprint, TcpOption, TcpOptionType,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IncrementalIpv4Packet {
    pub ttl: u8,
    pub dont_fragment: bool,
    pub identification: u16,
}

impl IncrementalIpv4Packet {
    pub fn new(ttl: u8, dont_fragment: bool, identification: u16) -> Self {
        Self {
            ttl,
            dont_fragment,
            identification,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IncrementalTcpSegment {
    pub window_size: u16,
    pub data_offset_flags: u16,
}

impl IncrementalTcpSegment {
    pub fn new(window_size: u16, data_offset_flags: u16) -> Self {
        Self {
            window_size,
            data_offset_flags,
        }
    }

    pub fn syn(&self) -> bool {
        (self.data_offset_flags & 0x0002) != 0
    }

    pub fn ack(&self) -> bool {
        (self.data_offset_flags & 0x0010) != 0
    }

    pub fn fin(&self) -> bool {
        (self.data_offset_flags & 0x0001) != 0
    }

    pub fn rst(&self) -> bool {
        (self.data_offset_flags & 0x0004) != 0
    }

    pub fn psh(&self) -> bool {
        (self.data_offset_flags & 0x0008) != 0
    }

    pub fn urg(&self) -> bool {
        (self.data_offset_flags & 0x0020) != 0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IncrementalTcpOptionKind {
    EndOfList,
    NoOperation,
    MSS,
    WindowScale,
    SackPermitted,
    Timestamp,
    TcpFastOpen,
    Unknown(u8),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IncrementalTcpOption {
    pub kind: IncrementalTcpOptionKind,
    pub length: u8,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct IncrementalPacket {
    pub ipv4: Option<IncrementalIpv4Packet>,
    pub tcp: Option<IncrementalTcpSegment>,
    pub tcp_options: Vec<IncrementalTcpOption>,
}

impl IncrementalPacket {
    pub fn new(
        ipv4: Option<IncrementalIpv4Packet>,
        tcp: Option<IncrementalTcpSegment>,
        tcp_options: Vec<IncrementalTcpOption>,
    ) -> Self {
        Self {
            ipv4,
            tcp,
            tcp_options,
        }
    }

    pub fn is_syn(&self) -> bool {
        self.tcp.as_ref().is_some_and(IncrementalTcpSegment::syn)
    }

    pub fn is_ack(&self) -> bool {
        self.tcp.as_ref().is_some_and(IncrementalTcpSegment::ack)
    }

    pub fn is_syn_ack(&self) -> bool {
        self.is_syn() && self.is_ack()
    }

    pub fn is_rst(&self) -> bool {
        self.tcp.as_ref().is_some_and(IncrementalTcpSegment::rst)
    }
}

#[derive(Debug, Clone)]
pub struct IncrementalFingerprintResult {
    pub tcp_fingerprint: Option<TcpFingerprint>,
    pub handshake_fingerprint: Option<TcpHandshakeFingerprint>,
    pub anomaly_score: f32,
    pub alert_reasons: Vec<String>,
    pub packet_count: usize,
}

/// Incrementally accumulates TCP packets into a live fingerprint and anomaly score.
#[derive(Debug, Clone, Default)]
pub struct IncrementalTcpFingerprint {
    syn: Option<SynCharacteristics>,
    syn_ack: Option<SynAckCharacteristics>,
    ack: Option<AckCharacteristics>,
    current_tcp: Option<TcpFingerprint>,
    anomaly_score: f32,
    alert_reasons: Vec<String>,
    packet_count: usize,
}

impl IncrementalTcpFingerprint {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn update(&mut self, packet: &IncrementalPacket) -> Result<(), String> {
        let tcp = packet
            .tcp
            .as_ref()
            .ok_or_else(|| "incremental TCP fingerprint requires a TCP packet".to_string())?;
        let ipv4 = packet.ipv4.as_ref().ok_or_else(|| {
            "incremental TCP fingerprint currently requires IPv4 metadata".to_string()
        })?;

        self.packet_count += 1;

        if packet.is_rst() && !self.handshake_complete() {
            self.raise_alert("rst_before_handshake_complete", 0.55);
        }
        if !is_common_initial_ttl(ipv4.ttl) {
            self.raise_alert("unusual_packet_ttl", 0.20);
        }

        if packet.is_syn() && !packet.is_ack() {
            let syn = SynCharacteristics {
                ip: build_ip_characteristics(ipv4),
                flags: build_tcp_flags(packet),
                window_size: tcp.window_size,
                options: map_options(&packet.tcp_options),
                option_order: option_order(&packet.tcp_options),
            };
            self.evaluate_syn(packet, &syn);
            self.current_tcp = Some(build_tcp_fingerprint(&syn));
            self.syn = Some(syn);
        } else if packet.is_syn_ack() {
            if self.syn.is_none() {
                self.raise_alert("syn_ack_before_syn", 0.35);
            }
            let syn_ack = SynAckCharacteristics {
                ip: build_ip_characteristics(ipv4),
                flags: build_tcp_flags(packet),
                window_size: tcp.window_size,
                options: map_options(&packet.tcp_options),
                option_order: option_order(&packet.tcp_options),
            };
            self.syn_ack = Some(syn_ack);
        } else if packet.is_ack() {
            if self.syn.is_none() {
                self.raise_alert("ack_before_syn", 0.45);
            } else if self.syn_ack.is_none() {
                self.raise_alert("ack_before_syn_ack", 0.25);
            }

            let ack = AckCharacteristics {
                ip: build_ip_characteristics(ipv4),
                flags: build_tcp_flags(packet),
                window_size: tcp.window_size,
                options: map_options(&packet.tcp_options),
                option_order: option_order(&packet.tcp_options),
            };
            self.evaluate_ack(&ack);
            self.ack = Some(ack);
        }

        Ok(())
    }

    pub fn anomaly_score(&self) -> f32 {
        self.anomaly_score
    }

    pub fn alert_reasons(&self) -> &[String] {
        &self.alert_reasons
    }

    pub fn packet_count(&self) -> usize {
        self.packet_count
    }

    pub fn handshake_complete(&self) -> bool {
        self.syn.is_some() && self.syn_ack.is_some() && self.ack.is_some()
    }

    pub fn current_fingerprint(&self) -> Option<&TcpFingerprint> {
        self.current_tcp.as_ref()
    }

    pub fn finalize(&self) -> IncrementalFingerprintResult {
        let handshake_fingerprint = match (&self.syn, &self.syn_ack, &self.ack) {
            (Some(syn), Some(syn_ack), Some(ack)) => {
                let mut fp =
                    TcpHandshakeFingerprint::new(syn.clone(), syn_ack.clone(), ack.clone());
                fp.detected_os = TcpHandshakeAnalyzer::detect_os(fp.ttl_sequence());
                fp.detected_browser = TcpHandshakeAnalyzer::detect_browser(&fp.syn.option_order)
                    .or_else(|| {
                        extract_mss(&fp.syn.options).and_then(TcpHandshakeAnalyzer::detect_from_mss)
                    });
                fp.confidence = (1.0_f32 - self.anomaly_score).clamp(0.0, 1.0) as f64;
                Some(fp)
            }
            _ => None,
        };

        IncrementalFingerprintResult {
            tcp_fingerprint: self.current_tcp.clone(),
            handshake_fingerprint,
            anomaly_score: self.anomaly_score,
            alert_reasons: self.alert_reasons.clone(),
            packet_count: self.packet_count,
        }
    }

    fn evaluate_syn(&mut self, packet: &IncrementalPacket, syn: &SynCharacteristics) {
        if syn.window_size == 0 {
            self.raise_alert("zero_window_on_syn", 0.45);
        }

        if !is_common_initial_ttl(syn.ip.ttl) {
            self.raise_alert("unusual_syn_ttl", 0.20);
        }

        if extract_mss(&syn.options).is_none() {
            self.raise_alert("missing_mss_option", 0.15);
        }

        if packet.tcp_options.len() > 8 {
            self.raise_alert("excessive_tcp_option_count", 0.10);
        }
    }

    fn evaluate_ack(&mut self, ack: &AckCharacteristics) {
        if let Some(syn) = &self.syn {
            let ttl_delta = syn.ip.ttl.abs_diff(ack.ip.ttl);
            if ttl_delta > 32 {
                self.raise_alert("ttl_jump_between_syn_and_ack", 0.20);
            }
        }
    }

    fn raise_alert(&mut self, reason: &str, score: f32) {
        if !self.alert_reasons.iter().any(|existing| existing == reason) {
            self.alert_reasons.push(reason.to_string());
        }
        self.anomaly_score = (self.anomaly_score + score).clamp(0.0, 1.0);
    }
}

fn build_ip_characteristics(ipv4: &IncrementalIpv4Packet) -> IpCharacteristics {
    IpCharacteristics {
        ttl: ipv4.ttl,
        dont_fragment: ipv4.dont_fragment,
        ip_id: ipv4.identification as u32,
        ip_id_increment: None,
    }
}

fn build_tcp_flags(packet: &IncrementalPacket) -> TcpFlags {
    let tcp = packet.tcp.as_ref().expect("tcp packet already validated");
    TcpFlags {
        syn: tcp.syn(),
        ack: tcp.ack(),
        fin: tcp.fin(),
        rst: tcp.rst(),
        psh: tcp.psh(),
        urg: tcp.urg(),
    }
}

fn option_order(options: &[IncrementalTcpOption]) -> String {
    options
        .iter()
        .filter_map(|option| match option.kind {
            IncrementalTcpOptionKind::MSS => Some("MSS"),
            IncrementalTcpOptionKind::WindowScale => Some("WSCALE"),
            IncrementalTcpOptionKind::SackPermitted => Some("SACK"),
            IncrementalTcpOptionKind::Timestamp => Some("Timestamp"),
            IncrementalTcpOptionKind::TcpFastOpen => Some("TFO"),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn map_options(options: &[IncrementalTcpOption]) -> Vec<TcpOption> {
    options
        .iter()
        .filter_map(|option| match option.kind {
            IncrementalTcpOptionKind::MSS if option.data.len() >= 2 => {
                Some(TcpOption::mss(u16::from_be_bytes([
                    option.data[0],
                    option.data[1],
                ])))
            }
            IncrementalTcpOptionKind::WindowScale if !option.data.is_empty() => {
                Some(TcpOption::wscale(option.data[0]))
            }
            IncrementalTcpOptionKind::SackPermitted => Some(TcpOption::sack_permitted()),
            IncrementalTcpOptionKind::Timestamp if option.data.len() >= 8 => {
                Some(TcpOption::timestamp(
                    u32::from_be_bytes([
                        option.data[0],
                        option.data[1],
                        option.data[2],
                        option.data[3],
                    ]),
                    u32::from_be_bytes([
                        option.data[4],
                        option.data[5],
                        option.data[6],
                        option.data[7],
                    ]),
                ))
            }
            IncrementalTcpOptionKind::TcpFastOpen => Some(TcpOption {
                option_type: TcpOptionType::TFO,
                length: option.length,
                value: if option.data.is_empty() {
                    None
                } else {
                    Some(option.data.clone())
                },
            }),
            _ => None,
        })
        .collect()
}

fn build_tcp_fingerprint(syn: &SynCharacteristics) -> TcpFingerprint {
    let mss = extract_mss(&syn.options);
    let window_scale = syn
        .options
        .iter()
        .find_map(|option| match option.option_type {
            TcpOptionType::WSCALE => option
                .value
                .as_ref()
                .and_then(|value| value.first().copied()),
            _ => None,
        });
    TcpFingerprint::with_options(syn.ip.ttl, syn.window_size, mss, window_scale)
}

fn extract_mss(options: &[TcpOption]) -> Option<u16> {
    options.iter().find_map(|option| match option.option_type {
        TcpOptionType::MSS => option.value.as_ref().and_then(|value| {
            if value.len() >= 2 {
                Some(u16::from_be_bytes([value[0], value[1]]))
            } else {
                None
            }
        }),
        _ => None,
    })
}

fn is_common_initial_ttl(ttl: u8) -> bool {
    matches!(ttl, 32 | 60..=64 | 127..=128 | 254..=255)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fingerprint_parsers::packet_capture::{
        PacketParser, ParsedPacket, ParsedTcpOption, ParsedTcpOptionKind,
    };

    struct TestTcpFrame {
        src_ip: [u8; 4],
        dst_ip: [u8; 4],
        seq: u32,
        ack: u32,
        flags: u16,
        ttl: u8,
        window: u16,
    }

    fn build_tcp_frame(frame_spec: TestTcpFrame, options: &[u8]) -> Vec<u8> {
        let mut padded_options = options.to_vec();
        while !padded_options.len().is_multiple_of(4) {
            padded_options.push(1);
        }
        let tcp_header_len = 20 + padded_options.len();
        let total_len = 20 + tcp_header_len;
        let mut frame = Vec::with_capacity(14 + total_len);
        frame.extend_from_slice(&[0, 1, 2, 3, 4, 5]);
        frame.extend_from_slice(&[6, 7, 8, 9, 10, 11]);
        frame.extend_from_slice(&[0x08, 0x00]);
        frame.push(0x45);
        frame.push(0x00);
        frame.extend_from_slice(&(total_len as u16).to_be_bytes());
        frame.extend_from_slice(&[0x12, 0x34]);
        frame.extend_from_slice(&[0x40, 0x00]);
        frame.push(frame_spec.ttl);
        frame.push(6);
        frame.extend_from_slice(&[0x00, 0x00]);
        frame.extend_from_slice(&frame_spec.src_ip);
        frame.extend_from_slice(&frame_spec.dst_ip);
        frame.extend_from_slice(&54321u16.to_be_bytes());
        frame.extend_from_slice(&443u16.to_be_bytes());
        frame.extend_from_slice(&frame_spec.seq.to_be_bytes());
        frame.extend_from_slice(&frame_spec.ack.to_be_bytes());
        let data_offset = ((tcp_header_len / 4) as u8) << 4;
        frame.push(data_offset);
        frame.push((frame_spec.flags & 0xff) as u8);
        frame.extend_from_slice(&frame_spec.window.to_be_bytes());
        frame.extend_from_slice(&[0x00, 0x00]);
        frame.extend_from_slice(&[0x00, 0x00]);
        frame.extend_from_slice(&padded_options);
        frame
    }

    fn option_kind_from_parsed(kind: ParsedTcpOptionKind) -> IncrementalTcpOptionKind {
        match kind {
            ParsedTcpOptionKind::EndOfList => IncrementalTcpOptionKind::EndOfList,
            ParsedTcpOptionKind::NoOperation => IncrementalTcpOptionKind::NoOperation,
            ParsedTcpOptionKind::MSS => IncrementalTcpOptionKind::MSS,
            ParsedTcpOptionKind::WindowScale => IncrementalTcpOptionKind::WindowScale,
            ParsedTcpOptionKind::SackPermitted => IncrementalTcpOptionKind::SackPermitted,
            ParsedTcpOptionKind::Timestamp => IncrementalTcpOptionKind::Timestamp,
            ParsedTcpOptionKind::TcpFastOpen => IncrementalTcpOptionKind::TcpFastOpen,
            ParsedTcpOptionKind::Unknown(value) => IncrementalTcpOptionKind::Unknown(value),
        }
    }

    fn option_from_parsed(option: &ParsedTcpOption) -> IncrementalTcpOption {
        IncrementalTcpOption {
            kind: option_kind_from_parsed(option.kind),
            length: option.length,
            data: option.data.clone(),
        }
    }

    fn packet_from_parsed(packet: &ParsedPacket) -> IncrementalPacket {
        IncrementalPacket::new(
            packet.ipv4.as_ref().map(|ipv4| {
                IncrementalIpv4Packet::new(ipv4.ttl, ipv4.df_flag(), ipv4.identification)
            }),
            packet
                .tcp
                .as_ref()
                .map(|tcp| IncrementalTcpSegment::new(tcp.window_size, tcp.data_offset_flags)),
            packet.tcp_options.iter().map(option_from_parsed).collect(),
        )
    }

    #[test]
    fn incremental_tcp_fingerprint_tracks_syn_before_flow_completion() {
        let syn_options = [0x02, 0x04, 0x05, 0xb4, 0x03, 0x03, 0x08, 0x04, 0x02];
        let syn = build_tcp_frame(
            TestTcpFrame {
                src_ip: [192, 168, 1, 10],
                dst_ip: [93, 184, 216, 34],
                seq: 1,
                ack: 0,
                flags: 0x0002,
                ttl: 64,
                window: 64240,
            },
            &syn_options,
        );
        let parsed = PacketParser::parse_packet(&syn, 1, 0).unwrap();
        let observation = packet_from_parsed(&parsed);

        let mut incremental = IncrementalTcpFingerprint::new();
        incremental.update(&observation).unwrap();

        let current = incremental.current_fingerprint().unwrap();
        assert_eq!(current.ttl, 64);
        assert_eq!(current.window_size, 64240);
        assert_eq!(current.mss, Some(1460));
        assert_eq!(current.window_scale, Some(8));
        assert!(!incremental.handshake_complete());
    }

    #[test]
    fn incremental_tcp_fingerprint_finalizes_after_handshake() {
        let syn_options = [0x02, 0x04, 0x05, 0xb4, 0x03, 0x03, 0x08, 0x04, 0x02];
        let syn_ack_options = [0x02, 0x04, 0x05, 0xb4, 0x03, 0x03, 0x08, 0x04, 0x02];
        let ack_options: [u8; 0] = [];

        let packets = [
            build_tcp_frame(
                TestTcpFrame {
                    src_ip: [192, 168, 1, 10],
                    dst_ip: [93, 184, 216, 34],
                    seq: 1,
                    ack: 0,
                    flags: 0x0002,
                    ttl: 64,
                    window: 64240,
                },
                &syn_options,
            ),
            build_tcp_frame(
                TestTcpFrame {
                    src_ip: [93, 184, 216, 34],
                    dst_ip: [192, 168, 1, 10],
                    seq: 10,
                    ack: 2,
                    flags: 0x0012,
                    ttl: 64,
                    window: 65535,
                },
                &syn_ack_options,
            ),
            build_tcp_frame(
                TestTcpFrame {
                    src_ip: [192, 168, 1, 10],
                    dst_ip: [93, 184, 216, 34],
                    seq: 2,
                    ack: 11,
                    flags: 0x0010,
                    ttl: 64,
                    window: 64240,
                },
                &ack_options,
            ),
        ];

        let mut incremental = IncrementalTcpFingerprint::new();
        for (idx, bytes) in packets.iter().enumerate() {
            let packet = PacketParser::parse_packet(bytes, idx as u32 + 1, 0).unwrap();
            let observation = packet_from_parsed(&packet);
            incremental.update(&observation).unwrap();
        }

        let result = incremental.finalize();
        assert!(incremental.handshake_complete());
        assert!(result.handshake_fingerprint.is_some());
        let handshake = result.handshake_fingerprint.unwrap();
        assert_eq!(handshake.signature(), "MSS,WSCALE,SACK-MSS,WSCALE,SACK-");
        assert_eq!(handshake.detected_os.as_deref(), Some("Unix-like"));
    }

    #[test]
    fn incremental_tcp_fingerprint_flags_unusual_sequence_early() {
        let ack = build_tcp_frame(
            TestTcpFrame {
                src_ip: [192, 168, 1, 10],
                dst_ip: [93, 184, 216, 34],
                seq: 2,
                ack: 11,
                flags: 0x0010,
                ttl: 17,
                window: 0,
            },
            &[],
        );
        let parsed = PacketParser::parse_packet(&ack, 1, 0).unwrap();
        let observation = packet_from_parsed(&parsed);

        let mut incremental = IncrementalTcpFingerprint::new();
        incremental.update(&observation).unwrap();

        assert!(incremental.anomaly_score() >= 0.65);
        assert!(incremental
            .alert_reasons()
            .iter()
            .any(|reason| reason == "ack_before_syn"));
    }
}
