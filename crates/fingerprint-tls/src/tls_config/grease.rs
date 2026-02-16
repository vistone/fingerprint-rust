//! GREASE valueprocessmodule
//!
//! reference：<https://datatracker.ietf.org/doc/html/draft-davidben-tls-grease-01>
//! reference：Huginn Net GREASE processimplement

/// TLS GREASE valueconstant
/// Based on RFC 8701, GREASE valuepattern is ：0x1a1a, 0x2a2a, 0x3a3a,..., 0xfafa
/// and 0x0a0a (specialvalue)
pub const TLS_GREASE_VALUES: [u16; 16] = [
    0x0a0a, 0x1a1a, 0x2a2a, 0x3a3a, 0x4a4a, 0x5a5a, 0x6a6a, 0x7a7a, 0x8a8a, 0x9a9a, 0xaaaa, 0xbaba,
    0xcaca, 0xdada, 0xeaea, 0xfafa,
];

/// Checkanvaluewhether is GREASE value
///
/// # Parameters
/// * `value` - needCheck u16 value
///
/// # Returns
/// * `true` if is GREASE value, `false` otherwise
///
/// # Examples
/// ```
/// use fingerprint_tls::tls_config::is_grease_value;
/// assert!(is_grease_value(0x0a0a));
/// assert!(is_grease_value(0x1a1a));
/// assert!(!is_grease_value(0x0017));
/// ```
pub fn is_grease_value(value: u16) -> bool {
    TLS_GREASE_VALUES.contains(&value)
}

/// from u16 list in filter掉 GREASE value
///
/// # Parameters
/// * `values` - needfilter u16 valuelist
///
/// # Returns
/// * filterback `Vec<u16>`, excluding GREASE value
///
/// # Examples
/// ```
/// use fingerprint_tls::tls_config::filter_grease_values;
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

/// from u16 list in remove GREASE value (in-placemodify)
///
/// # Parameters
/// * `values` - needmodify u16 valuelist (mutablereference)
///
/// # Examples
/// ```
/// use fingerprint_tls::tls_config::remove_grease_values;
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
