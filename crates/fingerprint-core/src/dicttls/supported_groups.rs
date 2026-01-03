//! TLS supportgroup (elliptic curve)constant
//!
//! fromsource：https://www.iana.org/assignments/tls-parameters/tls-parameters.xhtml#tls-parameters-8
//! finallyUpdate：March 2023

/// TLS supportgroupconstant
/// Corresponds to Go version's tls.CurveID constant
#[allow(clippy::module_inception)]
pub mod supported_groups {
 // elliptic curve
 pub const SECP256R1: u16 = 0x0017; // 23
 pub const SECP384R1: u16 = 0x0018; // 24
 pub const SECP521R1: u16 = 0x0019; // 25
 pub const X25519: u16 = 0x001d; // 29
 pub const X448: u16 = 0x001a; // 30

 // GREASE placeholder
 pub const GREASE_PLACEHOLDER: u16 = 0x0a0a;
}

pub use supported_groups::*;

/// CurveID typealias
/// Corresponds to Go version's tls.CurveID
pub type CurveID = u16;

/// in order to and Go versionkeepconsistent，providealias
/// Corresponds to Go version's tls.CurveP256
pub const CURVE_P256: u16 = supported_groups::SECP256R1;
/// Corresponds to Go version's tls.CurveP384
pub const CURVE_P384: u16 = supported_groups::SECP384R1;
/// Corresponds to Go version's tls.X25519
pub const X25519: u16 = supported_groups::X25519;
/// Corresponds to Go version's tls.X25519MLKEM768 (Chrome 133 new)
/// Note: this is ML-KEM curve
pub const X25519_MLKEM768: u16 = 0x6399;
