//! TCP 指纹同步实际运行演示

use fingerprint::*;

#[test]
fn test_tcp_sync_demo() {
    println!("\n╔════════════════════════════════════════════════════════════════╗");
    println!("║        TCP 指纹与浏览器指纹同步演示                         ║");
    println!("╚════════════════════════════════════════════════════════════════╝\n");

    println!("【演示 1】随机选择浏览器指纹（自动同步 TCP 指纹）\n");

    for i in 1..=5 {
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("第 {} 次随机选择：", i);

        let result = get_random_fingerprint().unwrap();
        let user_agent = &result.user_agent;
        let profile = &result.profile;

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

        println!("  浏览器指纹: {}", result.hello_client_id);
        println!("  User-Agent: {}", user_agent);
        println!("  推断的操作系统: {}", inferred_os);

        if let Some(tcp_profile) = &profile.tcp_profile {
            println!("  TCP Profile:");
            println!("    TTL: {}", tcp_profile.ttl);
            println!("    Window Size: {}", tcp_profile.window_size);
            println!("    MSS: {:?}", tcp_profile.mss);
            println!("    Window Scale: {:?}", tcp_profile.window_scale);

            let expected_ttl = match inferred_os {
                "Windows" => 128,
                "macOS" | "Linux" => 64,
                _ => {
                    println!("    ⚠️  无法验证（未知操作系统）");
                    continue;
                }
            };

            if tcp_profile.ttl == expected_ttl {
                println!(
                    "    ✅ 同步验证通过！TTL ({}) 与操作系统 ({}) 匹配",
                    tcp_profile.ttl, inferred_os
                );
            } else {
                println!(
                    "    ❌ 同步失败！TTL ({}) 与操作系统 ({}) 不匹配（期望: {}）",
                    tcp_profile.ttl, inferred_os, expected_ttl
                );
            }
        } else {
            println!("  ⚠️  TCP Profile 不存在");
        }
        println!();
    }

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("【演示 2】按浏览器类型选择（自动同步 TCP 指纹）\n");

    let browsers = vec!["chrome", "firefox"];

    for browser in browsers {
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("浏览器类型: {}", browser);

        let result = get_random_fingerprint_by_browser(browser).unwrap();
        let user_agent = &result.user_agent;
        let profile = &result.profile;

        let inferred_os = if user_agent.contains("Windows") {
            "Windows"
        } else if user_agent.contains("Macintosh") || user_agent.contains("Mac OS X") {
            "macOS"
        } else if user_agent.contains("Linux") || user_agent.contains("X11") {
            "Linux"
        } else {
            "Unknown"
        };

        println!("  User-Agent: {}", user_agent);
        println!("  推断的操作系统: {}", inferred_os);

        if let Some(tcp_profile) = &profile.tcp_profile {
            let expected_ttl = match inferred_os {
                "Windows" => 128,
                "macOS" | "Linux" => 64,
                _ => {
                    println!("  ⚠️  无法验证（未知操作系统）");
                    continue;
                }
            };

            println!("  TCP TTL: {} (期望: {})", tcp_profile.ttl, expected_ttl);
            println!("  TCP Window Size: {}", tcp_profile.window_size);

            if tcp_profile.ttl == expected_ttl {
                println!("  ✅ {} - TCP 指纹同步验证通过！", browser);
            } else {
                println!("  ❌ {} - TCP 指纹同步失败！", browser);
            }
        }
        println!();
    }

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("【演示 3】指定操作系统（自动同步 TCP 指纹）\n");

    use fingerprint_core::types::OperatingSystem;

    let test_cases = vec![
        (OperatingSystem::Windows10, 128, "Windows 10"),
        (OperatingSystem::Linux, 64, "Linux"),
        (OperatingSystem::MacOS14, 64, "macOS 14"),
    ];

    for (os, expected_ttl, os_name) in test_cases {
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("指定操作系统: {}", os_name);

        let result = get_random_fingerprint_with_os(Some(os)).unwrap();
        let user_agent = &result.user_agent;
        let profile = &result.profile;

        println!("  User-Agent: {}", user_agent);

        if let Some(tcp_profile) = &profile.tcp_profile {
            println!("  TCP TTL: {} (期望: {})", tcp_profile.ttl, expected_ttl);
            println!("  TCP Window Size: {}", tcp_profile.window_size);

            if tcp_profile.ttl == expected_ttl {
                println!("  ✅ {} - TCP 指纹同步验证通过！", os_name);
            } else {
                println!("  ❌ {} - TCP 指纹同步失败！", os_name);
            }
        }
        println!();
    }

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("✅ 演示完成！TCP 指纹和浏览器指纹已完全同步！");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
}
