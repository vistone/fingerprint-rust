//! TLS extensiontypeconstant
//!
//! fromsource：<https://www.iana.org/assignments/tls-extensiontype-values/tls-extensiontype-values.xhtml>
//! finallyUpdate：March 2023

/// TLS extensiontypeconstant
/// Corresponds to Go version's tls.Extension* constant
pub mod extension_types {
    pub const EXT_TYPE_SERVER_NAME: u16 = 0;
    pub const EXT_TYPE_STATUS_REQUEST: u16 = 5;
    pub const EXT_TYPE_SUPPORTED_GROUPS: u16 = 10;
    pub const EXT_TYPE_EC_POINT_FORMATS: u16 = 11;
    pub const EXT_TYPE_SIGNATURE_ALGORITHMS: u16 = 13;
    pub const EXT_TYPE_APPLICATION_LAYER_PROTOCOL_NEGOTIATION: u16 = 16;
    pub const EXT_TYPE_SIGNED_CERTIFICATE_TIMESTAMP: u16 = 18;
    pub const EXT_TYPE_PADDING: u16 = 21;
    pub const EXT_TYPE_EXTENDED_MASTER_SECRET: u16 = 23;
    pub const EXT_TYPE_SESSION_TICKET: u16 = 35;
    pub const EXT_TYPE_SUPPORTED_VERSIONS: u16 = 43;
    pub const EXT_TYPE_PSK_KEY_EXCHANGE_MODES: u16 = 45;
    pub const EXT_TYPE_KEY_SHARE: u16 = 51;
    pub const EXT_TYPE_RENEGOTIATION_INFO: u16 = 65281;
    pub const EXT_TYPE_PRE_SHARED_KEY: u16 = 41;
    pub const EXT_TYPE_COMPRESS_CERTIFICATE: u16 = 27;
    pub const EXT_TYPE_ECH: u16 = 0x0042; // Encrypted Client Hello (RFC 9180)

    // non IANA allocate's extensions
    pub const EXT_TYPE_APPLICATION_SETTINGS: u16 = 17513;
    pub const EXT_TYPE_APPLICATION_SETTINGS_NEW: u16 = 17613;
}

pub use extension_types::*;

/// compressionmethodconstant
pub mod compression {
    pub const COMPRESSION_NONE: u8 = 0;
}

pub use compression::*;

/// pointformatconstant
pub mod point_formats {
    pub const POINT_FORMAT_UNCOMPRESSED: u8 = 0;
}

pub use point_formats::*;

/// PSK patternconstant
pub mod psk_modes {
    pub const PSK_MODE_DHE: u8 = 1;
}

pub use psk_modes::*;

/// TLS versionconstant
pub mod versions {
    pub const VERSION_TLS12: u16 = 0x0303;
    pub const VERSION_TLS13: u16 = 0x0304;
}

pub use versions::*;

/// certificatecompressionalgorithmconstant
pub mod cert_compression {
    pub const CERT_COMPRESSION_BROTLI: u16 = 0x0002;
}

pub use cert_compression::*;

/// renegotiateconstant
pub mod renegotiation {
    pub const RENEGOTIATE_ONCE_AS_CLIENT: u8 = 1;
}

pub use renegotiation::*;
