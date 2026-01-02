//! TLS dictionarymodule
//!
//! provide TLS 相close的constantdefine，Corresponds to Go version's dicttls 包
//! count据来source：IANA TLS Parameters

pub mod cipher_suites;
pub mod extensions;
pub mod signature_schemes;
pub mod supported_groups;

pub use cipher_suites::*;
pub use extensions::*;
pub use signature_schemes::*;
// Note: supported_groups 中 GREASE_PLACEHOLDER  and cipher_suites in冲突
// when used needexplicitspecifiedmodulepath
pub use supported_groups::{
    CURVE_P256, CURVE_P384, SECP256R1, SECP384R1, SECP521R1, X25519, X25519_MLKEM768, X448,
};
