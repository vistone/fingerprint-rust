//! TLS 指纹统计模块
//!
//! 提供指纹统计和分析功能
//! 参考：Huginn Net Profiler 的统计功能

use crate::tls_config::signature::ClientHelloSignature;
use crate::tls_config::spec::ClientHelloSpec;
use std::collections::HashMap;

/// TLS 指纹统计信息
#[derive(Debug, Clone)]
pub struct FingerprintStats {
    /// 总指纹数量
    pub total_fingerprints: usize,
    /// 包含 GREASE 的指纹数量
    pub fingerprints_with_grease: usize,
    /// 包含 SNI 的指纹数量
    pub fingerprints_with_sni: usize,
    /// 包含 ALPN 的指纹数量
    pub fingerprints_with_alpn: usize,
    /// TLS 版本分布
    pub version_distribution: HashMap<String, usize>,
    /// 最常见的密码套件（前 10）
    pub top_cipher_suites: Vec<(u16, usize)>,
    /// 最常见的扩展（前 10）
    pub top_extensions: Vec<(u16, usize)>,
}

impl FingerprintStats {
    /// 从多个 ClientHelloSpec 计算统计信息
    pub fn from_specs(specs: &[ClientHelloSpec]) -> Self {
        let mut stats = Self {
            total_fingerprints: specs.len(),
            fingerprints_with_grease: 0,
            fingerprints_with_sni: 0,
            fingerprints_with_alpn: 0,
            version_distribution: HashMap::new(),
            top_cipher_suites: Vec::new(),
            top_extensions: Vec::new(),
        };

        let mut cipher_suite_counts: HashMap<u16, usize> = HashMap::new();
        let mut extension_counts: HashMap<u16, usize> = HashMap::new();

        for spec in specs {
            let signature = crate::tls_config::extract::extract_signature(spec);

            // 检查 GREASE
            if signature.has_grease() {
                stats.fingerprints_with_grease += 1;
            }

            // 检查 SNI
            if signature.sni.is_some() {
                stats.fingerprints_with_sni += 1;
            }

            // 检查 ALPN
            if signature.alpn.is_some() {
                stats.fingerprints_with_alpn += 1;
            }

            // TLS 版本分布
            let version_str = format!("{}", signature.version);
            *stats.version_distribution.entry(version_str).or_insert(0) += 1;

            // 统计密码套件
            for suite in &signature.cipher_suites {
                *cipher_suite_counts.entry(*suite).or_insert(0) += 1;
            }

            // 统计扩展
            for ext in &signature.extensions {
                *extension_counts.entry(*ext).or_insert(0) += 1;
            }
        }

        // 获取最常见的密码套件
        let mut cipher_vec: Vec<(u16, usize)> = cipher_suite_counts.into_iter().collect();
        cipher_vec.sort_by(|a, b| b.1.cmp(&a.1));
        stats.top_cipher_suites = cipher_vec.into_iter().take(10).collect();

        // 获取最常见的扩展
        let mut ext_vec: Vec<(u16, usize)> = extension_counts.into_iter().collect();
        ext_vec.sort_by(|a, b| b.1.cmp(&a.1));
        stats.top_extensions = ext_vec.into_iter().take(10).collect();

        stats
    }

    /// 从多个签名计算统计信息
    pub fn from_signatures(signatures: &[ClientHelloSignature]) -> Self {
        // 直接从签名计算统计信息
        let mut stats = Self {
            total_fingerprints: signatures.len(),
            fingerprints_with_grease: 0,
            fingerprints_with_sni: 0,
            fingerprints_with_alpn: 0,
            version_distribution: HashMap::new(),
            top_cipher_suites: Vec::new(),
            top_extensions: Vec::new(),
        };

        let mut cipher_suite_counts: HashMap<u16, usize> = HashMap::new();
        let mut extension_counts: HashMap<u16, usize> = HashMap::new();

        for signature in signatures {
            // 检查 GREASE
            if signature.has_grease() {
                stats.fingerprints_with_grease += 1;
            }

            // 检查 SNI
            if signature.sni.is_some() {
                stats.fingerprints_with_sni += 1;
            }

            // 检查 ALPN
            if signature.alpn.is_some() {
                stats.fingerprints_with_alpn += 1;
            }

            // TLS 版本分布
            let version_str = format!("{}", signature.version);
            *stats.version_distribution.entry(version_str).or_insert(0) += 1;

            // 统计密码套件
            for suite in &signature.cipher_suites {
                *cipher_suite_counts.entry(*suite).or_insert(0) += 1;
            }

            // 统计扩展
            for ext in &signature.extensions {
                *extension_counts.entry(*ext).or_insert(0) += 1;
            }
        }

        // 获取最常见的密码套件
        let mut cipher_vec: Vec<(u16, usize)> = cipher_suite_counts.into_iter().collect();
        cipher_vec.sort_by(|a, b| b.1.cmp(&a.1));
        stats.top_cipher_suites = cipher_vec.into_iter().take(10).collect();

        // 获取最常见的扩展
        let mut ext_vec: Vec<(u16, usize)> = extension_counts.into_iter().collect();
        ext_vec.sort_by(|a, b| b.1.cmp(&a.1));
        stats.top_extensions = ext_vec.into_iter().take(10).collect();

        stats
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stats_from_specs() {
        let specs = vec![
            ClientHelloSpec::chrome_133(),
            ClientHelloSpec::chrome_103(),
            ClientHelloSpec::firefox_133(),
        ];
        let stats = FingerprintStats::from_specs(&specs);
        assert_eq!(stats.total_fingerprints, 3);
        assert!(!stats.version_distribution.is_empty());
    }
}
