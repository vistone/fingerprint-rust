//! configurationexportmodule
//!
// ! will ClientHelloSpec export as JSON format, so thatprovideotherlanguage (如 Go uTLS)use.

#[cfg(feature = "export")]
use fingerprint_tls::tls_config::ClientHelloSpec;
#[cfg(feature = "export")]
use fingerprint_tls::tls_extensions::*;
#[cfg(feature = "export")]
use serde::{Deserialize, Serialize};

/// exportconfigurationstruct
#[cfg(feature = "export")]
#[derive(Serialize, Deserialize, Debug)]
pub struct ExportConfig {
    pub cipher_suites: Vec<u16>,
    pub compression_methods: Vec<u8>,
    pub extensions: Vec<ExportExtension>,
    pub tls_vers_min: u16,
    pub tls_vers_max: u16,
}

/// export KeyShare
#[cfg(feature = "export")]
#[derive(Serialize, Deserialize, Debug)]
pub struct ExportKeyShare {
    pub group: u16,
    pub data_hex: String,
}

/// export's extensionsenum
#[cfg(feature = "export")]
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type", content = "data")]
pub enum ExportExtension {
    SNI, // onlytype，nocountdata (由clientrun when decide)
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

/// will ClientHelloSpec convert to JSON string
#[cfg(feature = "export")]
pub fn export_config_json(spec: &ClientHelloSpec) -> Result<String, serde_json::Error> {
    let export = ExportConfig::from(spec);
    serde_json::to_string_pretty(&export)
}

#[cfg(feature = "export")]
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
                    // use as_any performtowarddowntransform
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
                        // SignatureScheme is u16
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
                                data_hex: {
                                    #[cfg(feature = "hex")]
                                    {
                                        hex::encode(&ks.data)
                                    }
                                    #[cfg(not(feature = "hex"))]
                                    {
                                        // Ifno hex feature, usehexadecimalformatmanualencoding
                                        ks.data
                                            .iter()
                                            .map(|b| format!("{:02x}", b))
                                            .collect::<String>()
                                    }
                                },
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
