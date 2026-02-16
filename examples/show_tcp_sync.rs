//! TCP 指纹演示
//!
//! 展示浏览器指纹和 TCP 配置

use fingerprint::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║        浏览器指纹演示                                        ║");
    println!("╚════════════════════════════════════════════════════════════════╝\n");

    // 演示 1: 随机选择浏览器指纹
    println!("【演示 1】随机选择浏览器指纹\n");
    
    for i in 1..=5 {
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("第 {} 次随机选择：", i);
        
        let result = get_random_fingerprint()?;
        let user_agent = &result.user_agent;
        
        // 从 User-Agent 推断操作系统
        let inferred_os = if user_agent.contains("Windows NT 10.0") || user_agent.contains("Windows NT 11.0") {
            "Windows"
        } else if user_agent.contains("Macintosh") || user_agent.contains("Mac OS X") {
            "macOS"
        } else if user_agent.contains("Linux") || user_agent.contains("X11") {
            "Linux"
        } else {
            "Unknown"
        };
        
        println!("  浏览器指纹: {}", result.profile_id);
        println!("  User-Agent: {}", user_agent);
        println!("  浏览器类型: {:?}", result.browser_type);
        println!("  推断的操作系统: {}", inferred_os);
        println!("  Accept-Language: {}", result.headers.accept_language);
        println!();
    }

    // 演示 2: 按浏览器类型选择
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("【演示 2】按浏览器类型选择\n");
    
    let browsers = vec!["chrome", "firefox"];
    
    for browser in browsers {
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("浏览器类型: {}", browser);
        
        let result = get_random_fingerprint_by_browser(browser)?;
        let user_agent = &result.user_agent;
        
        let inferred_os = if user_agent.contains("Windows") {
            "Windows"
        } else if user_agent.contains("Macintosh") || user_agent.contains("Mac OS X") {
            "macOS"
        } else if user_agent.contains("Linux") || user_agent.contains("X11") {
            "Linux"
        } else {
            "Unknown"
        };
        
        println!("  Profile ID: {}", result.profile_id);
        println!("  User-Agent: {}", user_agent);
        println!("  推断的操作系统: {}", inferred_os);
        println!("  ✅ {} - 指纹生成成功！", browser);
        println!();
    }

    // 演示 3: 指定操作系统
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("【演示 3】指定操作系统\n");
    
    let test_cases = vec![
        (OperatingSystem::Windows10, "Windows 10"),
        (OperatingSystem::Linux, "Linux"),
        (OperatingSystem::MacOS14, "macOS 14"),
    ];
    
    for (os, os_name) in test_cases {
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("指定操作系统: {}", os_name);
        
        let result = get_random_fingerprint_with_os(Some(os))?;
        let user_agent = &result.user_agent;
        
        println!("  Profile ID: {}", result.profile_id);
        println!("  User-Agent: {}", user_agent);
        println!("  ✅ {} - 指纹生成成功！", os_name);
        println!();
    }

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("✅ 演示完成！");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    
    Ok(())
}
