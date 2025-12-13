//! TLS 可观察性模块
//!
//! 提供 TLS ClientHello 的可观察性数据提取
//! 参考：Huginn Net Profiler 的 TlsClientObserved 设计

use crate::dicttls::supported_groups::CurveID;
use crate::tls_config::extract::extract_signature;
use crate::tls_config::signature::ClientHelloSignature;
use crate::tls_config::spec::ClientHelloSpec;

/// TLS ClientHello 可观察数据
/// 包含所有可以从 ClientHello 中观察到的信息
/// 参考：Huginn Net Profiler 的 TlsClientObserved
#[derive(Debug, Clone, PartialEq)]
pub struct TlsClientObserved {
    /// TLS 版本（字符串表示，如 "13", "12"）
    pub version: String,
    /// Server Name Indication
    pub sni: Option<String>,
    /// Application-Layer Protocol Negotiation
    pub alpn: Option<String>,
    /// 密码套件列表
    pub cipher_suites: Vec<u16>,
    /// 扩展列表
    pub extensions: Vec<u16>,
    /// 签名算法列表
    pub signature_algorithms: Vec<u16>,
    /// 椭圆曲线列表
    pub elliptic_curves: Vec<CurveID>,
}

impl TlsClientObserved {
    /// 从 ClientHelloSpec 创建可观察数据
    pub fn from_spec(spec: &ClientHelloSpec) -> Self {
        let signature = extract_signature(spec);
        Self::from_signature(&signature)
    }

    /// 从 ClientHelloSignature 创建可观察数据
    pub fn from_signature(signature: &ClientHelloSignature) -> Self {
        Self {
            version: format!("{}", signature.version),
            sni: signature.sni.clone(),
            alpn: signature.alpn.clone(),
            cipher_suites: signature.cipher_suites.clone(),
            extensions: signature.extensions.clone(),
            signature_algorithms: signature.signature_algorithms.clone(),
            elliptic_curves: signature.elliptic_curves.clone(),
        }
    }

    /// 获取密码套件数量
    pub fn cipher_suite_count(&self) -> usize {
        self.cipher_suites.len()
    }

    /// 获取扩展数量
    pub fn extension_count(&self) -> usize {
        self.extensions.len()
    }

    /// 获取签名算法数量
    pub fn signature_algorithm_count(&self) -> usize {
        self.signature_algorithms.len()
    }

    /// 检查是否包含特定扩展
    pub fn has_extension(&self, ext_id: u16) -> bool {
        self.extensions.contains(&ext_id)
    }

    /// 检查是否包含特定密码套件
    pub fn has_cipher_suite(&self, suite: u16) -> bool {
        self.cipher_suites.contains(&suite)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tls_config::version::TlsVersion;

    #[test]
    fn test_format_tls_version() {
        assert_eq!(format!("{}", TlsVersion::V1_3), "13");
        assert_eq!(format!("{}", TlsVersion::V1_2), "12");
        assert_eq!(format!("{}", TlsVersion::V1_0), "10");
    }

    #[test]
    fn test_from_spec() {
        let spec = ClientHelloSpec::chrome_133();
        let observed = TlsClientObserved::from_spec(&spec);
        assert!(!observed.cipher_suites.is_empty());
        assert!(!observed.extensions.is_empty());
    }

    #[test]
    fn test_has_extension() {
        let spec = ClientHelloSpec::chrome_133();
        let observed = TlsClientObserved::from_spec(&spec);
        // 检查是否包含 SNI 扩展（0x0000）
        let has_sni = observed.has_extension(0x0000);
        // Chrome 133 应该包含 SNI
        assert!(has_sni);
    }
}
