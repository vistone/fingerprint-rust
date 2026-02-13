//! QUIC Protocol Fingerprinting Module
//!
//! Implements detailed analysis of QUIC (RFC 9000) Initial packets to create fingerprints
//! that distinguish different QUIC clients and implementations.
//!
//! # QUIC Initial Packet Structure (RFC 9000)
//! ```text
//! Initial Packet {
//!   Header Form (1),              // 1 = Long Header Form
//!   Fixed Bit (1),                // Must be 1
//!   Spin Bit (1),                 // Latency sampling
//!   Reserved (2),                 // Must be 0
//!   Type (2),                      // 0x00 = Initial
//!   Version (32),                 // QUIC version
//!   DCID Len (8),                 // Destination Connection ID length
//!   Destination Connection ID,
//!   SCID Len (8),                 // Source Connection ID length
//!   Source Connection ID,
//!   Token Length (i),             // Variable length integer
//!   Token,
//!   Length (i),                   // Length of Packet Number + Payload
//!   Packet Number (8..32),
//!   Payload (encrypted)
//! }
//! ```

use std::fmt;

/// QUIC Version enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum QuicVersion {
    V1, // RFC 9000
    V2, // RFC 9369
    Draft29,
    Draft30,
    Draft31,
    Draft32,
    Unknown(u32),
}

impl QuicVersion {
    pub fn from_u32(v: u32) -> Self {
        match v {
            0x00000001 => QuicVersion::V1,
            0x6b3343cf => QuicVersion::V2,
            0xff00001d => QuicVersion::Draft29,
            0xff00001e => QuicVersion::Draft30,
            0xff00001f => QuicVersion::Draft31,
            0xff000020 => QuicVersion::Draft32,
            _ => QuicVersion::Unknown(v),
        }
    }

    pub fn to_u32(&self) -> u32 {
        match self {
            QuicVersion::V1 => 0x00000001,
            QuicVersion::V2 => 0x6b3343cf,
            QuicVersion::Draft29 => 0xff00001d,
            QuicVersion::Draft30 => 0xff00001e,
            QuicVersion::Draft31 => 0xff00001f,
            QuicVersion::Draft32 => 0xff000020,
            QuicVersion::Unknown(v) => *v,
        }
    }
}

impl fmt::Display for QuicVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            QuicVersion::V1 => write!(f, "QUIC v1 (RFC 9000)"),
            QuicVersion::V2 => write!(f, "QUIC v2 (RFC 9369)"),
            QuicVersion::Draft29 => write!(f, "QUIC Draft-29"),
            QuicVersion::Draft30 => write!(f, "QUIC Draft-30"),
            QuicVersion::Draft31 => write!(f, "QUIC Draft-31"),
            QuicVersion::Draft32 => write!(f, "QUIC Draft-32"),
            QuicVersion::Unknown(v) => write!(f, "QUIC Unknown (0x{:08x})", v),
        }
    }
}

/// QUIC Long Packet Type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum QuicPacketType {
    Initial,
    ZeroRTT,
    Handshake,
    Retry,
    Unknown(u8),
}

impl QuicPacketType {
    pub fn from_type_bits(bits: u8) -> Self {
        match bits & 0x03 {
            0x00 => QuicPacketType::Initial,
            0x01 => QuicPacketType::ZeroRTT,
            0x02 => QuicPacketType::Handshake,
            0x03 => QuicPacketType::Retry,
            _ => QuicPacketType::Unknown(bits),
        }
    }
}

impl fmt::Display for QuicPacketType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            QuicPacketType::Initial => write!(f, "Initial"),
            QuicPacketType::ZeroRTT => write!(f, "0-RTT"),
            QuicPacketType::Handshake => write!(f, "Handshake"),
            QuicPacketType::Retry => write!(f, "Retry"),
            QuicPacketType::Unknown(b) => write!(f, "Unknown (0x{:02x})", b),
        }
    }
}

/// QUIC Initial Packet Structure
#[derive(Debug, Clone)]
pub struct QuicInitialPacket {
    /// QUIC version
    pub version: QuicVersion,

    /// Packet type (Initial, 0-RTT, Handshake, Retry)
    pub packet_type: QuicPacketType,

    /// Destination Connection ID (DCID)
    pub dcid: Vec<u8>,

    /// Source Connection ID (SCID)
    pub scid: Vec<u8>,

    /// Token (user by Retry and Address Validation)
    pub token: Vec<u8>,

    /// Spin bit value
    pub spin_bit: bool,

    /// Reserved bits (should be 0x00)
    pub reserved_bits: u8,

    /// Packet Number length (1, 2, 4, or 8 bytes)
    pub packet_number_length: u8,

    /// First byte of packet (raw)
    pub first_byte: u8,

    /// Total packet length (if available)
    pub packet_length: Option<usize>,
}

impl QuicInitialPacket {
    /// Parse QUIC packet from raw bytes
    ///
    /// # Arguments
    /// * `data` - Raw packet bytes
    ///
    /// # Returns
    /// * `Ok(packet)` - Parsed QUIC packet
    /// * `Err(msg)` - Parse error message
    pub fn parse(data: &[u8]) -> Result<Self, String> {
        if data.is_empty() {
            return Err("Empty packet data".to_string());
        }

        let first_byte = data[0];

        // Check header form: must be 1 for long header
        let header_form = (first_byte >> 7) & 0x01;
        if header_form != 1 {
            return Err("Not a long header form packet".to_string());
        }

        // Check fixed bit: must be 1
        let fixed_bit = (first_byte >> 6) & 0x01;
        if fixed_bit != 1 {
            return Err("Fixed bit is not 1".to_string());
        }

        // Extract spin bit
        let spin_bit = ((first_byte >> 5) & 0x01) != 0;

        // Extract reserved bits (should be 0)
        let reserved_bits = (first_byte >> 3) & 0x03;

        // Extract packet type
        let packet_type = QuicPacketType::from_type_bits(first_byte);

        // Parse version
        if data.len() < 5 {
            return Err("Insufficient data for version field".to_string());
        }
        let version_bytes = [data[1], data[2], data[3], data[4]];
        let version_u32 = u32::from_be_bytes(version_bytes);
        let version = QuicVersion::from_u32(version_u32);

        let mut pos = 5;

        // Parse DCID
        if pos >= data.len() {
            return Err("Insufficient data for DCID length".to_string());
        }
        let dcid_len = data[pos] as usize;
        pos += 1;

        if pos + dcid_len > data.len() {
            return Err("Insufficient data for DCID".to_string());
        }
        let dcid = data[pos..pos + dcid_len].to_vec();
        pos += dcid_len;

        // Parse SCID
        if pos >= data.len() {
            return Err("Insufficient data for SCID length".to_string());
        }
        let scid_len = data[pos] as usize;
        pos += 1;

        if pos + scid_len > data.len() {
            return Err("Insufficient data for SCID".to_string());
        }
        let scid = data[pos..pos + scid_len].to_vec();
        pos += scid_len;

        // Parse Token (variable length integer)
        let (token_len, bytes_read) = parse_variable_length_integer(&data[pos..])?;
        pos += bytes_read;

        let token_len = token_len as usize;
        if pos + token_len > data.len() {
            return Err("Insufficient data for token".to_string());
        }
        let token = data[pos..pos + token_len].to_vec();
        pos += token_len;

        // Parse Length (variable length integer)
        if pos >= data.len() {
            return Err("Insufficient data for length field".to_string());
        }
        let (length, _) = parse_variable_length_integer(&data[pos..])?;
        let packet_length = length as usize;

        // Determine packet number length from reserved bits
        let packet_number_length = match (first_byte >> 3) & 0x03 {
            0 => 1,
            1 => 2,
            2 => 4,
            _ => 8,
        };

        Ok(QuicInitialPacket {
            version,
            packet_type,
            dcid,
            scid,
            token,
            spin_bit,
            reserved_bits,
            packet_number_length,
            first_byte,
            packet_length: Some(packet_length),
        })
    }

    /// Create a fingerprint string from QUIC packet characteristics
    ///
    /// Format: `quic_<version>_<dcid_len>_<scid_len>_<pkt_num_len>_<reserved>`
    pub fn fingerprint(&self) -> String {
        let version_str = match self.version {
            QuicVersion::V1 => "v1",
            QuicVersion::V2 => "v2",
            QuicVersion::Draft29 => "d29",
            QuicVersion::Draft30 => "d30",
            QuicVersion::Draft31 => "d31",
            QuicVersion::Draft32 => "d32",
            QuicVersion::Unknown(_) => "unknown",
        };

        format!(
            "quic_{}_{}_{}_{}_{}",
            version_str,
            self.dcid.len(),
            self.scid.len(),
            self.packet_number_length,
            self.reserved_bits
        )
    }

    /// Create a detailed analysis string
    pub fn analyze(&self) -> String {
        format!(
            "QUIC Initial Packet Analysis:\n\
             - Version: {}\n\
             - Packet Type: {}\n\
             - DCID Length: {} bytes\n\
             - SCID Length: {} bytes\n\
             - Token Length: {} bytes\n\
             - Spin Bit: {}\n\
             - Reserved Bits: 0x{:02x}\n\
             - Packet Number Length: {} bytes\n\
             - First Byte: 0x{:02x}\n\
             - Fingerprint: {}",
            self.version,
            self.packet_type,
            self.dcid.len(),
            self.scid.len(),
            self.token.len(),
            self.spin_bit,
            self.reserved_bits,
            self.packet_number_length,
            self.first_byte,
            self.fingerprint()
        )
    }
}

/// Parse variable-length integer (QUIC format, RFC 9000 Section 16)
fn parse_variable_length_integer(data: &[u8]) -> Result<(u64, usize), String> {
    if data.is_empty() {
        return Err("Empty data for variable length integer".to_string());
    }

    let first_byte = data[0];
    let prefix_bits = (first_byte >> 6) & 0x03;

    match prefix_bits {
        0 => Ok(((first_byte & 0x3f) as u64, 1)),
        1 => {
            if data.len() < 2 {
                return Err("Insufficient data for 2-byte varint".to_string());
            }
            let value = (((first_byte & 0x3f) as u64) << 8) | (data[1] as u64);
            Ok((value, 2))
        }
        2 => {
            if data.len() < 4 {
                return Err("Insufficient data for 4-byte varint".to_string());
            }
            let value = (((first_byte & 0x3f) as u64) << 24)
                | ((data[1] as u64) << 16)
                | ((data[2] as u64) << 8)
                | (data[3] as u64);
            Ok((value, 4))
        }
        3 => {
            if data.len() < 8 {
                return Err("Insufficient data for 8-byte varint".to_string());
            }
            let value = (((first_byte & 0x3f) as u64) << 56)
                | ((data[1] as u64) << 48)
                | ((data[2] as u64) << 40)
                | ((data[3] as u64) << 32)
                | ((data[4] as u64) << 24)
                | ((data[5] as u64) << 16)
                | ((data[6] as u64) << 8)
                | (data[7] as u64);
            Ok((value, 8))
        }
        _ => Err(format!("Invalid prefix bits: {}", prefix_bits)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quic_version_conversion() {
        let v1 = QuicVersion::from_u32(0x00000001);
        assert_eq!(v1, QuicVersion::V1);
        assert_eq!(v1.to_u32(), 0x00000001);

        let v2 = QuicVersion::from_u32(0x6b3343cf);
        assert_eq!(v2, QuicVersion::V2);

        let unknown = QuicVersion::from_u32(0xdeadbeef);
        assert!(matches!(unknown, QuicVersion::Unknown(0xdeadbeef)));
    }

    #[test]
    fn test_packet_type_extraction() {
        let initial = QuicPacketType::from_type_bits(0x00);
        assert_eq!(initial, QuicPacketType::Initial);

        let zero_rtt = QuicPacketType::from_type_bits(0x01);
        assert_eq!(zero_rtt, QuicPacketType::ZeroRTT);

        let handshake = QuicPacketType::from_type_bits(0x02);
        assert_eq!(handshake, QuicPacketType::Handshake);

        let retry = QuicPacketType::from_type_bits(0x03);
        assert_eq!(retry, QuicPacketType::Retry);
    }

    #[test]
    fn test_variable_length_integer() {
        // Single byte: 0x25 = 37
        let data = vec![0x25];
        let (val, len) = parse_variable_length_integer(&data).unwrap();
        assert_eq!(val, 37);
        assert_eq!(len, 1);

        // Two bytes: 0x7bbd = 15293
        let data = vec![0x40, 0x25];
        let (val, len) = parse_variable_length_integer(&data).unwrap();
        assert_eq!(val, 0x25);
        assert_eq!(len, 2);
    }

    #[test]
    fn test_quic_initial_packet_fingerprint() {
        let packet = QuicInitialPacket {
            version: QuicVersion::V1,
            packet_type: QuicPacketType::Initial,
            dcid: vec![1, 2, 3, 4],
            scid: vec![5, 6, 7, 8],
            token: vec![],
            spin_bit: false,
            reserved_bits: 0x00,
            packet_number_length: 4,
            first_byte: 0xc0,
            packet_length: Some(1200),
        };

        let fp = packet.fingerprint();
        assert!(fp.contains("quic_v1"));
        assert!(fp.contains("4_4")); // DCID and SCID lengths
        assert!(fp.contains("4")); // Packet number length
    }
}
