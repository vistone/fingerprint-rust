//! ClientHelloSpec 提取模块
//!
//! 从 ClientHelloSpec 中提取签名信息，用于指纹比较和匹配
//!
//! 参考：Huginn Net 的 Signature 提取实现

use crate::tls_config::signature::ClientHelloSignature;
use crate::tls_config::spec::ClientHelloSpec;
use crate::tls_config::version::TlsVersion;

/// 从 ClientHelloSpec 中提取签名信息
/// 
/// # 参数
/// * `spec` - 要提取签名的 ClientHelloSpec
/// 
/// # 返回
/// * `ClientHelloSignature` - 提取的签名信息
/// 
/// # 注意
/// 如果 spec 包含 metadata，会从中提取 SNI、ALPN 等信息。
/// 否则只能提取扩展 ID。
/// 
/// # 示例
/// ```
/// use fingerprint::{ClientHelloSpec, extract_signature};
/// let spec = ClientHelloSpec::chrome_133();
/// let signature = extract_signature(&spec);
/// ```
pub fn extract_signature(spec: &ClientHelloSpec) -> ClientHelloSignature {
    let mut signature = ClientHelloSignature::new();

    // 提取密码套件
    signature.cipher_suites = spec.cipher_suites.clone();

    // 提取 TLS 版本
    signature.version = TlsVersion::from_u16(spec.tls_vers_max); // 使用最大版本

    // 提取扩展 ID
    signature.extensions = spec.extensions.iter().map(|ext| ext.extension_id()).collect();

    // 从元数据中提取扩展的具体数据
    if let Some(ref metadata) = spec.metadata {
        // 提取 SNI
        if let Some(sni) = metadata.get_sni() {
            signature.sni = Some(sni.clone());
        }

        // 提取 ALPN（取第一个）
        if let Some(alpn) = metadata.get_first_alpn() {
            signature.alpn = Some(alpn);
        }

        // 提取椭圆曲线
        if let Some(ext_meta) = metadata.extension_metadata.get(
            &crate::dicttls::extensions::EXT_TYPE_SUPPORTED_GROUPS
        ) {
            if let Some(curves) = &ext_meta.elliptic_curves {
                signature.elliptic_curves = curves.clone();
            }
        }

        // 提取椭圆曲线点格式
        if let Some(ext_meta) = metadata.extension_metadata.get(
            &crate::dicttls::extensions::EXT_TYPE_EC_POINT_FORMATS
        ) {
            if let Some(formats) = &ext_meta.elliptic_curve_point_formats {
                signature.elliptic_curve_point_formats = formats.clone();
            }
        }

        // 提取签名算法
        if let Some(ext_meta) = metadata.extension_metadata.get(
            &crate::dicttls::extensions::EXT_TYPE_SIGNATURE_ALGORITHMS
        ) {
            if let Some(algs) = &ext_meta.signature_algorithms {
                signature.signature_algorithms = algs.clone();
            }
        }
    }

    signature
}
