//! TLS signature方案constant
//!
//! 来source：https://www.iana.org/assignments/tls-parameters/tls-signaturescheme.csv
//! finallyUpdate：March 2023

/// TLS signature方案constant
/// Corresponds to Go version's tls.SignatureScheme constant
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

/// SignatureScheme type别名
/// Corresponds to Go version's tls.SignatureScheme
pub type SignatureScheme = u16;

/// 为了 and Go versionkeep一致，provide别名
/// Corresponds to Go version's tls.ECDSAWithP256AndSHA256
pub const ECDSA_WITH_P256_AND_SHA256: u16 = signature_schemes::ECDSA_WITH_P256_AND_SHA256;
/// Corresponds to Go version's tls.PSSWithSHA256
pub const PSS_WITH_SHA256: u16 = signature_schemes::RSA_PSS_RSAE_SHA256;
/// Corresponds to Go version's tls.PKCS1WithSHA256
pub const PKCS1_WITH_SHA256: u16 = signature_schemes::RSA_PKCS1_SHA256;
/// Corresponds to Go version's tls.ECDSAWithP384AndSHA384
pub const ECDSA_WITH_P384_AND_SHA384: u16 = signature_schemes::ECDSA_WITH_P384_AND_SHA384;
/// Corresponds to Go version's tls.PSSWithSHA384
pub const PSS_WITH_SHA384: u16 = signature_schemes::RSA_PSS_RSAE_SHA384;
/// Corresponds to Go version's tls.PKCS1WithSHA384
pub const PKCS1_WITH_SHA384: u16 = signature_schemes::RSA_PKCS1_SHA384;
/// Corresponds to Go version's tls.PSSWithSHA512
pub const PSS_WITH_SHA512: u16 = signature_schemes::RSA_PSS_RSAE_SHA512;
/// Corresponds to Go version's tls.PKCS1WithSHA512
pub const PKCS1_WITH_SHA512: u16 = signature_schemes::RSA_PKCS1_SHA512;
