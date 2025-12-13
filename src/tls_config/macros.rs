//! TLS 配置宏
//!
//! 提供宏来简化常见配置的构建，减少重复代码

/// 创建 Chrome 扩展列表的辅助宏
/// 类似 wreq-util 的宏设计
#[macro_export]
macro_rules! chrome_extensions {
    (
        curves: [$($curve:expr),*],
        key_shares: [$($key_share:expr),*],
        alpn: [$($alpn:expr),*],
        app_settings: [$($app_setting:expr),*],
    ) => {{
        use $crate::tls_config::builder::ClientHelloSpecBuilder;
        use $crate::tls_extensions::{
            ALPNExtension, ApplicationSettingsExtensionNew, ExtendedMasterSecretExtension,
            KeyShare, KeyShareExtension, PSKKeyExchangeModesExtension,
            RenegotiationInfoExtension, SCTExtension, SNIExtension, SignatureAlgorithmsExtension,
            StatusRequestExtension, SupportedCurvesExtension, SupportedPointsExtension,
            SupportedVersionsExtension, UtlsCompressCertExtension, UtlsGREASEExtension,
            UtlsPaddingExtension,
        };
        use $crate::tls_config::spec::{
            COMPRESSION_NONE, POINT_FORMAT_UNCOMPRESSED, PSK_MODE_DHE,
            RENEGOTIATE_ONCE_AS_CLIENT, CERT_COMPRESSION_BROTLI,
            VERSION_TLS12, VERSION_TLS13,
        };
        use $crate::dicttls::supported_groups::GREASE_PLACEHOLDER as GREASE_SG;

        vec![
            Box::new(UtlsGREASEExtension::new()),
            Box::new(SNIExtension::new(String::new())),
            Box::new(ExtendedMasterSecretExtension),
            Box::new(RenegotiationInfoExtension::new(RENEGOTIATE_ONCE_AS_CLIENT)),
            Box::new(SupportedCurvesExtension::new(vec![$($curve),*])),
            Box::new(SupportedPointsExtension::new(vec![POINT_FORMAT_UNCOMPRESSED])),
            Box::new($crate::tls_extensions::SessionTicketExtension),
            Box::new(ALPNExtension::new(vec![$($alpn.to_string()),*])),
            Box::new(StatusRequestExtension),
            Box::new(SignatureAlgorithmsExtension::new(
                ClientHelloSpecBuilder::chrome_signature_algorithms()
            )),
            Box::new(SCTExtension),
            Box::new(KeyShareExtension::new(vec![$($key_share),*])),
            Box::new(PSKKeyExchangeModesExtension::new(vec![PSK_MODE_DHE])),
            Box::new(SupportedVersionsExtension::new(vec![
                GREASE_SG,
                VERSION_TLS13,
                VERSION_TLS12,
            ])),
            Box::new(UtlsCompressCertExtension::new(vec![CERT_COMPRESSION_BROTLI])),
            Box::new(ApplicationSettingsExtensionNew::new(vec![$($app_setting.to_string()),*])),
            Box::new($crate::tls_extensions::GREASEEncryptedClientHelloExtension::new()),
            Box::new(UtlsGREASEExtension::new()),
            Box::new(UtlsPaddingExtension::new()),
        ]
    }};
}
