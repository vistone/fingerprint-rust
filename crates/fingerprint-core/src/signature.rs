//! TLS ClientHello Signature 模块
//!
//! 提供 TLS ClientHello 的签名提取和比较功能
//! 参考：Huginn Net 的 Signature 结构设计

use crate::dicttls::supported_groups::CurveID;
use crate::fingerprint::{Fingerprint, FingerprintType};
use crate::grease::{filter_grease_values, is_grease_value};
use crate::metadata::FingerprintMetadata;
use crate::version::TlsVersion;
use sha2::{Digest, Sha256};

/// TLS ClientHello 签名
/// 包含从 ClientHello 消息中提取的所有关键 information
#[derive(Debug, Clone, PartialEq)]
pub struct ClientHelloSignature {
    /// 指纹 ID（基于 JA4 hash 或签名特征的哈希）
    pub id: String,

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

    /// 元数据
    pub metadata: FingerprintMetadata,
}

impl ClientHelloSignature {
    /// 创建新的签名
    pub fn new() -> Self {
        let mut sig = Self {
            id: String::new(),
            version: TlsVersion::V1_2, // 默认 TLS 1.2
            cipher_suites: Vec::new(),
            extensions: Vec::new(),
            elliptic_curves: Vec::new(),
            elliptic_curve_point_formats: Vec::new(),
            signature_algorithms: Vec::new(),
            sni: None,
            alpn: None,
            metadata: FingerprintMetadata::new(),
        };
        sig.id = sig.calculate_id();
        sig
    }

    /// 计算指纹 ID（基于签名特征）
    pub fn calculate_id(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(self.version.to_u16().to_be_bytes());
        hasher.update(self.cipher_suites_without_grease().len().to_be_bytes());
        for &cs in &self.cipher_suites_without_grease() {
            hasher.update(cs.to_be_bytes());
        }
        hasher.update(self.extensions_without_grease().len().to_be_bytes());
        for &ext in &self.extensions_without_grease() {
            hasher.update(ext.to_be_bytes());
        }
        for &curve in &self.elliptic_curves {
            hasher.update(curve.to_be_bytes());
        }
        if let Some(ref sni) = self.sni {
            hasher.update(sni.as_bytes());
        }
        if let Some(ref alpn) = self.alpn {
            hasher.update(alpn.as_bytes());
        }
        format!("{:x}", hasher.finalize())
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
            || self
                .signature_algorithms
                .iter()
                .any(|&v| is_grease_value(v))
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
            && self.signature_algorithms_without_grease()
                == other.signature_algorithms_without_grease()
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

impl Fingerprint for ClientHelloSignature {
    fn fingerprint_type(&self) -> FingerprintType {
        FingerprintType::Tls
    }

    fn id(&self) -> String {
        self.id.clone()
    }

    fn metadata(&self) -> &FingerprintMetadata {
        &self.metadata
    }

    fn metadata_mut(&mut self) -> &mut FingerprintMetadata {
        &mut self.metadata
    }

    fn hash(&self) -> u64 {
        self.hash()
    }

    fn similar_to(&self, other: &dyn Fingerprint) -> bool {
        if other.fingerprint_type() != FingerprintType::Tls {
            return false;
        }

        // 尝试转换为 ClientHelloSignature
        // 由于 trait 的限制，我们只能比较哈希值
        // 实际使用中，应该通过类型转换来比较
        self.hash() == other.hash()
    }

    fn to_string(&self) -> String {
        format!(
            "ClientHelloSignature(id={}, version={:?}, cipher_suites={}, extensions={})",
            self.id,
            self.version,
            self.cipher_suites_without_grease().len(),
            self.extensions_without_grease().len()
        )
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
