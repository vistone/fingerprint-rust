//! TCP 指纹同步测试
//!
//! 验证每次选择浏览器指纹时，TCP 指纹都会与 User-Agent 同步

use fingerprint::*;

#[test]
fn test_tcp_fingerprint_sync_with_random_fingerprint() {
    println!("\n=== TCP 指纹同步测试 ===\n");

    // 测试多次随机选择，确保每次 TCP 指纹都与 User-Agent 同步
    for i in 1..=5 {
        println!("【测试 {}】随机选择浏览器指纹", i);

        let result = get_random_fingerprint().unwrap();
        let user_agent = &result.user_agent;
        let profile = &result.profile;

        println!("  User-Agent: {}", user_agent);
        println!("  浏览器指纹: {}", result.hello_client_id);

        // 验证 TCP Profile 存在
        assert!(profile.tcp_profile.is_some(), "TCP Profile 应该存在");

        let tcp_profile = profile.tcp_profile.as_ref().unwrap();
        println!("  TCP Profile:");
        println!("    TTL: {}", tcp_profile.ttl);
        println!("    Window Size: {}", tcp_profile.window_size);

        // 验证一致性：从 User-Agent 推断操作系统，检查 TCP Profile 是否匹配
        let inferred_os =
            if user_agent.contains("Windows NT 10.0") || user_agent.contains("Windows NT 11.0") {
                "Windows"
            } else if user_agent.contains("Macintosh") || user_agent.contains("Mac OS X") {
                "macOS"
            } else if user_agent.contains("Linux") || user_agent.contains("X11") {
                "Linux"
            } else {
                "Unknown"
            };

        let expected_ttl = match inferred_os {
            "Windows" => 128,
            "macOS" | "Linux" => 64,
            _ => {
                println!("    ⚠️  无法推断操作系统，跳过验证");
                continue;
            }
        };

        assert_eq!(
            tcp_profile.ttl, expected_ttl,
            "TCP TTL ({}) 应该与 User-Agent 操作系统 ({}) 匹配",
            tcp_profile.ttl, inferred_os
        );

        println!("  ✅ TCP 指纹与 User-Agent 同步验证通过");
        println!("    操作系统: {}, TTL: {}", inferred_os, tcp_profile.ttl);
        println!();
    }

    println!("✅ 所有测试通过：TCP 指纹与浏览器指纹完全同步！\n");
}

#[test]
fn test_tcp_fingerprint_sync_by_browser() {
    println!("\n=== 按浏览器类型测试 TCP 指纹同步 ===\n");

    let browsers = vec!["chrome", "firefox"];

    for browser in browsers {
        println!("【测试 {}】", browser);

        let result = get_random_fingerprint_by_browser(browser).unwrap();
        let user_agent = &result.user_agent;
        let profile = &result.profile;

        println!("  User-Agent: {}", user_agent);

        // 验证 TCP Profile 存在并同步
        assert!(profile.tcp_profile.is_some(), "TCP Profile 应该存在");

        let tcp_profile = profile.tcp_profile.as_ref().unwrap();

        // 从 User-Agent 推断操作系统
        let inferred_os = if user_agent.contains("Windows") {
            "Windows"
        } else if user_agent.contains("Macintosh") || user_agent.contains("Mac OS X") {
            "macOS"
        } else if user_agent.contains("Linux") || user_agent.contains("X11") {
            "Linux"
        } else {
            "Unknown"
        };

        let expected_ttl = match inferred_os {
            "Windows" => 128,
            "macOS" | "Linux" => 64,
            _ => {
                println!("    ⚠️  无法推断操作系统，跳过验证");
                continue;
            }
        };

        assert_eq!(
            tcp_profile.ttl, expected_ttl,
            "TCP TTL ({}) 应该与 User-Agent 操作系统 ({}) 匹配",
            tcp_profile.ttl, inferred_os
        );

        println!(
            "  ✅ {} - TCP 指纹同步验证通过 (OS: {}, TTL: {})",
            browser, inferred_os, tcp_profile.ttl
        );
        println!();
    }

    println!("✅ 所有浏览器类型的 TCP 指纹同步测试通过！\n");
}

#[test]
fn test_tcp_fingerprint_sync_with_specific_os() {
    println!("\n=== 指定操作系统测试 TCP 指纹同步 ===\n");

    use fingerprint_core::types::OperatingSystem;

    let test_cases = vec![
        (OperatingSystem::Windows10, 128, "Windows"),
        (OperatingSystem::Linux, 64, "Linux"),
        (OperatingSystem::MacOS14, 64, "macOS"),
    ];

    for (os, expected_ttl, os_name) in test_cases {
        println!("【测试 {}】", os_name);

        let result = get_random_fingerprint_with_os(Some(os)).unwrap();
        let user_agent = &result.user_agent;
        let profile = &result.profile;

        println!("  User-Agent: {}", user_agent);

        // 验证 TCP Profile 存在
        assert!(profile.tcp_profile.is_some(), "TCP Profile 应该存在");

        let tcp_profile = profile.tcp_profile.as_ref().unwrap();

        assert_eq!(
            tcp_profile.ttl, expected_ttl,
            "TCP TTL ({}) 应该与指定的操作系统 ({}) 匹配",
            tcp_profile.ttl, os_name
        );

        println!(
            "  ✅ TCP 指纹同步验证通过 (OS: {}, TTL: {})",
            os_name, tcp_profile.ttl
        );
        println!();
    }

    println!("✅ 指定操作系统的 TCP 指纹同步测试通过！\n");
}
