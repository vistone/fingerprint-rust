//! TLS 扩展类型常量
//!
//! 来源：https://www.iana.org/assignments/tls-extensiontype-values/tls-extensiontype-values.xhtml
//! 最后更新：March 2023

/// TLS 扩展类型常量
/// 对应 Go 版本的 tls.Extension* 常量
pub mod extension_types {
    pub const SERVER_NAME: u16 = 0;
    pub const STATUS_REQUEST: u16 = 5;
    pub const SUPPORTED_GROUPS: u16 = 10;
    pub const EC_POINT_FORMATS: u16 = 11;
    pub const SIGNATURE_ALGORITHMS: u16 = 13;
    pub const ALPN: u16 = 16;
    pub const SCT: u16 = 18;
    pub const PADDING: u16 = 21;
    pub const EXTENDED_MASTER_SECRET: u16 = 23;
    pub const SESSION_TICKET: u16 = 35;
    pub const SUPPORTED_VERSIONS: u16 = 43;
    pub const PSK_KEY_EXCHANGE_MODES: u16 = 45;
    pub const KEY_SHARE: u16 = 51;
    pub const RENEGOTIATION_INFO: u16 = 65281;

    // 非 IANA 分配的扩展
    pub const APPLICATION_SETTINGS: u16 = 17513;
    pub const APPLICATION_SETTINGS_NEW: u16 = 17613;
    pub const ECH: u16 = 0xfe0d;
}

pub use extension_types::*;

/// 压缩方法常量
pub mod compression {
    pub const COMPRESSION_NONE: u8 = 0;
}

pub use compression::*;

/// 点格式常量
pub mod point_formats {
    pub const POINT_FORMAT_UNCOMPRESSED: u8 = 0;
}

pub use point_formats::*;

/// PSK 模式常量
pub mod psk_modes {
    pub const PSK_MODE_DHE: u8 = 1;
}

pub use psk_modes::*;

/// TLS 版本常量
pub mod versions {
    pub const VERSION_TLS12: u16 = 0x0303;
    pub const VERSION_TLS13: u16 = 0x0304;
}

pub use versions::*;

/// 证书压缩算法常量
pub mod cert_compression {
    pub const CERT_COMPRESSION_BROTLI: u16 = 0x0002;
}

pub use cert_compression::*;

/// 重新协商常量
pub mod renegotiation {
    pub const RENEGOTIATE_ONCE_AS_CLIENT: u8 = 1;
}

pub use renegotiation::*;
