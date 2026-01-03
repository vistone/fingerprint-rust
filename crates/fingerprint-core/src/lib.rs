//! # fingerprint-core
//!
//! **system levelprotectioncoreabstractlayer**
//!
//! from **singleserviceprotection**improve to **system levelprotection**，providesystem levelcoreabstract and interface。
//!
//! ## core定bit
//!
//! `fingerprint-core` is system levelprotectioncore， all outside part component all 围绕thiscore展 open ：
//!
//! - **system levelabstract**: system updown text 、networktraffic、protectiondecision etc.
//! - **offense and defenseunifiedinterface**: fingerprintabstract、analysisinterface、protectioninterface etc.
//! - **coretype and tool**: typedefine、metadata、toolfunction etc.
//!
//! ## coreFeatures
//!
//! ### system levelabstract
//!
//! - **system updown text ** (`SystemContext`): networkentitycompleteinfo (IP、port、protocol、direction etc.)
//! - **networktraffic** (`NetworkFlow`): system levelnetworktraffic，includingupdown text and fingerprintinfo
//! - **system protectioninterface** (`SystemProtector`): system levelprotectionunifiedinterface
//! - **system analysisinterface** (`SystemAnalyzer`): system levelanalysisunifiedinterface
//!
//! ### offense and defenseunifiedabstract
//!
//! - **fingerprintabstract** (`Fingerprint` trait): support TLS、HTTP、TCP etc.multiplefingerprinttype
//! - **fingerprintmetadata** (`FingerprintMetadata`): includingbrowser、operating system 、confidence etc.info
//! - **TLS fingerprint** (`ClientHelloSignature`): TLS ClientHello signature
//! - **HTTP fingerprint** (`HttpFingerprint`): HTTP requestfingerprint
//! - **TCP fingerprint** (`TcpFingerprint`): TCP connectionfingerprint
//!
//! ### coretype and tool
//!
//! - **typesystem **: `BrowserType`、`OperatingSystem` etc.coretype
//! - **toolfunction**: GREASE process、randomly select etc.toolfunction

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

// Re-export public API

// fingerprintabstract
pub use fingerprint::{Fingerprint, FingerprintComparator, FingerprintComparison, FingerprintType};

// metadata
pub use metadata::FingerprintMetadata;

// TLS phase close
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

// HTTP phase close
pub use http::{Http2Settings, HttpFingerprint};

// TCP phase close
pub use tcp::{TcpFingerprint, TcpProfile};

// typesystem 
pub use types::{
 BrowserType, OperatingSystem, OperatingSystems, UserAgentTemplate, OPERATING_SYSTEMS,
};

// toolfunction
pub use utils::{
 extract_chrome_version, extract_platform, infer_browser_from_profile_name, is_ mobile _profile,
 random_choice, random_choice_string,
};

// system levelabstract
pub use system ::{
 AnalysisDetails,
 FlowCharacteristics,
 // networktraffic
 NetworkFlow,
 ProtocolType,
 SystemAnalysisResult,
 // system analysis
 SystemAnalyzer,
 // system updown text 
 SystemContext,
 SystemProtectionDecision,
 SystemProtectionResult,
 // system statistics
 SystemProtectionStats,
 // system protection
 SystemProtector,
 ThreatType,
 TrafficDirection,
};
