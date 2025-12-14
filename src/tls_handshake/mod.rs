//! 自定义 TLS 握手实现
//!
//! 根据 ClientHelloSpec 生成真实的 TLS ClientHello 消息
//! 完全使用我们自己的指纹库，不依赖外部 TLS 库

pub mod builder;
pub mod handshake;
pub mod messages;
pub mod record;

pub use builder::TLSHandshakeBuilder;
pub use handshake::{TLSHandshake, TLSHandshakeType};
pub use messages::ClientHelloMessage;
pub use record::{TLSRecord, TLSRecordType};
