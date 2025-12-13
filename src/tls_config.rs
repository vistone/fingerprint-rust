//! TLS 配置模块
//!
//! 提供真实的 TLS Client Hello 配置，对应 Go 版本的 utls.ClientHelloID

use crate::dicttls::{
    cipher_suites::{self as cs, GREASE_PLACEHOLDER as GREASE_CS},
    extensions::*,
    signature_schemes::{
        self as ss,
        ECDSA_WITH_P256_AND_SHA256, ECDSA_WITH_P384_AND_SHA384,
        PSS_WITH_SHA256, PSS_WITH_SHA384, PSS_WITH_SHA512,
        PKCS1_WITH_SHA256, PKCS1_WITH_SHA384, PKCS1_WITH_SHA512,
    },
    supported_groups::{GREASE_PLACEHOLDER as GREASE_SG, CURVE_P256, CURVE_P384, SECP521R1, X25519, X25519_MLKEM768},
};
use std::collections::HashMap;

/// TLS 版本
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TLSVersion {
    TLS1_2,
    TLS1_3,
}

/// 密码套件 ID
pub type CipherSuiteID = u16;

/// TLS Client Hello 配置
/// 对应 Go 版本的 tls.ClientHelloSpec
#[derive(Debug, Clone)]
pub struct ClientHelloSpec {
    /// TLS 版本
    pub tls_versions: Vec<u16>,
    /// 密码套件列表
    pub cipher_suites: Vec<CipherSuiteID>,
    /// 压缩方法
    pub compression_methods: Vec<u8>,
    /// 扩展列表
    pub extensions: Vec<Extension>,
    /// 椭圆曲线列表
    pub elliptic_curves: Vec<u16>,
    /// 椭圆曲线点格式
    pub elliptic_curve_point_formats: Vec<u8>,
    /// ALPN 协议列表
    pub alpn_protocols: Vec<String>,
    /// 签名算法
    pub signature_algorithms: Vec<u16>,
    /// 签名算法证书
    pub signature_algorithms_cert: Vec<u16>,
    /// 支持的组（用于 TLS 1.3）
    pub supported_groups: Vec<u16>,
    /// 支持的版本（用于 TLS 1.3）
    pub supported_versions: Vec<u16>,
    /// PSK 密钥交换模式（用于 TLS 1.3）
    pub psk_key_exchange_modes: Vec<u8>,
    /// 其他自定义扩展
    pub custom_extensions: HashMap<u16, Vec<u8>>,
}

/// TLS 扩展
/// 对应 Go 版本的 tls.TLSExtension
#[derive(Debug, Clone)]
pub enum Extension {
    /// GREASE 扩展（对应 &tls.UtlsGREASEExtension{}）
    GREASE,
    /// Server Name Indication（对应 &tls.SNIExtension{}）
    ServerName(Vec<String>),
    /// Status Request（对应 &tls.StatusRequestExtension{}）
    StatusRequest,
    /// Supported Curves（对应 &tls.SupportedCurvesExtension{}）
    SupportedCurves(Vec<u16>),
    /// Supported Points（对应 &tls.SupportedPointsExtension{}）
    SupportedPoints(Vec<u8>),
    /// Signature Algorithms（对应 &tls.SignatureAlgorithmsExtension{}）
    SignatureAlgorithms(Vec<u16>),
    /// ALPN（对应 &tls.ALPNExtension{}）
    ALPN(Vec<String>),
    /// Extended Master Secret（对应 &tls.ExtendedMasterSecretExtension{}）
    ExtendedMasterSecret,
    /// Session Ticket（对应 &tls.SessionTicketExtension{}）
    SessionTicket,
    /// Supported Versions（对应 &tls.SupportedVersionsExtension{}）
    SupportedVersions(Vec<u16>),
    /// PSK Key Exchange Modes（对应 &tls.PSKKeyExchangeModesExtension{}）
    PSKKeyExchangeModes(Vec<u8>),
    /// Key Share（对应 &tls.KeyShareExtension{}）
    KeyShare(Vec<KeyShareEntry>),
    /// SCT（对应 &tls.SCTExtension{}）
    SCT,
    /// Renegotiation Info（对应 &tls.RenegotiationInfoExtension{}）
    RenegotiationInfo(u8),
    /// Application Settings New（对应 &tls.ApplicationSettingsExtensionNew{}）
    ApplicationSettingsNew(Vec<String>),
    /// Compress Certificate（对应 &tls.UtlsCompressCertExtension{}）
    CompressCertificate(Vec<u16>),
    /// GREASE ECH（对应 tls.BoringGREASEECH()）
    GREASEECH,
    /// Pre-Shared Key（对应 &tls.UtlsPreSharedKeyExtension{}）
    PreSharedKey,
    /// 自定义扩展
    Custom { id: u16, data: Vec<u8> },
}

/// Key Share Entry（用于 TLS 1.3）
#[derive(Debug, Clone)]
pub struct KeyShareEntry {
    pub group: u16,
    pub data: Vec<u8>,
}

impl ClientHelloSpec {
    /// 创建新的 ClientHelloSpec
    pub fn new() -> Self {
        Self {
            tls_versions: vec![0x0303, 0x0304], // TLS 1.2, TLS 1.3
            cipher_suites: Vec::new(),
            compression_methods: vec![0], // 无压缩
            extensions: Vec::new(),
            elliptic_curves: Vec::new(),
            elliptic_curve_point_formats: vec![0], // 未压缩
            alpn_protocols: Vec::new(),
            signature_algorithms: Vec::new(),
            signature_algorithms_cert: Vec::new(),
            supported_groups: Vec::new(),
            supported_versions: vec![0x0304], // TLS 1.3
            psk_key_exchange_modes: Vec::new(),
            custom_extensions: HashMap::new(),
        }
    }
}

impl ClientHelloSpec {

    /// 创建 Chrome 103 指纹的 ClientHelloSpec
    /// 对应 Go 版本的 Chrome_103 SpecFactory
    pub fn chrome_103() -> Self {
        let mut spec = Self::new();
        
        // Chrome 103 的密码套件（使用 dicttls 常量）
        spec.cipher_suites = vec![
            GREASE_CS,
            cs::TLS_AES_128_GCM_SHA256,
            cs::TLS_AES_256_GCM_SHA384,
            cs::TLS_CHACHA20_POLY1305_SHA256,
            cs::TLS_ECDHE_ECDSA_WITH_AES_128_GCM_SHA256,
            cs::TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256,
            cs::TLS_ECDHE_ECDSA_WITH_AES_256_GCM_SHA384,
            cs::TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384,
            cs::TLS_ECDHE_ECDSA_WITH_CHACHA20_POLY1305_SHA256,
            cs::TLS_ECDHE_RSA_WITH_CHACHA20_POLY1305_SHA256,
            cs::TLS_ECDHE_RSA_WITH_AES_128_CBC_SHA,
            cs::TLS_ECDHE_RSA_WITH_AES_256_CBC_SHA,
            cs::TLS_RSA_WITH_AES_128_GCM_SHA256,
            cs::TLS_RSA_WITH_AES_256_GCM_SHA384,
            cs::TLS_RSA_WITH_AES_128_CBC_SHA,
            cs::TLS_RSA_WITH_AES_256_CBC_SHA,
        ];

        // 压缩方法
        spec.compression_methods = vec![COMPRESSION_NONE];

        // 椭圆曲线（用于 SupportedCurves 扩展）
        spec.elliptic_curves = vec![
            GREASE_SG,
            X25519,
            CURVE_P256,
            CURVE_P384,
        ];

        // 签名算法（用于 SignatureAlgorithms 扩展）
        spec.signature_algorithms = vec![
            ECDSA_WITH_P256_AND_SHA256,
            PSS_WITH_SHA256,
            PKCS1_WITH_SHA256,
            ECDSA_WITH_P384_AND_SHA384,
            PSS_WITH_SHA384,
            PKCS1_WITH_SHA384,
            PSS_WITH_SHA512,
            PKCS1_WITH_SHA512,
        ];

        // ALPN 协议
        spec.alpn_protocols = vec![
            "h2".to_string(),
            "http/1.1".to_string(),
        ];

        // 支持的版本（Chrome 133）
        spec.supported_versions = vec![
            GREASE_SG, // 使用 supported_groups 的 GREASE
            VERSION_TLS13,
            VERSION_TLS12,
        ];

        // PSK 密钥交换模式
        spec.psk_key_exchange_modes = vec![PSK_MODE_DHE];

        // 扩展列表（注意：顺序很重要！）
        // Chrome 103 的扩展顺序（简化版本，实际需要更详细的实现）
        spec.extensions = vec![
            Extension::GREASE, // UtlsGREASEExtension
            Extension::SessionTicket,
            Extension::SignatureAlgorithms(spec.signature_algorithms.clone()),
            Extension::ApplicationSettingsNew(vec!["h2".to_string()]), // ApplicationSettingsExtensionNew
            Extension::KeyShare(vec![
                KeyShareEntry { group: GREASE_SG, data: vec![0] },
                KeyShareEntry { group: X25519, data: vec![] }, // 实际需要生成密钥
            ]),
            Extension::SCT, // SCTExtension
            Extension::SupportedPoints(vec![POINT_FORMAT_UNCOMPRESSED]),
            Extension::SupportedVersions(spec.supported_versions.clone()),
            Extension::StatusRequest,
            Extension::ALPN(spec.alpn_protocols.clone()),
            Extension::ServerName(vec![]), // SNIExtension (会在实际使用时填充)
            Extension::GREASEECH, // BoringGREASEECH()
            Extension::CompressCertificate(vec![CERT_COMPRESSION_BROTLI]),
            Extension::SupportedCurves(spec.elliptic_curves.clone()),
            Extension::PSKKeyExchangeModes(spec.psk_key_exchange_modes.clone()),
            Extension::ExtendedMasterSecret,
            Extension::RenegotiationInfo(RENEGOTIATE_ONCE_AS_CLIENT),
            Extension::GREASE, // UtlsGREASEExtension
        ];

        spec
    }

    /// 创建 Chrome 133 指纹的 ClientHelloSpec
    /// 对应 Go 版本的 Chrome_133 SpecFactory
    /// 注意：Chrome 133 的扩展顺序与 Chrome 103 不同！
    pub fn chrome_133() -> Self {
        let mut spec = Self::chrome_103();
        
        // Chrome 133 的密码套件（与 103 相同）
        spec.cipher_suites = vec![
            GREASE_CS,
            cs::TLS_AES_128_GCM_SHA256,
            cs::TLS_AES_256_GCM_SHA384,
            cs::TLS_CHACHA20_POLY1305_SHA256,
            cs::TLS_ECDHE_ECDSA_WITH_AES_128_GCM_SHA256,
            cs::TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256,
            cs::TLS_ECDHE_ECDSA_WITH_AES_256_GCM_SHA384,
            cs::TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384,
            cs::TLS_ECDHE_ECDSA_WITH_CHACHA20_POLY1305_SHA256,
            cs::TLS_ECDHE_RSA_WITH_CHACHA20_POLY1305_SHA256,
            cs::TLS_ECDHE_RSA_WITH_AES_128_CBC_SHA,
            cs::TLS_ECDHE_RSA_WITH_AES_256_CBC_SHA,
            cs::TLS_RSA_WITH_AES_128_GCM_SHA256,
            cs::TLS_RSA_WITH_AES_256_GCM_SHA384,
            cs::TLS_RSA_WITH_AES_128_CBC_SHA,
            cs::TLS_RSA_WITH_AES_256_CBC_SHA,
        ];

        // Chrome 133 的椭圆曲线（包含 X25519MLKEM768）
        spec.elliptic_curves = vec![
            GREASE_SG,
            X25519_MLKEM768,
            X25519,
            CURVE_P256,
            CURVE_P384,
        ];

        // Chrome 133 的 ALPN（包含 h3）
        spec.alpn_protocols = vec![
            "h3".to_string(),
            "h2".to_string(),
            "http/1.1".to_string(),
        ];

        // Chrome 133 的扩展顺序（与 Chrome 103 不同！）
        // 根据 Go 版本的实现，Chrome_133 的扩展顺序是：
        spec.extensions = vec![
            Extension::GREASE, // &tls.UtlsGREASEExtension{}
            Extension::SessionTicket, // &tls.SessionTicketExtension{}
            Extension::SignatureAlgorithms(spec.signature_algorithms.clone()), // &tls.SignatureAlgorithmsExtension{}
            Extension::ApplicationSettingsNew(vec!["h3".to_string(), "h2".to_string()]), // &tls.ApplicationSettingsExtensionNew{}
            Extension::KeyShare(vec![
                KeyShareEntry { group: GREASE_SG, data: vec![0] },
                KeyShareEntry { group: X25519_MLKEM768, data: vec![] },
                KeyShareEntry { group: X25519, data: vec![] },
            ]), // &tls.KeyShareExtension{}
            Extension::SCT, // &tls.SCTExtension{}
            Extension::SupportedPoints(vec![POINT_FORMAT_UNCOMPRESSED]), // &tls.SupportedPointsExtension{}
            Extension::SupportedVersions(spec.supported_versions.clone()), // &tls.SupportedVersionsExtension{}
            Extension::StatusRequest, // &tls.StatusRequestExtension{}
            Extension::ALPN(spec.alpn_protocols.clone()), // &tls.ALPNExtension{}
            Extension::ServerName(vec![]), // &tls.SNIExtension{}
            Extension::GREASEECH, // tls.BoringGREASEECH()
            Extension::CompressCertificate(vec![CERT_COMPRESSION_BROTLI]), // &tls.UtlsCompressCertExtension{}
            Extension::SupportedCurves(spec.elliptic_curves.clone()), // &tls.SupportedCurvesExtension{}
            Extension::PSKKeyExchangeModes(spec.psk_key_exchange_modes.clone()), // &tls.PSKKeyExchangeModesExtension{}
            Extension::ExtendedMasterSecret, // &tls.ExtendedMasterSecretExtension{}
            Extension::RenegotiationInfo(RENEGOTIATE_ONCE_AS_CLIENT), // &tls.RenegotiationInfoExtension{}
            Extension::GREASE, // &tls.UtlsGREASEExtension{}
        ];

        spec
    }

    /// 创建 Firefox 133 指纹的 ClientHelloSpec
    /// 对应 Go 版本的 Firefox_133 SpecFactory
    pub fn firefox_133() -> Self {
        let mut spec = Self::new();
        
        // Firefox 133 的密码套件（使用 dicttls 常量）
        spec.cipher_suites = vec![
            cs::TLS_AES_256_GCM_SHA384,
            cs::TLS_AES_128_GCM_SHA256,
            cs::TLS_CHACHA20_POLY1305_SHA256,
            cs::TLS_ECDHE_ECDSA_WITH_AES_256_GCM_SHA384,
            cs::TLS_ECDHE_ECDSA_WITH_AES_128_GCM_SHA256,
            cs::TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384,
            cs::TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256,
            cs::TLS_ECDHE_ECDSA_WITH_CHACHA20_POLY1305_SHA256,
            cs::TLS_ECDHE_RSA_WITH_CHACHA20_POLY1305_SHA256,
        ];

        // 压缩方法
        spec.compression_methods = vec![COMPRESSION_NONE];

        // Firefox 的椭圆曲线
        spec.elliptic_curves = vec![
            CURVE_P256, // secp256r1
            CURVE_P384, // secp384r1
            SECP521R1,
            X25519,
        ];

        // Firefox 的签名算法
        spec.signature_algorithms = vec![
            PSS_WITH_SHA256,
            PSS_WITH_SHA384,
            PSS_WITH_SHA512,
            PKCS1_WITH_SHA256,
            PKCS1_WITH_SHA384,
            PKCS1_WITH_SHA512,
            ECDSA_WITH_P256_AND_SHA256,
            ECDSA_WITH_P384_AND_SHA384,
            ss::ECDSA_WITH_P521_AND_SHA512,
        ];

        spec.alpn_protocols = vec![
            "h2".to_string(),
            "http/1.1".to_string(),
        ];

        spec.supported_versions = vec![VERSION_TLS13];
        spec.psk_key_exchange_modes = vec![PSK_MODE_DHE];

        // Firefox 133 的扩展（简化版本，实际需要查看 Go 版本的完整实现）
        spec.extensions = vec![
            Extension::SupportedCurves(spec.elliptic_curves.clone()),
            Extension::SupportedPoints(vec![POINT_FORMAT_UNCOMPRESSED]),
            Extension::SignatureAlgorithms(spec.signature_algorithms.clone()),
            Extension::ALPN(spec.alpn_protocols.clone()),
            Extension::ExtendedMasterSecret,
            Extension::SupportedVersions(spec.supported_versions.clone()),
        ];

        spec
    }

    /// 创建 Safari 16.0 指纹的 ClientHelloSpec
    /// 对应 Go 版本的 Safari_16_0 SpecFactory
    pub fn safari_16_0() -> Self {
        let mut spec = Self::new();
        
        // Safari 16.0 的密码套件（使用 dicttls 常量）
        spec.cipher_suites = vec![
            cs::TLS_AES_128_GCM_SHA256,
            cs::TLS_AES_256_GCM_SHA384,
            cs::TLS_CHACHA20_POLY1305_SHA256,
            cs::TLS_ECDHE_ECDSA_WITH_AES_128_GCM_SHA256,
            cs::TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256,
            cs::TLS_ECDHE_ECDSA_WITH_AES_256_GCM_SHA384,
            cs::TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384,
        ];

        // 压缩方法
        spec.compression_methods = vec![COMPRESSION_NONE];

        // Safari 的椭圆曲线
        spec.elliptic_curves = vec![
            CURVE_P256, // secp256r1
            CURVE_P384, // secp384r1
            X25519,
        ];

        // Safari 的签名算法
        spec.signature_algorithms = vec![
            PKCS1_WITH_SHA256,
            PKCS1_WITH_SHA384,
            PKCS1_WITH_SHA512,
            ECDSA_WITH_P256_AND_SHA256,
            ECDSA_WITH_P384_AND_SHA384,
        ];

        spec.alpn_protocols = vec![
            "h2".to_string(),
            "http/1.1".to_string(),
        ];

        spec.supported_versions = vec![VERSION_TLS13];

        // Safari 16.0 的扩展（简化版本）
        spec.extensions = vec![
            Extension::SupportedCurves(spec.elliptic_curves.clone()),
            Extension::SupportedPoints(vec![POINT_FORMAT_UNCOMPRESSED]),
            Extension::SignatureAlgorithms(spec.signature_algorithms.clone()),
            Extension::ALPN(spec.alpn_protocols.clone()),
            Extension::SupportedVersions(spec.supported_versions.clone()),
        ];

        spec
    }
}

/// Chrome 103 Spec Factory
/// 对应 Go 版本的 Chrome_103 SpecFactory
pub fn chrome_103_spec() -> Result<ClientHelloSpec, String> {
    Ok(ClientHelloSpec::chrome_103())
}

/// Chrome 133 Spec Factory
/// 对应 Go 版本的 Chrome_133 SpecFactory
pub fn chrome_133_spec() -> Result<ClientHelloSpec, String> {
    Ok(ClientHelloSpec::chrome_133())
}

/// Firefox 133 Spec Factory
/// 对应 Go 版本的 Firefox_133 SpecFactory
pub fn firefox_133_spec() -> Result<ClientHelloSpec, String> {
    Ok(ClientHelloSpec::firefox_133())
}

/// Safari 16.0 Spec Factory
/// 对应 Go 版本的 Safari_16_0 SpecFactory
pub fn safari_16_0_spec() -> Result<ClientHelloSpec, String> {
    Ok(ClientHelloSpec::safari_16_0())
}

impl Default for ClientHelloSpec {
    fn default() -> Self {
        Self::chrome_133()
    }
}
