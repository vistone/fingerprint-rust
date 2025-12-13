//! TLS 扩展模块
//!
//! 实现各种 TLS 扩展，对应 Go 版本的 tls.TLSExtension
//!
//! 参考：https://github.com/refraction-networking/utls/blob/master/u_tls_extensions.go

use crate::dicttls::extensions::*;
use crate::dicttls::signature_schemes::SignatureScheme;
use crate::dicttls::supported_groups::CurveID;
use std::io;

/// TLS 扩展 ID
pub type ExtensionID = u16;

/// Key Share Entry
/// 对应 Go 版本的 tls.KeyShare
#[derive(Debug, Clone)]
pub struct KeyShare {
    pub group: CurveID,
    pub data: Vec<u8>,
}

/// TLS 扩展 trait
/// 对应 Go 版本的 tls.TLSExtension 接口
pub trait TLSExtension: std::fmt::Debug {
    /// 获取扩展的长度（包括头部）
    /// 对应 Go 版本的 Len() int
    fn len(&self) -> usize;

    /// 读取扩展数据到字节缓冲区
    /// 对应 Go 版本的 Read(p []byte) (n int, err error)
    fn read(&self, buf: &mut [u8]) -> io::Result<usize>;

    /// 获取扩展 ID
    fn extension_id(&self) -> ExtensionID;
}

/// TLS 扩展 Writer trait
/// 对应 Go 版本的 tls.TLSExtensionWriter 接口
pub trait TLSExtensionWriter: TLSExtension {
    /// 从字节缓冲区写入扩展数据
    /// 对应 Go 版本的 Write(b []byte) (n int, err error)
    fn write(&mut self, buf: &[u8]) -> io::Result<usize>;
}

/// GREASE 扩展
/// 对应 Go 版本的 &tls.UtlsGREASEExtension{}
#[derive(Debug, Clone)]
pub struct UtlsGREASEExtension {
    pub value: u16,
}

impl UtlsGREASEExtension {
    pub fn new() -> Self {
        Self {
            value: 0x0a0a, // GREASE placeholder
        }
    }
}

impl TLSExtension for UtlsGREASEExtension {
    fn len(&self) -> usize {
        4 // extension_id (2) + length (2)
    }

    fn read(&self, buf: &mut [u8]) -> io::Result<usize> {
        if buf.len() < self.len() {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "buffer too short"));
        }
        buf[0] = (self.value >> 8) as u8;
        buf[1] = (self.value & 0xff) as u8;
        buf[2] = 0; // length = 0
        buf[3] = 0;
        Ok(self.len())
    }

    fn extension_id(&self) -> ExtensionID {
        self.value
    }
}

impl Default for UtlsGREASEExtension {
    fn default() -> Self {
        Self::new()
    }
}

/// SNI 扩展
/// 对应 Go 版本的 &tls.SNIExtension{}
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
    fn len(&self) -> usize {
        if self.server_name.is_empty() {
            return 0;
        }
        // extension_id (2) + length (2) + server_name_list_length (2) + name_type (1) + host_name_length (2) + host_name
        4 + 2 + 1 + 2 + self.server_name.len()
    }

    fn read(&self, buf: &mut [u8]) -> io::Result<usize> {
        if self.server_name.is_empty() {
            return Ok(0);
        }
        let len = self.len();
        if buf.len() < len {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "buffer too short"));
        }

        // Extension ID
        buf[0] = (EXT_TYPE_SERVER_NAME >> 8) as u8;
        buf[1] = (EXT_TYPE_SERVER_NAME & 0xff) as u8;

        let host_name_len = self.server_name.len();
        let total_len = 5 + host_name_len;

        // Extension length
        buf[2] = (total_len >> 8) as u8;
        buf[3] = (total_len & 0xff) as u8;

        // Server name list length
        buf[4] = ((host_name_len + 3) >> 8) as u8;
        buf[5] = ((host_name_len + 3) & 0xff) as u8;

        // Name type: host_name (0)
        buf[6] = 0;

        // Host name length
        buf[7] = (host_name_len >> 8) as u8;
        buf[8] = (host_name_len & 0xff) as u8;

        // Host name
        buf[9..9 + host_name_len].copy_from_slice(self.server_name.as_bytes());

        Ok(len)
    }

    fn extension_id(&self) -> ExtensionID {
        EXT_TYPE_SERVER_NAME
    }
}

impl TLSExtensionWriter for SNIExtension {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        // SNI Write 是 no-op，因为 SNI 不应该被指纹化，是用户控制的
        Ok(buf.len())
    }
}

/// Status Request 扩展
/// 对应 Go 版本的 &tls.StatusRequestExtension{}
#[derive(Debug, Clone)]
pub struct StatusRequestExtension;

impl TLSExtension for StatusRequestExtension {
    fn len(&self) -> usize {
        9 // extension_id (2) + length (2) + status_type (1) + responder_id_list_length (2) + request_extensions_length (2)
    }

    fn read(&self, buf: &mut [u8]) -> io::Result<usize> {
        if buf.len() < self.len() {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "buffer too short"));
        }

        // Extension ID
        buf[0] = (EXT_TYPE_STATUS_REQUEST >> 8) as u8;
        buf[1] = (EXT_TYPE_STATUS_REQUEST & 0xff) as u8;

        // Extension length
        buf[2] = 0;
        buf[3] = 5;

        // Status type: OCSP (1)
        buf[4] = 1;

        // Responder ID list length (0)
        buf[5] = 0;
        buf[6] = 0;

        // Request extensions length (0)
        buf[7] = 0;
        buf[8] = 0;

        Ok(self.len())
    }

    fn extension_id(&self) -> ExtensionID {
        EXT_TYPE_STATUS_REQUEST
    }
}

impl TLSExtensionWriter for StatusRequestExtension {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        Ok(buf.len())
    }
}

/// Supported Curves 扩展
/// 对应 Go 版本的 &tls.SupportedCurvesExtension{}
#[derive(Debug, Clone)]
pub struct SupportedCurvesExtension {
    pub curves: Vec<CurveID>,
}

impl SupportedCurvesExtension {
    pub fn new(curves: Vec<CurveID>) -> Self {
        Self { curves }
    }
}

impl TLSExtension for SupportedCurvesExtension {
    fn len(&self) -> usize {
        6 + 2 * self.curves.len() // extension_id (2) + length (2) + curves_length (2) + curves
    }

    fn read(&self, buf: &mut [u8]) -> io::Result<usize> {
        let len = self.len();
        if buf.len() < len {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "buffer too short"));
        }

        // Extension ID
        buf[0] = (EXT_TYPE_SUPPORTED_GROUPS >> 8) as u8;
        buf[1] = (EXT_TYPE_SUPPORTED_GROUPS & 0xff) as u8;

        let curves_len = 2 * self.curves.len();
        let total_len = 2 + curves_len;

        // Extension length
        buf[2] = (total_len >> 8) as u8;
        buf[3] = (total_len & 0xff) as u8;

        // Curves length
        buf[4] = (curves_len >> 8) as u8;
        buf[5] = (curves_len & 0xff) as u8;

        // Curves
        for (i, curve) in self.curves.iter().enumerate() {
            buf[6 + 2 * i] = (*curve >> 8) as u8;
            buf[7 + 2 * i] = (*curve & 0xff) as u8;
        }

        Ok(len)
    }

    fn extension_id(&self) -> ExtensionID {
        EXT_TYPE_SUPPORTED_GROUPS
    }
}

impl TLSExtensionWriter for SupportedCurvesExtension {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        // 简化实现
        Ok(buf.len())
    }
}

/// Supported Points 扩展
/// 对应 Go 版本的 &tls.SupportedPointsExtension{}
#[derive(Debug, Clone)]
pub struct SupportedPointsExtension {
    pub supported_points: Vec<u8>,
}

impl SupportedPointsExtension {
    pub fn new(supported_points: Vec<u8>) -> Self {
        Self { supported_points }
    }
}

impl TLSExtension for SupportedPointsExtension {
    fn len(&self) -> usize {
        5 + self.supported_points.len() // extension_id (2) + length (2) + point_formats_length (1) + point_formats
    }

    fn read(&self, buf: &mut [u8]) -> io::Result<usize> {
        let len = self.len();
        if buf.len() < len {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "buffer too short"));
        }

        // Extension ID
        buf[0] = (EXT_TYPE_EC_POINT_FORMATS >> 8) as u8;
        buf[1] = (EXT_TYPE_EC_POINT_FORMATS & 0xff) as u8;

        let total_len = 1 + self.supported_points.len();

        // Extension length
        buf[2] = (total_len >> 8) as u8;
        buf[3] = (total_len & 0xff) as u8;

        // Point formats length
        buf[4] = self.supported_points.len() as u8;

        // Point formats
        buf[5..5 + self.supported_points.len()].copy_from_slice(&self.supported_points);

        Ok(len)
    }

    fn extension_id(&self) -> ExtensionID {
        EXT_TYPE_EC_POINT_FORMATS
    }
}

impl TLSExtensionWriter for SupportedPointsExtension {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        // 简化实现
        Ok(buf.len())
    }
}

/// Signature Algorithms 扩展
/// 对应 Go 版本的 &tls.SignatureAlgorithmsExtension{}
#[derive(Debug, Clone)]
pub struct SignatureAlgorithmsExtension {
    pub supported_signature_algorithms: Vec<SignatureScheme>,
}

impl SignatureAlgorithmsExtension {
    pub fn new(supported_signature_algorithms: Vec<SignatureScheme>) -> Self {
        Self {
            supported_signature_algorithms,
        }
    }
}

impl TLSExtension for SignatureAlgorithmsExtension {
    fn len(&self) -> usize {
        6 + 2 * self.supported_signature_algorithms.len() // extension_id (2) + length (2) + algorithms_length (2) + algorithms
    }

    fn read(&self, buf: &mut [u8]) -> io::Result<usize> {
        let len = self.len();
        if buf.len() < len {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "buffer too short"));
        }

        // Extension ID
        buf[0] = (EXT_TYPE_SIGNATURE_ALGORITHMS >> 8) as u8;
        buf[1] = (EXT_TYPE_SIGNATURE_ALGORITHMS & 0xff) as u8;

        let algorithms_len = 2 * self.supported_signature_algorithms.len();
        let total_len = 2 + algorithms_len;

        // Extension length
        buf[2] = (total_len >> 8) as u8;
        buf[3] = (total_len & 0xff) as u8;

        // Algorithms length
        buf[4] = (algorithms_len >> 8) as u8;
        buf[5] = (algorithms_len & 0xff) as u8;

        // Algorithms
        for (i, scheme) in self.supported_signature_algorithms.iter().enumerate() {
            buf[6 + 2 * i] = (*scheme >> 8) as u8;
            buf[7 + 2 * i] = (*scheme & 0xff) as u8;
        }

        Ok(len)
    }

    fn extension_id(&self) -> ExtensionID {
        EXT_TYPE_SIGNATURE_ALGORITHMS
    }
}

impl TLSExtensionWriter for SignatureAlgorithmsExtension {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        // 简化实现
        Ok(buf.len())
    }
}

/// ALPN 扩展
/// 对应 Go 版本的 &tls.ALPNExtension{}
#[derive(Debug, Clone)]
pub struct ALPNExtension {
    pub alpn_protocols: Vec<String>,
}

impl ALPNExtension {
    pub fn new(alpn_protocols: Vec<String>) -> Self {
        Self { alpn_protocols }
    }
}

impl TLSExtension for ALPNExtension {
    fn len(&self) -> usize {
        let mut total = 2 + 2 + 2; // extension_id (2) + length (2) + protocol_name_list_length (2)
        for protocol in &self.alpn_protocols {
            total += 1 + protocol.len(); // protocol_length (1) + protocol
        }
        total
    }

    fn read(&self, buf: &mut [u8]) -> io::Result<usize> {
        let len = self.len();
        if buf.len() < len {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "buffer too short"));
        }

        // Extension ID
        buf[0] = (EXT_TYPE_APPLICATION_LAYER_PROTOCOL_NEGOTIATION >> 8) as u8;
        buf[1] = (EXT_TYPE_APPLICATION_LAYER_PROTOCOL_NEGOTIATION & 0xff) as u8;

        let mut protocol_name_list_len = 0;
        for protocol in &self.alpn_protocols {
            protocol_name_list_len += 1 + protocol.len();
        }

        let total_len = 2 + protocol_name_list_len;

        // Extension length
        buf[2] = (total_len >> 8) as u8;
        buf[3] = (total_len & 0xff) as u8;

        // Protocol name list length
        buf[4] = (protocol_name_list_len >> 8) as u8;
        buf[5] = (protocol_name_list_len & 0xff) as u8;

        // Protocols
        let mut offset = 6;
        for protocol in &self.alpn_protocols {
            buf[offset] = protocol.len() as u8;
            offset += 1;
            buf[offset..offset + protocol.len()].copy_from_slice(protocol.as_bytes());
            offset += protocol.len();
        }

        Ok(len)
    }

    fn extension_id(&self) -> ExtensionID {
        EXT_TYPE_APPLICATION_LAYER_PROTOCOL_NEGOTIATION
    }
}

impl TLSExtensionWriter for ALPNExtension {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        // 简化实现
        Ok(buf.len())
    }
}

/// Extended Master Secret 扩展
/// 对应 Go 版本的 &tls.ExtendedMasterSecretExtension{}
#[derive(Debug, Clone)]
pub struct ExtendedMasterSecretExtension;

impl TLSExtension for ExtendedMasterSecretExtension {
    fn len(&self) -> usize {
        4 // extension_id (2) + length (2, value = 0)
    }

    fn read(&self, buf: &mut [u8]) -> io::Result<usize> {
        if buf.len() < self.len() {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "buffer too short"));
        }

        // Extension ID
        buf[0] = (EXT_TYPE_EXTENDED_MASTER_SECRET >> 8) as u8;
        buf[1] = (EXT_TYPE_EXTENDED_MASTER_SECRET & 0xff) as u8;

        // Extension length (0)
        buf[2] = 0;
        buf[3] = 0;

        Ok(self.len())
    }

    fn extension_id(&self) -> ExtensionID {
        EXT_TYPE_EXTENDED_MASTER_SECRET
    }
}

impl TLSExtensionWriter for ExtendedMasterSecretExtension {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        Ok(buf.len())
    }
}

/// Session Ticket 扩展
/// 对应 Go 版本的 &tls.SessionTicketExtension{}
#[derive(Debug, Clone)]
pub struct SessionTicketExtension;

impl TLSExtension for SessionTicketExtension {
    fn len(&self) -> usize {
        4 // extension_id (2) + length (2, value = 0)
    }

    fn read(&self, buf: &mut [u8]) -> io::Result<usize> {
        if buf.len() < self.len() {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "buffer too short"));
        }

        // Extension ID
        buf[0] = (EXT_TYPE_SESSION_TICKET >> 8) as u8;
        buf[1] = (EXT_TYPE_SESSION_TICKET & 0xff) as u8;

        // Extension length (0)
        buf[2] = 0;
        buf[3] = 0;

        Ok(self.len())
    }

    fn extension_id(&self) -> ExtensionID {
        EXT_TYPE_SESSION_TICKET
    }
}

impl TLSExtensionWriter for SessionTicketExtension {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        Ok(buf.len())
    }
}

/// Supported Versions 扩展
/// 对应 Go 版本的 &tls.SupportedVersionsExtension{}
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
    fn len(&self) -> usize {
        6 + 2 * self.versions.len() // extension_id (2) + length (2) + versions_length (1) + versions
    }

    fn read(&self, buf: &mut [u8]) -> io::Result<usize> {
        let len = self.len();
        if buf.len() < len {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "buffer too short"));
        }

        // Extension ID
        buf[0] = (EXT_TYPE_SUPPORTED_VERSIONS >> 8) as u8;
        buf[1] = (EXT_TYPE_SUPPORTED_VERSIONS & 0xff) as u8;

        let versions_len = 1 + 2 * self.versions.len();
        let total_len = versions_len;

        // Extension length
        buf[2] = (total_len >> 8) as u8;
        buf[3] = (total_len & 0xff) as u8;

        // Versions length
        buf[4] = (2 * self.versions.len()) as u8;

        // Versions
        for (i, version) in self.versions.iter().enumerate() {
            buf[5 + 2 * i] = (*version >> 8) as u8;
            buf[6 + 2 * i] = (*version & 0xff) as u8;
        }

        Ok(len)
    }

    fn extension_id(&self) -> ExtensionID {
        EXT_TYPE_SUPPORTED_VERSIONS
    }
}

impl TLSExtensionWriter for SupportedVersionsExtension {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        // 简化实现
        Ok(buf.len())
    }
}

/// PSK Key Exchange Modes 扩展
/// 对应 Go 版本的 &tls.PSKKeyExchangeModesExtension{}
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
    fn len(&self) -> usize {
        5 + self.modes.len() // extension_id (2) + length (2) + modes_length (1) + modes
    }

    fn read(&self, buf: &mut [u8]) -> io::Result<usize> {
        let len = self.len();
        if buf.len() < len {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "buffer too short"));
        }

        // Extension ID
        buf[0] = (EXT_TYPE_PSK_KEY_EXCHANGE_MODES >> 8) as u8;
        buf[1] = (EXT_TYPE_PSK_KEY_EXCHANGE_MODES & 0xff) as u8;

        let total_len = 1 + self.modes.len();

        // Extension length
        buf[2] = (total_len >> 8) as u8;
        buf[3] = (total_len & 0xff) as u8;

        // Modes length
        buf[4] = self.modes.len() as u8;

        // Modes
        buf[5..5 + self.modes.len()].copy_from_slice(&self.modes);

        Ok(len)
    }

    fn extension_id(&self) -> ExtensionID {
        EXT_TYPE_PSK_KEY_EXCHANGE_MODES
    }
}

impl TLSExtensionWriter for PSKKeyExchangeModesExtension {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        // 简化实现
        Ok(buf.len())
    }
}

/// Key Share 扩展
/// 对应 Go 版本的 &tls.KeyShareExtension{}
#[derive(Debug, Clone)]
pub struct KeyShareExtension {
    pub key_shares: Vec<KeyShare>,
}

impl KeyShareExtension {
    pub fn new(key_shares: Vec<KeyShare>) -> Self {
        Self { key_shares }
    }

    fn key_shares_len(&self) -> usize {
        let mut len = 0;
        for ks in &self.key_shares {
            len += 4 + ks.data.len(); // group (2) + length (2) + data
        }
        len
    }
}

impl TLSExtension for KeyShareExtension {
    fn len(&self) -> usize {
        4 + 2 + self.key_shares_len() // extension_id (2) + length (2) + client_shares_length (2) + key_shares
    }

    fn read(&self, buf: &mut [u8]) -> io::Result<usize> {
        let len = self.len();
        if buf.len() < len {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "buffer too short"));
        }

        // Extension ID
        buf[0] = (EXT_TYPE_KEY_SHARE >> 8) as u8;
        buf[1] = (EXT_TYPE_KEY_SHARE & 0xff) as u8;

        let key_shares_len = self.key_shares_len();
        let total_len = 2 + key_shares_len;

        // Extension length
        buf[2] = (total_len >> 8) as u8;
        buf[3] = (total_len & 0xff) as u8;

        // Client shares length
        buf[4] = (key_shares_len >> 8) as u8;
        buf[5] = (key_shares_len & 0xff) as u8;

        // Key shares
        let mut offset = 6;
        for ks in &self.key_shares {
            buf[offset] = (ks.group >> 8) as u8;
            buf[offset + 1] = (ks.group & 0xff) as u8;
            buf[offset + 2] = (ks.data.len() >> 8) as u8;
            buf[offset + 3] = (ks.data.len() & 0xff) as u8;
            buf[offset + 4..offset + 4 + ks.data.len()].copy_from_slice(&ks.data);
            offset += 4 + ks.data.len();
        }

        Ok(len)
    }

    fn extension_id(&self) -> ExtensionID {
        EXT_TYPE_KEY_SHARE
    }
}

impl TLSExtensionWriter for KeyShareExtension {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        // 简化实现
        Ok(buf.len())
    }
}

/// SCT 扩展
/// 对应 Go 版本的 &tls.SCTExtension{}
#[derive(Debug, Clone)]
pub struct SCTExtension;

impl TLSExtension for SCTExtension {
    fn len(&self) -> usize {
        4 // extension_id (2) + length (2, value = 0)
    }

    fn read(&self, buf: &mut [u8]) -> io::Result<usize> {
        if buf.len() < self.len() {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "buffer too short"));
        }

        // Extension ID
        buf[0] = (EXT_TYPE_SIGNED_CERTIFICATE_TIMESTAMP >> 8) as u8;
        buf[1] = (EXT_TYPE_SIGNED_CERTIFICATE_TIMESTAMP & 0xff) as u8;

        // Extension length (0)
        buf[2] = 0;
        buf[3] = 0;

        Ok(self.len())
    }

    fn extension_id(&self) -> ExtensionID {
        EXT_TYPE_SIGNED_CERTIFICATE_TIMESTAMP
    }
}

impl TLSExtensionWriter for SCTExtension {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        Ok(buf.len())
    }
}

/// Renegotiation Info 扩展
/// 对应 Go 版本的 &tls.RenegotiationInfoExtension{}
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
    fn len(&self) -> usize {
        5 // extension_id (2) + length (2) + renegotiated_connection_length (1)
    }

    fn read(&self, buf: &mut [u8]) -> io::Result<usize> {
        if buf.len() < self.len() {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "buffer too short"));
        }

        // Extension ID
        buf[0] = (EXT_TYPE_RENEGOTIATION_INFO >> 8) as u8;
        buf[1] = (EXT_TYPE_RENEGOTIATION_INFO & 0xff) as u8;

        // Extension length
        buf[2] = 0;
        buf[3] = 1;

        // Renegotiated connection length
        buf[4] = self.renegotiation;

        Ok(self.len())
    }

    fn extension_id(&self) -> ExtensionID {
        EXT_TYPE_RENEGOTIATION_INFO
    }
}

impl TLSExtensionWriter for RenegotiationInfoExtension {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        Ok(buf.len())
    }
}

/// Application Settings Extension New
/// 对应 Go 版本的 &tls.ApplicationSettingsExtensionNew{}
#[derive(Debug, Clone)]
pub struct ApplicationSettingsExtensionNew {
    pub supported_protocols: Vec<String>,
}

impl ApplicationSettingsExtensionNew {
    pub fn new(supported_protocols: Vec<String>) -> Self {
        Self {
            supported_protocols,
        }
    }
}

impl TLSExtension for ApplicationSettingsExtensionNew {
    fn len(&self) -> usize {
        let mut total = 2 + 2 + 2; // extension_id (2) + length (2) + protocol_list_length (2)
        for protocol in &self.supported_protocols {
            total += 1 + protocol.len(); // protocol_length (1) + protocol
        }
        total
    }

    fn read(&self, buf: &mut [u8]) -> io::Result<usize> {
        let len = self.len();
        if buf.len() < len {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "buffer too short"));
        }

        // Extension ID
        buf[0] = (EXT_TYPE_APPLICATION_SETTINGS_NEW >> 8) as u8;
        buf[1] = (EXT_TYPE_APPLICATION_SETTINGS_NEW & 0xff) as u8;

        let mut protocol_list_len = 0;
        for protocol in &self.supported_protocols {
            protocol_list_len += 1 + protocol.len();
        }

        let total_len = 2 + protocol_list_len;

        // Extension length
        buf[2] = (total_len >> 8) as u8;
        buf[3] = (total_len & 0xff) as u8;

        // Protocol list length
        buf[4] = (protocol_list_len >> 8) as u8;
        buf[5] = (protocol_list_len & 0xff) as u8;

        // Protocols
        let mut offset = 6;
        for protocol in &self.supported_protocols {
            buf[offset] = protocol.len() as u8;
            offset += 1;
            buf[offset..offset + protocol.len()].copy_from_slice(protocol.as_bytes());
            offset += protocol.len();
        }

        Ok(len)
    }

    fn extension_id(&self) -> ExtensionID {
        EXT_TYPE_APPLICATION_SETTINGS_NEW
    }
}

impl TLSExtensionWriter for ApplicationSettingsExtensionNew {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        Ok(buf.len())
    }
}

/// Compress Certificate 扩展
/// 对应 Go 版本的 &tls.UtlsCompressCertExtension{}
#[derive(Debug, Clone)]
pub struct UtlsCompressCertExtension {
    pub algorithms: Vec<u16>,
}

impl UtlsCompressCertExtension {
    pub fn new(algorithms: Vec<u16>) -> Self {
        Self { algorithms }
    }
}

impl TLSExtension for UtlsCompressCertExtension {
    fn len(&self) -> usize {
        6 + 2 * self.algorithms.len() // extension_id (2) + length (2) + algorithms_length (2) + algorithms
    }

    fn read(&self, buf: &mut [u8]) -> io::Result<usize> {
        let len = self.len();
        if buf.len() < len {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "buffer too short"));
        }

        // Extension ID
        buf[0] = (EXT_TYPE_COMPRESS_CERTIFICATE >> 8) as u8;
        buf[1] = (EXT_TYPE_COMPRESS_CERTIFICATE & 0xff) as u8;

        let algorithms_len = 2 * self.algorithms.len();
        let total_len = 2 + algorithms_len;

        // Extension length
        buf[2] = (total_len >> 8) as u8;
        buf[3] = (total_len & 0xff) as u8;

        // Algorithms length
        buf[4] = (algorithms_len >> 8) as u8;
        buf[5] = (algorithms_len & 0xff) as u8;

        // Algorithms
        for (i, alg) in self.algorithms.iter().enumerate() {
            buf[6 + 2 * i] = (*alg >> 8) as u8;
            buf[7 + 2 * i] = (*alg & 0xff) as u8;
        }

        Ok(len)
    }

    fn extension_id(&self) -> ExtensionID {
        EXT_TYPE_COMPRESS_CERTIFICATE
    }
}

impl TLSExtensionWriter for UtlsCompressCertExtension {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        Ok(buf.len())
    }
}

/// Pre-Shared Key 扩展
/// 对应 Go 版本的 &tls.UtlsPreSharedKeyExtension{}
#[derive(Debug, Clone)]
pub struct UtlsPreSharedKeyExtension;

impl TLSExtension for UtlsPreSharedKeyExtension {
    fn len(&self) -> usize {
        4 // extension_id (2) + length (2, will be set dynamically)
    }

    fn read(&self, buf: &mut [u8]) -> io::Result<usize> {
        if buf.len() < self.len() {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "buffer too short"));
        }

        // Extension ID
        buf[0] = (EXT_TYPE_PRE_SHARED_KEY >> 8) as u8;
        buf[1] = (EXT_TYPE_PRE_SHARED_KEY & 0xff) as u8;

        // Extension length (simplified, actual implementation would be more complex)
        buf[2] = 0;
        buf[3] = 0;

        Ok(self.len())
    }

    fn extension_id(&self) -> ExtensionID {
        EXT_TYPE_PRE_SHARED_KEY
    }
}

impl TLSExtensionWriter for UtlsPreSharedKeyExtension {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        Ok(buf.len())
    }
}

/// GREASE ECH 扩展
/// 对应 Go 版本的 tls.BoringGREASEECH()
#[derive(Debug, Clone)]
pub struct GREASEEncryptedClientHelloExtension {
    pub value: u16,
}

impl GREASEEncryptedClientHelloExtension {
    pub fn new() -> Self {
        Self {
            value: 0xfe0d, // ECH extension ID
        }
    }
}

impl TLSExtension for GREASEEncryptedClientHelloExtension {
    fn len(&self) -> usize {
        4 // extension_id (2) + length (2)
    }

    fn read(&self, buf: &mut [u8]) -> io::Result<usize> {
        if buf.len() < self.len() {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "buffer too short"));
        }

        // Extension ID
        buf[0] = (self.value >> 8) as u8;
        buf[1] = (self.value & 0xff) as u8;

        // Extension length (0)
        buf[2] = 0;
        buf[3] = 0;

        Ok(self.len())
    }

    fn extension_id(&self) -> ExtensionID {
        self.value
    }
}

impl TLSExtensionWriter for GREASEEncryptedClientHelloExtension {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        Ok(buf.len())
    }
}

impl Default for GREASEEncryptedClientHelloExtension {
    fn default() -> Self {
        Self::new()
    }
}

/// 从扩展 ID 创建扩展实例
/// 对应 Go 版本的 ExtensionFromID 函数
pub fn extension_from_id(id: ExtensionID) -> Option<Box<dyn TLSExtension>> {
    match id {
        EXT_TYPE_SERVER_NAME => Some(Box::new(SNIExtension::new(String::new()))),
        EXT_TYPE_STATUS_REQUEST => Some(Box::new(StatusRequestExtension)),
        EXT_TYPE_SUPPORTED_GROUPS => Some(Box::new(SupportedCurvesExtension::new(vec![]))),
        EXT_TYPE_EC_POINT_FORMATS => Some(Box::new(SupportedPointsExtension::new(vec![]))),
        EXT_TYPE_SIGNATURE_ALGORITHMS => Some(Box::new(SignatureAlgorithmsExtension::new(vec![]))),
        EXT_TYPE_APPLICATION_LAYER_PROTOCOL_NEGOTIATION => Some(Box::new(ALPNExtension::new(vec![]))),
        EXT_TYPE_EXTENDED_MASTER_SECRET => Some(Box::new(ExtendedMasterSecretExtension)),
        EXT_TYPE_SESSION_TICKET => Some(Box::new(SessionTicketExtension)),
        EXT_TYPE_SUPPORTED_VERSIONS => Some(Box::new(SupportedVersionsExtension::new(vec![]))),
        EXT_TYPE_PSK_KEY_EXCHANGE_MODES => Some(Box::new(PSKKeyExchangeModesExtension::new(vec![]))),
        EXT_TYPE_KEY_SHARE => Some(Box::new(KeyShareExtension::new(vec![]))),
        EXT_TYPE_SIGNED_CERTIFICATE_TIMESTAMP => Some(Box::new(SCTExtension)),
        EXT_TYPE_RENEGOTIATION_INFO => Some(Box::new(RenegotiationInfoExtension::new(1))),
        EXT_TYPE_APPLICATION_SETTINGS_NEW => Some(Box::new(ApplicationSettingsExtensionNew::new(vec![]))),
        EXT_TYPE_COMPRESS_CERTIFICATE => Some(Box::new(UtlsCompressCertExtension::new(vec![]))),
        EXT_TYPE_PRE_SHARED_KEY => Some(Box::new(UtlsPreSharedKeyExtension)),
        EXT_TYPE_ECH => Some(Box::new(GREASEEncryptedClientHelloExtension::new())),
        _ => {
            // 检查是否是 GREASE
            if is_grease_uint16(id) {
                Some(Box::new(UtlsGREASEExtension::new()))
            } else {
                None
            }
        }
    }
}

/// 检查是否是 GREASE 值
fn is_grease_uint16(v: u16) -> bool {
    // GREASE 值的模式：0x1a1a, 0x2a2a, 0x3a3a, ..., 0xfafa
    let low = v & 0xff;
    let high = (v >> 8) & 0xff;
    low == high && (low & 0x0f) == 0x0a
}
