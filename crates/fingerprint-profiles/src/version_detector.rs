/// Browser Version Detection from User-Agent
///
/// Parses User-Agent strings to identify browser type and version
use super::version_registry::BrowserType;
use regex::Regex;
use std::sync::OnceLock;

/// Browser detection result
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrowserInfo {
    /// Browser type
    pub browser: BrowserType,
    /// Version number
    pub version: u32,
    /// Full User-Agent string
    pub user_agent: String,
    /// Operating system (if detected)
    pub os: Option<String>,
    /// is Mobile device
    pub is_mobile: bool,
}

/// Version detector for User-Agent parsing
pub struct VersionDetector;

impl VersionDetector {
    /// Detect browser from User-Agent string
    ///
    /// # Examples
    ///
    /// ```
    /// use fingerprint_profiles::VersionDetector;
    ///
    /// let ua = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/133.0.0.0 Safari/537.36";
    /// let info = VersionDetector::detect(ua);
    /// assert!(info.is_some());
    /// ```
    pub fn detect(user_agent: &str) -> Option<BrowserInfo> {
        // Edge detection (must come before Chrome detection)
        if let Some(version) = Self::extract_edge_version(user_agent) {
            return Some(BrowserInfo {
                browser: BrowserType::Edge,
                version,
                user_agent: user_agent.to_string(),
                os: Self::extract_os(user_agent),
                is_mobile: Self::is_mobile(user_agent),
            });
        }

        // Opera detection (must come before Chrome detection)
        if let Some(version) = Self::extract_opera_version(user_agent) {
            return Some(BrowserInfo {
                browser: BrowserType::Opera,
                version,
                user_agent: user_agent.to_string(),
                os: Self::extract_os(user_agent),
                is_mobile: Self::is_mobile(user_agent),
            });
        }

        // Chrome detection (must come before Safari/Android detection)
        if let Some(version) = Self::extract_chrome_version(user_agent) {
            return Some(BrowserInfo {
                browser: BrowserType::Chrome,
                version,
                user_agent: user_agent.to_string(),
                os: Self::extract_os(user_agent),
                is_mobile: Self::is_mobile(user_agent),
            });
        }

        // Firefox detection
        if let Some(version) = Self::extract_firefox_version(user_agent) {
            return Some(BrowserInfo {
                browser: BrowserType::Firefox,
                version,
                user_agent: user_agent.to_string(),
                os: Self::extract_os(user_agent),
                is_mobile: Self::is_mobile(user_agent),
            });
        }

        // Safari detection (must come after Chrome detection)
        if let Some(version) = Self::extract_safari_version(user_agent) {
            return Some(BrowserInfo {
                browser: BrowserType::Safari,
                version,
                user_agent: user_agent.to_string(),
                os: Self::extract_os(user_agent),
                is_mobile: Self::is_mobile(user_agent),
            });
        }

        None
    }

    /// Extract Chrome version from User-Agent
    fn extract_chrome_version(user_agent: &str) -> Option<u32> {
        // Chrome 133.0.0.0 or Chromium/133.0.0.0
        let pattern = static_regex(r"Chrome[/\s]+(\d+)");
        pattern
            .captures(user_agent)
            .and_then(|c| c.get(1))
            .and_then(|m| m.as_str().parse().ok())
    }

    /// Extract Firefox version from User-Agent
    fn extract_firefox_version(user_agent: &str) -> Option<u32> {
        // Firefox/133.0 or Thunderbird/138.0
        let pattern = static_regex(r"Firefox[/\s]+(\d+)");
        pattern
            .captures(user_agent)
            .and_then(|c| c.get(1))
            .and_then(|m| m.as_str().parse().ok())
    }

    /// Extract Safari version from User-Agent
    fn extract_safari_version(user_agent: &str) -> Option<u32> {
        // Safari without Chrome (to avoid false positives)
        if user_agent.contains("Chrome") || user_agent.contains("Chromium") {
            return None;
        }

        // Version pattern for Safari
        let pattern = static_regex(r"Version[/\s]+(\d+)");
        pattern
            .captures(user_agent)
            .and_then(|c| c.get(1))
            .and_then(|m| m.as_str().parse().ok())
    }

    /// Extract Edge version from User-Agent
    fn extract_edge_version(user_agent: &str) -> Option<u32> {
        // Edg/133.0.0.0 (note: lowercase 'dg')
        let pattern = static_regex(r"Edg[e]?[/\s]+(\d+)");
        pattern
            .captures(user_agent)
            .and_then(|c| c.get(1))
            .and_then(|m| m.as_str().parse().ok())
    }

    /// Extract Opera version from User-Agent
    fn extract_opera_version(user_agent: &str) -> Option<u32> {
        // OPR/94.0.0.0 or Opera/94.0.0.0
        let pattern = static_regex(r"(?:OPR|Opera)[/\s]+(\d+)");
        pattern
            .captures(user_agent)
            .and_then(|c| c.get(1))
            .and_then(|m| m.as_str().parse().ok())
    }

    /// Extract operating system from User-Agent
    fn extract_os(user_agent: &str) -> Option<String> {
        if user_agent.contains("Windows") {
            Some("Windows".to_string())
        } else if user_agent.contains("Macintosh") || user_agent.contains("Mac OS X") {
            Some("macOS".to_string())
        } else if user_agent.contains("Linux") {
            Some("Linux".to_string())
        } else if user_agent.contains("iPhone") || user_agent.contains("iPad") {
            Some("iOS".to_string())
        } else if user_agent.contains("Android") {
            Some("Android".to_string())
        } else {
            None
        }
    }

    /// Check if User-Agent indicates mobile device
    pub fn is_mobile(user_agent: &str) -> bool {
        user_agent.contains("Mobile")
            || user_agent.contains("iPhone")
            || user_agent.contains("iPad")
            || user_agent.contains("Android")
            || user_agent.contains("webOS")
    }

    /// Parse version range (e.g., "133" -> 133, "130-135" -> [130-135])
    pub fn parse_version_range(version_str: &str) -> Result<Vec<u32>, String> {
        if version_str.contains('-') {
            let parts: Vec<&str> = version_str.split('-').collect();
            if parts.len() != 2 {
                return Err("Invalid version range format".to_string());
            }

            let start: u32 = parts[0]
                .trim()
                .parse()
                .map_err(|_| "Invalid start version")?;
            let end: u32 = parts[1].trim().parse().map_err(|_| "Invalid end version")?;

            Ok((start..=end).collect())
        } else {
            version_str
                .trim()
                .parse::<u32>()
                .map(|v| vec![v])
                .map_err(|_| "Invalid version".to_string())
        }
    }
}

/// Helper function to get or create regex (with caching)
fn static_regex(pattern: &str) -> &'static Regex {
    // This is a simplified version - in production, use a proper regex cache
    match pattern {
        r"Chrome[/\s]+(\d+)" => {
            static CHROME: OnceLock<Regex> = OnceLock::new();
            CHROME.get_or_init(|| Regex::new(r"Chrome[/\s]+(\d+)").unwrap())
        }
        r"Firefox[/\s]+(\d+)" => {
            static FIREFOX: OnceLock<Regex> = OnceLock::new();
            FIREFOX.get_or_init(|| Regex::new(r"Firefox[/\s]+(\d+)").unwrap())
        }
        r"Version[/\s]+(\d+)" => {
            static VERSION: OnceLock<Regex> = OnceLock::new();
            VERSION.get_or_init(|| Regex::new(r"Version[/\s]+(\d+)").unwrap())
        }
        r"Edg[e]?[/\s]+(\d+)" => {
            static EDGE: OnceLock<Regex> = OnceLock::new();
            EDGE.get_or_init(|| Regex::new(r"Edg[e]?[/\s]+(\d+)").unwrap())
        }
        r"(?:OPR|Opera)[/\s]+(\d+)" => {
            static OPERA: OnceLock<Regex> = OnceLock::new();
            OPERA.get_or_init(|| Regex::new(r"(?:OPR|Opera)[/\s]+(\d+)").unwrap())
        }
        _ => panic!("Unknown regex pattern"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_chrome_133() {
        let ua = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/133.0.0.0 Safari/537.36";
        let info = VersionDetector::detect(ua);
        assert!(info.is_some());
        let info = info.unwrap();
        assert_eq!(info.browser, BrowserType::Chrome);
        assert_eq!(info.version, 133);
    }

    #[test]
    fn test_detect_firefox_133() {
        let ua = "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:133.0) Gecko/20100101 Firefox/133.0";
        let info = VersionDetector::detect(ua);
        assert!(info.is_some());
        let info = info.unwrap();
        assert_eq!(info.browser, BrowserType::Firefox);
        assert_eq!(info.version, 133);
    }

    #[test]
    fn test_detect_safari() {
        let ua = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Version/18.0 Safari/537.36";
        let info = VersionDetector::detect(ua);
        assert!(info.is_some());
        let info = info.unwrap();
        assert_eq!(info.browser, BrowserType::Safari);
        assert_eq!(info.version, 18);
    }

    #[test]
    fn test_detect_edge_133() {
        let ua = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/133.0.0.0 Safari/537.36 Edg/133.0.0.0";
        let info = VersionDetector::detect(ua);
        assert!(info.is_some());
        let info = info.unwrap();
        assert_eq!(info.browser, BrowserType::Edge);
        assert_eq!(info.version, 133);
    }

    #[test]
    fn test_parse_version_range() {
        let range = VersionDetector::parse_version_range("130-135").unwrap();
        assert_eq!(range.len(), 6);
        assert_eq!(range[0], 130);
        assert_eq!(range[5], 135);

        let single = VersionDetector::parse_version_range("133").unwrap();
        assert_eq!(single.len(), 1);
        assert_eq!(single[0], 133);
    }

    #[test]
    fn test_is_mobile() {
        assert!(VersionDetector::is_mobile(
            "Mozilla/5.0 (iPhone; CPU iPhone OS 18_0 like Mac OS X) AppleWebKit/537.36"
        ));
        assert!(!VersionDetector::is_mobile(
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36"
        ));
    }
}
