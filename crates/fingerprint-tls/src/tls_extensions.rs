//! TLS extensionmodule
//!
//! implementeach种 TLS extension, Corresponds to Go version's tls.TLSExtension
//!
//! reference：https://github.com/refraction-networking/utls/blob/master/u_tls_extensions.go

use fingerprint_core::dicttls::extensions::*;
use fingerprint_core::dicttls::signature_schemes::SignatureScheme;
use fingerprint_core::dicttls::supported_groups::CurveID;
use std::any::Any;
use std::io;

/// TLS extension ID
pub type ExtensionID = u16;

/// Padding length calculation function type
pub type PaddingLengthFn = Box<dyn Fn(usize) -> (usize, bool)>;

/// Key Share Entry
/// Corresponds to Go version's tls.KeyShare
#[derive(Debug, Clone)]
pub struct KeyShare {
    pub group: CurveID,
    pub data: Vec<u8>,
}

/// TLS extension trait
/// Corresponds to Go version's tls.TLSExtension interface
pub trait TLSExtension: std::fmt::Debug + Any {
    /// Getextensionlength (includeheader)
    /// Corresponds to Go version's Len() int
    fn len(&self) -> usize;

    /// Checkextensionwhether as empty
    /// defaultimplement：length as 0 when as empty
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// readextensioncountdata to bytesbuffer
    /// Corresponds to Go version's Read(p []byte) (n int, err error)
    fn read(&self, buf: &mut [u8]) -> io::Result<usize>;

    /// Getextension ID
    fn extension_id(&self) -> ExtensionID;

    /// convert to Any trait object,  for towarddowntransform
    fn as_any(&self) -> &dyn Any;
}

/// TLS extension Writer trait
/// Corresponds to Go version's tls.TLSExtensionWriter interface
pub trait TLSExtensionWriter: TLSExtension {
    /// from bytesbufferwriteextensioncountdata
    /// Corresponds to Go version's Write(b []byte) (n int, err error)
    fn write(&mut self, buf: &[u8]) -> io::Result<usize>;
}

/// GREASE extension
/// Corresponds to Go version's &tls.UtlsGREASEExtension{}
#[derive(Debug, Clone)]
pub struct UtlsGREASEExtension {
    pub value: u16,
}

impl UtlsGREASEExtension {
    pub fn new() -> Self {
        Self {
            value: fingerprint_core::grease::get_random_grease(),
        }
    }
}

impl TLSExtension for UtlsGREASEExtension {
    fn len(&self) -> usize {
        4 // extension_id (2) + length (2)
    }

    fn read(&self, buf: &mut [u8]) -> io::Result<usize> {
        if buf.len() < self.len() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "buffer too short",
            ));
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

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Default for UtlsGREASEExtension {
    fn default() -> Self {
        Self::new()
    }
}

/// SNI extension
/// Corresponds to Go version's &tls.SNIExtension{}
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
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "buffer too short",
            ));
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

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl TLSExtensionWriter for SNIExtension {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        // SNI Write is no-op, because SNI 不should被fingerprint化,  is usercontrol的
        Ok(buf.len())
    }
}

/// Status Request extension
/// Corresponds to Go version's &tls.StatusRequestExtension{}
#[derive(Debug, Clone)]
pub struct StatusRequestExtension;

impl TLSExtension for StatusRequestExtension {
    fn len(&self) -> usize {
        9 // extension_id (2) + length (2) + status_type (1) + responder_id_list_length (2) + request_extensions_length (2)
    }

    fn read(&self, buf: &mut [u8]) -> io::Result<usize> {
        if buf.len() < self.len() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "buffer too short",
            ));
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

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl TLSExtensionWriter for StatusRequestExtension {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        Ok(buf.len())
    }
}

/// Supported Curves extension
/// Corresponds to Go version's &tls.SupportedCurvesExtension{}
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
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "buffer too short",
            ));
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
            buf[6 + 2 * i] = (curve >> 8) as u8;
            buf[7 + 2 * i] = (curve & 0xff) as u8;
        }

        Ok(len)
    }

    fn extension_id(&self) -> ExtensionID {
        EXT_TYPE_SUPPORTED_GROUPS
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl TLSExtensionWriter for SupportedCurvesExtension {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        // Simplified implementation
        Ok(buf.len())
    }
}

/// Supported Points extension
/// Corresponds to Go version's &tls.SupportedPointsExtension{}
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
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "buffer too short",
            ));
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

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl TLSExtensionWriter for SupportedPointsExtension {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        // Simplified implementation
        Ok(buf.len())
    }
}

/// Signature Algorithms extension
/// Corresponds to Go version's &tls.SignatureAlgorithmsExtension{}
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
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "buffer too short",
            ));
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
            buf[6 + 2 * i] = (scheme >> 8) as u8;
            buf[7 + 2 * i] = (scheme & 0xff) as u8;
        }

        Ok(len)
    }

    fn extension_id(&self) -> ExtensionID {
        EXT_TYPE_SIGNATURE_ALGORITHMS
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl TLSExtensionWriter for SignatureAlgorithmsExtension {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        // Simplified implementation
        Ok(buf.len())
    }
}

/// ALPN extension
/// Corresponds to Go version's &tls.ALPNExtension{}
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
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "buffer too short",
            ));
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

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl TLSExtensionWriter for ALPNExtension {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        // Simplified implementation
        Ok(buf.len())
    }
}

/// Extended Master Secret extension
/// Corresponds to Go version's &tls.ExtendedMasterSecretExtension{}
#[derive(Debug, Clone)]
pub struct ExtendedMasterSecretExtension;

impl TLSExtension for ExtendedMasterSecretExtension {
    fn len(&self) -> usize {
        4 // extension_id (2) + length (2, value = 0)
    }

    fn read(&self, buf: &mut [u8]) -> io::Result<usize> {
        if buf.len() < self.len() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "buffer too short",
            ));
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

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl TLSExtensionWriter for ExtendedMasterSecretExtension {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        Ok(buf.len())
    }
}

/// Session Ticket extension
/// Corresponds to Go version's &tls.SessionTicketExtension{}
#[derive(Debug, Clone)]
pub struct SessionTicketExtension;

impl TLSExtension for SessionTicketExtension {
    fn len(&self) -> usize {
        4 // extension_id (2) + length (2, value = 0)
    }

    fn read(&self, buf: &mut [u8]) -> io::Result<usize> {
        if buf.len() < self.len() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "buffer too short",
            ));
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

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl TLSExtensionWriter for SessionTicketExtension {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        Ok(buf.len())
    }
}

/// Supported Versions extension
/// Corresponds to Go version's &tls.SupportedVersionsExtension{}
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
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "buffer too short",
            ));
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
            buf[5 + 2 * i] = (version >> 8) as u8;
            buf[6 + 2 * i] = (version & 0xff) as u8;
        }

        Ok(len)
    }

    fn extension_id(&self) -> ExtensionID {
        EXT_TYPE_SUPPORTED_VERSIONS
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl TLSExtensionWriter for SupportedVersionsExtension {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        // Simplified implementation
        Ok(buf.len())
    }
}

/// PSK Key Exchange Modes extension
/// Corresponds to Go version's &tls.PSKKeyExchangeModesExtension{}
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
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "buffer too short",
            ));
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

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl TLSExtensionWriter for PSKKeyExchangeModesExtension {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        // Simplified implementation
        Ok(buf.len())
    }
}

/// Key Share extension
/// Corresponds to Go version's &tls.KeyShareExtension{}
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
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "buffer too short",
            ));
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

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl TLSExtensionWriter for KeyShareExtension {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        // Simplified implementation
        Ok(buf.len())
    }
}

/// SCT extension
/// Corresponds to Go version's &tls.SCTExtension{}
#[derive(Debug, Clone)]
pub struct SCTExtension;

impl TLSExtension for SCTExtension {
    fn len(&self) -> usize {
        4 // extension_id (2) + length (2, value = 0)
    }

    fn read(&self, buf: &mut [u8]) -> io::Result<usize> {
        if buf.len() < self.len() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "buffer too short",
            ));
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

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl TLSExtensionWriter for SCTExtension {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        Ok(buf.len())
    }
}

/// Renegotiation Info extension
/// Corresponds to Go version's &tls.RenegotiationInfoExtension{}
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
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "buffer too short",
            ));
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

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl TLSExtensionWriter for RenegotiationInfoExtension {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        Ok(buf.len())
    }
}

/// Application Settings Extension New
/// Corresponds to Go version's &tls.ApplicationSettingsExtensionNew{}
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
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "buffer too short",
            ));
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

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl TLSExtensionWriter for ApplicationSettingsExtensionNew {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        Ok(buf.len())
    }
}

/// Compress Certificate extension
/// Corresponds to Go version's &tls.UtlsCompressCertExtension{}
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
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "buffer too short",
            ));
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

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl TLSExtensionWriter for UtlsCompressCertExtension {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        Ok(buf.len())
    }
}

/// Pre-Shared Key extension
/// Corresponds to Go version's &tls.UtlsPreSharedKeyExtension{}
#[derive(Debug, Clone)]
pub struct UtlsPreSharedKeyExtension;

impl TLSExtension for UtlsPreSharedKeyExtension {
    fn len(&self) -> usize {
        4 // extension_id (2) + length (2, will be set dynamically)
    }

    fn read(&self, buf: &mut [u8]) -> io::Result<usize> {
        if buf.len() < self.len() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "buffer too short",
            ));
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

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl TLSExtensionWriter for UtlsPreSharedKeyExtension {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        Ok(buf.len())
    }
}

/// GREASE ECH extension
/// Corresponds to Go version's tls.BoringGREASEECH()
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
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "buffer too short",
            ));
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

    fn as_any(&self) -> &dyn Any {
        self
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

/// Padding extension
/// Corresponds to Go version's &tls.UtlsPaddingExtension{}
pub struct UtlsPaddingExtension {
    pub padding_len: usize,
    pub will_pad: bool,
    pub get_padding_len: Option<PaddingLengthFn>,
}

impl std::fmt::Debug for UtlsPaddingExtension {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UtlsPaddingExtension")
            .field("padding_len", &self.padding_len)
            .field("will_pad", &self.will_pad)
            .field("get_padding_len", &self.get_padding_len.is_some())
            .finish()
    }
}

impl Clone for UtlsPaddingExtension {
    fn clone(&self) -> Self {
        Self {
            padding_len: self.padding_len,
            will_pad: self.will_pad,
            get_padding_len: None, // 不clonefunctionpointer
        }
    }
}

impl UtlsPaddingExtension {
    pub fn new() -> Self {
        Self {
            padding_len: 0,
            will_pad: false,
            get_padding_len: None,
        }
    }

    /// BoringPaddingStyle
    /// Corresponds to Go version's BoringPaddingStyle function
    pub fn boring_padding_style(unpadded_len: usize) -> (usize, bool) {
        if unpadded_len > 0xff && unpadded_len < 0x200 {
            let mut padding_len = 0x200 - unpadded_len;
            if padding_len > 4 {
                padding_len -= 4;
            } else {
                padding_len = 1;
            }
            return (padding_len, true);
        }
        (0, false)
    }
}

impl TLSExtension for UtlsPaddingExtension {
    fn len(&self) -> usize {
        if self.will_pad {
            4 + self.padding_len // extension_id (2) + length (2) + padding
        } else {
            0
        }
    }

    fn read(&self, buf: &mut [u8]) -> io::Result<usize> {
        if !self.will_pad {
            return Ok(0);
        }
        let len = self.len();
        if buf.len() < len {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "buffer too short",
            ));
        }

        // Extension ID
        buf[0] = (EXT_TYPE_PADDING >> 8) as u8;
        buf[1] = (EXT_TYPE_PADDING & 0xff) as u8;

        // Extension length
        buf[2] = (self.padding_len >> 8) as u8;
        buf[3] = (self.padding_len & 0xff) as u8;

        // Padding bytes (zeros)
        for i in 0..self.padding_len {
            buf[4 + i] = 0;
        }

        Ok(len)
    }

    fn extension_id(&self) -> ExtensionID {
        EXT_TYPE_PADDING
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl TLSExtensionWriter for UtlsPaddingExtension {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        // settings BoringPaddingStyle
        self.get_padding_len = Some(Box::new(Self::boring_padding_style));
        Ok(buf.len())
    }
}

impl Default for UtlsPaddingExtension {
    fn default() -> Self {
        Self::new()
    }
}

/// from extension ID Createextensioninstance
/// Corresponds to Go version's ExtensionFromID function
pub fn extension_from_id(id: ExtensionID) -> Option<Box<dyn TLSExtension>> {
    match id {
        EXT_TYPE_SERVER_NAME => Some(Box::new(SNIExtension::new(String::new()))),
        EXT_TYPE_STATUS_REQUEST => Some(Box::new(StatusRequestExtension)),
        EXT_TYPE_SUPPORTED_GROUPS => Some(Box::new(SupportedCurvesExtension::new(vec![]))),
        EXT_TYPE_EC_POINT_FORMATS => Some(Box::new(SupportedPointsExtension::new(vec![]))),
        EXT_TYPE_SIGNATURE_ALGORITHMS => Some(Box::new(SignatureAlgorithmsExtension::new(vec![]))),
        EXT_TYPE_APPLICATION_LAYER_PROTOCOL_NEGOTIATION => {
            Some(Box::new(ALPNExtension::new(vec![])))
        }
        EXT_TYPE_EXTENDED_MASTER_SECRET => Some(Box::new(ExtendedMasterSecretExtension)),
        EXT_TYPE_SESSION_TICKET => Some(Box::new(SessionTicketExtension)),
        EXT_TYPE_SUPPORTED_VERSIONS => Some(Box::new(SupportedVersionsExtension::new(vec![]))),
        EXT_TYPE_PSK_KEY_EXCHANGE_MODES => {
            Some(Box::new(PSKKeyExchangeModesExtension::new(vec![])))
        }
        EXT_TYPE_KEY_SHARE => Some(Box::new(KeyShareExtension::new(vec![]))),
        EXT_TYPE_SIGNED_CERTIFICATE_TIMESTAMP => Some(Box::new(SCTExtension)),
        EXT_TYPE_RENEGOTIATION_INFO => Some(Box::new(RenegotiationInfoExtension::new(1))),
        EXT_TYPE_APPLICATION_SETTINGS_NEW => {
            Some(Box::new(ApplicationSettingsExtensionNew::new(vec![])))
        }
        EXT_TYPE_COMPRESS_CERTIFICATE => Some(Box::new(UtlsCompressCertExtension::new(vec![]))),
        EXT_TYPE_PRE_SHARED_KEY => Some(Box::new(UtlsPreSharedKeyExtension)),
        EXT_TYPE_ECH => Some(Box::new(GREASEEncryptedClientHelloExtension::new())),
        _ => {
            // Checkwhether is GREASE
            if is_grease_uint16(id) {
                Some(Box::new(UtlsGREASEExtension::new()))
            } else {
                None
            }
        }
    }
}

/// Checkwhether is GREASE value
fn is_grease_uint16(v: u16) -> bool {
    // GREASE valuepattern：0x1a1a, 0x2a2a, 0x3a3a,..., 0xfafa
    let low = v & 0xff;
    let high = (v >> 8) & 0xff;
    low == high && (low & 0x0f) == 0x0a
}
