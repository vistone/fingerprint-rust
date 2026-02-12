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

/// Getrandom GREASE value
pub fn get_random_grease() -> u16 {
    use rand::seq::SliceRandom;
    let mut rng = rand::thread_rng();
    *TLS_GREASE_VALUES.choose(&mut rng).unwrap_or(&0x0a0a)
}

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
/// use fingerprint_core::grease::is_grease_value;
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
/// * filterback的 `Vec<u16>`, excluding GREASE value
///
/// # Examples
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

/// from u16 list in remove GREASE value (in-placemodify)
///
/// # Parameters
/// * `values` - needmodify u16 valuelist (mutablereference)
///
/// # Examples
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

/// Normalize JA3 string by removing GREASE values
///
/// JA3 format: SSLVersion,Ciphers,Extensions,EllipticCurves,EllipticCurvePointFormats
/// Removes GREASE values from ciphers, extensions, and curves lists.
///
/// # Examples
/// ```
/// use fingerprint_core::grease::normalize_ja3_string;
/// let ja3 = "771,4865-4866-4867-1a1a,0-23-65281,29-23-24,0";
/// let normalized = normalize_ja3_string(ja3);
/// assert!(!normalized.contains("1a1a"));
/// ```
pub fn normalize_ja3_string(ja3: &str) -> String {
    let parts: Vec<&str> = ja3.split(',').collect();

    if parts.len() != 5 {
        return ja3.to_string(); // Invalid format, return as-is
    }

    [
        parts[0].to_string(),
        remove_grease_from_hex_list(parts[1]),
        remove_grease_from_hex_list(parts[2]),
        remove_grease_from_hex_list(parts[3]),
        parts[4].to_string(),
    ]
    .join(",")
}

/// Remove GREASE values from comma-separated list of hex values
fn remove_grease_from_hex_list(list: &str) -> String {
    let values: Vec<&str> = list.split('-').collect();

    let filtered: Vec<String> = values
        .iter()
        .filter_map(|val| {
            // Try to parse as hex number
            if let Ok(num) = u16::from_str_radix(val, 16) {
                // Check if it's a GREASE value
                if is_grease_value(num) {
                    None // Remove GREASE values
                } else {
                    Some(val.to_string())
                }
            } else {
                Some(val.to_string()) // Keep non-numeric values
            }
        })
        .collect();

    filtered.join("-")
}

/// Compare two JA3 fingerprints, ignoring GREASE differences
///
/// # Examples
/// ```
/// use fingerprint_core::grease::ja3_equal_ignore_grease;
/// let ja3_a = "771,4865-4866-4867-0a0a,0-23-65281,29-23-24,0";
/// let ja3_b = "771,4865-4866-4867-1a1a,0-23-65281,29-23-24,0";
/// assert!(ja3_equal_ignore_grease(ja3_a, ja3_b));
/// ```
pub fn ja3_equal_ignore_grease(ja3_a: &str, ja3_b: &str) -> bool {
    normalize_ja3_string(ja3_a) == normalize_ja3_string(ja3_b)
}

/// Calculate similarity between two JA3 fingerprints (0.0 - 1.0)
///
/// Takes into account normalized (GREASE-free) components.
pub fn ja3_similarity(ja3_a: &str, ja3_b: &str) -> f64 {
    let normalized_a = normalize_ja3_string(ja3_a);
    let normalized_b = normalize_ja3_string(ja3_b);

    if normalized_a == normalized_b {
        return 1.0;
    }

    let parts_a: Vec<&str> = normalized_a.split(',').collect();
    let parts_b: Vec<&str> = normalized_b.split(',').collect();

    if parts_a.len() != 5 || parts_b.len() != 5 {
        return 0.0;
    }

    let mut total_score = 0.0;
    let weights = [0.1, 0.4, 0.3, 0.15, 0.05]; // Version, Ciphers, Extensions, Curves, Formats

    for i in 0..5 {
        let similarity = component_similarity(parts_a[i], parts_b[i]);
        total_score += similarity * weights[i];
    }

    total_score
}

/// Calculate similarity between two JA3 components (uses Jaccard similarity)
fn component_similarity(comp_a: &str, comp_b: &str) -> f64 {
    let items_a: std::collections::HashSet<&str> = comp_a.split('-').collect();
    let items_b: std::collections::HashSet<&str> = comp_b.split('-').collect();

    if items_a.is_empty() && items_b.is_empty() {
        return 1.0;
    }

    let intersection = items_a.intersection(&items_b).count();
    let union = items_a.union(&items_b).count();

    if union == 0 {
        0.0
    } else {
        intersection as f64 / union as f64
    }
}

#[cfg(test)]
mod ja3_normalization_tests {
    use super::*;

    #[test]
    fn test_normalize_ja3_removes_grease() {
        let ja3 = "771,4865-4866-4867-1a1a,0-23-65281-0a0a,29-23-24,0";
        let normalized = normalize_ja3_string(ja3);

        assert!(!normalized.contains("1a1a"));
        assert!(!normalized.contains("0a0a"));
        assert!(normalized.contains("4865"));
        assert!(normalized.contains("4866"));
    }

    #[test]
    fn test_ja3_equal_ignore_grease() {
        let ja3_a = "771,4865-4866-4867-0a0a,0-23-65281,29-23-24,0";
        let ja3_b = "771,4865-4866-4867-1a1a,0-23-65281,29-23-24,0";

        assert!(ja3_equal_ignore_grease(ja3_a, ja3_b));
    }

    #[test]
    fn test_ja3_not_equal_ignore_grease() {
        let ja3_a = "771,4865-4866-4867,0-23-65281,29-23-24,0";
        let ja3_b = "771,4865-4866,0-23-65281,29-23-24,0"; // Missing cipher

        assert!(!ja3_equal_ignore_grease(ja3_a, ja3_b));
    }

    #[test]
    fn test_ja3_similarity_identical() {
        let ja3_a = "771,4865-4866-4867,0-23-65281,29-23-24,0";
        let ja3_b = "771,4865-4866-4867,0-23-65281,29-23-24,0";

        let sim = ja3_similarity(ja3_a, ja3_b);
        assert_eq!(sim, 1.0);
    }

    #[test]
    fn test_ja3_similarity_with_grease_differences() {
        let ja3_a = "771,4865-4866-4867-0a0a,0-23-65281,29-23-24,0";
        let ja3_b = "771,4865-4866-4867-1a1a,0-23-65281,29-23-24,0";

        let sim = ja3_similarity(ja3_a, ja3_b);
        assert_eq!(sim, 1.0); // Same after normalization
    }

    #[test]
    fn test_ja3_similarity_partial() {
        let ja3_a = "771,4865-4866-4867,0-23-65281,29-23-24,0";
        let ja3_b = "771,4865-4866,0-23-65281,29-23-24,0";

        let sim = ja3_similarity(ja3_a, ja3_b);
        assert!(sim > 0.7);
        assert!(sim < 1.0);
    }

    #[test]
    fn test_ja3_similarity_very_different() {
        let ja3_a = "771,4865-4866-4867,0-23-65281,29-23-24,0";
        let ja3_b = "771,1301-1302,0-23,23-24,0";

        let sim = ja3_similarity(ja3_a, ja3_b);
        assert!(sim < 0.5);
    }
}
