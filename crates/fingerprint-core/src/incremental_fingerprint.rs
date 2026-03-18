use crate::tcp::TcpFingerprint;
use crate::tcp_handshake::{
    AckCharacteristics, IpCharacteristics, SynAckCharacteristics, SynCharacteristics, TcpFlags,
    TcpHandshakeAnalyzer, TcpHandshakeFingerprint, TcpOption, TcpOptionType,
};
use fingerprint_parsers::packet_capture::{
    Ipv4Header, ParsedPacket, ParsedTcpOption, ParsedTcpOptionKind,
};

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

    pub fn update(&mut self, packet: &ParsedPacket) -> Result<(), String> {
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

    fn evaluate_syn(&mut self, packet: &ParsedPacket, syn: &SynCharacteristics) {
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

fn build_ip_characteristics(ipv4: &Ipv4Header) -> IpCharacteristics {
    IpCharacteristics {
        ttl: ipv4.ttl,
        dont_fragment: ipv4.df_flag(),
        ip_id: ipv4.identification as u32,
        ip_id_increment: None,
    }
}

fn build_tcp_flags(packet: &ParsedPacket) -> TcpFlags {
    let tcp = packet.tcp.as_ref().expect("tcp packet already validated");
    TcpFlags {
        syn: tcp.syn(),
        ack: tcp.ack(),
        fin: tcp.fin(),
        rst: tcp.rst(),
        psh: tcp.psh(),
        urg: (tcp.data_offset_flags & 0x0020) != 0,
    }
}

fn option_order(options: &[ParsedTcpOption]) -> String {
    options
        .iter()
        .filter_map(|option| match option.kind {
            ParsedTcpOptionKind::MSS => Some("MSS"),
            ParsedTcpOptionKind::WindowScale => Some("WSCALE"),
            ParsedTcpOptionKind::SackPermitted => Some("SACK"),
            ParsedTcpOptionKind::Timestamp => Some("Timestamp"),
            ParsedTcpOptionKind::TcpFastOpen => Some("TFO"),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn map_options(options: &[ParsedTcpOption]) -> Vec<TcpOption> {
    options
        .iter()
        .filter_map(|option| match option.kind {
            ParsedTcpOptionKind::MSS if option.data.len() >= 2 => {
                Some(TcpOption::mss(u16::from_be_bytes([
                    option.data[0],
                    option.data[1],
                ])))
            }
            ParsedTcpOptionKind::WindowScale if !option.data.is_empty() => {
                Some(TcpOption::wscale(option.data[0]))
            }
            ParsedTcpOptionKind::SackPermitted => Some(TcpOption::sack_permitted()),
            ParsedTcpOptionKind::Timestamp if option.data.len() >= 8 => Some(TcpOption::timestamp(
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
            )),
            ParsedTcpOptionKind::TcpFastOpen => Some(TcpOption {
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
    use fingerprint_parsers::packet_capture::PacketParser;

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

        let mut incremental = IncrementalTcpFingerprint::new();
        incremental.update(&parsed).unwrap();

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
            incremental.update(&packet).unwrap();
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

        let mut incremental = IncrementalTcpFingerprint::new();
        incremental.update(&parsed).unwrap();

        assert!(incremental.anomaly_score() >= 0.65);
        assert!(incremental
            .alert_reasons()
            .iter()
            .any(|reason| reason == "ack_before_syn"));
    }
}
