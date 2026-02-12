//! # fingerprint-http
//!
//! HTTP client implementation module supporting HTTP/1.1, HTTP/2, and HTTP/3 protocols.
//! Also includes QUIC (RFC 9000) initial packet fingerprinting.

pub mod http_client;
pub mod quic_fingerprint;

pub use http_client::*;
pub use quic_fingerprint::{QuicInitialPacket, QuicPacketType, QuicVersion};
