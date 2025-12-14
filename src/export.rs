//! 配置导出模块
//!
//! 将 ClientHelloSpec 导出为 JSON 格式，以便供其他语言（如 Go uTLS）使用。

use crate::tls_config::ClientHelloSpec;
use crate::tls_extensions::*;
// use crate::dicttls::extensions::*; // Unused
use serde::{Deserialize, Serialize};

/// 导出的配置结构体
#[derive(Serialize, Deserialize, Debug)]
pub struct ExportConfig {
    pub cipher_suites: Vec<u16>,
    pub compression_methods: Vec<u8>,
    pub extensions: Vec<ExportExtension>,
    pub tls_vers_min: u16,
    pub tls_vers_max: u16,
}

/// 导出的 KeyShare
#[derive(Serialize, Deserialize, Debug)]
pub struct ExportKeyShare {
    pub group: u16,
    pub data_hex: String,
}

/// 导出的扩展枚举
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type", content = "data")]
pub enum ExportExtension {
    SNI, // 只有类型，没有数据（由客户端运行时决定）
    StatusRequest,
    SupportedCurves(Vec<u16>),
    SupportedPoints(Vec<u8>),
    SignatureAlgorithms(Vec<u16>),
    ALPN(Vec<String>),
    ExtendedMasterSecret,
    SessionTicket,
    SupportedVersions(Vec<u16>),
    PSKKeyExchangeModes(Vec<u8>),
    KeyShare(Vec<ExportKeyShare>),
    SCT,
    RenegotiationInfo(u8),
    ApplicationSettings(Vec<String>),
    CompressCertificate(Vec<u16>),
    PreSharedKey,
    GREASE(u16),
    Padding { padding_len: usize, will_pad: bool },
    ECH(u16),
    Unknown(u16),
}

/// 将 ClientHelloSpec 转换为 JSON 字符串
pub fn export_config_json(spec: &ClientHelloSpec) -> Result<String, serde_json::Error> {
    let export = ExportConfig::from(spec);
    serde_json::to_string_pretty(&export)
}

impl From<&ClientHelloSpec> for ExportConfig {
    fn from(spec: &ClientHelloSpec) -> Self {
        Self {
            cipher_suites: spec.cipher_suites.clone(),
            compression_methods: spec.compression_methods.clone(),
            tls_vers_min: spec.tls_vers_min,
            tls_vers_max: spec.tls_vers_max,
            extensions: spec
                .extensions
                .iter()
                .map(|ext| {
                    // 使用 as_any 进行向下转型
                    let any_ext = ext.as_any();

                    if let Some(e) = any_ext.downcast_ref::<UtlsGREASEExtension>() {
                        return ExportExtension::GREASE(e.value);
                    }

                    if let Some(_e) = any_ext.downcast_ref::<SNIExtension>() {
                        return ExportExtension::SNI;
                    }

                    if let Some(_e) = any_ext.downcast_ref::<StatusRequestExtension>() {
                        return ExportExtension::StatusRequest;
                    }

                    if let Some(e) = any_ext.downcast_ref::<SupportedCurvesExtension>() {
                        return ExportExtension::SupportedCurves(e.curves.clone());
                    }

                    if let Some(e) = any_ext.downcast_ref::<SupportedPointsExtension>() {
                        return ExportExtension::SupportedPoints(e.supported_points.clone());
                    }

                    if let Some(e) = any_ext.downcast_ref::<SignatureAlgorithmsExtension>() {
                        // SignatureScheme 是 u16
                        return ExportExtension::SignatureAlgorithms(
                            e.supported_signature_algorithms.clone(),
                        );
                    }

                    if let Some(e) = any_ext.downcast_ref::<ALPNExtension>() {
                        return ExportExtension::ALPN(e.alpn_protocols.clone());
                    }

                    if let Some(_e) = any_ext.downcast_ref::<ExtendedMasterSecretExtension>() {
                        return ExportExtension::ExtendedMasterSecret;
                    }

                    if let Some(_e) = any_ext.downcast_ref::<SessionTicketExtension>() {
                        return ExportExtension::SessionTicket;
                    }

                    if let Some(e) = any_ext.downcast_ref::<SupportedVersionsExtension>() {
                        return ExportExtension::SupportedVersions(e.versions.clone());
                    }

                    if let Some(e) = any_ext.downcast_ref::<PSKKeyExchangeModesExtension>() {
                        return ExportExtension::PSKKeyExchangeModes(e.modes.clone());
                    }

                    if let Some(e) = any_ext.downcast_ref::<KeyShareExtension>() {
                        let shares = e
                            .key_shares
                            .iter()
                            .map(|ks| ExportKeyShare {
                                group: ks.group,
                                data_hex: hex::encode(&ks.data),
                            })
                            .collect();
                        return ExportExtension::KeyShare(shares);
                    }

                    if let Some(_e) = any_ext.downcast_ref::<SCTExtension>() {
                        return ExportExtension::SCT;
                    }

                    if let Some(e) = any_ext.downcast_ref::<RenegotiationInfoExtension>() {
                        return ExportExtension::RenegotiationInfo(e.renegotiation);
                    }

                    if let Some(e) = any_ext.downcast_ref::<ApplicationSettingsExtensionNew>() {
                        return ExportExtension::ApplicationSettings(e.supported_protocols.clone());
                    }

                    if let Some(e) = any_ext.downcast_ref::<UtlsCompressCertExtension>() {
                        return ExportExtension::CompressCertificate(e.algorithms.clone());
                    }

                    if let Some(_e) = any_ext.downcast_ref::<UtlsPreSharedKeyExtension>() {
                        return ExportExtension::PreSharedKey;
                    }

                    if let Some(e) = any_ext.downcast_ref::<GREASEEncryptedClientHelloExtension>() {
                        return ExportExtension::ECH(e.value);
                    }

                    if let Some(e) = any_ext.downcast_ref::<UtlsPaddingExtension>() {
                        return ExportExtension::Padding {
                            padding_len: e.padding_len,
                            will_pad: e.will_pad,
                        };
                    }

                    ExportExtension::Unknown(ext.extension_id())
                })
                .collect(),
        }
    }
}
