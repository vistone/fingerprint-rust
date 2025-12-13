//! User-Agent 生成示例
//!
//! 展示如何生成不同浏览器的 User-Agent

use fingerprint::*;

fn main() {
    println!("=== User-Agent 生成示例 ===\n");

    // 1. 根据 profile 名称获取 User-Agent
    println!("1. 根据 profile 名称获取 User-Agent：");
    let profiles = vec!["chrome_120", "firefox_133", "safari_16_0", "opera_91"];
    for profile in profiles {
        match get_user_agent_by_profile_name(profile) {
            Ok(ua) => println!("   {}: {}", profile, ua),
            Err(e) => println!("   {}: 错误 - {}", profile, e),
        }
    }

    println!("\n2. 指定操作系统获取 User-Agent：");
    match get_user_agent_by_profile_name_with_os("chrome_120", OperatingSystem::Windows11) {
        Ok(ua) => println!("   Chrome 120 (Windows 11): {}", ua),
        Err(e) => println!("   错误: {}", e),
    }

    match get_user_agent_by_profile_name_with_os("firefox_133", OperatingSystem::MacOS14) {
        Ok(ua) => println!("   Firefox 133 (macOS 14): {}", ua),
        Err(e) => println!("   错误: {}", e),
    }

    println!("\n3. 移动端 User-Agent：");
    let mobile_profiles = vec!["safari_ios_17_0", "okhttp4_android_13"];
    for profile in mobile_profiles {
        match get_user_agent_by_profile_name(profile) {
            Ok(ua) => println!("   {}: {}", profile, ua),
            Err(e) => println!("   {}: 错误 - {}", profile, e),
        }
    }

    println!("\n4. 随机操作系统：");
    for _ in 0..3 {
        let os = random_os();
        println!("   随机操作系统: {}", os.as_str());
    }
}
