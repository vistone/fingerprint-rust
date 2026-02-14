//! TLS dictionary module
//!
//! Provide TLS related constant define, corresponds to Go version's dicttls package.
//! Count data from source: IANA TLS Parameters.

pub mod cipher_suites;
pub mod extensions;
pub mod signature_schemes;
pub mod supported_groups;

pub use cipher_suites::*;
pub use extensions::*;
pub use signature_schemes::*;
// Note: supported_groups in GREASE_PLACEHOLDER and cipher_suites in conflict
// when used need explicit specified module path
pub use supported_groups::{
    CURVE_P256, CURVE_P384, SECP256R1, SECP384R1, SECP521R1, X25519, X25519_MLKEM768, X448,
};
