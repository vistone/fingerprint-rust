//! GREASE 值处理模块
//!
//! 参考：<https://datatracker.ietf.org/doc/html/draft-davidben-tls-grease-01>
//! 参考：Huginn Net 的 GREASE 处理实现

/// TLS GREASE 值常量
/// 根据 RFC 8701，GREASE 值的模式是：0x1a1a, 0x2a2a, 0x3a3a, ..., 0xfafa
/// 以及 0x0a0a (特殊值)
pub const TLS_GREASE_VALUES: [u16; 16] = [
    0x0a0a, 0x1a1a, 0x2a2a, 0x3a3a, 0x4a4a, 0x5a5a, 0x6a6a, 0x7a7a, 0x8a8a, 0x9a9a, 0xaaaa, 0xbaba,
    0xcaca, 0xdada, 0xeaea, 0xfafa,
];

/// 获取随机 GREASE 值
pub fn get_random_grease() -> u16 {
    use rand::seq::SliceRandom;
    let mut rng = rand::thread_rng();
    *TLS_GREASE_VALUES.choose(&mut rng).unwrap_or(&0x0a0a)
}

/// 检查一个值是否是 GREASE 值
///
/// # 参数
/// * `value` - 要检查的 u16 值
///
/// # 返回
/// * `true` 如果是 GREASE 值，`false` 否则
///
/// # 示例
/// ```
/// use fingerprint_core::grease::is_grease_value;
/// assert!(is_grease_value(0x0a0a));
/// assert!(is_grease_value(0x1a1a));
/// assert!(!is_grease_value(0x0017));
/// ```
pub fn is_grease_value(value: u16) -> bool {
    TLS_GREASE_VALUES.contains(&value)
}

/// 从 u16 列表中过滤掉 GREASE 值
///
/// # 参数
/// * `values` - 要过滤的 u16 值列表
///
/// # 返回
/// * 过滤后的 `Vec<u16>`，不包含 GREASE 值
///
/// # 示例
/// ```
/// use fingerprint_core::grease::filter_grease_values;
/// let values = vec![0x0a0a, 0x0017, 0x1a1a, 0x0018];
/// let filtered = filter_grease_values(&values);
/// assert_eq!(filtered, vec![0x0017, 0x0018]);
/// ```
pub fn filter_grease_values(values: &[u16]) -> Vec<u16> {
    values
        .iter()
        .filter(|&&v| !is_grease_value(v))
        .copied()
        .collect()
}

/// 从 u16 列表中移除 GREASE 值（原地修改）
///
/// # 参数
/// * `values` - 要修改的 u16 值列表（可变引用）
///
/// # 示例
/// ```
/// use fingerprint_core::grease::remove_grease_values;
/// let mut values = vec![0x0a0a, 0x0017, 0x1a1a, 0x0018];
/// remove_grease_values(&mut values);
/// assert_eq!(values, vec![0x0017, 0x0018]);
/// ```
pub fn remove_grease_values(values: &mut Vec<u16>) {
    values.retain(|&v| !is_grease_value(v));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_grease_value() {
        assert!(is_grease_value(0x0a0a));
        assert!(is_grease_value(0x1a1a));
        assert!(is_grease_value(0xfafa));
        assert!(!is_grease_value(0x0017));
        assert!(!is_grease_value(0x0018));
        assert!(!is_grease_value(0x0303));
    }

    #[test]
    fn test_filter_grease_values() {
        let values = vec![0x0a0a, 0x0017, 0x1a1a, 0x0018, 0x2a2a];
        let filtered = filter_grease_values(&values);
        assert_eq!(filtered, vec![0x0017, 0x0018]);
    }

    #[test]
    fn test_remove_grease_values() {
        let mut values = vec![0x0a0a, 0x0017, 0x1a1a, 0x0018];
        remove_grease_values(&mut values);
        assert_eq!(values, vec![0x0017, 0x0018]);
    }

    #[test]
    fn test_all_grease_values() {
        for &grease in &TLS_GREASE_VALUES {
            assert!(
                is_grease_value(grease),
                "{} should be a GREASE value",
                grease
            );
        }
    }
}
