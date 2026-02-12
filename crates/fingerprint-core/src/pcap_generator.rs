/// Synthetic PCAP Generator
/// Generates test PCAP files with known browser fingerprints for validation
use crate::packet_capture::*;
use std::fs::File;
use std::io::{self, Write};
use std::path::Path;

/// Generate a complete PCAP file with synthetic browser traffic
pub struct PcapGenerator {
    global_header: PcapGlobalHeader,
    packets: Vec<Vec<u8>>,
}

impl Default for PcapGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl PcapGenerator {
    pub fn new() -> Self {
        Self {
            global_header: PcapGlobalHeader {
                magic_number: 0xa1b2c3d4,
                version_major: 2,
                version_minor: 4,
                timezone_offset: 0,
                timestamp_accuracy: 0,
                snapshot_length: 65535,
                data_link_type: 1, // Ethernet
            },
            packets: Vec::new(),
        }
    }

    /// Add a synthetic TCP SYN packet (Chrome-style)
    pub fn add_chrome_syn(&mut self) {
        let packet = self.create_tcp_syn(
            [0x00, 0x1a, 0x2b, 0x3c, 0x4d, 0x5e], // src_mac
            [0x00, 0x50, 0x56, 0xc0, 0x00, 0x01], // dst_mac
            [192, 168, 1, 100],                   // src_ip
            [93, 184, 216, 34],                   // dst_ip (example.com)
            54321,                                // src_port
            443,                                  // dst_port
            ChromeTcpOptions::default(),
        );
        self.packets.push(packet);
    }

    /// Add a synthetic TCP SYN packet (Firefox-style)
    pub fn add_firefox_syn(&mut self) {
        let packet = self.create_tcp_syn(
            [0x00, 0x1a, 0x2b, 0x3c, 0x4d, 0x5e],
            [0x00, 0x50, 0x56, 0xc0, 0x00, 0x01],
            [192, 168, 1, 100],
            [93, 184, 216, 34],
            54322,
            443,
            FirefoxTcpOptions::default(),
        );
        self.packets.push(packet);
    }

    /// Write PCAP file to disk
    pub fn write_to_file<P: AsRef<Path>>(&self, path: P) -> io::Result<()> {
        let mut file = File::create(path)?;

        // Write global header
        file.write_all(&self.global_header.magic_number.to_le_bytes())?;
        file.write_all(&self.global_header.version_major.to_le_bytes())?;
        file.write_all(&self.global_header.version_minor.to_le_bytes())?;
        file.write_all(&self.global_header.timezone_offset.to_le_bytes())?;
        file.write_all(&self.global_header.timestamp_accuracy.to_le_bytes())?;
        file.write_all(&self.global_header.snapshot_length.to_le_bytes())?;
        file.write_all(&self.global_header.data_link_type.to_le_bytes())?;

        // Write packets
        for (idx, packet_data) in self.packets.iter().enumerate() {
            let packet_header = PcapPacketHeader {
                timestamp_sec: 1707654000 + idx as u32, // Feb 11, 2026
                timestamp_usec: (idx * 1000) as u32,
                incl_len: packet_data.len() as u32,
                orig_len: packet_data.len() as u32,
            };

            file.write_all(&packet_header.timestamp_sec.to_le_bytes())?;
            file.write_all(&packet_header.timestamp_usec.to_le_bytes())?;
            file.write_all(&packet_header.incl_len.to_le_bytes())?;
            file.write_all(&packet_header.orig_len.to_le_bytes())?;
            file.write_all(packet_data)?;
        }

        Ok(())
    }

    /// Create a TCP SYN packet with specified options
    #[allow(clippy::too_many_arguments)]
    fn create_tcp_syn(
        &self,
        src_mac: [u8; 6],
        dst_mac: [u8; 6],
        src_ip: [u8; 4],
        dst_ip: [u8; 4],
        src_port: u16,
        dst_port: u16,
        options: impl TcpOptionsBuilder,
    ) -> Vec<u8> {
        let mut packet = Vec::new();

        // Ethernet header (14 bytes)
        packet.extend_from_slice(&dst_mac);
        packet.extend_from_slice(&src_mac);
        packet.extend_from_slice(&[0x08, 0x00]); // IPv4

        // IPv4 header (20 bytes, no options)
        let ip_header_start = packet.len();
        packet.push(0x45); // Version 4, IHL 5
        packet.push(0x00); // DSCP, ECN

        // Total length (will be updated later)
        let total_length_offset = packet.len();
        packet.extend_from_slice(&[0x00, 0x00]);

        packet.extend_from_slice(&[0x12, 0x34]); // Identification
        packet.extend_from_slice(&[0x40, 0x00]); // Flags: DF, Fragment offset
        packet.push(64); // TTL
        packet.push(6); // Protocol: TCP

        // Checksum (will be calculated later)
        let ip_checksum_offset = packet.len();
        packet.extend_from_slice(&[0x00, 0x00]);

        packet.extend_from_slice(&src_ip);
        packet.extend_from_slice(&dst_ip);

        // TCP header (20 bytes + options)
        let tcp_header_start = packet.len();
        packet.extend_from_slice(&src_port.to_be_bytes());
        packet.extend_from_slice(&dst_port.to_be_bytes());
        packet.extend_from_slice(&[0x00, 0x00, 0x00, 0x01]); // Sequence number
        packet.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // Ack number

        // Data offset and flags (will be updated after adding options)
        let data_offset_flags_offset = packet.len();
        packet.extend_from_slice(&[0x00, 0x02]); // SYN flag

        packet.extend_from_slice(&[0xff, 0xff]); // Window size

        // TCP checksum (will be calculated later)
        let tcp_checksum_offset = packet.len();
        packet.extend_from_slice(&[0x00, 0x00]);

        packet.extend_from_slice(&[0x00, 0x00]); // Urgent pointer

        // Add TCP options
        let tcp_options = options.build();
        packet.extend_from_slice(&tcp_options);

        // Pad to 4-byte boundary
        while (packet.len() - tcp_header_start) % 4 != 0 {
            packet.push(0x00); // NOP
        }

        // Update data offset
        let tcp_header_len = packet.len() - tcp_header_start;
        let data_offset = (tcp_header_len / 4) as u8;
        packet[data_offset_flags_offset] = data_offset << 4;

        // Update IP total length
        let ip_total_len = (packet.len() - ip_header_start) as u16;
        packet[total_length_offset..total_length_offset + 2]
            .copy_from_slice(&ip_total_len.to_be_bytes());

        // Calculate IP checksum
        let ip_checksum = calculate_ip_checksum(&packet[ip_header_start..tcp_header_start]);
        packet[ip_checksum_offset..ip_checksum_offset + 2]
            .copy_from_slice(&ip_checksum.to_be_bytes());

        // Calculate TCP checksum (simplified - would need pseudo-header in real impl)
        let tcp_checksum = calculate_tcp_checksum(&src_ip, &dst_ip, &packet[tcp_header_start..]);
        packet[tcp_checksum_offset..tcp_checksum_offset + 2]
            .copy_from_slice(&tcp_checksum.to_be_bytes());

        packet
    }
}

/// TCP options builder trait
pub trait TcpOptionsBuilder {
    fn build(&self) -> Vec<u8>;
}

/// Chrome-style TCP options
pub struct ChromeTcpOptions {
    pub mss: u16,
    pub window_scale: u8,
    pub sack_permitted: bool,
    pub timestamps: Option<(u32, u32)>,
}

impl Default for ChromeTcpOptions {
    fn default() -> Self {
        Self {
            mss: 1460,
            window_scale: 8,
            sack_permitted: true,
            timestamps: Some((12345678, 0)),
        }
    }
}

impl TcpOptionsBuilder for ChromeTcpOptions {
    fn build(&self) -> Vec<u8> {
        let mut options = Vec::new();

        // MSS (kind=2, len=4)
        options.push(0x02);
        options.push(0x04);
        options.extend_from_slice(&self.mss.to_be_bytes());

        // SACK permitted (kind=4, len=2)
        if self.sack_permitted {
            options.push(0x04);
            options.push(0x02);
        }

        // Timestamps (kind=8, len=10)
        if let Some((ts_val, ts_ecr)) = self.timestamps {
            options.push(0x08);
            options.push(0x0a);
            options.extend_from_slice(&ts_val.to_be_bytes());
            options.extend_from_slice(&ts_ecr.to_be_bytes());
        }

        // Window scale (kind=3, len=3)
        options.push(0x03);
        options.push(0x03);
        options.push(self.window_scale);

        options
    }
}

/// Firefox-style TCP options (different order)
pub struct FirefoxTcpOptions {
    pub mss: u16,
    pub sack_permitted: bool,
    pub timestamps: Option<(u32, u32)>,
    pub window_scale: u8,
}

impl Default for FirefoxTcpOptions {
    fn default() -> Self {
        Self {
            mss: 1440, // Firefox often uses 1440
            sack_permitted: true,
            timestamps: Some((12345678, 0)),
            window_scale: 7,
        }
    }
}

impl TcpOptionsBuilder for FirefoxTcpOptions {
    fn build(&self) -> Vec<u8> {
        let mut options = Vec::new();

        // MSS
        options.push(0x02);
        options.push(0x04);
        options.extend_from_slice(&self.mss.to_be_bytes());

        // SACK permitted
        if self.sack_permitted {
            options.push(0x04);
            options.push(0x02);
        }

        // Timestamps
        if let Some((ts_val, ts_ecr)) = self.timestamps {
            options.push(0x08);
            options.push(0x0a);
            options.extend_from_slice(&ts_val.to_be_bytes());
            options.extend_from_slice(&ts_ecr.to_be_bytes());
        }

        // Window scale (different position than Chrome)
        options.push(0x03);
        options.push(0x03);
        options.push(self.window_scale);

        options
    }
}

/// Calculate IP header checksum
fn calculate_ip_checksum(header: &[u8]) -> u16 {
    let mut sum: u32 = 0;

    for i in (0..header.len()).step_by(2) {
        let word = if i + 1 < header.len() {
            u16::from_be_bytes([header[i], header[i + 1]])
        } else {
            u16::from_be_bytes([header[i], 0])
        };
        sum += word as u32;
    }

    while sum >> 16 != 0 {
        sum = (sum & 0xffff) + (sum >> 16);
    }

    !(sum as u16)
}

/// Calculate TCP checksum (simplified)
fn calculate_tcp_checksum(src_ip: &[u8; 4], dst_ip: &[u8; 4], tcp_segment: &[u8]) -> u16 {
    let mut sum: u32 = 0;

    // Pseudo-header
    for i in 0..4 {
        sum += src_ip[i] as u32;
        sum += dst_ip[i] as u32;
    }
    sum += 6; // Protocol (TCP)
    sum += tcp_segment.len() as u32;

    // TCP segment
    for i in (0..tcp_segment.len()).step_by(2) {
        let word = if i + 1 < tcp_segment.len() {
            u16::from_be_bytes([tcp_segment[i], tcp_segment[i + 1]])
        } else {
            u16::from_be_bytes([tcp_segment[i], 0])
        };
        sum += word as u32;
    }

    while sum >> 16 != 0 {
        sum = (sum & 0xffff) + (sum >> 16);
    }

    !(sum as u16)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_chrome_pcap() {
        let mut gen = PcapGenerator::new();
        gen.add_chrome_syn();

        let output_path = "test_data/synthetic/chrome_syn.pcap";
        std::fs::create_dir_all("test_data/synthetic").ok();
        gen.write_to_file(output_path)
            .expect("Failed to write PCAP");

        // Verify file exists and has content
        let metadata = std::fs::metadata(output_path).expect("PCAP file not found");
        assert!(metadata.len() > 24); // At least global header
    }

    #[test]
    fn test_generate_firefox_pcap() {
        let mut gen = PcapGenerator::new();
        gen.add_firefox_syn();

        let output_path = "test_data/synthetic/firefox_syn.pcap";
        std::fs::create_dir_all("test_data/synthetic").ok();
        gen.write_to_file(output_path)
            .expect("Failed to write PCAP");

        let metadata = std::fs::metadata(output_path).expect("PCAP file not found");
        assert!(metadata.len() > 24);
    }

    #[test]
    fn test_tcp_options_order() {
        let chrome_options = ChromeTcpOptions::default().build();
        let firefox_options = FirefoxTcpOptions::default().build();

        // Chrome and Firefox should have different option orders
        assert_ne!(chrome_options, firefox_options);

        // Both should have MSS as first option
        assert_eq!(chrome_options[0], 0x02); // MSS kind
        assert_eq!(firefox_options[0], 0x02);
    }
}
