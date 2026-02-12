//! ClientHelloSpec Builder module
//!
//! provide Builder patternfromBuild ClientHelloSpec, 使codeclearer, typesecurity

use crate::tls_config::spec::{
    ClientHelloSpec, CERT_COMPRESSION_BROTLI, POINT_FORMAT_UNCOMPRESSED, PSK_MODE_DHE,
    RENEGOTIATE_ONCE_AS_CLIENT, VERSION_TLS12, VERSION_TLS13,
};
use crate::tls_extensions::{
    ALPNExtension, ApplicationSettingsExtensionNew, EarlyDataExtension,
    EncryptedClientHelloExtension, ExtendedMasterSecretExtension,
    GREASEEncryptedClientHelloExtension, KeyShare, KeyShareExtension, PSKKeyExchangeModesExtension,
    PreSharedKeyExtension, RenegotiationInfoExtension, SCTExtension, SNIExtension,
    SignatureAlgorithmsExtension, StatusRequestExtension, SupportedCurvesExtension,
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
/// use Builder patternBuild ClientHelloSpec, 使codeclearer
#[derive(Debug, Default)]
pub struct ClientHelloSpecBuilder {
    cipher_suites: Vec<u16>,
    compression_methods: Vec<u8>,
    extensions: Vec<Box<dyn TLSExtension>>,
    tls_vers_min: u16,
    tls_vers_max: u16,
}

impl ClientHelloSpecBuilder {
    /// Create a new Builder
    pub fn new() -> Self {
        Self::default()
    }

    /// settingscipher suite
    pub fn cipher_suites(mut self, suites: Vec<u16>) -> Self {
        self.cipher_suites = suites;
        self
    }

    /// Addcipher suite
    pub fn add_cipher_suite(mut self, suite: u16) -> Self {
        self.cipher_suites.push(suite);
        self
    }

    /// settingscompressionmethod
    pub fn compression_methods(mut self, methods: Vec<u8>) -> Self {
        self.compression_methods = methods;
        self
    }

    /// Addextension
    pub fn add_extension(mut self, ext: Box<dyn TLSExtension>) -> Self {
        self.extensions.push(ext);
        self
    }

    /// settingsextensionlist
    pub fn extensions(mut self, exts: Vec<Box<dyn TLSExtension>>) -> Self {
        self.extensions = exts;
        self
    }

    /// settings TLS versionrange
    pub fn tls_versions(mut self, min: u16, max: u16) -> Self {
        self.tls_vers_min = min;
        self.tls_vers_max = max;
        self
    }

    /// Build ClientHelloSpec
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

    /// Chrome 136 defaultcipher suite
    /// in 136 version in , Chrome furtheroptimize了encryptionsuiteweight, completelypriorityconsidermodern AEAD suite
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
            // 136 version in 大multiplecountplatformupalreadyalmostno longer preferredtheseoldsuite
            cs::TLS_ECDHE_RSA_WITH_AES_128_CBC_SHA,
            cs::TLS_ECDHE_RSA_WITH_AES_256_CBC_SHA,
            cs::TLS_RSA_WITH_AES_128_GCM_SHA256,
            cs::TLS_RSA_WITH_AES_256_GCM_SHA384,
            cs::TLS_RSA_WITH_AES_128_CBC_SHA,
            cs::TLS_RSA_WITH_AES_256_CBC_SHA,
        ]
    }

    /// Chrome 133 defaultcipher suite
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

    /// Chrome defaultsignaturealgorithm
    /// returnstaticreference, avoidunnecessaryinsidesaveallocate
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

    /// Chrome default ALPN protocol
    pub fn chrome_alpn_protocols() -> &'static [&'static str] {
        &["h2", "http/1.1"]
    }

    /// Build Chrome 133 's extensionslist
    /// returnextensionlist and metadata
    pub fn chrome_133_extensions() -> (
        Vec<Box<dyn TLSExtension>>,
        crate::tls_config::metadata::SpecMetadata,
    ) {
        let mut metadata = crate::tls_config::metadata::SpecMetadata::new();

        // settings ALPN
        metadata.set_alpn(vec!["h2".to_string(), "http/1.1".to_string()]);

        // settingselliptic curve
        metadata.set_elliptic_curves(vec![
            GREASE_SG,
            X25519_MLKEM768,
            X25519,
            CURVE_P256,
            CURVE_P384,
        ]);

        // settingselliptic curvepointformat
        metadata.set_elliptic_curve_point_formats(vec![POINT_FORMAT_UNCOMPRESSED]);

        // settingssignaturealgorithm
        metadata.set_signature_algorithms(Self::chrome_signature_algorithms().to_vec());

        // settingssupportversion
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
            Box::new(EncryptedClientHelloExtension::outer()), // Real ECH (RFC 9180)
            Box::new(GREASEEncryptedClientHelloExtension::new()), // GREASE variant
            Box::new(UtlsGREASEExtension::new()),
            Box::new(UtlsPaddingExtension::new()),
        ];

        (extensions, metadata)
    }

    /// Build Chrome 136 's extensionslist
    /// returnextensionlist and metadata
    pub fn chrome_136_extensions() -> (
        Vec<Box<dyn TLSExtension>>,
        crate::tls_config::metadata::SpecMetadata,
    ) {
        let (mut extensions, mut metadata) = Self::chrome_133_extensions();

        // needlepair 136 fine-tune：ensure ALPN including h3 并put firstbit (Chrome 136 strongchangedpair h3 support)
        let alpn_protocols = vec!["h3".to_string(), "h2".to_string(), "http/1.1".to_string()];
        metadata.set_alpn(alpn_protocols.clone());

        // adjustextensionlist in ALPN
        for ext in extensions.iter_mut() {
            if ext.extension_id() == fingerprint_core::dicttls::extensions::EXT_TYPE_APPLICATION_LAYER_PROTOCOL_NEGOTIATION {
 *ext = Box::new(ALPNExtension::new(alpn_protocols.clone()));
 }
        }

        (extensions, metadata)
    }

    /// Build Chrome 103 's extensionslist (excluding X25519MLKEM768)
    /// returnextensionlist and metadata
    pub fn chrome_103_extensions() -> (
        Vec<Box<dyn TLSExtension>>,
        crate::tls_config::metadata::SpecMetadata,
    ) {
        let mut metadata = crate::tls_config::metadata::SpecMetadata::new();

        // settings ALPN
        metadata.set_alpn(vec!["h2".to_string(), "http/1.1".to_string()]);

        // settingselliptic curve (excluding X25519MLKEM768)
        metadata.set_elliptic_curves(vec![GREASE_SG, X25519, CURVE_P256, CURVE_P384]);

        // settingselliptic curvepointformat
        metadata.set_elliptic_curve_point_formats(vec![POINT_FORMAT_UNCOMPRESSED]);

        // settingssignaturealgorithm
        metadata.set_signature_algorithms(Self::chrome_signature_algorithms().to_vec());

        // settingssupportversion
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

    /// Build Chrome PSK (Pre-Shared Key) Session Resumption extensions
    /// For TLS 1.3 session resumption with PSK
    pub fn chrome_psk_extensions() -> (
        Vec<Box<dyn TLSExtension>>,
        crate::tls_config::metadata::SpecMetadata,
    ) {
        let (mut extensions, metadata) = Self::chrome_133_extensions();

        // Insert PSK and PSK Key Exchange Modes before the last GREASE extension
        // Find the insertion point (before the last UtlsGREASEExtension)
        let psk_ext = Box::new(PreSharedKeyExtension::for_session_resumption(
            vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
            vec![0x20; 32], // 32-byte binder (SHA-256)
        ));

        // Add PSK extension before padding
        if !extensions.is_empty() {
            extensions.insert(extensions.len() - 1, psk_ext);
        } else {
            extensions.push(psk_ext);
        }

        (extensions, metadata)
    }

    /// Build Chrome 0-RTT (Early Data) extensions
    /// For TLS 1.3 zero-roundtrip connections
    pub fn chrome_0rtt_extensions() -> (
        Vec<Box<dyn TLSExtension>>,
        crate::tls_config::metadata::SpecMetadata,
    ) {
        let (mut extensions, metadata) = Self::chrome_133_extensions();

        // Insert Early Data extension
        let early_data = Box::new(EarlyDataExtension::standard());

        // Add Early Data before PSK if PSK is present, otherwise before padding
        if !extensions.is_empty() {
            extensions.insert(extensions.len() - 1, early_data);
        } else {
            extensions.push(early_data);
        }

        (extensions, metadata)
    }

    /// Build Chrome PSK + 0-RTT combined extensions
    /// For simultaneous session resumption with early data
    pub fn chrome_psk_0rtt_extensions() -> (
        Vec<Box<dyn TLSExtension>>,
        crate::tls_config::metadata::SpecMetadata,
    ) {
        let (mut extensions, metadata) = Self::chrome_133_extensions();

        // Insert both Early Data and PSK
        let early_data = Box::new(EarlyDataExtension::standard());
        let psk_ext = Box::new(PreSharedKeyExtension::for_session_resumption(
            vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
            vec![0x20; 32],
        ));

        // Add both before padding (last 1-2 extensions)
        if !extensions.is_empty() {
            extensions.insert(extensions.len() - 1, psk_ext);
        }
        if !extensions.is_empty() {
            extensions.insert(extensions.len() - 1, early_data);
        }

        (extensions, metadata)
    }
}
