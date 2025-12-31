//! 指纹核心抽象
//!
//! 定义统一的指纹抽象，支持 TLS、HTTP、TCP 等多种指纹类型。

use crate::metadata::FingerprintMetadata;

/// 指纹类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FingerprintType {
    /// TLS 指纹
    Tls,
    /// HTTP 指纹
    Http,
    /// TCP 指纹
    Tcp,
}

impl FingerprintType {
    /// 转换为字符串
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Tls => "tls",
            Self::Http => "http",
            Self::Tcp => "tcp",
        }
    }
}

impl std::fmt::Display for FingerprintType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// 指纹抽象 trait
///
/// 所有指纹类型（TLS、HTTP、TCP）都应该实现这个 trait
pub trait Fingerprint: Send + Sync {
    /// 获取指纹类型
    fn fingerprint_type(&self) -> FingerprintType;

    /// 获取指纹的唯一标识符（通常是哈希值）
    fn id(&self) -> String;

    /// 获取指纹的元数据
    fn metadata(&self) -> &FingerprintMetadata;

    /// 获取指纹的元数据（可变引用）
    fn metadata_mut(&mut self) -> &mut FingerprintMetadata;

    /// 计算指纹的哈希值（用于快速比较）
    fn hash(&self) -> u64;

    /// 比较两个指纹是否相似
    fn similar_to(&self, other: &dyn Fingerprint) -> bool;

    /// 获取指纹的字符串表示（用于调试和日志）
    fn to_string(&self) -> String;
}

/// 指纹比较结果
#[derive(Debug, Clone, PartialEq)]
pub struct FingerprintComparison {
    /// 相似度分数 (0.0 - 1.0)
    pub similarity: f64,

    /// 是否匹配
    pub matched: bool,

    /// 匹配的字段
    pub matched_fields: Vec<String>,

    /// 不匹配的字段
    pub unmatched_fields: Vec<String>,
}

impl FingerprintComparison {
    /// 创建新的比较结果
    pub fn new(similarity: f64, matched: bool) -> Self {
        Self {
            similarity,
            matched,
            matched_fields: Vec::new(),
            unmatched_fields: Vec::new(),
        }
    }

    /// 创建完全匹配的结果
    pub fn perfect_match() -> Self {
        Self {
            similarity: 1.0,
            matched: true,
            matched_fields: Vec::new(),
            unmatched_fields: Vec::new(),
        }
    }

    /// 创建完全不匹配的结果
    pub fn no_match() -> Self {
        Self {
            similarity: 0.0,
            matched: false,
            matched_fields: Vec::new(),
            unmatched_fields: Vec::new(),
        }
    }
}

/// 指纹比较器
pub struct FingerprintComparator;

impl FingerprintComparator {
    /// 比较两个指纹
    pub fn compare(f1: &dyn Fingerprint, f2: &dyn Fingerprint) -> FingerprintComparison {
        // 类型必须相同
        if f1.fingerprint_type() != f2.fingerprint_type() {
            return FingerprintComparison::no_match();
        }

        // 使用 similar_to 方法进行比较
        if f1.similar_to(f2) {
            FingerprintComparison::perfect_match()
        } else {
            // 计算相似度（基于哈希值）
            let h1 = f1.hash();
            let h2 = f2.hash();

            // 简单的相似度计算（基于哈希值的汉明距离）
            let similarity = if h1 == h2 {
                1.0
            } else {
                // 计算哈希值的差异
                let diff = (h1 ^ h2).count_ones() as f64;
                let max_diff = 64.0; // u64 的最大位数
                1.0 - (diff / max_diff)
            };

            FingerprintComparison {
                similarity,
                matched: similarity > 0.8, // 相似度阈值
                matched_fields: Vec::new(),
                unmatched_fields: Vec::new(),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fingerprint_type() {
        assert_eq!(FingerprintType::Tls.as_str(), "tls");
        assert_eq!(FingerprintType::Http.as_str(), "http");
        assert_eq!(FingerprintType::Tcp.as_str(), "tcp");
    }

    #[test]
    fn test_fingerprint_comparison() {
        let perfect = FingerprintComparison::perfect_match();
        assert_eq!(perfect.similarity, 1.0);
        assert!(perfect.matched);

        let no_match = FingerprintComparison::no_match();
        assert_eq!(no_match.similarity, 0.0);
        assert!(!no_match.matched);
    }
}
