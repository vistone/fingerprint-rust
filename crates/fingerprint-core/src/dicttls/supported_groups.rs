//! TLS 支持的组（椭圆曲线）常量
//!
//! 来源：https://www.iana.org/assignments/tls-parameters/tls-parameters.xhtml#tls-parameters-8
//! 最后更新：March 2023

/// TLS 支持的组常量
/// 对应 Go 版本的 tls.CurveID 常量
#[allow(clippy::module_inception)]
pub mod supported_groups {
    // 椭圆曲线
    pub const SECP256R1: u16 = 0x0017; // 23
    pub const SECP384R1: u16 = 0x0018; // 24
    pub const SECP521R1: u16 = 0x0019; // 25
    pub const X25519: u16 = 0x001d; // 29
    pub const X448: u16 = 0x001a; // 30

    // GREASE placeholder
    pub const GREASE_PLACEHOLDER: u16 = 0x0a0a;
}

pub use supported_groups::*;

/// CurveID 类型别名
/// 对应 Go 版本的 tls.CurveID
pub type CurveID = u16;

/// 为了与 Go 版本保持一致，提供别名
/// 对应 Go 版本的 tls.CurveP256
pub const CURVE_P256: u16 = supported_groups::SECP256R1;
/// 对应 Go 版本的 tls.CurveP384
pub const CURVE_P384: u16 = supported_groups::SECP384R1;
/// 对应 Go 版本的 tls.X25519
pub const X25519: u16 = supported_groups::X25519;
/// 对应 Go 版本的 tls.X25519MLKEM768 (Chrome 133 新增)
/// 注意：这是 ML-KEM 曲线
pub const X25519_MLKEM768: u16 = 0x6399;
