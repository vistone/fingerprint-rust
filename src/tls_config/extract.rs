//! ClientHelloSpec 提取模块
//!
//! 从 ClientHelloSpec 中提取签名信息，用于指纹比较和匹配
//!
//! 参考：Huginn Net 的 Signature 提取实现

use crate::tls_config::signature::ClientHelloSignature;
use crate::tls_config::spec::ClientHelloSpec;

/// 从 ClientHelloSpec 中提取签名信息
/// 
/// # 参数
/// * `spec` - 要提取签名的 ClientHelloSpec
/// 
/// # 返回
/// * `ClientHelloSignature` - 提取的签名信息
/// 
/// # 注意
/// 由于扩展是 trait 对象，我们只能提取扩展 ID，无法直接访问扩展的内部数据。
/// 如果需要提取扩展的具体数据（如 SNI、ALPN），需要在构建 ClientHelloSpec 时保存这些信息。
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
    signature.version = spec.tls_vers_max; // 使用最大版本

    // 提取扩展 ID
    signature.extensions = spec.extensions.iter().map(|ext| ext.extension_id()).collect();

    // 注意：由于扩展是 trait 对象，我们无法直接提取扩展的内部数据
    // 如果需要 SNI、ALPN 等数据，需要在构建时保存，或者使用辅助函数

    signature
}
