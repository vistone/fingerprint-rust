//! TLS ClientHello Parser
//!
//! Parses TLS ClientHello messages from TCP payloads for browser fingerprinting.
//! Extracts cipher suites, extensions, curves, and other fingerprint features.

use crate::signature::ClientHelloSignature;
use crate::version::TlsVersion;
use crate::dicttls::supported_groups::CurveID;
use std::fmt;

/// TLS Parse Error
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TlsParseError {
    TooShort,
    InvalidContentType,
    InvalidRecordVersion,
    InvalidHandshakeType,
    InvalidLength,
    MalformedExtension,
}

impl fmt::Display for TlsParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TlsParseError::TooShort => write!(f, "Buffer too short for TLS record"),
            TlsParseError::InvalidContentType => write!(f, "Invalid TLS content type"),
            TlsParseError::InvalidRecordVersion => write!(f, "Invalid TLS record version"),
            TlsParseError::InvalidHandshakeType => write!(f, "Not a ClientHello message"),
            TlsParseError::InvalidLength => write!(f, "Invalid length field"),
            TlsParseError::MalformedExtension => write!(f, "Malformed extension data"),
        }
    }
}

impl std::error::Error for TlsParseError {}

/// TLS Content Types
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TlsContentType {
    ChangeCipherSpec = 0x14,
    Alert = 0x15,
    Handshake = 0x16,
    ApplicationData = 0x17,
}

impl TryFrom<u8> for TlsContentType {
    type Error = TlsParseError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x14 => Ok(TlsContentType::ChangeCipherSpec),
            0x15 => Ok(TlsContentType::Alert),
            0x16 => Ok(TlsContentType::Handshake),
            0x17 => Ok(TlsContentType::ApplicationData),
            _ => Err(TlsParseError::InvalidContentType),
        }
    }
}

/// TLS Handshake Types
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TlsHandshakeType {
    ClientHello = 0x01,
    ServerHello = 0x02,
    Certificate = 0x0b,
    ServerKeyExchange = 0x0c,
    CertificateRequest = 0x0d,
    ServerHelloDone = 0x0e,
    CertificateVerify = 0x0f,
    ClientKeyExchange = 0x10,
    Finished = 0x14,
}

impl TryFrom<u8> for TlsHandshakeType {
    type Error = TlsParseError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x01 => Ok(TlsHandshakeType::ClientHello),
            0x02 => Ok(TlsHandshakeType::ServerHello),
            0x0b => Ok(TlsHandshakeType::Certificate),
            0x0c => Ok(TlsHandshakeType::ServerKeyExchange),
            0x0d => Ok(TlsHandshakeType::CertificateRequest),
            0x0e => Ok(TlsHandshakeType::ServerHelloDone),
            0x0f => Ok(TlsHandshakeType::CertificateVerify),
            0x10 => Ok(TlsHandshakeType::ClientKeyExchange),
            0x14 => Ok(TlsHandshakeType::Finished),
            _ => Err(TlsParseError::InvalidHandshakeType),
        }
    }
}

/// TLS Record Header (5 bytes)
#[derive(Debug, Clone)]
pub struct TlsRecordHeader {
    pub content_type: TlsContentType,
    pub version: u16,
    pub length: u16,
}

impl TlsRecordHeader {
    /// Parse TLS record header from bytes
    pub fn parse(data: &[u8]) -> Result<Self, TlsParseError> {
        if data.len() < 5 {
            return Err(TlsParseError::TooShort);
        }

        let content_type = TlsContentType::try_from(data[0])?;
        let version = u16::from_be_bytes([data[1], data[2]]);
        let length = u16::from_be_bytes([data[3], data[4]]);

        Ok(TlsRecordHeader {
            content_type,
            version,
            length,
        })
    }

    /// Check if this is a handshake record
    pub fn is_handshake(&self) -> bool {
        self.content_type == TlsContentType::Handshake
    }
}

/// Find ClientHello in TCP payload
///
/// Scans through TLS records to find the first ClientHello message.
/// Returns the ClientHelloSignature if found.
pub fn find_client_hello(data: &[u8]) -> Option<ClientHelloSignature> {
    let mut offset = 0;

    // Scan for TLS records
    while offset + 5 <= data.len() {
        // Try to parse TLS record header
        if let Ok(header) = TlsRecordHeader::parse(&data[offset..]) {
            if header.is_handshake() {
                let record_end = offset + 5 + header.length as usize;
                if record_end <= data.len() {
                    let handshake_data = &data[offset + 5..record_end];
                    
                    // Try to parse ClientHello
                    if let Ok(client_hello) = parse_client_hello(handshake_data) {
                        return Some(client_hello);
                    }
                }
            }
            
            // Move to next record
            offset += 5 + header.length as usize;
        } else {
            // Not a valid TLS record, try next byte
            offset += 1;
        }
    }

    None
}

/// Parse ClientHello message
fn parse_client_hello(data: &[u8]) -> Result<ClientHelloSignature, TlsParseError> {
    if data.len() < 4 {
        return Err(TlsParseError::TooShort);
    }

    // Check handshake type
    let handshake_type = TlsHandshakeType::try_from(data[0])?;
    if handshake_type != TlsHandshakeType::ClientHello {
        return Err(TlsParseError::InvalidHandshakeType);
    }

    // Handshake length (3 bytes)
    let length = u32::from_be_bytes([0, data[1], data[2], data[3]]) as usize;
    
    if data.len() < 4 + length {
        return Err(TlsParseError::TooShort);
    }

    let body = &data[4..4 + length];
    
    // Parse ClientHello body
    parse_client_hello_body(body)
}

/// Parse ClientHello body
fn parse_client_hello_body(data: &[u8]) -> Result<ClientHelloSignature, TlsParseError> {
    if data.len() < 38 {
        return Err(TlsParseError::TooShort);
    }

    let mut offset = 0;

    // Client version (2 bytes)
    let client_version = u16::from_be_bytes([data[offset], data[offset + 1]]);
    let tls_version = match client_version {
        0x0301 => TlsVersion::V1_0,
        0x0302 => TlsVersion::V1_1,
        0x0303 => TlsVersion::V1_2,
        0x0304 => TlsVersion::V1_3,
        _ => TlsVersion::V1_2, // Default
    };
    offset += 2;

    // Random (32 bytes) - skip
    offset += 32;

    // Session ID length (1 byte)
    if offset >= data.len() {
        return Err(TlsParseError::TooShort);
    }
    let session_id_len = data[offset] as usize;
    offset += 1 + session_id_len;

    // Cipher suites length (2 bytes)
    if offset + 2 > data.len() {
        return Err(TlsParseError::TooShort);
    }
    let cipher_suites_len = u16::from_be_bytes([data[offset], data[offset + 1]]) as usize;
    offset += 2;

    // Cipher suites
    let mut cipher_suites = Vec::new();
    let cipher_end = offset + cipher_suites_len;
    if cipher_end > data.len() {
        return Err(TlsParseError::TooShort);
    }
    
    for i in (offset..cipher_end).step_by(2) {
        if i + 1 < data.len() {
            let cipher = u16::from_be_bytes([data[i], data[i + 1]]);
            cipher_suites.push(cipher);
        }
    }
    offset = cipher_end;

    // Compression methods length (1 byte)
    if offset >= data.len() {
        return Err(TlsParseError::TooShort);
    }
    let compression_len = data[offset] as usize;
    offset += 1 + compression_len;

    // Extensions
    let mut extensions = Vec::new();
    let mut elliptic_curves = Vec::new();
    let mut elliptic_curve_point_formats = Vec::new();
    let mut signature_algorithms = Vec::new();
    let mut sni: Option<String> = None;
    let mut alpn: Option<String> = None;

    if offset + 2 <= data.len() {
        let extensions_len = u16::from_be_bytes([data[offset], data[offset + 1]]) as usize;
        offset += 2;

        let extensions_end = offset + extensions_len;
        if extensions_end <= data.len() {
            // Parse extensions
            while offset + 4 <= extensions_end {
                let ext_type = u16::from_be_bytes([data[offset], data[offset + 1]]);
                let ext_len = u16::from_be_bytes([data[offset + 2], data[offset + 3]]) as usize;
                offset += 4;

                extensions.push(ext_type);

                if offset + ext_len <= data.len() {
                    let ext_data = &data[offset..offset + ext_len];

                    // Parse specific extensions
                    match ext_type {
                        0x000a => {
                            // Supported Groups (Elliptic Curves)
                            if let Ok(curves) = parse_supported_groups(ext_data) {
                                elliptic_curves = curves;
                            }
                        }
                        0x000b => {
                            // EC Point Formats
                            if let Ok(formats) = parse_ec_point_formats(ext_data) {
                                elliptic_curve_point_formats = formats;
                            }
                        }
                        0x000d => {
                            // Signature Algorithms
                            if let Ok(algorithms) = parse_signature_algorithms(ext_data) {
                                signature_algorithms = algorithms;
                            }
                        }
                        0x0000 => {
                            // Server Name Indication (SNI)
                            if let Ok(server_name) = parse_sni(ext_data) {
                                sni = Some(server_name);
                            }
                        }
                        0x0010 => {
                            // ALPN
                            if let Ok(protocol) = parse_alpn(ext_data) {
                                alpn = Some(protocol);
                            }
                        }
                        _ => {}
                    }

                    offset += ext_len;
                } else {
                    break;
                }
            }
        }
    }

    let mut signature = ClientHelloSignature::new();
    signature.version = tls_version;
    signature.cipher_suites = cipher_suites;
    signature.extensions = extensions;
    signature.elliptic_curves = elliptic_curves;
    signature.elliptic_curve_point_formats = elliptic_curve_point_formats;
    signature.signature_algorithms = signature_algorithms;
    signature.sni = sni;
    signature.alpn = alpn;
    signature.id = signature.calculate_id();

    Ok(signature)
}

/// Parse Supported Groups extension
fn parse_supported_groups(data: &[u8]) -> Result<Vec<CurveID>, TlsParseError> {
    if data.len() < 2 {
        return Err(TlsParseError::MalformedExtension);
    }

    let list_len = u16::from_be_bytes([data[0], data[1]]) as usize;
    let mut curves = Vec::new();

    for i in (2..2 + list_len).step_by(2) {
        if i + 1 < data.len() {
            let curve = u16::from_be_bytes([data[i], data[i + 1]]);
            curves.push(curve);
        }
    }

    Ok(curves)
}

/// Parse EC Point Formats extension
fn parse_ec_point_formats(data: &[u8]) -> Result<Vec<u8>, TlsParseError> {
    if data.is_empty() {
        return Err(TlsParseError::MalformedExtension);
    }

    let list_len = data[0] as usize;
    let formats = data[1..1 + list_len].to_vec();

    Ok(formats)
}

/// Parse Signature Algorithms extension
fn parse_signature_algorithms(data: &[u8]) -> Result<Vec<u16>, TlsParseError> {
    if data.len() < 2 {
        return Err(TlsParseError::MalformedExtension);
    }

    let list_len = u16::from_be_bytes([data[0], data[1]]) as usize;
    let mut algorithms = Vec::new();

    for i in (2..2 + list_len).step_by(2) {
        if i + 1 < data.len() {
            let algo = u16::from_be_bytes([data[i], data[i + 1]]);
            algorithms.push(algo);
        }
    }

    Ok(algorithms)
}

/// Parse SNI extension
fn parse_sni(data: &[u8]) -> Result<String, TlsParseError> {
    if data.len() < 5 {
        return Err(TlsParseError::MalformedExtension);
    }

    let list_len = u16::from_be_bytes([data[0], data[1]]) as usize;
    if data.len() < 2 + list_len {
        return Err(TlsParseError::MalformedExtension);
    }

    // Type (1 byte) - should be 0x00 for host_name
    let name_type = data[2];
    if name_type != 0x00 {
        return Err(TlsParseError::MalformedExtension);
    }

    // Name length (2 bytes)
    let name_len = u16::from_be_bytes([data[3], data[4]]) as usize;
    if data.len() < 5 + name_len {
        return Err(TlsParseError::MalformedExtension);
    }

    // Server name
    let name_bytes = &data[5..5 + name_len];
    String::from_utf8(name_bytes.to_vec())
        .map_err(|_| TlsParseError::MalformedExtension)
}

/// Parse ALPN extension
fn parse_alpn(data: &[u8]) -> Result<String, TlsParseError> {
    if data.len() < 3 {
        return Err(TlsParseError::MalformedExtension);
    }

    let list_len = u16::from_be_bytes([data[0], data[1]]) as usize;
    if data.len() < 2 + list_len {
        return Err(TlsParseError::MalformedExtension);
    }

    // First protocol length
    let proto_len = data[2] as usize;
    if data.len() < 3 + proto_len {
        return Err(TlsParseError::MalformedExtension);
    }

    // Protocol name
    let proto_bytes = &data[3..3 + proto_len];
    String::from_utf8(proto_bytes.to_vec())
        .map_err(|_| TlsParseError::MalformedExtension)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_tls_record_header() {
        let data = vec![
            0x16, // Handshake
            0x03, 0x03, // TLS 1.2
            0x00, 0x10, // Length: 16
        ];

        let header = TlsRecordHeader::parse(&data).unwrap();
        assert_eq!(header.content_type, TlsContentType::Handshake);
        assert_eq!(header.version, 0x0303);
        assert_eq!(header.length, 16);
        assert!(header.is_handshake());
    }

    #[test]
    fn test_content_type_conversion() {
        assert_eq!(
            TlsContentType::try_from(0x16).unwrap(),
            TlsContentType::Handshake
        );
        assert_eq!(
            TlsContentType::try_from(0x17).unwrap(),
            TlsContentType::ApplicationData
        );
        assert!(TlsContentType::try_from(0xFF).is_err());
    }

    #[test]
    fn test_handshake_type_conversion() {
        assert_eq!(
            TlsHandshakeType::try_from(0x01).unwrap(),
            TlsHandshakeType::ClientHello
        );
        assert_eq!(
            TlsHandshakeType::try_from(0x02).unwrap(),
            TlsHandshakeType::ServerHello
        );
        assert!(TlsHandshakeType::try_from(0xFF).is_err());
    }
}
