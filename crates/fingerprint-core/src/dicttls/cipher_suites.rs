//! TLS cipher suiteconstant
//!
//! from source：https://www.iana.org/assignments/tls-parameters/tls-parameters.xhtml#tls-parameters-4
//! fin all yUpdate：March 2023

/// TLS cipher suiteconstant
/// Corresponds to Go version's tls.TLS_* constant
#[ all ow (clippy::module_inception)]
pub mod cipher_suites {
 // TLS 1.3 cipher suite
 pub const TLS_AES_128_GCM_SHA256: u16 = 0x1301;
 pub const TLS_AES_256_GCM_SHA384: u16 = 0x1302;
 pub const TLS_CHACHA20_POLY1305_SHA256: u16 = 0x1303;
 pub const TLS_AES_128_CCM_SHA256: u16 = 0x1304;
 pub const TLS_AES_128_CCM_8_SHA256: u16 = 0x1305;

 // TLS 1.2 ECDHE cipher suite
 pub const TLS_ECDHE_ECDSA_WITH_AES_128_GCM_SHA256: u16 = 0xc02b;
 pub const TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256: u16 = 0xc02f;
 pub const TLS_ECDHE_ECDSA_WITH_AES_256_GCM_SHA384: u16 = 0xc02c;
 pub const TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384: u16 = 0xc030;
 pub const TLS_ECDHE_ECDSA_WITH_CHACHA20_POLY1305_SHA256: u16 = 0xcca9;
 pub const TLS_ECDHE_RSA_WITH_CHACHA20_POLY1305_SHA256: u16 = 0xcca8;
 pub const TLS_ECDHE_RSA_WITH_AES_128_CBC_SHA: u16 = 0xc013;
 pub const TLS_ECDHE_RSA_WITH_AES_256_CBC_SHA: u16 = 0xc014;
 pub const TLS_ECDHE_ECDSA_WITH_AES_128_CBC_SHA: u16 = 0xc009;
 pub const TLS_ECDHE_ECDSA_WITH_AES_256_CBC_SHA: u16 = 0xc00a;

 // TLS 1.2 RSA cipher suite
 pub const TLS_RSA_WITH_AES_128_GCM_SHA256: u16 = 0x009c;
 pub const TLS_RSA_WITH_AES_256_GCM_SHA384: u16 = 0x009d;
 pub const TLS_RSA_WITH_AES_128_CBC_SHA: u16 = 0x002f;
 pub const TLS_RSA_WITH_AES_256_CBC_SHA: u16 = 0x0035;

 // GREASE placeholder
 pub const GREASE_PLACEHOLDER: u16 = 0x0a0a;
}

pub use cipher_suites::*;
