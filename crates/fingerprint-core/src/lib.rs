//! # fingerprint-core
//!
//! **systemlevelprotection的core抽象layer**
//!
//!  from **单一serviceprotection**提升 to **systemlevelprotection**，providesystemlevel的core抽象 and interface。
//!
//! ## core定bit
//!
//! `fingerprint-core` 是systemlevelprotection的core，alloutside部component都围绕这个core展开：
//!
//! - **systemlevel抽象**: systemupdown文、networktraffic、protection决策等
//! - **攻防统一interface**: fingerprint抽象、analysisinterface、protectioninterface等
//! - **coretype and 工具**: typedefine、metadata、工具function等
//!
//! ## coreFeatures
//!
//! ### systemlevel抽象
//!
//! - **systemupdown文** (`SystemContext`): networkentity的completeinfo（IP、port、protocol、方向等）
//! - **networktraffic** (`NetworkFlow`): systemlevel的networktraffic，includingupdown文 and fingerprintinfo
//! - **systemprotectioninterface** (`SystemProtector`): systemlevelprotection的统一interface
//! - **systemanalysisinterface** (`SystemAnalyzer`): systemlevelanalysis的统一interface
//!
//! ### 攻防统一抽象
//!
//! - **fingerprint抽象** (`Fingerprint` trait): support TLS、HTTP、TCP 等多种fingerprinttype
//! - **fingerprintmetadata** (`FingerprintMetadata`): includingbrowser、operating system、置信度等info
//! - **TLS fingerprint** (`ClientHelloSignature`): TLS ClientHello signature
//! - **HTTP fingerprint** (`HttpFingerprint`): HTTP requestfingerprint
//! - **TCP fingerprint** (`TcpFingerprint`): TCP connectionfingerprint
//!
//! ### coretype and 工具
//!
//! - **typesystem**: `BrowserType`、`OperatingSystem` 等coretype
//! - **工具function**: GREASE process、randomly select等工具function

pub mod database;
pub mod dicttls;
pub mod fingerprint;
pub mod grease;
pub mod hassh;
pub mod http;
pub mod ja3;
pub mod ja4;
pub mod jarm;
pub mod metadata;
pub mod signature;
pub mod system;
pub mod tcp;
pub mod types;
pub mod utils;
pub mod version;

// Re-export 公共 API

// fingerprint抽象
pub use fingerprint::{Fingerprint, FingerprintComparator, FingerprintComparison, FingerprintType};

// metadata
pub use metadata::FingerprintMetadata;

// TLS 相关
pub use dicttls::*;
pub use grease::{
    filter_grease_values, get_random_grease, is_grease_value, remove_grease_values,
    TLS_GREASE_VALUES,
};
pub use hassh::{HASSH, HASSHServer, JA4SSH, SSHKexInit};
pub use ja3::{JA3, JA3S};
pub use ja4::{ConsistencyReport, JA4, JA4H, JA4L, JA4S, JA4T};
pub use signature::ClientHelloSignature;
pub use version::TlsVersion;

// HTTP 相关
pub use http::{Http2Settings, HttpFingerprint};

// TCP 相关
pub use tcp::{TcpFingerprint, TcpProfile};

// typesystem
pub use types::{
    BrowserType, OperatingSystem, OperatingSystems, UserAgentTemplate, OPERATING_SYSTEMS,
};

// 工具function
pub use utils::{
    extract_chrome_version, extract_platform, infer_browser_from_profile_name, is_mobile_profile,
    random_choice, random_choice_string,
};

// systemlevel抽象
pub use system::{
    AnalysisDetails,
    FlowCharacteristics,
    // networktraffic
    NetworkFlow,
    ProtocolType,
    SystemAnalysisResult,
    // systemanalysis
    SystemAnalyzer,
    // systemupdown文
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
