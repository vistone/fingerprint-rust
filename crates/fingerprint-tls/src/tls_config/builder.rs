//! ClientHelloSpec Builder 模块
//!
//! 提供 Builder 模式来构建 ClientHelloSpec，使代码更清晰、类型安全

use crate::tls_config::spec::{
    ClientHelloSpec, CERT_COMPRESSION_BROTLI, POINT_FORMAT_UNCOMPRESSED, PSK_MODE_DHE,
    RENEGOTIATE_ONCE_AS_CLIENT, VERSION_TLS12, VERSION_TLS13,
};
use crate::tls_extensions::{
    ALPNExtension, ApplicationSettingsExtensionNew, ExtendedMasterSecretExtension, KeyShare,
    KeyShareExtension, PSKKeyExchangeModesExtension, RenegotiationInfoExtension, SCTExtension,
    SNIExtension, SignatureAlgorithmsExtension, StatusRequestExtension, SupportedCurvesExtension,
    SupportedPointsExtension, SupportedVersionsExtension, TLSExtension, UtlsCompressCertExtension,
    UtlsGREASEExtension, UtlsPaddingExtension,
};
use fingerprint_core::dicttls::{
    cipher_suites::{self as cs, GREASE_PLACEHOLDER as GREASE_CS},
    signature_schemes::{
        ECDSA_WITH_P256_AND_SHA256, ECDSA_WITH_P384_AND_SHA384, PKCS1_WITH_SHA256,
        PKCS1_WITH_SHA384, PKCS1_WITH_SHA512, PSS_WITH_SHA256, PSS_WITH_SHA384, PSS_WITH_SHA512,
    },
    supported_groups::{
        CURVE_P256, CURVE_P384, GREASE_PLACEHOLDER as GREASE_SG, X25519, X25519_MLKEM768,
    },
};

/// ClientHelloSpec Builder
/// 使用 Builder 模式构建 ClientHelloSpec，使代码更清晰
#[derive(Debug, Default)]
pub struct ClientHelloSpecBuilder {
    cipher_suites: Vec<u16>,
    compression_methods: Vec<u8>,
    extensions: Vec<Box<dyn TLSExtension>>,
    tls_vers_min: u16,
    tls_vers_max: u16,
}

impl ClientHelloSpecBuilder {
    /// 创建新的 Builder
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置密码套件
    pub fn cipher_suites(mut self, suites: Vec<u16>) -> Self {
        self.cipher_suites = suites;
        self
    }

    /// 添加密码套件
    pub fn add_cipher_suite(mut self, suite: u16) -> Self {
        self.cipher_suites.push(suite);
        self
    }

    /// 设置压缩方法
    pub fn compression_methods(mut self, methods: Vec<u8>) -> Self {
        self.compression_methods = methods;
        self
    }

    /// 添加扩展
    pub fn add_extension(mut self, ext: Box<dyn TLSExtension>) -> Self {
        self.extensions.push(ext);
        self
    }

    /// 设置扩展列表
    pub fn extensions(mut self, exts: Vec<Box<dyn TLSExtension>>) -> Self {
        self.extensions = exts;
        self
    }

    /// 设置 TLS 版本范围
    pub fn tls_versions(mut self, min: u16, max: u16) -> Self {
        self.tls_vers_min = min;
        self.tls_vers_max = max;
        self
    }

    /// 构建 ClientHelloSpec
    pub fn build(self) -> ClientHelloSpec {
        ClientHelloSpec {
            cipher_suites: self.cipher_suites,
            compression_methods: self.compression_methods,
            extensions: self.extensions,
            tls_vers_min: self.tls_vers_min,
            tls_vers_max: self.tls_vers_max,
            metadata: None,
        }
    }

    /// Chrome 136 的默认密码套件
    /// 在 136 版本中，Chrome 进一步优化了加密套件权重，完全优先考虑现代 AEAD 套件
    pub fn chrome_136_cipher_suites() -> Vec<u16> {
        vec![
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
            // 136 版本在大多数平台上已经几乎不再首选这些旧套件
            cs::TLS_ECDHE_RSA_WITH_AES_128_CBC_SHA,
            cs::TLS_ECDHE_RSA_WITH_AES_256_CBC_SHA,
            cs::TLS_RSA_WITH_AES_128_GCM_SHA256,
            cs::TLS_RSA_WITH_AES_256_GCM_SHA384,
            cs::TLS_RSA_WITH_AES_128_CBC_SHA,
            cs::TLS_RSA_WITH_AES_256_CBC_SHA,
        ]
    }

    /// Chrome 133 的默认密码套件
    pub fn chrome_cipher_suites() -> Vec<u16> {
        vec![
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
        ]
    }

    /// Chrome 的默认签名算法
    /// 返回静态引用，避免不必要的内存分配
    pub fn chrome_signature_algorithms() -> &'static [u16] {
        &[
            ECDSA_WITH_P256_AND_SHA256,
            PSS_WITH_SHA256,
            PKCS1_WITH_SHA256,
            ECDSA_WITH_P384_AND_SHA384,
            PSS_WITH_SHA384,
            PKCS1_WITH_SHA384,
            PSS_WITH_SHA512,
            PKCS1_WITH_SHA512,
        ]
    }

    /// Chrome 的默认 ALPN 协议
    pub fn chrome_alpn_protocols() -> &'static [&'static str] {
        &["h2", "http/1.1"]
    }

    /// 构建 Chrome 133 的扩展列表
    /// 返回扩展列表和元数据
    pub fn chrome_133_extensions() -> (
        Vec<Box<dyn TLSExtension>>,
        crate::tls_config::metadata::SpecMetadata,
    ) {
        let mut metadata = crate::tls_config::metadata::SpecMetadata::new();

        // 设置 ALPN
        metadata.set_alpn(vec!["h2".to_string(), "http/1.1".to_string()]);

        // 设置椭圆曲线
        metadata.set_elliptic_curves(vec![
            GREASE_SG,
            X25519_MLKEM768,
            X25519,
            CURVE_P256,
            CURVE_P384,
        ]);

        // 设置椭圆曲线点格式
        metadata.set_elliptic_curve_point_formats(vec![POINT_FORMAT_UNCOMPRESSED]);

        // 设置签名算法
        metadata.set_signature_algorithms(Self::chrome_signature_algorithms().to_vec());

        // 设置支持的版本
        metadata.set_supported_versions(vec![GREASE_SG, VERSION_TLS13, VERSION_TLS12]);

        let extensions: Vec<Box<dyn TLSExtension>> = vec![
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
            Box::new(SupportedPointsExtension::new(vec![
                POINT_FORMAT_UNCOMPRESSED,
            ])),
            Box::new(crate::tls_extensions::SessionTicketExtension),
            Box::new(ALPNExtension::new(
                Self::chrome_alpn_protocols()
                    .iter()
                    .map(|s| s.to_string())
                    .collect(),
            )),
            Box::new(StatusRequestExtension),
            Box::new(SignatureAlgorithmsExtension::new(
                Self::chrome_signature_algorithms().to_vec(),
            )),
            Box::new(SCTExtension),
            Box::new(KeyShareExtension::new(vec![
                KeyShare {
                    group: GREASE_SG,
                    data: vec![0],
                },
                KeyShare {
                    group: X25519_MLKEM768,
                    data: vec![],
                },
                KeyShare {
                    group: X25519,
                    data: vec![],
                },
            ])),
            Box::new(PSKKeyExchangeModesExtension::new(vec![PSK_MODE_DHE])),
            Box::new(SupportedVersionsExtension::new(vec![
                GREASE_SG,
                VERSION_TLS13,
                VERSION_TLS12,
            ])),
            Box::new(UtlsCompressCertExtension::new(vec![
                CERT_COMPRESSION_BROTLI,
            ])),
            Box::new(ApplicationSettingsExtensionNew::new(vec!["h2".to_string()])),
            Box::new(crate::tls_extensions::GREASEEncryptedClientHelloExtension::new()),
            Box::new(UtlsGREASEExtension::new()),
            Box::new(UtlsPaddingExtension::new()),
        ];

        (extensions, metadata)
    }

    /// 构建 Chrome 136 的扩展列表
    /// 返回扩展列表和元数据
    pub fn chrome_136_extensions() -> (
        Vec<Box<dyn TLSExtension>>,
        crate::tls_config::metadata::SpecMetadata,
    ) {
        let (mut extensions, mut metadata) = Self::chrome_133_extensions();

        // 针对 136 的微调：确保 ALPN 包含 h3 并置于首位（Chrome 136 强化了对 h3 的支持）
        let alpn_protocols = vec!["h3".to_string(), "h2".to_string(), "http/1.1".to_string()];
        metadata.set_alpn(alpn_protocols.clone());

        // 调整扩展列表中的 ALPN
        for ext in extensions.iter_mut() {
            if ext.extension_id() == fingerprint_core::dicttls::extensions::EXT_TYPE_APPLICATION_LAYER_PROTOCOL_NEGOTIATION {
                *ext = Box::new(ALPNExtension::new(alpn_protocols.clone()));
            }
        }

        (extensions, metadata)
    }

    /// 构建 Chrome 103 的扩展列表（不包含 X25519MLKEM768）
    /// 返回扩展列表和元数据
    pub fn chrome_103_extensions() -> (
        Vec<Box<dyn TLSExtension>>,
        crate::tls_config::metadata::SpecMetadata,
    ) {
        let mut metadata = crate::tls_config::metadata::SpecMetadata::new();

        // 设置 ALPN
        metadata.set_alpn(vec!["h2".to_string(), "http/1.1".to_string()]);

        // 设置椭圆曲线（不包含 X25519MLKEM768）
        metadata.set_elliptic_curves(vec![GREASE_SG, X25519, CURVE_P256, CURVE_P384]);

        // 设置椭圆曲线点格式
        metadata.set_elliptic_curve_point_formats(vec![POINT_FORMAT_UNCOMPRESSED]);

        // 设置签名算法
        metadata.set_signature_algorithms(Self::chrome_signature_algorithms().to_vec());

        // 设置支持的版本
        metadata.set_supported_versions(vec![GREASE_SG, VERSION_TLS13, VERSION_TLS12]);

        let extensions: Vec<Box<dyn TLSExtension>> = vec![
            Box::new(UtlsGREASEExtension::new()),
            Box::new(SNIExtension::new(String::new())),
            Box::new(ExtendedMasterSecretExtension),
            Box::new(RenegotiationInfoExtension::new(RENEGOTIATE_ONCE_AS_CLIENT)),
            Box::new(SupportedCurvesExtension::new(vec![
                GREASE_SG, X25519, CURVE_P256, CURVE_P384,
            ])),
            Box::new(SupportedPointsExtension::new(vec![
                POINT_FORMAT_UNCOMPRESSED,
            ])),
            Box::new(crate::tls_extensions::SessionTicketExtension),
            Box::new(ALPNExtension::new(
                Self::chrome_alpn_protocols()
                    .iter()
                    .map(|s| s.to_string())
                    .collect(),
            )),
            Box::new(StatusRequestExtension),
            Box::new(SignatureAlgorithmsExtension::new(
                Self::chrome_signature_algorithms().to_vec(),
            )),
            Box::new(SCTExtension),
            Box::new(KeyShareExtension::new(vec![
                KeyShare {
                    group: GREASE_SG,
                    data: vec![0],
                },
                KeyShare {
                    group: X25519,
                    data: vec![],
                },
            ])),
            Box::new(PSKKeyExchangeModesExtension::new(vec![PSK_MODE_DHE])),
            Box::new(SupportedVersionsExtension::new(vec![
                GREASE_SG,
                VERSION_TLS13,
                VERSION_TLS12,
            ])),
            Box::new(UtlsCompressCertExtension::new(vec![
                CERT_COMPRESSION_BROTLI,
            ])),
            Box::new(ApplicationSettingsExtensionNew::new(vec!["h2".to_string()])),
            Box::new(UtlsGREASEExtension::new()),
            Box::new(UtlsPaddingExtension::new()),
        ];

        (extensions, metadata)
    }
}
