//! TLS 字典模块
//!
//! 提供 TLS 相关的常量定义，对应 Go 版本的 dicttls 包
//! 数据来源：IANA TLS Parameters

pub mod cipher_suites;
pub mod extensions;
pub mod signature_schemes;
pub mod supported_groups;

pub use cipher_suites::*;
pub use extensions::*;
pub use signature_schemes::*;
// 注意：supported_groups 中的 GREASE_PLACEHOLDER 与 cipher_suites 中的冲突
// 使用时需要明确指定模块路径
pub use supported_groups::{CURVE_P256, CURVE_P384, SECP256R1, SECP384R1, SECP521R1, X25519, X25519_MLKEM768, X448};
