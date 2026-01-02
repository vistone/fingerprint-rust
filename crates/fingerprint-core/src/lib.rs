//! # fingerprint-core
//!
//! **系统级别防护的核心抽象层**
//!
//! 从**单一服务防护**提升到**系统级别防护**，提供系统级别的核心抽象和接口。
//!
//! ## 核心定位
//!
//! `fingerprint-core` 是系统级别防护的核心，所有外部组件都围绕这个核心展开：
//!
//! - **系统级别抽象**: 系统上下文、网络流量、防护决策等
//! - **攻防统一接口**: 指纹抽象、分析接口、防护接口等
//! - **核心类型和工具**: 类型定义、元数据、工具函数等
//!
//! ## 核心功能
//!
//! ### 系统级别抽象
//!
//! - **系统上下文** (`SystemContext`): 网络实体的完整信息（IP、端口、协议、方向等）
//! - **网络流量** (`NetworkFlow`): 系统级别的网络流量，包含上下文和指纹信息
//! - **系统防护接口** (`SystemProtector`): 系统级别防护的统一接口
//! - **系统分析接口** (`SystemAnalyzer`): 系统级别分析的统一接口
//!
//! ### 攻防统一抽象
//!
//! - **指纹抽象** (`Fingerprint` trait): 支持 TLS、HTTP、TCP 等多种指纹类型
//! - **指纹元数据** (`FingerprintMetadata`): 包含浏览器、操作系统、置信度等信息
//! - **TLS 指纹** (`ClientHelloSignature`): TLS ClientHello 签名
//! - **HTTP 指纹** (`HttpFingerprint`): HTTP 请求指纹
//! - **TCP 指纹** (`TcpFingerprint`): TCP 连接指纹
//!
//! ### 核心类型和工具
//!
//! - **类型系统**: `BrowserType`、`OperatingSystem` 等核心类型
//! - **工具函数**: GREASE 处理、随机选择等工具函数

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

// 指纹抽象
pub use fingerprint::{Fingerprint, FingerprintComparator, FingerprintComparison, FingerprintType};

// 元数据
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

// 类型系统
pub use types::{
    BrowserType, OperatingSystem, OperatingSystems, UserAgentTemplate, OPERATING_SYSTEMS,
};

// 工具函数
pub use utils::{
    extract_chrome_version, extract_platform, infer_browser_from_profile_name, is_mobile_profile,
    random_choice, random_choice_string,
};

// 系统级别抽象
pub use system::{
    AnalysisDetails,
    FlowCharacteristics,
    // 网络流量
    NetworkFlow,
    ProtocolType,
    SystemAnalysisResult,
    // 系统分析
    SystemAnalyzer,
    // 系统上下文
    SystemContext,
    SystemProtectionDecision,
    SystemProtectionResult,
    // 系统统计
    SystemProtectionStats,
    // 系统防护
    SystemProtector,
    ThreatType,
    TrafficDirection,
};
