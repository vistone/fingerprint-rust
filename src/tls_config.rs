//! TLS 配置模块
//!
//! 提供真实的 TLS Client Hello 配置，对应 Go 版本的 utls.ClientHelloID

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
#[derive(Debug, Clone)]
pub enum Extension {
    ServerName(Vec<String>),
    StatusRequest,
    SupportedCurves(Vec<u16>),
    SupportedPoints(Vec<u8>),
    SignatureAlgorithms(Vec<u16>),
    ALPN(Vec<String>),
    ExtendedMasterSecret,
    SessionTicket,
    SupportedVersions(Vec<u16>),
    Cookie(Vec<u8>),
    PSKKeyExchangeModes(Vec<u8>),
    KeyShare(Vec<KeyShareEntry>),
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

    /// 创建 Chrome 指纹的 ClientHelloSpec
    pub fn chrome_103() -> Self {
        let mut spec = Self::new();
        
        // Chrome 103 的密码套件
        spec.cipher_suites = vec![
            0x1301, // TLS_AES_128_GCM_SHA256
            0x1302, // TLS_AES_256_GCM_SHA384
            0x1303, // TLS_CHACHA20_POLY1305_SHA256
            0xc02f, // TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256
            0xc030, // TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384
            0xc02b, // TLS_ECDHE_ECDSA_WITH_AES_128_GCM_SHA256
            0xc02c, // TLS_ECDHE_ECDSA_WITH_AES_256_GCM_SHA384
            0xcca8, // TLS_ECDHE_RSA_WITH_CHACHA20_POLY1305_SHA256
            0xcca9, // TLS_ECDHE_ECDSA_WITH_CHACHA20_POLY1305_SHA256
            0xc013, // TLS_ECDHE_RSA_WITH_AES_128_CBC_SHA
            0xc014, // TLS_ECDHE_RSA_WITH_AES_256_CBC_SHA
            0xc009, // TLS_ECDHE_ECDSA_WITH_AES_128_CBC_SHA
            0xc00a, // TLS_ECDHE_ECDSA_WITH_AES_256_CBC_SHA
            0x009c, // TLS_RSA_WITH_AES_128_GCM_SHA256
            0x009d, // TLS_RSA_WITH_AES_256_GCM_SHA384
            0x002f, // TLS_RSA_WITH_AES_128_CBC_SHA
            0x0035, // TLS_RSA_WITH_AES_256_CBC_SHA
        ];

        // 椭圆曲线
        spec.elliptic_curves = vec![
            0x001d, // secp256r1
            0x0017, // secp384r1
            0x0018, // secp521r1
            0x0019, // x25519
            0x001a, // x448
        ];

        // 签名算法
        spec.signature_algorithms = vec![
            0x0804, // rsa_pss_rsae_sha256
            0x0805, // rsa_pss_rsae_sha384
            0x0806, // rsa_pss_rsae_sha512
            0x0401, // rsa_pkcs1_sha256
            0x0501, // rsa_pkcs1_sha384
            0x0601, // rsa_pkcs1_sha512
            0x0201, // ecdsa_secp256r1_sha256
            0x0202, // ecdsa_secp384r1_sha384
            0x0203, // ecdsa_secp521r1_sha512
            0x0807, // ed25519
            0x0808, // ed448
        ];

        // ALPN
        spec.alpn_protocols = vec![
            "h2".to_string(),
            "http/1.1".to_string(),
        ];

        // 扩展
        spec.extensions = vec![
            Extension::SupportedCurves(spec.elliptic_curves.clone()),
            Extension::SupportedPoints(vec![0, 1, 2]), // 未压缩, ansiX962_compressed_prime, ansiX962_compressed_char2
            Extension::SignatureAlgorithms(spec.signature_algorithms.clone()),
            Extension::ALPN(spec.alpn_protocols.clone()),
            Extension::ExtendedMasterSecret,
            Extension::SupportedVersions(vec![0x0304]), // TLS 1.3
            Extension::PSKKeyExchangeModes(vec![0, 1]), // psk_ke, psk_dhe_ke
        ];

        spec
    }

    /// 创建 Chrome 133 指纹的 ClientHelloSpec
    pub fn chrome_133() -> Self {
        // Chrome 133 与 Chrome 103 基本相同，但可能有一些细微差别
        Self::chrome_103()
    }

    /// 创建 Firefox 指纹的 ClientHelloSpec
    pub fn firefox_133() -> Self {
        let mut spec = Self::new();
        
        // Firefox 的密码套件（与 Chrome 略有不同）
        spec.cipher_suites = vec![
            0x1302, // TLS_AES_256_GCM_SHA384
            0x1301, // TLS_AES_128_GCM_SHA256
            0x1303, // TLS_CHACHA20_POLY1305_SHA256
            0xc02c, // TLS_ECDHE_ECDSA_WITH_AES_256_GCM_SHA384
            0xc02b, // TLS_ECDHE_ECDSA_WITH_AES_128_GCM_SHA256
            0xc030, // TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384
            0xc02f, // TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256
            0xcca9, // TLS_ECDHE_ECDSA_WITH_CHACHA20_POLY1305_SHA256
            0xcca8, // TLS_ECDHE_RSA_WITH_CHACHA20_POLY1305_SHA256
        ];

        // Firefox 的椭圆曲线
        spec.elliptic_curves = vec![
            0x001d, // secp256r1
            0x0017, // secp384r1
            0x0018, // secp521r1
            0x0019, // x25519
        ];

        // Firefox 的签名算法
        spec.signature_algorithms = vec![
            0x0804, // rsa_pss_rsae_sha256
            0x0805, // rsa_pss_rsae_sha384
            0x0806, // rsa_pss_rsae_sha512
            0x0401, // rsa_pkcs1_sha256
            0x0501, // rsa_pkcs1_sha384
            0x0601, // rsa_pkcs1_sha512
            0x0201, // ecdsa_secp256r1_sha256
            0x0202, // ecdsa_secp384r1_sha384
            0x0203, // ecdsa_secp521r1_sha512
        ];

        spec.alpn_protocols = vec![
            "h2".to_string(),
            "http/1.1".to_string(),
        ];

        spec.extensions = vec![
            Extension::SupportedCurves(spec.elliptic_curves.clone()),
            Extension::SupportedPoints(vec![0]),
            Extension::SignatureAlgorithms(spec.signature_algorithms.clone()),
            Extension::ALPN(spec.alpn_protocols.clone()),
            Extension::ExtendedMasterSecret,
            Extension::SupportedVersions(vec![0x0304]),
        ];

        spec
    }

    /// 创建 Safari 指纹的 ClientHelloSpec
    pub fn safari_16_0() -> Self {
        let mut spec = Self::new();
        
        // Safari 的密码套件
        spec.cipher_suites = vec![
            0x1301, // TLS_AES_128_GCM_SHA256
            0x1302, // TLS_AES_256_GCM_SHA384
            0x1303, // TLS_CHACHA20_POLY1305_SHA256
            0xc02b, // TLS_ECDHE_ECDSA_WITH_AES_128_GCM_SHA256
            0xc02f, // TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256
            0xc02c, // TLS_ECDHE_ECDSA_WITH_AES_256_GCM_SHA384
            0xc030, // TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384
        ];

        spec.elliptic_curves = vec![
            0x001d, // secp256r1
            0x0017, // secp384r1
            0x0019, // x25519
        ];

        spec.signature_algorithms = vec![
            0x0401, // rsa_pkcs1_sha256
            0x0501, // rsa_pkcs1_sha384
            0x0601, // rsa_pkcs1_sha512
            0x0201, // ecdsa_secp256r1_sha256
            0x0202, // ecdsa_secp384r1_sha384
        ];

        spec.alpn_protocols = vec![
            "h2".to_string(),
            "http/1.1".to_string(),
        ];

        spec.extensions = vec![
            Extension::SupportedCurves(spec.elliptic_curves.clone()),
            Extension::SupportedPoints(vec![0]),
            Extension::SignatureAlgorithms(spec.signature_algorithms.clone()),
            Extension::ALPN(spec.alpn_protocols.clone()),
            Extension::SupportedVersions(vec![0x0304]),
        ];

        spec
    }
}

impl Default for ClientHelloSpec {
    fn default() -> Self {
        Self::chrome_133()
    }
}
