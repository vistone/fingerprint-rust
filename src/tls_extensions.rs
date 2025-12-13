//! TLS 扩展模块
//!
//! 实现各种 TLS 扩展，对应 Go 版本的 tls.TLSExtension

use crate::tls_config::KeyShareEntry;

/// TLS 扩展 ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExtensionID {
    ServerName = 0,
    StatusRequest = 5,
    SupportedCurves = 10,
    SupportedPoints = 11,
    SignatureAlgorithms = 13,
    ALPN = 16,
    ExtendedMasterSecret = 23,
    SessionTicket = 35,
    SupportedVersions = 43,
    PSKKeyExchangeModes = 45,
    KeyShare = 51,
    RenegotiationInfo = 65281,
    SCT = 18,
    ApplicationSettings = 17513,
    ApplicationSettingsNew = 17613,
    CompressCertificate = 27,
    GREASE = 0x0a0a, // GREASE placeholder
    ECH = 0xfe0d,
}

/// TLS 扩展 trait
pub trait TLSExtension: std::fmt::Debug {
    fn extension_id(&self) -> ExtensionID;
    fn marshal(&self) -> Vec<u8>;
}

/// GREASE 扩展
#[derive(Debug, Clone)]
pub struct GREASEExtension {
    pub value: u16,
}

impl GREASEExtension {
    pub fn new() -> Self {
        Self { value: 0x0a0a }
    }
}

impl TLSExtension for GREASEExtension {
    fn extension_id(&self) -> ExtensionID {
        ExtensionID::GREASE
    }

    fn marshal(&self) -> Vec<u8> {
        vec![(self.value >> 8) as u8, (self.value & 0xff) as u8]
    }
}

/// SNI (Server Name Indication) 扩展
#[derive(Debug, Clone)]
pub struct SNIExtension {
    pub server_name: String,
}

impl SNIExtension {
    pub fn new(server_name: String) -> Self {
        Self { server_name }
    }
}

impl TLSExtension for SNIExtension {
    fn extension_id(&self) -> ExtensionID {
        ExtensionID::ServerName
    }

    fn marshal(&self) -> Vec<u8> {
        // 简化的 SNI 编码
        let name_bytes = self.server_name.as_bytes();
        let mut data = Vec::new();
        data.push(0); // NameType: host_name
        data.push((name_bytes.len() >> 8) as u8);
        data.push((name_bytes.len() & 0xff) as u8);
        data.extend_from_slice(name_bytes);
        data
    }
}

/// ALPN (Application-Layer Protocol Negotiation) 扩展
#[derive(Debug, Clone)]
pub struct ALPNExtension {
    pub protocols: Vec<String>,
}

impl ALPNExtension {
    pub fn new(protocols: Vec<String>) -> Self {
        Self { protocols }
    }
}

impl TLSExtension for ALPNExtension {
    fn extension_id(&self) -> ExtensionID {
        ExtensionID::ALPN
    }

    fn marshal(&self) -> Vec<u8> {
        let mut data = Vec::new();
        let mut total_len = 0;
        for protocol in &self.protocols {
            total_len += protocol.len() + 1;
        }
        data.push(total_len as u8);
        for protocol in &self.protocols {
            data.push(protocol.len() as u8);
            data.extend_from_slice(protocol.as_bytes());
        }
        data
    }
}

/// Signature Algorithms 扩展
#[derive(Debug, Clone)]
pub struct SignatureAlgorithmsExtension {
    pub algorithms: Vec<u16>,
}

impl SignatureAlgorithmsExtension {
    pub fn new(algorithms: Vec<u16>) -> Self {
        Self { algorithms }
    }
}

impl TLSExtension for SignatureAlgorithmsExtension {
    fn extension_id(&self) -> ExtensionID {
        ExtensionID::SignatureAlgorithms
    }

    fn marshal(&self) -> Vec<u8> {
        let mut data = Vec::new();
        data.push((self.algorithms.len() * 2 >> 8) as u8);
        data.push((self.algorithms.len() * 2 & 0xff) as u8);
        for alg in &self.algorithms {
            data.push((alg >> 8) as u8);
            data.push((alg & 0xff) as u8);
        }
        data
    }
}

/// Supported Versions 扩展
#[derive(Debug, Clone)]
pub struct SupportedVersionsExtension {
    pub versions: Vec<u16>,
}

impl SupportedVersionsExtension {
    pub fn new(versions: Vec<u16>) -> Self {
        Self { versions }
    }
}

impl TLSExtension for SupportedVersionsExtension {
    fn extension_id(&self) -> ExtensionID {
        ExtensionID::SupportedVersions
    }

    fn marshal(&self) -> Vec<u8> {
        let mut data = Vec::new();
        data.push((self.versions.len() * 2) as u8);
        for version in &self.versions {
            data.push((version >> 8) as u8);
            data.push((version & 0xff) as u8);
        }
        data
    }
}

/// Supported Curves 扩展
#[derive(Debug, Clone)]
pub struct SupportedCurvesExtension {
    pub curves: Vec<u16>,
}

impl SupportedCurvesExtension {
    pub fn new(curves: Vec<u16>) -> Self {
        Self { curves }
    }
}

impl TLSExtension for SupportedCurvesExtension {
    fn extension_id(&self) -> ExtensionID {
        ExtensionID::SupportedCurves
    }

    fn marshal(&self) -> Vec<u8> {
        let mut data = Vec::new();
        data.push((self.curves.len() * 2 >> 8) as u8);
        data.push((self.curves.len() * 2 & 0xff) as u8);
        for curve in &self.curves {
            data.push((curve >> 8) as u8);
            data.push((curve & 0xff) as u8);
        }
        data
    }
}

/// Supported Points 扩展
#[derive(Debug, Clone)]
pub struct SupportedPointsExtension {
    pub point_formats: Vec<u8>,
}

impl SupportedPointsExtension {
    pub fn new(point_formats: Vec<u8>) -> Self {
        Self { point_formats }
    }
}

impl TLSExtension for SupportedPointsExtension {
    fn extension_id(&self) -> ExtensionID {
        ExtensionID::SupportedPoints
    }

    fn marshal(&self) -> Vec<u8> {
        let mut data = Vec::new();
        data.push(self.point_formats.len() as u8);
        data.extend_from_slice(&self.point_formats);
        data
    }
}

/// Key Share 扩展
#[derive(Debug, Clone)]
pub struct KeyShareExtension {
    pub key_shares: Vec<KeyShareEntry>,
}

impl KeyShareExtension {
    pub fn new(key_shares: Vec<KeyShareEntry>) -> Self {
        Self { key_shares }
    }
}

impl TLSExtension for KeyShareExtension {
    fn extension_id(&self) -> ExtensionID {
        ExtensionID::KeyShare
    }

    fn marshal(&self) -> Vec<u8> {
        let mut data = Vec::new();
        // 计算总长度
        let mut total_len = 0;
        for share in &self.key_shares {
            total_len += 2 + 2 + share.data.len(); // group (2) + length (2) + data
        }
        data.push((total_len >> 8) as u8);
        data.push((total_len & 0xff) as u8);
        for share in &self.key_shares {
            data.push((share.group >> 8) as u8);
            data.push((share.group & 0xff) as u8);
            data.push((share.data.len() >> 8) as u8);
            data.push((share.data.len() & 0xff) as u8);
            data.extend_from_slice(&share.data);
        }
        data
    }
}

/// PSK Key Exchange Modes 扩展
#[derive(Debug, Clone)]
pub struct PSKKeyExchangeModesExtension {
    pub modes: Vec<u8>,
}

impl PSKKeyExchangeModesExtension {
    pub fn new(modes: Vec<u8>) -> Self {
        Self { modes }
    }
}

impl TLSExtension for PSKKeyExchangeModesExtension {
    fn extension_id(&self) -> ExtensionID {
        ExtensionID::PSKKeyExchangeModes
    }

    fn marshal(&self) -> Vec<u8> {
        let mut data = Vec::new();
        data.push(self.modes.len() as u8);
        data.extend_from_slice(&self.modes);
        data
    }
}

/// Session Ticket 扩展
#[derive(Debug, Clone)]
pub struct SessionTicketExtension;

impl TLSExtension for SessionTicketExtension {
    fn extension_id(&self) -> ExtensionID {
        ExtensionID::SessionTicket
    }

    fn marshal(&self) -> Vec<u8> {
        vec![] // 空数据
    }
}

/// Extended Master Secret 扩展
#[derive(Debug, Clone)]
pub struct ExtendedMasterSecretExtension;

impl TLSExtension for ExtendedMasterSecretExtension {
    fn extension_id(&self) -> ExtensionID {
        ExtensionID::ExtendedMasterSecret
    }

    fn marshal(&self) -> Vec<u8> {
        vec![] // 空数据
    }
}

/// Renegotiation Info 扩展
#[derive(Debug, Clone)]
pub struct RenegotiationInfoExtension {
    pub renegotiation: u8,
}

impl RenegotiationInfoExtension {
    pub fn new(renegotiation: u8) -> Self {
        Self { renegotiation }
    }
}

impl TLSExtension for RenegotiationInfoExtension {
    fn extension_id(&self) -> ExtensionID {
        ExtensionID::RenegotiationInfo
    }

    fn marshal(&self) -> Vec<u8> {
        vec![1, self.renegotiation]
    }
}

/// SCT (Signed Certificate Timestamp) 扩展
#[derive(Debug, Clone)]
pub struct SCTExtension;

impl TLSExtension for SCTExtension {
    fn extension_id(&self) -> ExtensionID {
        ExtensionID::SCT
    }

    fn marshal(&self) -> Vec<u8> {
        vec![] // 空数据
    }
}

/// Status Request 扩展
#[derive(Debug, Clone)]
pub struct StatusRequestExtension;

impl TLSExtension for StatusRequestExtension {
    fn extension_id(&self) -> ExtensionID {
        ExtensionID::StatusRequest
    }

    fn marshal(&self) -> Vec<u8> {
        vec![1, 1] // CertificateStatusType: OCSP
    }
}

/// Application Settings 扩展（新版本）
#[derive(Debug, Clone)]
pub struct ApplicationSettingsExtensionNew {
    pub protocols: Vec<String>,
}

impl ApplicationSettingsExtensionNew {
    pub fn new(protocols: Vec<String>) -> Self {
        Self { protocols }
    }
}

impl TLSExtension for ApplicationSettingsExtensionNew {
    fn extension_id(&self) -> ExtensionID {
        ExtensionID::ApplicationSettingsNew
    }

    fn marshal(&self) -> Vec<u8> {
        let mut data = Vec::new();
        let mut total_len = 0;
        for protocol in &self.protocols {
            total_len += protocol.len() + 1;
        }
        data.push((total_len >> 8) as u8);
        data.push((total_len & 0xff) as u8);
        for protocol in &self.protocols {
            data.push(protocol.len() as u8);
            data.extend_from_slice(protocol.as_bytes());
        }
        data
    }
}

/// Compress Certificate 扩展
#[derive(Debug, Clone)]
pub struct CompressCertExtension {
    pub algorithms: Vec<u16>,
}

impl CompressCertExtension {
    pub fn new(algorithms: Vec<u16>) -> Self {
        Self { algorithms }
    }
}

impl TLSExtension for CompressCertExtension {
    fn extension_id(&self) -> ExtensionID {
        ExtensionID::CompressCertificate
    }

    fn marshal(&self) -> Vec<u8> {
        let mut data = Vec::new();
        data.push((self.algorithms.len() * 2) as u8);
        for alg in &self.algorithms {
            data.push((alg >> 8) as u8);
            data.push((alg & 0xff) as u8);
        }
        data
    }
}

/// Pre-Shared Key 扩展
#[derive(Debug, Clone)]
pub struct PreSharedKeyExtension;

impl TLSExtension for PreSharedKeyExtension {
    fn extension_id(&self) -> ExtensionID {
        ExtensionID::PSKKeyExchangeModes // 注意：PSK 扩展使用不同的 ID
    }

    fn marshal(&self) -> Vec<u8> {
        vec![] // 简化实现
    }
}
