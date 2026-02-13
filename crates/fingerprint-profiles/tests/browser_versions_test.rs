#[cfg(test)]
mod tests {
    use fingerprint_core::fingerprint::Fingerprint;
    use fingerprint_profiles::profiles::*;

    #[test]
    fn test_chrome_versions_exist() {
        // 测试新增的Chrome版本函数是否存在且能正常调用
        let chrome_104 = chrome_104();
        assert_eq!(chrome_104.id(), "chrome_104");

        let chrome_117 = chrome_117();
        assert_eq!(chrome_117.id(), "chrome_117");

        let chrome_133 = chrome_133();
        assert_eq!(chrome_133.id(), "chrome_133");
    }

    #[test]
    fn test_firefox_versions_exist() {
        // 测试新增的Firefox版本函数
        let firefox_102 = firefox_102();
        assert_eq!(firefox_102.id(), "firefox_102");

        let firefox_123 = firefox_123();
        assert_eq!(firefox_123.id(), "firefox_123");
    }

    #[test]
    fn test_safari_versions_exist() {
        // 测试新增的Safari版本函数
        let safari_15_6_1 = safari_15_6_1();
        assert_eq!(safari_15_6_1.id(), "safari_15_6_1");

        let safari_ios_18_5 = safari_ios_18_5();
        assert_eq!(safari_ios_18_5.id(), "safari_ios_18_5");
    }

    #[test]
    fn test_opera_versions_exist() {
        // 测试新增的Opera版本函数
        let opera_89 = opera_89();
        assert_eq!(opera_89.id(), "opera_89");

        let opera_90 = opera_90();
        assert_eq!(opera_90.id(), "opera_90");
    }

    #[test]
    fn test_version_registry_completeness() {
        use fingerprint_profiles::version_registry::VersionRegistry;

        let registry = VersionRegistry::new();

        // 验证Chrome版本注册
        assert!(registry.get_chrome_version(104).is_some());
        assert!(registry.get_chrome_version(117).is_some());
        assert!(registry.get_chrome_version(133).is_some());

        // 验证Firefox版本注册
        assert!(registry.get_firefox_version(102).is_some());
        assert!(registry.get_firefox_version(123).is_some());

        // 验证Safari版本注册
        assert!(registry.get_safari_version(156).is_some());
        assert!(registry.get_safari_version(1850).is_some()); // iOS 18.5

        // 验证Opera版本注册
        assert!(registry.get_opera_version(89).is_some());
        assert!(registry.get_opera_version(90).is_some());
    }

    #[test]
    fn test_version_adapter_loading() {
        use fingerprint_profiles::version_adapter::VersionAdapter;

        let adapter = VersionAdapter::new();

        // 测试加载新增的版本
        let chrome_104_profile = adapter.load_profile_by_version("Chrome", 104);
        assert!(chrome_104_profile.is_some());

        let firefox_123_profile = adapter.load_profile_by_version("Firefox", 123);
        assert!(firefox_123_profile.is_some());

        let safari_15_6_1_profile = adapter.load_profile_by_version("Safari", 1561);
        assert!(safari_15_6_1_profile.is_some());

        let opera_90_profile = adapter.load_profile_by_version("Opera", 90);
        assert!(opera_90_profile.is_some());
    }

    #[test]
    fn test_profile_consistency() {
        // 验证所有配置文件具有一致的结构
        let profiles = vec![chrome_104(), firefox_102(), safari_15_6_1(), opera_89()];

        for profile in profiles {
            // 确保都有有效的ClientHello ID
            assert!(!profile.client_hello_id.name.is_empty());
            assert!(!profile.client_hello_id.version.is_empty());

            // 确保都有HTTP/2设置
            assert!(!profile.settings.is_empty());
            assert!(!profile.settings_order.is_empty());
        }
    }

    #[test]
    fn test_mobile_variants() {
        // 测试移动端变体
        let chrome_mobile_134 = chrome_mobile_134();
        assert_eq!(chrome_mobile_134.id(), "chrome_mobile_134");

        let firefox_mobile_130 = firefox_mobile_130();
        assert_eq!(firefox_mobile_130.id(), "firefox_mobile_130");
    }
}
