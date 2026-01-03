//! TLS recordlayer (Record Layer)
//!
//! TLS recordformat：
//! ```text
//! struct {
//! ContentType type; // 1 byte
//! ProtocolVersion version; // 2 bytes
//! uint16 length; // 2 bytes
//! opaque fragment[length]; // length bytes
//! } TLSPlaintext;
//! ```

/// TLS recordtype
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum TLSRecordType {
 ChangeCipherSpec = 20,
 Alert = 21,
 Handshake = 22,
 ApplicationData = 23,
}

impl TLSRecordType {
 pub fn as_u8(&self) -> u8 {
 *self as u8
 }
}

/// TLS record
#[derive(Debug, Clone)]
pub struct TLSRecord {
 /// insidecontaintype
 pub content_type: TLSRecordType,
 /// protocolversion (usually is TLS 1.0 = 0x0301,  for compatibleproperty)
 pub version: u16,
 /// countdatainsidecontain
 pub fragment: Vec<u8>,
}

impl TLSRecord {
 /// Create a new TLS record
 pub fn new(content_type: TLSRecordType, version: u16, fragment: Vec<u8>) -> Self {
 Self {
 content_type,
 version,
 fragment,
 }
 }

 /// Createhandshakerecord
 pub fn handshake(version: u16, data: Vec<u8>) -> Self {
 Self::new(TLSRecordType::Handshake, version, data)
 }

 /// serialize as bytesstream
 pub fn to_bytes(&self) -> Vec<u8> {
 let mut bytes = Vec::new();

 // Content Type (1 byte)
 bytes.push(self.content_type.as_u8());

 // Version (2 bytes)
 bytes.extend_from_slice(&self.version.to_be_bytes());

 // Length (2 bytes)
 let length = self.fragment.len() as u16;
 bytes.extend_from_slice(&length.to_be_bytes());

 // Fragment
 bytes.extend_from_slice(&self.fragment);

 bytes
 }

 /// from bytesstreamParse
 pub fn from_bytes(data: &[u8]) -> Result<(Self, usize), String> {
 if data.len() < 5 {
 return Err("countdatatoo short，unable toParse TLS record".to_string());
 }

 let content_type = match data[0] {
 20 => TLSRecordType::ChangeCipherSpec,
 21 => TLSRecordType::Alert,
 22 => TLSRecordType::Handshake,
 23 => TLSRecordType::ApplicationData,
 _ => return Err(format!("not知inside容type: {}", data[0])),
 };

 let version = u16::from_be_bytes([data[1], data[2]]);
 let length = u16::from_be_bytes([data[3], data[4]]) as usize;

 if data.len() < 5 + length {
 return Err(format!(
 "countdata不complete，need {} bytes，actualonly {} bytes",
 5 + length,
 data.len()
 ));
 }

 let fragment = data[5..5 + length].to_vec();

 Ok((Self::new(content_type, version, fragment), 5 + length))
 }
}

#[cfg(test)]
mod tests {
 use super::*;

 #[test]
 fn test_tls_record_serialization() {
 let data = vec![1, 2, 3, 4, 5];
 let record = TLSRecord::handshake(0x0303, data.clone());

 let bytes = record.to_bytes();

 // Validateformat
 assert_eq!(bytes[0], 22); // Handshake
 assert_eq!(u16::from_be_bytes([bytes[1], bytes[2]]), 0x0303); // TLS 1.2
 assert_eq!(u16::from_be_bytes([bytes[3], bytes[4]]), 5); // Length
 assert_eq!(&bytes[5..], &data);
 }

 #[test]
 fn test_tls_record_deserialization() {
 let data = vec![22, 0x03, 0x03, 0x00, 0x05, 1, 2, 3, 4, 5];
 let (record, consumed) = TLSRecord::from_bytes(&data).unwrap();

 assert_eq!(record.content_type, TLSRecordType::Handshake);
 assert_eq!(record.version, 0x0303);
 assert_eq!(record.fragment, vec![1, 2, 3, 4, 5]);
 assert_eq!(consumed, 10);
 }
}
