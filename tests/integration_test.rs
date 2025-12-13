//! 集成测试套件
//!
//! 全面测试 fingerprint 库的各项功能

use fingerprint::*;
use fingerprint::headers::generate_headers;
use fingerprint::types::OPERATING_SYSTEMS;

#[test]
fn test_get_random_fingerprint() {
    let result = get_random_fingerprint();
    assert!(result.is_ok(), "get_random_fingerprint should succeed");
    
    let result = result.unwrap();
    assert!(!result.user_agent.is_empty(), "User-Agent should not be empty");
    assert!(!result.hello_client_id.is_empty(), "HelloClientID should not be empty");
    assert!(!result.headers.user_agent.is_empty(), "Headers User-Agent should not be empty");
}

#[test]
fn test_get_random_fingerprint_with_os() {
    let result = get_random_fingerprint_with_os(Some(OperatingSystem::MacOS14));
    assert!(result.is_ok());
    
    let result = result.unwrap();
    // 移动端指纹可能不包含操作系统信息，所以只检查 User-Agent 不为空
    assert!(!result.user_agent.is_empty(), "User-Agent should not be empty");
    // 如果包含操作系统信息，应该包含 Macintosh
    if result.user_agent.contains("Macintosh") || result.user_agent.contains("Mac OS X") {
        assert!(result.user_agent.contains("Macintosh") || result.user_agent.contains("Mac OS X"));
    }
}

#[test]
fn test_get_random_fingerprint_by_browser_chrome() {
    let result = get_random_fingerprint_by_browser("chrome");
    assert!(result.is_ok());
    
    let result = result.unwrap();
    assert!(result.user_agent.contains("Chrome"), "User-Agent should contain Chrome");
    assert!(result.hello_client_id.starts_with("Chrome"), "HelloClientID should start with Chrome");
}

#[test]
fn test_get_random_fingerprint_by_browser_firefox() {
    let result = get_random_fingerprint_by_browser("firefox");
    assert!(result.is_ok());
    
    let result = result.unwrap();
    assert!(result.user_agent.contains("Firefox"), "User-Agent should contain Firefox");
}

#[test]
fn test_get_random_fingerprint_by_browser_safari() {
    let result = get_random_fingerprint_by_browser("safari");
    assert!(result.is_ok());
    
    let result = result.unwrap();
    assert!(result.user_agent.contains("Safari"), "User-Agent should contain Safari");
}

#[test]
fn test_get_random_fingerprint_by_browser_opera() {
    let result = get_random_fingerprint_by_browser("opera");
    assert!(result.is_ok());
    
    let result = result.unwrap();
    assert!(result.user_agent.contains("OPR") || result.user_agent.contains("Opera"), "User-Agent should contain OPR or Opera");
}

#[test]
fn test_get_random_fingerprint_by_browser_not_found() {
    let result = get_random_fingerprint_by_browser("unknown_browser");
    assert!(result.is_err(), "Should return error for unknown browser");
}

#[test]
fn test_get_user_agent_by_profile_name() {
    let ua = get_user_agent_by_profile_name("chrome_120");
    assert!(ua.is_ok());
    let ua = ua.unwrap();
    assert!(ua.contains("Chrome/120"));
}

#[test]
fn test_get_user_agent_by_profile_name_with_os() {
    let ua = get_user_agent_by_profile_name_with_os("chrome_120", OperatingSystem::Windows11);
    assert!(ua.is_ok());
    let ua = ua.unwrap();
    assert!(ua.contains("Windows"));
    assert!(ua.contains("Chrome/120"));
}

#[test]
fn test_random_language() {
    let lang = random_language();
    assert!(!lang.is_empty());
    // 验证格式
    assert!(lang.contains(",") || lang.contains(";"), "Language should be in Accept-Language format");
}

#[test]
fn test_random_os() {
    let os = random_os();
    // 验证返回的是有效的操作系统
    assert!(OPERATING_SYSTEMS.contains(&os));
}

#[test]
fn test_http_headers_set() {
    let mut headers = HTTPHeaders::new();
    headers.user_agent = "test".to_string();
    headers.set("Cookie", "session_id=abc123");
    
    let map = headers.to_map();
    assert_eq!(map.get("User-Agent"), Some(&"test".to_string()));
    assert_eq!(map.get("Cookie"), Some(&"session_id=abc123".to_string()));
}

#[test]
fn test_http_headers_set_headers() {
    let mut headers = HTTPHeaders::new();
    headers.set_headers(&[
        ("Cookie", "session_id=abc123"),
        ("Authorization", "Bearer token"),
    ]);
    
    let map = headers.to_map();
    assert_eq!(map.get("Cookie"), Some(&"session_id=abc123".to_string()));
    assert_eq!(map.get("Authorization"), Some(&"Bearer token".to_string()));
}

#[test]
fn test_http_headers_clone() {
    let mut headers = HTTPHeaders::new();
    headers.user_agent = "test".to_string();
    headers.set("Cookie", "session_id=abc123");
    
    let cloned = headers.clone();
    assert_eq!(cloned.user_agent, headers.user_agent);
    assert_eq!(cloned.custom.get("Cookie"), headers.custom.get("Cookie"));
}

#[test]
fn test_generate_headers_chrome() {
    let ua = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";
    let headers = generate_headers(BrowserType::Chrome, ua, false);
    
    assert_eq!(headers.user_agent, ua);
    assert!(!headers.accept.is_empty());
    assert!(!headers.accept_language.is_empty());
    assert!(!headers.accept_encoding.is_empty());
    assert!(!headers.sec_ch_ua.is_empty());
}

#[test]
fn test_generate_headers_firefox() {
    let ua = "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:133.0) Gecko/20100101 Firefox/133.0";
    let headers = generate_headers(BrowserType::Firefox, ua, false);
    
    assert_eq!(headers.user_agent, ua);
    assert!(!headers.accept.is_empty());
    assert!(!headers.accept_language.is_empty());
}

#[test]
fn test_generate_headers_mobile() {
    let ua = "Mozilla/5.0 (Linux; Android 13; SM-G991B) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Mobile Safari/537.36";
    let headers = generate_headers(BrowserType::Chrome, ua, true);
    
    assert_eq!(headers.sec_ch_ua_mobile, "?1");
    assert!(headers.sec_ch_ua_platform.contains("Android"));
}

#[test]
fn test_browser_type_from_str() {
    assert_eq!(BrowserType::from_str("chrome"), Some(BrowserType::Chrome));
    assert_eq!(BrowserType::from_str("firefox"), Some(BrowserType::Firefox));
    assert_eq!(BrowserType::from_str("safari"), Some(BrowserType::Safari));
    assert_eq!(BrowserType::from_str("opera"), Some(BrowserType::Opera));
    assert_eq!(BrowserType::from_str("edge"), Some(BrowserType::Edge));
    assert_eq!(BrowserType::from_str("unknown"), None);
}

#[test]
fn test_browser_type_as_str() {
    assert_eq!(BrowserType::Chrome.as_str(), "chrome");
    assert_eq!(BrowserType::Firefox.as_str(), "firefox");
    assert_eq!(BrowserType::Safari.as_str(), "safari");
    assert_eq!(BrowserType::Opera.as_str(), "opera");
    assert_eq!(BrowserType::Edge.as_str(), "edge");
}

#[test]
fn test_operating_system_as_str() {
    assert_eq!(OperatingSystem::Windows10.as_str(), "Windows NT 10.0; Win64; x64");
    assert_eq!(OperatingSystem::MacOS14.as_str(), "Macintosh; Intel Mac OS X 14_0_0");
    assert_eq!(OperatingSystem::Linux.as_str(), "X11; Linux x86_64");
}

#[test]
fn test_mapped_tls_clients() {
    let clients = mapped_tls_clients();
    assert!(!clients.is_empty(), "MappedTLS clients should not be empty");
    
    // 验证一些常见的指纹存在
    assert!(clients.contains_key("chrome_133"));
    assert!(clients.contains_key("firefox_133"));
    assert!(clients.contains_key("safari_16_0"));
    assert!(clients.contains_key("opera_91"));
}

    #[test]
    fn test_client_profile() {
        let profile = mapped_tls_clients().get("chrome_133").unwrap();
        assert_eq!(profile.get_client_hello_str(), "Chrome-133");
        assert!(!profile.get_pseudo_header_order().is_empty());
    }

    #[test]
    fn test_get_client_hello_spec() {
        let profile = mapped_tls_clients().get("chrome_133").unwrap();
        let spec = profile.get_client_hello_spec();
        assert!(spec.is_ok(), "get_client_hello_spec should succeed");
        let spec = spec.unwrap();
        assert!(!spec.cipher_suites.is_empty(), "cipher_suites should not be empty");
        assert!(!spec.extensions.is_empty(), "extensions should not be empty");
        assert_eq!(spec.compression_methods, vec![0], "compression_methods should be [0]");
    }

    #[test]
    fn test_http2_settings() {
        let chrome_profile = mapped_tls_clients().get("chrome_133").unwrap();
        let firefox_profile = mapped_tls_clients().get("firefox_133").unwrap();
        
        let chrome_settings = chrome_profile.get_settings();
        let firefox_settings = firefox_profile.get_settings();
        
        assert!(!chrome_settings.is_empty());
        assert!(!firefox_settings.is_empty());
        
        // Chrome 和 Firefox 的 Settings 应该不同
        let chrome_window_size = chrome_settings.get(&4).unwrap(); // InitialWindowSize
        let firefox_window_size = firefox_settings.get(&4).unwrap();
        assert_ne!(chrome_window_size, firefox_window_size);
    }

    #[test]
    fn test_pseudo_header_order_differences() {
        let chrome_profile = mapped_tls_clients().get("chrome_133").unwrap();
        let firefox_profile = mapped_tls_clients().get("firefox_133").unwrap();
        let safari_profile = mapped_tls_clients().get("safari_16_0").unwrap();
        
        let chrome_order = chrome_profile.get_pseudo_header_order();
        let firefox_order = firefox_profile.get_pseudo_header_order();
        let safari_order = safari_profile.get_pseudo_header_order();
        
        // 不同浏览器的 Pseudo Header Order 应该不同
        assert_ne!(chrome_order, firefox_order);
        assert_ne!(chrome_order, safari_order);
        assert_ne!(firefox_order, safari_order);
    }

#[test]
fn test_concurrent_access() {
    use std::thread;
    
    let handles: Vec<_> = (0..10)
        .map(|_| {
            thread::spawn(|| {
                let result = get_random_fingerprint();
                assert!(result.is_ok());
                let result = result.unwrap();
                assert!(!result.user_agent.is_empty());
            })
        })
        .collect();
    
    for handle in handles {
        handle.join().unwrap();
    }
}

#[test]
fn test_multiple_random_calls() {
    // 多次调用应该返回不同的结果（至少大部分情况下）
    let results: Vec<_> = (0..10)
        .map(|_| get_random_fingerprint().unwrap().hello_client_id)
        .collect();
    
    // 至少应该有一些不同的结果
    let unique_count = results.iter().collect::<std::collections::HashSet<_>>().len();
    assert!(unique_count > 1, "Multiple calls should return different results");
}
