//! TLS handshakelayer (Handshake Layer)
//!
//! TLS handshakemessageformat：
//! ```text
//! struct {
//! HandshakeType msg_type; // 1 byte
//! uint24 length; // 3 bytes
//! opaque body[length]; // length bytes
//! } Handshake;
//! ```

/// TLS handshaketype
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum TLSHandshakeType {
    ClientHello = 1,
    ServerHello = 2,
    NewSessionTicket = 4,
    Certificate = 11,
    ServerKeyExchange = 12,
    CertificateRequest = 13,
    ServerHelloDone = 14,
    CertificateVerify = 15,
    ClientKeyExchange = 16,
    Finished = 20,
}

impl TLSHandshakeType {
    pub fn as_u8(&self) -> u8 {
        *self as u8
    }
}

/// TLS handshakemessage
#[derive(Debug, Clone)]
pub struct TLSHandshake {
    /// messagetype
    pub msg_type: TLSHandshakeType,
    // / message体
    pub body: Vec<u8>,
}

impl TLSHandshake {
    /// Create a newhandshakemessage
    pub fn new(msg_type: TLSHandshakeType, body: Vec<u8>) -> Self {
        Self { msg_type, body }
    }

    /// Create ClientHello handshakemessage
    pub fn client_hello(body: Vec<u8>) -> Self {
        Self::new(TLSHandshakeType::ClientHello, body)
    }

    /// serialize as bytesstream
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        // Message Type (1 byte)
        bytes.push(self.msg_type.as_u8());

        // Length (3 bytes, big-endian)
        let length = self.body.len() as u32;
        bytes.push(((length >> 16) & 0xFF) as u8);
        bytes.push(((length >> 8) & 0xFF) as u8);
        bytes.push((length & 0xFF) as u8);

        // Body
        bytes.extend_from_slice(&self.body);

        bytes
    }

    /// from bytesstreamParse
    pub fn from_bytes(data: &[u8]) -> Result<(Self, usize), String> {
        if data.len() < 4 {
            return Err("countdatatoo short，unable toParsehandshakemessage".to_string());
        }

        let msg_type = match data[0] {
            1 => TLSHandshakeType::ClientHello,
            2 => TLSHandshakeType::ServerHello,
            4 => TLSHandshakeType::NewSessionTicket,
            11 => TLSHandshakeType::Certificate,
            12 => TLSHandshakeType::ServerKeyExchange,
            13 => TLSHandshakeType::CertificateRequest,
            14 => TLSHandshakeType::ServerHelloDone,
            15 => TLSHandshakeType::CertificateVerify,
            16 => TLSHandshakeType::ClientKeyExchange,
            20 => TLSHandshakeType::Finished,
            _ => return Err(format!("not知handshaketype: {}", data[0])),
        };

        // 3 byteslength
        let length = ((data[1] as u32) << 16) | ((data[2] as u32) << 8) | (data[3] as u32);
        let length = length as usize;

        if data.len() < 4 + length {
            return Err(format!(
                "countdata不complete，need {} bytes，actualonly {} bytes",
                4 + length,
                data.len()
            ));
        }

        let body = data[4..4 + length].to_vec();

        Ok((Self::new(msg_type, body), 4 + length))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handshake_serialization() {
        let body = vec![1, 2, 3, 4, 5];
        let handshake = TLSHandshake::client_hello(body.clone());

        let bytes = handshake.to_bytes();

        // Validateformat
        assert_eq!(bytes[0], 1); // ClientHello
        assert_eq!(bytes[1], 0); // Length high byte
        assert_eq!(bytes[2], 0); // Length mid byte
        assert_eq!(bytes[3], 5); // Length low byte
        assert_eq!(&bytes[4..], &body);
    }

    #[test]
    fn test_handshake_deserialization() {
        let data = vec![1, 0, 0, 5, 1, 2, 3, 4, 5];
        let (handshake, consumed) = TLSHandshake::from_bytes(&data).unwrap();

        assert_eq!(handshake.msg_type, TLSHandshakeType::ClientHello);
        assert_eq!(handshake.body, vec![1, 2, 3, 4, 5]);
        assert_eq!(consumed, 9);
    }
}
