//! # fingerprint-core
//!
//! **system-level protection core abstract layer**
//!
//! from **single service protection**improve to **system-level protection**, provides system-level core abstractions and interface.
//!
//! ## core positioning
//!
//! `fingerprint-core` is system-level protectioncore, all external components revolve around this core：
//!
//! - **system-level abstractions**: system context, network traffic, protection decision etc.
//! - **offense and defense unified interface**: fingerprint abstractions, analysis interface, protection interface etc.
//! - **core types and utilities**: type definitions, metadata, utility functions etc.
//!
//! ## Core Features
//!
//! ### system-level abstractions
//!
//! - **system context** (`SystemContext`): complete network entity information (IP, port, protocol, direction etc.)
//! - **network traffic** (`NetworkFlow`): systemlevelnetwork traffic, including context and fingerprint info
//! - **systemprotection interface** (`SystemProtector`): system-level protectionunifiedinterface
//! - **systemanalysis interface** (`SystemAnalyzer`): unified system-level analysis interface
//!
//! ### offense and defense unified abstractions
//!
//! - **fingerprint abstractions** (`Fingerprint` trait): support TLS, HTTP, TCP etc.multiple fingerprint types
//! - **fingerprintmetadata** (`FingerprintMetadata`): including browser, operating system, confidence etc.info
//! - **TLS fingerprint** (`ClientHelloSignature`): TLS ClientHello signature
//! - **HTTP fingerprint** (`HttpFingerprint`): HTTP request fingerprint
//! - **TCP fingerprint** (`TcpFingerprint`): TCP connection fingerprint
//!
//! ### core types and utilities
//!
//! - **type system**: `BrowserType`, `OperatingSystem` etc.coretype
//! - **utility functions**: GREASE process, randomly select etc.utility functions

pub mod benchmark;
pub mod database;
pub mod dicttls;
pub mod fingerprint;
pub mod grease;
pub mod hassh;
pub mod hpack;
pub mod http;
pub mod http2_frame_parser;
pub mod ja3;
pub mod ja4;
pub mod jarm;
pub mod metadata;
pub mod packet_capture;
pub mod pcap_generator;
pub mod signature;
pub mod system;
pub mod tcp;
pub mod tcp_handshake;
pub mod types;
pub mod utils;
pub mod version; // Performance benchmarking utilities

// Re-export public API

// fingerprint abstractions
pub use fingerprint::{Fingerprint, FingerprintComparator, FingerprintComparison, FingerprintType};

// metadata
pub use metadata::FingerprintMetadata;

// TLS related
pub use dicttls::*;
pub use grease::{
    filter_grease_values, get_random_grease, is_grease_value, remove_grease_values,
    TLS_GREASE_VALUES,
};
pub use hassh::{HASSHServer, SSHKexInit, HASSH, JA4SSH};
pub use ja3::{JA3, JA3S};
pub use ja4::{ConsistencyReport, JA4, JA4H, JA4L, JA4S, JA4T};
pub use signature::ClientHelloSignature;
pub use version::TlsVersion;

// HTTP related
pub use hpack::{
    static_table, DynamicTableEntry, DynamicTableSnapshot, EncodedHeaderField, HpackAnalyzer,
    HpackFingerprint, HpackHeaderList, HuffmanEncoding, IndexType, StaticTableEntry,
};
pub use http::{Http2Settings, HttpFingerprint};
pub use http2_frame_parser::{
    find_settings_frame, is_http2_connection, Http2FrameHeader, Http2FrameType,
    Http2ParseError, Http2SettingsFrame, Http2SettingsMatcher, HTTP2_PREFACE,
};
pub use packet_capture::{
    EthernetHeader, Ipv4Header, Ipv6Header, NetworkProtocol, PacketFlowAnalyzer, PacketParser,
    ParsedPacket, PcapGlobalHeader, PcapPacketHeader, TcpFlow, TcpHeader, TransportProtocol,
    UdpHeader,
};

// TCP related
pub use tcp::{TcpFingerprint, TcpProfile};
pub use tcp_handshake::{
    signatures, AckCharacteristics, IpCharacteristics, SynAckCharacteristics, SynCharacteristics,
    TcpFlags, TcpHandshakeAnalyzer, TcpHandshakeFingerprint, TcpOption, TcpOptionType,
};

// type system
pub use types::{
    BrowserType, OperatingSystem, OperatingSystems, UserAgentTemplate, OPERATING_SYSTEMS,
};

// utility functions
pub use utils::{
    extract_chrome_version, extract_platform, infer_browser_from_profile_name, is_mobile_profile,
    random_choice, random_choice_string,
};

// benchmarking (optional, for performance testing)
pub use benchmark::{Benchmark, HttpMetrics, Timer};

// system-level abstractions
pub use system::{
    AnalysisDetails,
    FlowCharacteristics,
    // network traffic
    NetworkFlow,
    ProtocolType,
    SystemAnalysisResult,
    // systemanalysis
    SystemAnalyzer,
    // system context
    SystemContext,
    SystemProtectionDecision,
    SystemProtectionResult,
    // systemstatistics
    SystemProtectionStats,
    // systemprotection
    SystemProtector,
    ThreatType,
    TrafficDirection,
};
