#[cfg(test)]
mod tests {
    use fingerprint_profiles::profiles::*;

    #[test]
    fn test_chrome_versions_exist() {
        // testingnewly addedofChromeversionfunction是否存在且能normalcall
        let chrome_104 = chrome_104();
        assert_eq!(chrome_104.id(), "chrome_104");

        let chrome_117 = chrome_117();
        assert_eq!(chrome_117.id(), "chrome_117");

        let chrome_133 = chrome_133();
        assert_eq!(chrome_133.id(), "chrome_133");
    }

    #[test]
    fn test_firefox_versions_exist() {
        // testingnewly addedofFirefoxversionfunction
        let firefox_102 = firefox_102();
        assert_eq!(firefox_102.id(), "firefox_102");

        let firefox_123 = firefox_123();
        assert_eq!(firefox_123.id(), "firefox_123");
    }

    #[test]
    fn test_safari_versions_exist() {
        // testingnewly addedofSafariversionfunction
        let safari_15_6_1 = safari_15_6_1();
        assert_eq!(safari_15_6_1.id(), "safari_15_6_1");

        let safari_ios_18_5 = safari_ios_18_5();
        assert_eq!(safari_ios_18_5.id(), "safari_ios_18_5");
    }

    #[test]
    fn test_opera_versions_exist() {
        // testingnewly addedofOperaversionfunction
        let opera_89 = opera_89();
        assert_eq!(opera_89.id(), "opera_89");

        let opera_90 = opera_90();
        assert_eq!(opera_90.id(), "opera_90");
    }

    #[test]
    fn test_version_registry_completeness() {
        use fingerprint_profiles::version_registry::{BrowserType, VersionRegistry};

        let registry = VersionRegistry::new();

        // validateChromeversionregister
        assert!(registry.get_version(BrowserType::Chrome, 104).is_some());
        assert!(registry.get_version(BrowserType::Chrome, 117).is_some());
        assert!(registry.get_version(BrowserType::Chrome, 133).is_some());

        // validateFirefoxversionregister
        assert!(registry.get_version(BrowserType::Firefox, 102).is_some());
        assert!(registry.get_version(BrowserType::Firefox, 123).is_some());

        // validateSafariversionregister
        assert!(registry.get_version(BrowserType::Safari, 156).is_some());
        assert!(registry.get_version(BrowserType::Safari, 1850).is_some()); // iOS 18.5

        // validateOperaversionregister
        assert!(registry.get_version(BrowserType::Opera, 89).is_some());
        assert!(registry.get_version(BrowserType::Opera, 90).is_some());
    }

    #[test]
    fn test_version_adapter_loading() {
        use fingerprint_profiles::version_adapter::VersionAdapter;
        use fingerprint_profiles::version_registry::BrowserType;

        let adapter = VersionAdapter::new();

        // testingloadnewly addedofversion
        let chrome_104_profile = adapter.get_profile(BrowserType::Chrome, 104);
        assert!(chrome_104_profile.is_some());

        let firefox_123_profile = adapter.get_profile(BrowserType::Firefox, 123);
        assert!(firefox_123_profile.is_some());

        let safari_15_6_1_profile = adapter.get_profile(BrowserType::Safari, 1561);
        assert!(safari_15_6_1_profile.is_some());

        let opera_90_profile = adapter.get_profile(BrowserType::Opera, 90);
        assert!(opera_90_profile.is_some());
    }

    #[test]
    fn test_profile_consistency() {
        // validateallconfigurefilehave consistentofstructure
        let profiles = vec![chrome_104(), firefox_102(), safari_15_6_1(), opera_89()];

        for profile in profiles {
            assert!(!profile.metadata.user_agent.is_empty());
            assert!(!profile.tls_config.cipher_suites.is_empty());
            assert!(!profile.http2_settings.is_empty());
            assert!(!profile.http2_settings_order.is_empty());
        }
    }

    #[test]
    fn test_mobile_variants() {
        // testingmove端variant
        let chrome_mobile_134 = chrome_mobile_134();
        assert_eq!(chrome_mobile_134.id(), "chrome_mobile_134");

        let firefox_mobile_130 = firefox_mobile_130();
        assert_eq!(firefox_mobile_130.id(), "firefox_mobile_130");
    }
}
