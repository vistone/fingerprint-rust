//! TLS 签名方案常量
//!
//! 来源：https://www.iana.org/assignments/tls-parameters/tls-signaturescheme.csv
//! 最后更新：March 2023

/// TLS 签名方案常量
/// 对应 Go 版本的 tls.SignatureScheme 常量
#[allow(clippy::module_inception)]
pub mod signature_schemes {
    // RSA PKCS1
    pub const RSA_PKCS1_SHA256: u16 = 0x0401;
    pub const RSA_PKCS1_SHA384: u16 = 0x0501;
    pub const RSA_PKCS1_SHA512: u16 = 0x0601;

    // RSA PSS
    pub const RSA_PSS_RSAE_SHA256: u16 = 0x0804;
    pub const RSA_PSS_RSAE_SHA384: u16 = 0x0805;
    pub const RSA_PSS_RSAE_SHA512: u16 = 0x0806;

    // ECDSA
    pub const ECDSA_WITH_P256_AND_SHA256: u16 = 0x0403;
    pub const ECDSA_WITH_P384_AND_SHA384: u16 = 0x0503;
    pub const ECDSA_WITH_P521_AND_SHA512: u16 = 0x0603;

    // EdDSA
    pub const ED25519: u16 = 0x0807;
    pub const ED448: u16 = 0x0808;
}

pub use signature_schemes::*;

/// SignatureScheme 类型别名
/// 对应 Go 版本的 tls.SignatureScheme
pub type SignatureScheme = u16;

/// 为了与 Go 版本保持一致，提供别名
/// 对应 Go 版本的 tls.ECDSAWithP256AndSHA256
pub const ECDSA_WITH_P256_AND_SHA256: u16 = signature_schemes::ECDSA_WITH_P256_AND_SHA256;
/// 对应 Go 版本的 tls.PSSWithSHA256
pub const PSS_WITH_SHA256: u16 = signature_schemes::RSA_PSS_RSAE_SHA256;
/// 对应 Go 版本的 tls.PKCS1WithSHA256
pub const PKCS1_WITH_SHA256: u16 = signature_schemes::RSA_PKCS1_SHA256;
/// 对应 Go 版本的 tls.ECDSAWithP384AndSHA384
pub const ECDSA_WITH_P384_AND_SHA384: u16 = signature_schemes::ECDSA_WITH_P384_AND_SHA384;
/// 对应 Go 版本的 tls.PSSWithSHA384
pub const PSS_WITH_SHA384: u16 = signature_schemes::RSA_PSS_RSAE_SHA384;
/// 对应 Go 版本的 tls.PKCS1WithSHA384
pub const PKCS1_WITH_SHA384: u16 = signature_schemes::RSA_PKCS1_SHA384;
/// 对应 Go 版本的 tls.PSSWithSHA512
pub const PSS_WITH_SHA512: u16 = signature_schemes::RSA_PSS_RSAE_SHA512;
/// 对应 Go 版本的 tls.PKCS1WithSHA512
pub const PKCS1_WITH_SHA512: u16 = signature_schemes::RSA_PKCS1_SHA512;
