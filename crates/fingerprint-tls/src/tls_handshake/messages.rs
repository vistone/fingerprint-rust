//! TLS ClientHello messageBuild
//!
//! Based on ClientHelloSpec Generatereal TLS ClientHello message
//!
//! ClientHello format (RFC 5246):
//! ```text
//! struct {
//! ProtocolVersion client_version;
//! Random random;
//! SessionID session_id;
//! CipherSuite cipher_suites<2..2^16-2>;
//! CompressionMethod compression_methods<1..2^8-1>;
//! Extension extensions<0..2^16-1>;
//! } ClientHello;
//! ```

use crate::tls_config::ClientHelloSpec;
use crate::tls_extensions::TLSExtension;

/// ClientHello message
#[derive(Debug, Clone)]
pub struct ClientHelloMessage {
 /// clientversion
 pub client_version: u16,
 /// randomcount (32 bytes)
 pub random: Vec<u8>,
 /// session ID
 pub session_id: Vec<u8>,
 /// cipher suitelist
 pub cipher_suites: Vec<u16>,
 /// compressionmethod
 pub compression_methods: Vec<u8>,
 /// extensionlist
 pub extensions: Vec<u8>,
}

impl ClientHelloMessage {
 /// from ClientHelloSpec Create ClientHello message
 ///
 /// # Errors
 ///
 /// Ifunable toGetencryptionsecurityrandomcount（ in no `crypto` feature when ）, willreturnerror。
 /// suggest in productionenvironment in enabled `crypto` feature 以ensuresecurity性。
 pub fn from_spec(spec: &ClientHelloSpec, server_name: &str) -> Result<Self, String> {
 // use TLS 1.2 asclientversion（in order tocompatible性）
 let client_version = spec.tls_vers_max.max(0x0303);

 // Generaterandomcount (32 bytes)
 let mut random = Vec::with_capacity(32);

 // front 4 bytes: Unix when between戳
 // usecurrent when between， if Getfailure则use 0（虽然不太mayfailure）
 // fix 2038 year overflowissue：explicittruncatehighbit，ensure u32 rangeinside
 let timestamp = std::time::SystemTime::now()
.duration_since(std::time::UNIX_EPOCH)
.map(|d| (d.as_secs() & 0xFFFFFFFF) as u32) // explicittruncatehighbit，prevent 2038 year overflow
.unwrap_or(0);
 random.extend_from_slice(&timestamp.to_be_bytes());

 // back 28 bytes: randomcount
 #[cfg(feature = "crypto")]
 {
 use rand::Rng;
 let mut rng = rand::thread_rng();
 for _ in 0..28 {
 random.push(rng.gen());
 }
 }
 #[cfg(not(feature = "crypto"))]
 {
 // Ifno crypto feature, try from systemrandomcountsourceGetencryptionsecurityrandomcount
 // Ifunable toGet, directlyreturnerror，不allowuse不security降levelsolution
 use std::io::Read;
 let mut random_bytes = [0u8; 28];

 // try from /dev/urandom (Unix) Getrandomcount
 let mut rng = std::fs::File::open("/dev/urandom")
.map_err(|e| format!(
 "unable toaccesssystemrandomcountsource /dev/urandom: {}. suggestenabled 'crypto' feature 以useencryptionsecurityrandomcountGenerator",
 e
 ))?;

 rng.read_exact(&mut random_bytes)
.map_err(|e| format!(
 "unable to from /dev/urandom readrandomcount: {}. suggestenabled 'crypto' feature 以useencryptionsecurityrandomcountGenerator",
 e
 ))?;

 random.extend_from_slice(&random_bytes);
 }

 // emptysession ID（新session）
 let session_id = Vec::new();

 // cipher suite
 let cipher_suites = spec.cipher_suites.clone();

 // compressionmethod
 let compression_methods = if spec.compression_methods.is_empty() {
 vec![0] // 无compression
 } else {
 spec.compression_methods.clone()
 };

 // serializeextension
 let extensions = Self::serialize_extensions(&spec.extensions, server_name);

 Ok(Self {
 client_version,
 random,
 session_id,
 cipher_suites,
 compression_methods,
 extensions,
 })
 }

 /// serializeextension
 fn serialize_extensions(extensions: &[Box<dyn TLSExtension>], server_name: &str) -> Vec<u8> {
 let mut ext_bytes = Vec::new();
 let mut has_sni = false;

 for ext in extensions {
 let ext_id = ext.extension_id();

 // If is SNI extension（ID == 0）, weneedspecialprocess
 if ext_id == 0 {
 // skipduplicate SNI extension
 if has_sni {
 continue;
 }
 has_sni = true;

 // dynamicBuild SNI extensioncountdata
 let sni_data = Self::build_sni_extension(server_name);

 // extensionformat: ID (2 bytes) + Length (2 bytes) + Data
 ext_bytes.extend_from_slice(&ext_id.to_be_bytes());
 ext_bytes.extend_from_slice(&(sni_data.len() as u16).to_be_bytes());
 ext_bytes.extend_from_slice(&sni_data);
 continue;
 }

 // otherextension：normalserialize
 let ext_len = ext.len();
 if ext_len == 0 {
 // emptyextensionalsoneedwrite ID and length
 ext_bytes.extend_from_slice(&ext_id.to_be_bytes());
 ext_bytes.extend_from_slice(&0u16.to_be_bytes());
 continue;
 }

 // readextensioncountdata (including ID and length)
 let mut ext_data = vec![0u8; ext_len];
 if ext.read(&mut ext_data).is_ok() {
 ext_bytes.extend_from_slice(&ext_data);
 }
 }

 // Ifno SNI extension, Addan
 if !has_sni && !server_name.is_empty() {
 let sni_data = Self::build_sni_extension(server_name);
 ext_bytes.extend_from_slice(&0u16.to_be_bytes()); // SNI extension ID
 ext_bytes.extend_from_slice(&(sni_data.len() as u16).to_be_bytes());
 ext_bytes.extend_from_slice(&sni_data);
 }

 ext_bytes
 }

 /// Build SNI extensioncountdata（excludingextension ID and lengthfield）
 fn build_sni_extension(server_name: &str) -> Vec<u8> {
 let mut data = Vec::new();

 // Server Name List Length (2 bytes)
 let list_len = 3 + server_name.len();
 data.extend_from_slice(&(list_len as u16).to_be_bytes());

 // Server Name Type (1 byte): 0 = host_name
 data.push(0);

 // Server Name Length (2 bytes)
 data.extend_from_slice(&(server_name.len() as u16).to_be_bytes());

 // Server Name
 data.extend_from_slice(server_name.as_bytes());

 data
 }

 /// serialize as bytesstream
 pub fn to_bytes(&self) -> Vec<u8> {
 let mut bytes = Vec::new();

 // Client Version (2 bytes)
 bytes.extend_from_slice(&self.client_version.to_be_bytes());

 // Random (32 bytes)
 bytes.extend_from_slice(&self.random);

 // Session ID Length (1 byte) + Session ID
 bytes.push(self.session_id.len() as u8);
 bytes.extend_from_slice(&self.session_id);

 // Cipher Suites Length (2 bytes) + Cipher Suites
 let cs_len = (self.cipher_suites.len() * 2) as u16;
 bytes.extend_from_slice(&cs_len.to_be_bytes());
 for cs in &self.cipher_suites {
 bytes.extend_from_slice(&cs.to_be_bytes());
 }

 // Compression Methods Length (1 byte) + Compression Methods
 bytes.push(self.compression_methods.len() as u8);
 bytes.extend_from_slice(&self.compression_methods);

 // Extensions Length (2 bytes) + Extensions
 bytes.extend_from_slice(&(self.extensions.len() as u16).to_be_bytes());
 bytes.extend_from_slice(&self.extensions);

 bytes
 }

 /// printdebuginfo
 pub fn debug_info(&self) -> String {
 format!(
 "ClientHello:\n\
 - Version: 0x{:04x}\n\
 - Random: {} bytes\n\
 - Session ID: {} bytes\n\
 - Cipher Suites: {} suites\n\
 - Compression: {} methods\n\
 - Extensions: {} bytes",
 self.client_version,
 self.random.len(),
 self.session_id.len(),
 self.cipher_suites.len(),
 self.compression_methods.len(),
 self.extensions.len()
 )
 }
}

#[cfg(test)]
mod tests {
 use super::*;

 #[test]
 fn test_clienthello_basic() {
 // Createansimple ClientHelloSpec
 let spec = ClientHelloSpec {
 cipher_suites: vec![0xc02f, 0xc030], // 两cipher suite
 compression_methods: vec![0],
 extensions: vec![],
 tls_vers_min: 0x0303,
 tls_vers_max: 0x0303,
 metadata: None,
 };

 let msg = ClientHelloMessage::from_spec(&spec, "example.com").unwrap();

 // Validatebasicfield
 assert_eq!(msg.client_version, 0x0303);
 assert_eq!(msg.random.len(), 32);
 assert_eq!(msg.cipher_suites.len(), 2);
 assert_eq!(msg.compression_methods, vec![0]);

 // serialize
 let bytes = msg.to_bytes();
 println!("ClientHello size: {} bytes", bytes.len());
 println!("{}", msg.debug_info());

 // Validateformat
 assert!(bytes.len() >= 41); // minimumlength
 }

 #[test]
 fn test_sni_extension() {
 let data = ClientHelloMessage::build_sni_extension("example.com");

 // SNI formatValidate
 assert!(data.len() > 5);

 // Server Name List Length
 let list_len = u16::from_be_bytes([data[0], data[1]]) as usize;
 assert_eq!(list_len, data.len() - 2);

 // Server Name Type
 assert_eq!(data[2], 0); // host_name

 // Server Name Length
 let name_len = u16::from_be_bytes([data[3], data[4]]) as usize;
 assert_eq!(name_len, 11); // "example.com".len()
 }
}
