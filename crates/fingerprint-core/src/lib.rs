//! # fingerprint-core
//!
//! **System-level protection core abstract layer**
//!
//! From **single service protection** improve to **system-level protection**, provides system-level core abstractions and interface.
//!
//! ## Core Positioning
//!
//! `fingerprint-core` is system-level protection core, all external components revolve around this core:
//!
//! - **System-level abstractions**: system context, network traffic, protection decision etc.
//! - **Offense and defense unified interface**: fingerprint abstractions, analysis interface, protection interface etc.
//! - **Core types and utilities**: type definitions, metadata, utility functions etc.
//!
//! ## Core Features
//!
//! ### System-level Abstractions
//!
//! - **System context** (`SystemContext`): complete network entity information (IP, port, protocol, direction etc.)
//! - **Network traffic** (`NetworkFlow`): system level network traffic, including context and fingerprint info
//! - **System protection interface** (`SystemProtector`): system-level protection unified interface
//! - **System analysis interface** (`SystemAnalyzer`): unified system-level analysis interface
//!
//! ### Offense and defense unified abstractions
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
pub mod cache; // Multi-tier caching (L1/L2/L3)
pub mod cache_redis; // Redis-backed cache implementation
pub mod database;
pub mod dicttls;
pub mod error; // Comprehensive error types
pub mod fingerprint;
pub mod grease;
pub mod hassh;
pub mod hpack;
pub mod http;
pub mod http2_frame_parser;
pub mod ja3;
pub mod ja3_database;
pub mod ja4;
pub mod jarm;
pub mod metadata;
pub mod packet_capture;
pub mod pcap_generator;
pub mod pqc; // Post-Quantum Cryptography detection
pub mod rate_limiting; // Distributed rate limiting service (Phase 9.4)
pub mod rate_limiting_metrics; // Prometheus metrics for rate limiting
pub mod rate_limiting_redis; // Redis integration for rate limiting
pub mod signature;
pub mod system;
pub mod tcp;
pub mod tcp_handshake;
pub mod tls_parser;
pub mod types;
pub mod utils;
pub mod version; // Performance benchmarking utilities
pub mod wasm; // WebAssembly fingerprinting detection

// Re-export public API

// Error types
pub use error::{
    CacheError, ConfigError, DatabaseError, DnsError, FingerprintError, HttpError, ParseError,
    RateLimitError, Result, TcpError, TlsError, ValidationError,
};

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
pub use ja4::{ConsistencyReport, JA4, JA4H, JA4L, JA4S, JA4T, JA4X};
pub use signature::ClientHelloSignature;
pub use version::TlsVersion;

// Post-Quantum Cryptography
pub use pqc::{PQCAlgorithm, PQCBrowserSupport, PQCCapabilities};

// WebAssembly fingerprinting
pub use wasm::{
    WasmBrowserSupport, WasmCapabilities, WasmMemoryFingerprint, WasmTableFingerprint, WasmVersion,
};

// HTTP related
pub use hpack::{
    static_table, DynamicTableEntry, DynamicTableSnapshot, EncodedHeaderField, HpackAnalyzer,
    HpackFingerprint, HpackHeaderList, HuffmanEncoding, IndexType, StaticTableEntry,
};
pub use http::{Http2Settings, HttpFingerprint};
pub use http2_frame_parser::{
    find_settings_frame, is_http2_connection, Http2FrameHeader, Http2FrameType, Http2ParseError,
    Http2SettingsFrame, Http2SettingsMatcher, HTTP2_PREFACE,
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
pub use benchmark::{Benchmark, CacheBenchmark, CacheBenchmarkSuite, HttpMetrics, Timer};

// rate limiting service (Phase 9.4)
pub use rate_limiting::{
    current_unix_timestamp, EndpointConfig, MetricsSnapshot, QuotaTier, RateLimitResponse,
    RateLimiter, UserQuota,
};

// rate limiting Redis backend
pub use rate_limiting_redis::{
    RedisBackendError, RedisConfig, RedisQuotaEntry, RedisRateLimitBackend, RedisResult,
};

// rate limiting Prometheus metrics
pub use rate_limiting_metrics::{MetricsHandler, PrometheusMetrics, TierMetrics};

// cache (Phase 9.3)
pub use cache::{Cache, CacheResult, CacheStats, CacheTTL, CacheTier, DistributedLock, LockGuard};

// Redis cache (optional, requires redis-cache feature)
pub use cache_redis::RedisCacheConfig;

#[cfg(feature = "redis-cache")]
pub use cache_redis::RedisCache;

#[cfg(feature = "redis-cache")]
pub use cache_redis::RedisClusterCache;

#[cfg(feature = "redis-cache")]
pub use cache_redis::RedisClusterConfig;

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
