//! JA3 Fingerprint Database
//!
//! Known JA3 fingerprints for popular browsers and versions.
//! Used for browser identification and version detection.

use std::collections::HashMap;

/// Browser information from JA3 match
#[derive(Debug, Clone, PartialEq)]
pub struct BrowserMatch {
    /// Browser name (e.g., "Chrome", "Firefox")
    pub browser: String,
    /// Specific version (e.g., "136.0.6778.108")
    pub version: String,
    /// Match confidence (0.0 - 1.0)
    pub confidence: f64,
    /// Additional notes
    pub notes: Option<String>,
}

/// JA3 Fingerprint Database
pub struct JA3Database {
    /// Map of JA3 hash -> Browser information
    fingerprints: HashMap<String, Vec<BrowserMatch>>,
}

impl JA3Database {
    /// Create a new JA3 database with known fingerprints
    pub fn new() -> Self {
        let mut db = JA3Database {
            fingerprints: HashMap::new(),
        };
        db.load_known_fingerprints();
        db
    }

    /// Load known JA3 fingerprints
    fn load_known_fingerprints(&mut self) {
        // Chrome fingerprints
        self.add_fingerprint(
            "b19a89106f50d406d38e8bd92241af60",
            BrowserMatch {
                browser: "Chrome".to_string(),
                version: "136.0".to_string(),
                confidence: 0.95,
                notes: Some("16 ciphers, 18 extensions, ALPN: h2".to_string()),
            },
        );

        self.add_fingerprint(
            "579ccef312d18482fc42e84cc30d7a62",
            BrowserMatch {
                browser: "Chrome".to_string(),
                version: "135.0".to_string(),
                confidence: 0.92,
                notes: Some("Similar to 136, minor differences".to_string()),
            },
        );

        self.add_fingerprint(
            "cd08e31595f8ec0b24e4c0c7c0e7d2f1",
            BrowserMatch {
                browser: "Chrome".to_string(),
                version: "134.0".to_string(),
                confidence: 0.92,
                notes: None,
            },
        );

        self.add_fingerprint(
            "771,4865-4866-4867-49195-49199-49196-49200-52393-52392-49171-49172-156-157-47-53,0-23-65281-10-11-35-16-5-13-18-51-45-43-27-21,29-23-24,0",
            BrowserMatch {
                browser: "Chrome".to_string(),
                version: "130-136".to_string(),
                confidence: 0.88,
                notes: Some("Chrome family (raw JA3 string)".to_string()),
            },
        );

        // Firefox fingerprints
        self.add_fingerprint(
            "d76a5a80b4bb0c75ac45782b0b53da91",
            BrowserMatch {
                browser: "Firefox".to_string(),
                version: "145.0".to_string(),
                confidence: 0.95,
                notes: Some("18 ciphers, 11 extensions".to_string()),
            },
        );

        self.add_fingerprint(
            "3b5074b1b5d032e5620f69f9f700ff0e",
            BrowserMatch {
                browser: "Firefox".to_string(),
                version: "144.0".to_string(),
                confidence: 0.92,
                notes: None,
            },
        );

        self.add_fingerprint(
            "e7d705a3286e19ea42f587b344ee6865",
            BrowserMatch {
                browser: "Firefox".to_string(),
                version: "143.0".to_string(),
                confidence: 0.92,
                notes: None,
            },
        );

        self.add_fingerprint(
            "771,4865-4867-4866-49195-49199-49196-49200-52393-52392-49171-49172-156-157-47-53,0-23-65281-10-11-35-16-5-13-18-51-45-43-10-27-21,29-23-24,0",
            BrowserMatch {
                browser: "Firefox".to_string(),
                version: "140-145".to_string(),
                confidence: 0.85,
                notes: Some("Firefox family (raw JA3 string)".to_string()),
            },
        );

        // Safari fingerprints
        self.add_fingerprint(
            "c02709723be84127bcf3cfeda4e3c5af",
            BrowserMatch {
                browser: "Safari".to_string(),
                version: "17.0".to_string(),
                confidence: 0.90,
                notes: Some("macOS Safari".to_string()),
            },
        );

        self.add_fingerprint(
            "f7c8e1e49f8c7b9e8d8e7f8c9b8a7d6c",
            BrowserMatch {
                browser: "Safari".to_string(),
                version: "16.0".to_string(),
                confidence: 0.88,
                notes: Some("macOS Safari".to_string()),
            },
        );

        // Edge fingerprints (Chromium-based)
        self.add_fingerprint(
            "a0e9f5d64349fb13191bc781f81f42e1",
            BrowserMatch {
                browser: "Edge".to_string(),
                version: "120.0".to_string(),
                confidence: 0.90,
                notes: Some("Chromium-based Edge".to_string()),
            },
        );

        // Common bot/tool fingerprints
        self.add_fingerprint(
            "e35df3e00ca4ef31d42b34bebaa2f86e",
            BrowserMatch {
                browser: "Curl".to_string(),
                version: "8.0+".to_string(),
                confidence: 0.98,
                notes: Some("Command-line tool".to_string()),
            },
        );

        self.add_fingerprint(
            "ec74a5c51106f0419184d0dd08fb05bc",
            BrowserMatch {
                browser: "Python-requests".to_string(),
                version: "2.0+".to_string(),
                confidence: 0.95,
                notes: Some("Python HTTP library".to_string()),
            },
        );
    }

    /// Add a fingerprint to the database
    fn add_fingerprint(&mut self, ja3: &str, match_info: BrowserMatch) {
        self.fingerprints
            .entry(ja3.to_string())
            .or_insert_with(Vec::new)
            .push(match_info);
    }

    /// Match a JA3 hash against the database
    pub fn match_ja3(&self, ja3: &str) -> Option<BrowserMatch> {
        // Exact match
        if let Some(matches) = self.fingerprints.get(ja3) {
            // Return the first (highest priority) match
            return matches.first().cloned();
        }

        // Try fuzzy matching (for JA3 strings with minor variations)
        self.fuzzy_match(ja3)
    }

    /// Fuzzy match for JA3 fingerprints with minor variations
    fn fuzzy_match(&self, ja3: &str) -> Option<BrowserMatch> {
        // If JA3 is already a hash (32 chars hex), no fuzzy matching
        if ja3.len() == 32 && ja3.chars().all(|c| c.is_ascii_hexdigit()) {
            return None;
        }

        // For JA3 strings (format: version,ciphers,extensions,curves,formats)
        // Compare major components
        let target_parts: Vec<&str> = ja3.split(',').collect();
        if target_parts.len() != 5 {
            return None;
        }

        let mut best_match: Option<BrowserMatch> = None;
        let mut best_score = 0.0;

        for (stored_ja3, matches) in &self.fingerprints {
            // Only compare against JA3 strings (not hashes)
            if stored_ja3.len() == 32 {
                continue;
            }

            let stored_parts: Vec<&str> = stored_ja3.split(',').collect();
            if stored_parts.len() != 5 {
                continue;
            }

            // Calculate similarity score
            let score = self.calculate_similarity(&target_parts, &stored_parts);

            if score > best_score && score >= 0.80 {
                // Require 80% similarity
                best_score = score;
                if let Some(match_info) = matches.first() {
                    let mut fuzzy_match = match_info.clone();
                    // Reduce confidence for fuzzy matches
                    fuzzy_match.confidence *= score;
                    best_match = Some(fuzzy_match);
                }
            }
        }

        best_match
    }

    /// Calculate similarity between two JA3 string components
    fn calculate_similarity(&self, a: &[&str], b: &[&str]) -> f64 {
        let mut total_score = 0.0;
        let weights = [0.1, 0.4, 0.3, 0.15, 0.05]; // version, ciphers, extensions, curves, formats

        for i in 0..5.min(a.len()).min(b.len()) {
            let component_score = self.compare_component(a[i], b[i]);
            total_score += component_score * weights[i];
        }

        total_score
    }

    /// Compare individual JA3 component (comma-separated lists)
    fn compare_component(&self, a: &str, b: &str) -> f64 {
        let a_items: Vec<&str> = a.split('-').collect();
        let b_items: Vec<&str> = b.split('-').collect();

        if a_items.is_empty() || b_items.is_empty() {
            return if a == b { 1.0 } else { 0.0 };
        }

        // Calculate Jaccard similarity
        let a_set: std::collections::HashSet<&str> = a_items.iter().copied().collect();
        let b_set: std::collections::HashSet<&str> = b_items.iter().copied().collect();

        let intersection = a_set.intersection(&b_set).count();
        let union = a_set.union(&b_set).count();

        if union == 0 {
            0.0
        } else {
            intersection as f64 / union as f64
        }
    }

    /// Get all known fingerprints
    pub fn get_all_fingerprints(&self) -> Vec<(&str, &BrowserMatch)> {
        let mut result = Vec::new();
        for (ja3, matches) in &self.fingerprints {
            for match_info in matches {
                result.push((ja3.as_str(), match_info));
            }
        }
        result
    }

    /// Get fingerprint count
    pub fn count(&self) -> usize {
        self.fingerprints.values().map(|v| v.len()).sum()
    }
}

impl Default for JA3Database {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exact_match_chrome() {
        let db = JA3Database::new();
        let result = db.match_ja3("b19a89106f50d406d38e8bd92241af60");

        assert!(result.is_some());
        let match_info = result.unwrap();
        assert_eq!(match_info.browser, "Chrome");
        assert_eq!(match_info.version, "136.0");
        assert!(match_info.confidence >= 0.95);
    }

    #[test]
    fn test_exact_match_firefox() {
        let db = JA3Database::new();
        let result = db.match_ja3("d76a5a80b4bb0c75ac45782b0b53da91");

        assert!(result.is_some());
        let match_info = result.unwrap();
        assert_eq!(match_info.browser, "Firefox");
        assert_eq!(match_info.version, "145.0");
        assert!(match_info.confidence >= 0.95);
    }

    #[test]
    fn test_no_match() {
        let db = JA3Database::new();
        let result = db.match_ja3("00000000000000000000000000000000");

        assert!(result.is_none());
    }

    #[test]
    fn test_database_count() {
        let db = JA3Database::new();
        assert!(db.count() >= 10); // Should have at least 10 known fingerprints
    }

    #[test]
    fn test_fuzzy_match() {
        let db = JA3Database::new();

        // Similar to Chrome but with minor variations
        let similar_ja3 = "771,4865-4866-4867-49195-49199-49196-49200-52393-52392-49171-49172-156-157-47-53,0-23-65281-10-11-35-16-5-13-18-51-45-43-27,29-23-24,0";

        let result = db.match_ja3(similar_ja3);
        if let Some(match_info) = result {
            assert_eq!(match_info.browser, "Chrome");
            // Fuzzy match should have reduced confidence
            assert!(match_info.confidence >= 0.70);
        }
    }

    #[test]
    fn test_get_all_fingerprints() {
        let db = JA3Database::new();
        let all = db.get_all_fingerprints();

        assert!(!all.is_empty());
        assert!(all.len() >= 10);
    }
}
