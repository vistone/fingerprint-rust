//! TLS 指纹比较模块
//!
//! 提供指纹比较和匹配功能
//! 参考：Huginn Net 的指纹比较实现

use crate::tls_config::signature::ClientHelloSignature;
use crate::tls_config::spec::ClientHelloSpec;
use crate::tls_config::extract::extract_signature;

/// 指纹匹配结果
#[derive(Debug, Clone, PartialEq)]
pub enum FingerprintMatch {
    /// 完全匹配（包括 GREASE 值）
    Exact,
    /// 相似匹配（忽略 GREASE 值后相同）
    Similar,
    /// 不匹配
    None,
}

/// 比较两个 ClientHelloSpec 的相似度
/// 
/// # 参数
/// * `spec1` - 第一个 ClientHelloSpec
/// * `spec2` - 第二个 ClientHelloSpec
/// 
/// # 返回
/// * `FingerprintMatch` - 匹配结果
/// 
/// # 示例
/// ```
/// use fingerprint::{ClientHelloSpec, compare_specs};
/// let spec1 = ClientHelloSpec::chrome_133();
/// let spec2 = ClientHelloSpec::chrome_103();
/// let match_result = compare_specs(&spec1, &spec2);
/// ```
pub fn compare_specs(spec1: &ClientHelloSpec, spec2: &ClientHelloSpec) -> FingerprintMatch {
    let sig1 = extract_signature(spec1);
    let sig2 = extract_signature(spec2);

    compare_signatures(&sig1, &sig2)
}

/// 比较两个签名的相似度
/// 
/// # 参数
/// * `sig1` - 第一个签名
/// * `sig2` - 第二个签名
/// 
/// # 返回
/// * `FingerprintMatch` - 匹配结果
pub fn compare_signatures(sig1: &ClientHelloSignature, sig2: &ClientHelloSignature) -> FingerprintMatch {
    // 完全匹配
    if sig1 == sig2 {
        return FingerprintMatch::Exact;
    }

    // 相似匹配（忽略 GREASE）
    if sig1.similar_to(sig2) {
        return FingerprintMatch::Similar;
    }

    FingerprintMatch::None
}

/// 查找与给定签名最相似的指纹配置
/// 
/// # 参数
/// * `signature` - 要匹配的签名
/// * `specs` - 候选的 ClientHelloSpec 列表
/// 
/// # 返回
/// * `Option<usize>` - 最相似配置的索引，如果没有找到则返回 None
pub fn find_best_match(
    signature: &ClientHelloSignature,
    specs: &[ClientHelloSpec],
) -> Option<usize> {
    let mut best_index = None;
    let mut best_score = 0;

    for (index, spec) in specs.iter().enumerate() {
        let spec_sig = extract_signature(spec);
        let match_result = compare_signatures(signature, &spec_sig);

        let score = match match_result {
            FingerprintMatch::Exact => 100,
            FingerprintMatch::Similar => 50,
            FingerprintMatch::None => 0,
        };

        if score > best_score {
            best_score = score;
            best_index = Some(index);
        }
    }

    best_index
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compare_specs() {
        let spec1 = ClientHelloSpec::chrome_133();
        let spec2 = ClientHelloSpec::chrome_133();
        let result = compare_specs(&spec1, &spec2);
        assert_eq!(result, FingerprintMatch::Exact);
    }

    #[test]
    fn test_find_best_match() {
        let signature = extract_signature(&ClientHelloSpec::chrome_133());
        let specs = vec![
            ClientHelloSpec::chrome_103(),
            ClientHelloSpec::chrome_133(),
            ClientHelloSpec::firefox_133(),
        ];
        let best = find_best_match(&signature, &specs);
        assert_eq!(best, Some(1)); // chrome_133 应该是最匹配的
    }
}
