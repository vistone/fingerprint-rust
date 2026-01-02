//! ClientHelloSpec Extractmodule
//!
//!  from  ClientHelloSpec 中Extractsignatureinfo， for fingerprint比较 and match
//!
//! 参考：Huginn Net  Signature Extractimplement

use crate::tls_config::signature::ClientHelloSignature;
use crate::tls_config::spec::ClientHelloSpec;
use crate::tls_config::version::TlsVersion;

///  from  ClientHelloSpec 中Extractsignatureinfo
///
/// # Parameters
/// * `spec` - 要Extractsignature ClientHelloSpec
///
/// # Returns
/// * `ClientHelloSignature` - Extract的signatureinfo
///
/// # Notes
/// If spec including metadata, will from 中Extract SNI、ALPN 等info。
/// otherwise只能Extractextension ID。
///
/// # Examples
/// ```
/// use fingerprint_tls::tls_config::{ClientHelloSpec, extract_signature};
/// let spec = ClientHelloSpec::chrome_133();
/// let signature = extract_signature(&spec);
/// ```
pub fn extract_signature(spec: &ClientHelloSpec) -> ClientHelloSignature {
    let mut signature = ClientHelloSignature::new();

    // Extractcipher suite
    signature.cipher_suites = spec.cipher_suites.clone();

    // Extract TLS version
    signature.version = TlsVersion::from_u16(spec.tls_vers_max); // usemaximumversion

    // Extractextension ID
    signature.extensions = spec
        .extensions
        .iter()
        .map(|ext| ext.extension_id())
        .collect();

    //  from metadata中Extractextension的具体count据
    if let Some(ref metadata) = spec.metadata {
        // Extract SNI
        if let Some(sni) = metadata.get_sni() {
            signature.sni = Some(sni.clone());
        }

        // Extract ALPN（取first）
        if let Some(alpn) = metadata.get_first_alpn() {
            signature.alpn = Some(alpn);
        }

        // Extract椭圆曲线
        if let Some(ext_meta) = metadata
            .extension_metadata
            .get(&fingerprint_core::dicttls::extensions::EXT_TYPE_SUPPORTED_GROUPS)
        {
            if let Some(curves) = &ext_meta.elliptic_curves {
                signature.elliptic_curves = curves.clone();
            }
        }

        // Extract椭圆曲线点format
        if let Some(ext_meta) = metadata
            .extension_metadata
            .get(&fingerprint_core::dicttls::extensions::EXT_TYPE_EC_POINT_FORMATS)
        {
            if let Some(formats) = &ext_meta.elliptic_curve_point_formats {
                signature.elliptic_curve_point_formats = formats.clone();
            }
        }

        // Extractsignaturealgorithm
        if let Some(ext_meta) = metadata
            .extension_metadata
            .get(&fingerprint_core::dicttls::extensions::EXT_TYPE_SIGNATURE_ALGORITHMS)
        {
            if let Some(algs) = &ext_meta.signature_algorithms {
                signature.signature_algorithms = algs.clone();
            }
        }
    }

    signature
}
