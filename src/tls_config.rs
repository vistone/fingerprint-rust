//! TLS 配置模块
//!
//! 提供真实的 TLS Client Hello 配置，对应 Go 版本的 utls.ClientHelloID

use crate::dicttls::{
    cipher_suites::{self as cs, GREASE_PLACEHOLDER as GREASE_CS},
    signature_schemes::{
        self as ss,
        ECDSA_WITH_P256_AND_SHA256, ECDSA_WITH_P384_AND_SHA384,
        PSS_WITH_SHA256, PSS_WITH_SHA384, PSS_WITH_SHA512,
        PKCS1_WITH_SHA256, PKCS1_WITH_SHA384, PKCS1_WITH_SHA512,
    },
    supported_groups::{GREASE_PLACEHOLDER as GREASE_SG, CURVE_P256, CURVE_P384, SECP521R1, X25519, X25519_MLKEM768},
};
use crate::tls_extensions::{
    ALPNExtension, ApplicationSettingsExtensionNew, ExtendedMasterSecretExtension,
    KeyShare, KeyShareExtension, PSKKeyExchangeModesExtension,
    RenegotiationInfoExtension, SCTExtension, SNIExtension, SignatureAlgorithmsExtension,
    StatusRequestExtension, SupportedCurvesExtension, SupportedPointsExtension,
    SupportedVersionsExtension, TLSExtension, UtlsCompressCertExtension, UtlsGREASEExtension,
};

/// TLS 版本常量
pub const VERSION_TLS10: u16 = 0x0301;
pub const VERSION_TLS11: u16 = 0x0302;
pub const VERSION_TLS12: u16 = 0x0303;
pub const VERSION_TLS13: u16 = 0x0304;

/// 压缩方法常量
pub const COMPRESSION_NONE: u8 = 0x00;

/// 点格式常量
pub const POINT_FORMAT_UNCOMPRESSED: u8 = 0x00;

/// PSK 模式常量
pub const PSK_MODE_DHE: u8 = 0x01;

/// 重新协商常量
pub const RENEGOTIATE_ONCE_AS_CLIENT: u8 = 1;

/// 证书压缩算法常量
pub const CERT_COMPRESSION_BROTLI: u16 = 0x0002;

/// 密码套件 ID
pub type CipherSuiteID = u16;

/// TLS Client Hello 配置
/// 对应 Go 版本的 tls.ClientHelloSpec
#[derive(Debug)]
pub struct ClientHelloSpec {
    /// 密码套件列表
    /// 对应 Go 版本的 CipherSuites []uint16
    pub cipher_suites: Vec<CipherSuiteID>,
    /// 压缩方法
    /// 对应 Go 版本的 CompressionMethods []uint8
    pub compression_methods: Vec<u8>,
    /// 扩展列表
    /// 对应 Go 版本的 Extensions []TLSExtension
    pub extensions: Vec<Box<dyn TLSExtension>>,
    /// TLS 版本最小值
    /// 对应 Go 版本的 TLSVersMin uint16
    pub tls_vers_min: u16,
    /// TLS 版本最大值
    /// 对应 Go 版本的 TLSVersMax uint16
    pub tls_vers_max: u16,
}

impl ClientHelloSpec {
    /// 创建新的 ClientHelloSpec
    /// 对应 Go 版本的 ClientHelloSpec{} 零值
    pub fn new() -> Self {
        Self {
            cipher_suites: Vec::new(),
            compression_methods: Vec::new(),
            extensions: Vec::new(),
            tls_vers_min: 0,
            tls_vers_max: 0,
        }
    }

    /// 创建 Chrome 133 指纹的 ClientHelloSpec
    /// 对应 Go 版本的 Chrome_133 SpecFactory
    pub fn chrome_133() -> Self {
        let mut spec = Self::new();
        
        // Chrome 133 的密码套件
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

        // Chrome 133 的扩展顺序（对应 Go 版本的 ShuffleChromeTLSExtensions）
        spec.extensions = vec![
            Box::new(UtlsGREASEExtension::new()),
            Box::new(SNIExtension::new(String::new())),
            Box::new(ExtendedMasterSecretExtension),
            Box::new(RenegotiationInfoExtension::new(RENEGOTIATE_ONCE_AS_CLIENT)),
            Box::new(SupportedCurvesExtension::new(vec![
                GREASE_SG,
                X25519_MLKEM768,
                X25519,
                CURVE_P256,
                CURVE_P384,
            ])),
            Box::new(SupportedPointsExtension::new(vec![POINT_FORMAT_UNCOMPRESSED])),
            Box::new(crate::tls_extensions::SessionTicketExtension),
            Box::new(ALPNExtension::new(vec![
                "h2".to_string(),
                "http/1.1".to_string(),
            ])),
            Box::new(StatusRequestExtension),
            Box::new(SignatureAlgorithmsExtension::new(vec![
                ECDSA_WITH_P256_AND_SHA256,
                PSS_WITH_SHA256,
                PKCS1_WITH_SHA256,
                ECDSA_WITH_P384_AND_SHA384,
                PSS_WITH_SHA384,
                PKCS1_WITH_SHA384,
                PSS_WITH_SHA512,
                PKCS1_WITH_SHA512,
            ])),
            Box::new(SCTExtension),
            Box::new(KeyShareExtension::new(vec![
                KeyShare {
                    group: GREASE_SG,
                    data: vec![0],
                },
                KeyShare {
                    group: X25519_MLKEM768,
                    data: vec![], // 实际需要生成密钥
                },
                KeyShare {
                    group: X25519,
                    data: vec![], // 实际需要生成密钥
                },
            ])),
            Box::new(PSKKeyExchangeModesExtension::new(vec![PSK_MODE_DHE])),
            Box::new(SupportedVersionsExtension::new(vec![
                GREASE_SG,
                VERSION_TLS13,
                VERSION_TLS12,
                VERSION_TLS11,
                VERSION_TLS10,
            ])),
            Box::new(UtlsCompressCertExtension::new(vec![CERT_COMPRESSION_BROTLI])),
            Box::new(ApplicationSettingsExtensionNew::new(vec!["h2".to_string()])),
            Box::new(UtlsGREASEExtension::new()),
        ];

        spec
    }

    /// 创建 Chrome 103 指纹的 ClientHelloSpec
    /// 对应 Go 版本的 Chrome_103 SpecFactory
    pub fn chrome_103() -> Self {
        let mut spec = Self::chrome_133();
        
        // Chrome 103 的椭圆曲线（不包含 X25519MLKEM768）
        spec.extensions = vec![
            Box::new(UtlsGREASEExtension::new()),
            Box::new(SNIExtension::new(String::new())),
            Box::new(ExtendedMasterSecretExtension),
            Box::new(RenegotiationInfoExtension::new(RENEGOTIATE_ONCE_AS_CLIENT)),
            Box::new(SupportedCurvesExtension::new(vec![
                GREASE_SG,
                X25519,
                CURVE_P256,
                CURVE_P384,
            ])),
            Box::new(SupportedPointsExtension::new(vec![POINT_FORMAT_UNCOMPRESSED])),
            Box::new(crate::tls_extensions::SessionTicketExtension),
            Box::new(ALPNExtension::new(vec![
                "h2".to_string(),
                "http/1.1".to_string(),
            ])),
            Box::new(StatusRequestExtension),
            Box::new(SignatureAlgorithmsExtension::new(vec![
                ECDSA_WITH_P256_AND_SHA256,
                PSS_WITH_SHA256,
                PKCS1_WITH_SHA256,
                ECDSA_WITH_P384_AND_SHA384,
                PSS_WITH_SHA384,
                PKCS1_WITH_SHA384,
                PSS_WITH_SHA512,
                PKCS1_WITH_SHA512,
            ])),
            Box::new(SCTExtension),
            Box::new(KeyShareExtension::new(vec![
                KeyShare {
                    group: GREASE_SG,
                    data: vec![0],
                },
                KeyShare {
                    group: X25519,
                    data: vec![], // 实际需要生成密钥
                },
            ])),
            Box::new(PSKKeyExchangeModesExtension::new(vec![PSK_MODE_DHE])),
            Box::new(SupportedVersionsExtension::new(vec![
                GREASE_SG,
                VERSION_TLS13,
                VERSION_TLS12,
                VERSION_TLS11,
                VERSION_TLS10,
            ])),
            Box::new(UtlsCompressCertExtension::new(vec![CERT_COMPRESSION_BROTLI])),
            Box::new(ApplicationSettingsExtensionNew::new(vec!["h2".to_string()])),
            Box::new(UtlsGREASEExtension::new()),
        ];

        spec
    }

    /// 创建 Firefox 133 指纹的 ClientHelloSpec
    /// 对应 Go 版本的 Firefox_133 SpecFactory
    pub fn firefox_133() -> Self {
        let mut spec = Self::new();
        
        // Firefox 133 的密码套件
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

        // Firefox 133 的扩展（简化版本，实际需要查看 Go 版本的完整实现）
        spec.extensions = vec![
            Box::new(SupportedCurvesExtension::new(vec![
                CURVE_P256,
                CURVE_P384,
                SECP521R1,
                X25519,
            ])),
            Box::new(SupportedPointsExtension::new(vec![POINT_FORMAT_UNCOMPRESSED])),
            Box::new(SignatureAlgorithmsExtension::new(vec![
                PSS_WITH_SHA256,
                PSS_WITH_SHA384,
                PSS_WITH_SHA512,
                PKCS1_WITH_SHA256,
                PKCS1_WITH_SHA384,
                PKCS1_WITH_SHA512,
                ECDSA_WITH_P256_AND_SHA256,
                ECDSA_WITH_P384_AND_SHA384,
                ss::ECDSA_WITH_P521_AND_SHA512,
            ])),
            Box::new(ALPNExtension::new(vec![
                "h2".to_string(),
                "http/1.1".to_string(),
            ])),
            Box::new(ExtendedMasterSecretExtension),
            Box::new(SupportedVersionsExtension::new(vec![VERSION_TLS13])),
        ];

        spec
    }

    /// 创建 Safari 16.0 指纹的 ClientHelloSpec
    /// 对应 Go 版本的 Safari_16_0 SpecFactory
    pub fn safari_16_0() -> Self {
        let mut spec = Self::new();
        
        // Safari 16.0 的密码套件
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

        // Safari 16.0 的扩展（简化版本）
        spec.extensions = vec![
            Box::new(SupportedCurvesExtension::new(vec![
                CURVE_P256,
                CURVE_P384,
                X25519,
            ])),
            Box::new(SupportedPointsExtension::new(vec![POINT_FORMAT_UNCOMPRESSED])),
            Box::new(SignatureAlgorithmsExtension::new(vec![
                PKCS1_WITH_SHA256,
                PKCS1_WITH_SHA384,
                PKCS1_WITH_SHA512,
                ECDSA_WITH_P256_AND_SHA256,
                ECDSA_WITH_P384_AND_SHA384,
            ])),
            Box::new(ALPNExtension::new(vec![
                "h2".to_string(),
                "http/1.1".to_string(),
            ])),
            Box::new(SupportedVersionsExtension::new(vec![VERSION_TLS13])),
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
