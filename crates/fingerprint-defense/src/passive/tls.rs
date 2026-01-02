//! TLS 被动fingerprintidentify
//!
//! implement TLS ClientHello 的被动analysis and JA4 fingerprintGenerate。

use crate::passive::packet::Packet;

/// TLS analysis器
pub struct TlsAnalyzer;

use serde::{Deserialize, Serialize};

/// TLS fingerprint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsFingerprint {
    /// JA4 fingerprint
    pub ja4: Option<String>,

    /// JA4 raw fingerprint
    pub ja4_raw: Option<String>,

    /// TLS version
    pub version: Option<u16>,

    /// cipher suitecount
    pub cipher_suites_count: usize,

    /// extensioncount
    pub extensions_count: usize,

    /// fingerprintmetadata
    pub metadata: fingerprint_core::metadata::FingerprintMetadata,
}

impl fingerprint_core::fingerprint::Fingerprint for TlsFingerprint {
    fn fingerprint_type(&self) -> fingerprint_core::fingerprint::FingerprintType {
        fingerprint_core::fingerprint::FingerprintType::Tls
    }

    fn id(&self) -> String {
        self.ja4.clone().unwrap_or_else(|| "unknown".to_string())
    }

    fn metadata(&self) -> &fingerprint_core::metadata::FingerprintMetadata {
        &self.metadata
    }

    fn metadata_mut(&mut self) -> &mut fingerprint_core::metadata::FingerprintMetadata {
        &mut self.metadata
    }

    fn hash(&self) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        self.ja4.hash(&mut hasher);
        self.version.hash(&mut hasher);
        hasher.finish()
    }

    fn similar_to(&self, other: &dyn fingerprint_core::fingerprint::Fingerprint) -> bool {
        if other.fingerprint_type() != fingerprint_core::fingerprint::FingerprintType::Tls {
            return false;
        }
        self.id() == other.id()
    }

    fn to_string(&self) -> String {
        format!(
            "TLS Fingerprint (JA4: {:?}, Version: {:?})",
            self.ja4, self.version
        )
    }
}

impl TlsAnalyzer {
    /// Create a new TLS analysis器
    pub fn new() -> Result<Self, String> {
        Ok(Self)
    }

    /// analysis TLS countpacket
    pub fn analyze(&self, packet: &Packet) -> Option<TlsFingerprint> {
        // find TLS ClientHello
        if let Some(client_hello) = self.find_client_hello(&packet.payload) {
            return Some(self.analyze_client_hello(&client_hello));
        }

        None
    }

    /// find ClientHello
    fn find_client_hello(&self, data: &[u8]) -> Option<Vec<u8>> {
        // find TLS handshakemessage
        // TLS recordformat: [ContentType(1)][Version(2)][Length(2)][Data]
        // Handshake format: [Type(1)][Length(3)][Version(2)][Random(32)][SessionID][CipherSuites][Compression][Extensions]

        if data.len() < 5 {
            return None;
        }

        // find ContentType = 22 (Handshake)
        for i in 0..data.len().saturating_sub(5) {
            if data[i] == 0x16 {
                // Handshake
                if i + 1 < data.len() && data[i + 1] == 0x03 {
                    // TLS version starts with 0x03
                    let record_len = if i + 4 < data.len() {
                        u16::from_be_bytes([data[i + 3], data[i + 4]]) as usize
                    } else {
                        continue;
                    };

                    if i + 5 + record_len <= data.len() {
                        let handshake_data = &data[i + 5..i + 5 + record_len];

                        // Checkwhether是 ClientHello (Type = 1)
                        if !handshake_data.is_empty() && handshake_data[0] == 0x01 {
                            return Some(handshake_data.to_vec());
                        }
                    }
                }
            }
        }

        None
    }

    /// analysis ClientHello
    fn analyze_client_hello(&self, client_hello: &[u8]) -> TlsFingerprint {
        // Parse ClientHello
        // [Type(1)][Length(3)][Version(2)][Random(32)][SessionID][CipherSuites][Compression][Extensions]

        let mut offset = 0;

        // Type (should是 1 = ClientHello)
        if client_hello.is_empty() || client_hello[offset] != 0x01 {
            return TlsFingerprint::default();
        }
        offset += 1;

        // Length (3 bytes)
        if client_hello.len() < offset + 3 {
            return TlsFingerprint::default();
        }
        offset += 3;

        // Version (2 bytes)
        let version = if client_hello.len() >= offset + 2 {
            Some(u16::from_be_bytes([
                client_hello[offset],
                client_hello[offset + 1],
            ]))
        } else {
            None
        };
        offset += 2;

        // Random (32 bytes)
        if client_hello.len() < offset + 32 {
            return TlsFingerprint::default();
        }
        offset += 32;

        // SessionID
        if client_hello.len() <= offset {
            return TlsFingerprint::default();
        }
        let session_id_len = client_hello[offset] as usize;
        offset += 1;
        if client_hello.len() < offset + session_id_len {
            return TlsFingerprint::default();
        }
        offset += session_id_len;

        // CipherSuites
        let mut cipher_suites = Vec::new();
        if client_hello.len() >= offset + 2 {
            let cipher_suites_len =
                u16::from_be_bytes([client_hello[offset], client_hello[offset + 1]]) as usize;
            offset += 2;
            if client_hello.len() >= offset + cipher_suites_len {
                for i in (0..cipher_suites_len).step_by(2) {
                    cipher_suites.push(u16::from_be_bytes([
                        client_hello[offset + i],
                        client_hello[offset + i + 1],
                    ]));
                }
                offset += cipher_suites_len;
            }
        }

        // Compression
        if client_hello.len() <= offset {
            return TlsFingerprint {
                version,
                cipher_suites_count: cipher_suites.len(),
                extensions_count: 0,
                ja4: None,
                ja4_raw: None,
                metadata: fingerprint_core::metadata::FingerprintMetadata::new(),
            };
        }
        let compression_len = client_hello[offset] as usize;
        offset += 1;
        offset += compression_len;

        // Extensions
        let mut extensions = Vec::new();
        let mut has_sni = false;
        let mut first_alpn = None;
        let mut sig_algs = Vec::new();

        if client_hello.len() >= offset + 2 {
            let extensions_len =
                u16::from_be_bytes([client_hello[offset], client_hello[offset + 1]]) as usize;
            offset += 2;
            let ext_end = offset + extensions_len;

            while offset + 4 <= ext_end && offset + 4 <= client_hello.len() {
                let ext_type = u16::from_be_bytes([client_hello[offset], client_hello[offset + 1]]);
                let ext_len =
                    u16::from_be_bytes([client_hello[offset + 2], client_hello[offset + 3]])
                        as usize;
                offset += 4;

                extensions.push(ext_type);

                if offset + ext_len <= ext_end && offset + ext_len <= client_hello.len() {
                    match ext_type {
                        0 => has_sni = true, // Server Name
                        16 => {
                            // ALPN
                            if ext_len >= 2 {
                                let alpn_list_len = u16::from_be_bytes([
                                    client_hello[offset],
                                    client_hello[offset + 1],
                                ]) as usize;
                                if alpn_list_len >= 1 && offset + 2 + alpn_list_len <= ext_end {
                                    let first_protocol_len = client_hello[offset + 2] as usize;
                                    if 2 + 1 + first_protocol_len <= alpn_list_len + 2 {
                                        let protocol = &client_hello
                                            [offset + 3..offset + 3 + first_protocol_len];
                                        first_alpn =
                                            Some(String::from_utf8_lossy(protocol).to_string());
                                    }
                                }
                            }
                        }
                        13 => {
                            // Signature Algorithms
                            if ext_len >= 2 {
                                let sig_list_len = u16::from_be_bytes([
                                    client_hello[offset],
                                    client_hello[offset + 1],
                                ]) as usize;
                                for i in (0..sig_list_len.saturating_sub(1)).step_by(2) {
                                    if offset + 2 + i + 1 < client_hello.len() {
                                        sig_algs.push(u16::from_be_bytes([
                                            client_hello[offset + 2 + i],
                                            client_hello[offset + 2 + i + 1],
                                        ]));
                                    }
                                }
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

        // Generate JA4 fingerprint
        let tls_ver_str = match version {
            Some(0x0304) => "1.3",
            Some(0x0303) => "1.2",
            Some(0x0302) => "1.1",
            Some(0x0301) => "1.0",
            _ => "1.2", // default
        };

        let ja4 = fingerprint_core::ja4::JA4::generate(
            't', // 假设是 TCP，actual应 from  packet judge
            tls_ver_str,
            has_sni,
            &cipher_suites,
            &extensions,
            first_alpn.as_deref(),
            &sig_algs,
        );

        let ja4_string = ja4.to_fingerprint_string();
        let mut metadata = fingerprint_core::metadata::FingerprintMetadata::new();
        metadata.add_tag(format!("ja4:{}", ja4_string));

        TlsFingerprint {
            version,
            cipher_suites_count: cipher_suites.len(),
            extensions_count: extensions.len(),
            ja4: Some(ja4_string),
            ja4_raw: None,
            metadata,
        }
    }
}

impl Default for TlsFingerprint {
    fn default() -> Self {
        Self {
            ja4: None,
            ja4_raw: None,
            version: None,
            cipher_suites_count: 0,
            extensions_count: 0,
            metadata: fingerprint_core::metadata::FingerprintMetadata::new(),
        }
    }
}
