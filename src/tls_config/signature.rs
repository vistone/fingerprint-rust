//! TLS ClientHello Signature 模块
//!
//! 提供 TLS ClientHello 的签名提取和比较功能
//! 参考：Huginn Net 的 Signature 结构设计

use crate::dicttls::supported_groups::CurveID;
use crate::tls_config::grease::{filter_grease_values, is_grease_value};
use crate::tls_config::version::TlsVersion;

/// TLS ClientHello 签名
/// 包含从 ClientHello 消息中提取的所有关键信息
#[derive(Debug, Clone, PartialEq)]
pub struct ClientHelloSignature {
    /// TLS 版本
    pub version: TlsVersion,
    /// 密码套件列表（包含 GREASE）
    pub cipher_suites: Vec<u16>,
    /// 扩展列表（包含 GREASE）
    pub extensions: Vec<u16>,
    /// 椭圆曲线列表
    pub elliptic_curves: Vec<CurveID>,
    /// 椭圆曲线点格式
    pub elliptic_curve_point_formats: Vec<u8>,
    /// 签名算法列表
    pub signature_algorithms: Vec<u16>,
    /// Server Name Indication
    pub sni: Option<String>,
    /// Application-Layer Protocol Negotiation
    pub alpn: Option<String>,
}

impl ClientHelloSignature {
    /// 创建新的签名
    pub fn new() -> Self {
        Self {
            version: TlsVersion::V1_2, // 默认 TLS 1.2
            cipher_suites: Vec::new(),
            extensions: Vec::new(),
            elliptic_curves: Vec::new(),
            elliptic_curve_point_formats: Vec::new(),
            signature_algorithms: Vec::new(),
            sni: None,
            alpn: None,
        }
    }

    /// 获取过滤 GREASE 后的密码套件
    pub fn cipher_suites_without_grease(&self) -> Vec<u16> {
        filter_grease_values(&self.cipher_suites)
    }

    /// 获取过滤 GREASE 后的扩展
    pub fn extensions_without_grease(&self) -> Vec<u16> {
        filter_grease_values(&self.extensions)
    }

    /// 获取过滤 GREASE 后的签名算法
    pub fn signature_algorithms_without_grease(&self) -> Vec<u16> {
        filter_grease_values(&self.signature_algorithms)
    }

    /// 检查是否包含 GREASE 值
    pub fn has_grease(&self) -> bool {
        self.cipher_suites.iter().any(|&v| is_grease_value(v))
            || self.extensions.iter().any(|&v| is_grease_value(v))
            || self.signature_algorithms.iter().any(|&v| is_grease_value(v))
    }

    /// 比较两个签名是否相似（忽略 GREASE 值）
    /// 
    /// # 参数
    /// * `other` - 要比较的另一个签名
    /// 
    /// # 返回
    /// * `true` 如果签名相似（忽略 GREASE 后相同），`false` 否则
    pub fn similar_to(&self, other: &Self) -> bool {
        self.version == other.version
            && self.cipher_suites_without_grease() == other.cipher_suites_without_grease()
            && self.extensions_without_grease() == other.extensions_without_grease()
            && self.signature_algorithms_without_grease() == other.signature_algorithms_without_grease()
            && self.elliptic_curves == other.elliptic_curves
            && self.elliptic_curve_point_formats == other.elliptic_curve_point_formats
            && self.sni == other.sni
            && self.alpn == other.alpn
    }

    /// 计算签名的哈希值（用于快速比较）
    /// 使用过滤 GREASE 后的值
    pub fn hash(&self) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        self.version.to_u16().hash(&mut hasher);
        self.cipher_suites_without_grease().hash(&mut hasher);
        self.extensions_without_grease().hash(&mut hasher);
        self.signature_algorithms_without_grease().hash(&mut hasher);
        self.elliptic_curves.hash(&mut hasher);
        self.elliptic_curve_point_formats.hash(&mut hasher);
        self.sni.hash(&mut hasher);
        self.alpn.hash(&mut hasher);
        hasher.finish()
    }
}

impl Default for ClientHelloSignature {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_similar_to() {
        let mut sig1 = ClientHelloSignature::new();
        sig1.version = TlsVersion::V1_2;
        sig1.cipher_suites = vec![0x0a0a, 0x0017, 0x1a1a]; // 包含 GREASE
        sig1.extensions = vec![0x0000, 0x0010];

        let mut sig2 = ClientHelloSignature::new();
        sig2.version = TlsVersion::V1_2;
        sig2.cipher_suites = vec![0x0017, 0x2a2a]; // 不同的 GREASE，但过滤后相同
        sig2.extensions = vec![0x0000, 0x0010];

        // 过滤 GREASE 后应该相同
        assert_eq!(
            sig1.cipher_suites_without_grease(),
            sig2.cipher_suites_without_grease()
        );
        assert!(sig1.similar_to(&sig2));
    }

    #[test]
    fn test_has_grease() {
        let mut sig = ClientHelloSignature::new();
        assert!(!sig.has_grease());

        sig.cipher_suites = vec![0x0a0a, 0x0017];
        assert!(sig.has_grease());

        sig.cipher_suites = vec![0x0017];
        sig.extensions = vec![0x1a1a];
        assert!(sig.has_grease());
    }
}
