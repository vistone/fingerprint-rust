use super::profiles::BrowserProfile;
use super::version_detector::{BrowserInfo, VersionDetector};
/// Browser Version Adapter
///
/// Automatically selects and adapts browser profiles based on detected version
/// Provides fast adaption for new browser versions
use super::version_registry::{BrowserType, VersionRegistry};
use std::sync::OnceLock;

/// Version-aware profile adapter
pub struct VersionAdapter {
    registry: VersionRegistry,
}

impl VersionAdapter {
    /// Create new version adapter with default registry
    #[must_use]
    pub fn new() -> Self {
        Self {
            registry: VersionRegistry::new(),
        }
    }

    /// Get version adapter singleton (lazy initialization)
    pub fn instance() -> &'static Self {
        static INSTANCE: OnceLock<VersionAdapter> = OnceLock::new();
        INSTANCE.get_or_init(VersionAdapter::new)
    }

    /// Get profile for a specific browser and version
    pub fn get_profile(&self, browser: BrowserType, version: u32) -> Option<BrowserProfile> {
        // Try exact version first
        if let Some(entry) = self.registry.get_version(browser, version) {
            return Some(self.load_profile(&entry.profile_fn));
        }

        // Find nearest compatible version
        if let Some(nearest_version) = self.registry.find_nearest_compatible(browser, version) {
            if let Some(entry) = self.registry.get_version(browser, nearest_version) {
                return Some(self.load_profile(&entry.profile_fn));
            }
        }

        None
    }

    /// Get profile from User-Agent string
    pub fn get_profile_from_ua(&self, user_agent: &str) -> Option<BrowserProfile> {
        let info = VersionDetector::detect(user_agent)?;
        self.get_profile(info.browser, info.version)
    }

    /// Get profile for latest browser version
    pub fn get_latest_profile(&self, browser: BrowserType) -> Option<BrowserProfile> {
        let entry = self.registry.get_latest(browser)?;
        Some(self.load_profile(&entry.profile_fn))
    }

    /// Load profile by function name (dynamically)
    fn load_profile(&self, profile_fn: &str) -> BrowserProfile {
        // This is a dispatch function that maps profile function names to actual profiles
        // In a real implementation, this could use a plugin system or code generation

        use crate::profiles::*;

        match profile_fn {
            // Chrome versions
            "chrome_103" => chrome_103(),
            "chrome_104" => chrome_104(),
            "chrome_105" => chrome_105(),
            "chrome_106" => chrome_106(),
            "chrome_107" => chrome_107(),
            "chrome_108" => chrome_108(),
            "chrome_109" => chrome_109(),
            "chrome_110" => chrome_110(),
            "chrome_111" => chrome_111(),
            "chrome_112" => chrome_112(),
            "chrome_116_psk" => chrome_116_psk(),
            "chrome_116_psk_pq" => chrome_116_psk_pq(),
            "chrome_117" => chrome_117(),
            "chrome_120" => chrome_120(),
            "chrome_121" => chrome_121(),
            "chrome_122" => chrome_122(),
            "chrome_123" => chrome_123(),
            "chrome_124" => chrome_124(),
            "chrome_125" => chrome_125(),
            "chrome_126" => chrome_126(),
            "chrome_127" => chrome_127(),
            "chrome_128" => chrome_128(),
            "chrome_129" => chrome_129(),
            "chrome_130" => chrome_130(),
            "chrome_131" => chrome_131(),
            "chrome_132" => chrome_132(),
            "chrome_133" => chrome_133(),
            "chrome_133_psk" => chrome_133_psk(),
            "chrome_133_0rtt" => chrome_133_0rtt(),
            "chrome_133_psk_0rtt" => chrome_133_psk_0rtt(),
            "chrome_134" => chrome_134(),
            "chrome_135" => chrome_135(),
            "chrome_136" => chrome_136(),
            "chrome_137" => chrome_137(),
            "chrome_138" => chrome_138(),

            // Firefox versions
            "firefox_102" => firefox_102(),
            "firefox_104" => firefox_104(),
            "firefox_105" => firefox_105(),
            "firefox_106" => firefox_106(),
            "firefox_108" => firefox_108(),
            "firefox_110" => firefox_110(),
            "firefox_117" => firefox_117(),
            "firefox_120" => firefox_120(),
            "firefox_123" => firefox_123(),
            "firefox_130" => firefox_130(),
            "firefox_131" => firefox_131(),
            "firefox_132" => firefox_132(),
            "firefox_133" => firefox_133(),
            "firefox_134" => firefox_134(),
            "firefox_135" => firefox_135(),
            "firefox_136" => firefox_136(),
            "firefox_137" => firefox_137(),
            "firefox_138" => firefox_138(),

            // Safari versions
            "safari_15_0" => safari_15_0(),
            "safari_15_6_1" => safari_15_6_1(),
            "safari_15_7" => safari_15_7(),
            "safari_16_0" => safari_16_0(),
            "safari_17_0" => safari_17_0(),
            "safari_17_5" => safari_17_5(),
            "safari_18_0" => safari_18_0(),
            "safari_18_1" => safari_18_1(),
            "safari_18_2" => safari_18_2(),
            "safari_18_3" => safari_18_3(),

            // Safari iOS versions
            "safari_ios_15_5" => safari_ios_15_5(),
            "safari_ios_15_6" => safari_ios_15_6(),
            "safari_ios_16_0" => safari_ios_16_0(),
            "safari_ios_17_0" => safari_ios_17_0(),
            "safari_ios_18_0" => safari_ios_18_0(),
            "safari_ios_18_1" => safari_ios_18_1(),
            "safari_ios_18_2" => safari_ios_18_2(),
            "safari_ios_18_3" => safari_ios_18_3(),
            "safari_ios_18_5" => safari_ios_18_5(),

            // Safari iPad versions
            "safari_ipad_15_6" => safari_ipad_15_6(),

            // Edge versions
            "edge_120" => edge_120(),
            "edge_124" => edge_124(),
            "edge_125" => edge_125(),
            "edge_126" => edge_126(),
            "edge_130" => edge_130(),
            "edge_131" => edge_131(),
            "edge_132" => edge_132(),
            "edge_133" => edge_133(),
            "edge_134" => edge_134(),
            "edge_135" => edge_135(),
            "edge_137" => edge_137(),

            // Opera versions
            "opera_89" => opera_89(),
            "opera_90" => opera_90(),
            "opera_91" => opera_91(),
            "opera_92" => opera_92(),
            "opera_93" => opera_93(),
            "opera_94" => opera_94(),

            // Mobile variants
            "chrome_mobile_120" => chrome_mobile_120(),
            "chrome_mobile_130" => chrome_mobile_130(),
            "chrome_mobile_134" => chrome_mobile_134(),
            "chrome_mobile_135" => chrome_mobile_135(),
            "chrome_mobile_137" => chrome_mobile_137(),
            "firefox_mobile_120" => firefox_mobile_120(),
            "firefox_mobile_130" => firefox_mobile_130(),
            "firefox_mobile_135" => firefox_mobile_135(),

            // Default fallback
            _ => chrome_133(),
        }
    }

    /// Get versions with specific capability
    pub fn get_versions_with_capability(&self, browser: BrowserType, capability: &str) -> Vec<u32> {
        self.registry
            .get_with_feature(browser, capability)
            .into_iter()
            .map(|(v, _)| *v)
            .collect()
    }

    /// Check if a version supports a capability
    pub fn supports_capability(
        &self,
        browser: BrowserType,
        version: u32,
        capability: &str,
    ) -> bool {
        self.registry
            .get_version(browser, version)
            .map(|entry| match capability {
                "ech" => entry.ech_support,
                "http3" => entry.http3_support,
                "psk" => entry.psk_support,
                "early_data" => entry.early_data_support,
                "pq" => entry.pq_support,
                _ => false,
            })
            .unwrap_or(false)
    }

    /// Get version information
    pub fn get_version_info(&self, browser: BrowserType, version: u32) -> Option<String> {
        self.registry.get_version(browser, version).map(|entry| {
            format!(
                "{} v{}: {} | ECH: {} | HTTP3: {} | PSK: {} | 0-RTT: {} | PQ: {}",
                match browser {
                    BrowserType::Chrome => "Chrome",
                    BrowserType::Firefox => "Firefox",
                    BrowserType::Safari => "Safari",
                    BrowserType::Edge => "Edge",
                    BrowserType::Opera => "Opera",
                },
                entry.version,
                entry.release_date,
                if entry.ech_support { "✓" } else { "✗" },
                if entry.http3_support { "✓" } else { "✗" },
                if entry.psk_support { "✓" } else { "✗" },
                if entry.early_data_support {
                    "✓"
                } else {
                    "✗"
                },
                if entry.pq_support { "✓" } else { "✗" },
            )
        })
    }
}

impl Default for VersionAdapter {
    fn default() -> Self {
        Self::new()
    }
}

/// Quick API for version adaptation
pub mod quick {
    use super::*;

    /// Get profile from User-Agent string (quick API)
    pub fn profile_from_ua(user_agent: &str) -> Option<BrowserProfile> {
        VersionAdapter::instance().get_profile_from_ua(user_agent)
    }

    /// Get profile for specific browser and version (quick API)
    pub fn profile(browser: BrowserType, version: u32) -> Option<BrowserProfile> {
        VersionAdapter::instance().get_profile(browser, version)
    }

    /// Get latest profile (quick API)
    pub fn latest_profile(browser: BrowserType) -> Option<BrowserProfile> {
        VersionAdapter::instance().get_latest_profile(browser)
    }

    /// Detect browser from User-Agent (quick API)
    pub fn detect_browser(user_agent: &str) -> Option<BrowserInfo> {
        VersionDetector::detect(user_agent)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adapter_creation() {
        let adapter = VersionAdapter::new();
        assert!(adapter.registry.chrome.len() > 0);
    }

    #[test]
    fn test_get_profile_exact_version() {
        let adapter = VersionAdapter::new();
        let profile = adapter.get_profile(BrowserType::Chrome, 133);
        assert!(profile.is_some());
    }

    #[test]
    fn test_get_profile_from_ua() {
        let ua = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/133.0.0.0 Safari/537.36";
        let adapter = VersionAdapter::new();
        let profile = adapter.get_profile_from_ua(ua);
        assert!(profile.is_some());
    }

    #[test]
    fn test_get_latest_profile() {
        let adapter = VersionAdapter::new();
        let profile = adapter.get_latest_profile(BrowserType::Chrome);
        assert!(profile.is_some());
    }

    #[test]
    fn test_supports_capability() {
        let adapter = VersionAdapter::new();
        assert!(adapter.supports_capability(BrowserType::Chrome, 133, "ech"));
    }

    #[test]
    fn test_quick_api() {
        let ua = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/133.0.0.0 Safari/537.36";
        let profile = quick::profile_from_ua(ua);
        assert!(profile.is_some());

        let profile = quick::profile(BrowserType::Chrome, 133);
        assert!(profile.is_some());

        let profile = quick::latest_profile(BrowserType::Chrome);
        assert!(profile.is_some());
    }
}
