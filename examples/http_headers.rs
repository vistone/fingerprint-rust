//! HTTP Headers 和 User-Agent 示例
//!
//! 展示如何生成和使用 HTTP Headers 和 User-Agent
//!
//! 运行方式:
//! ```bash
//! cargo run --example http_headers
//! ```

use fingerprint::headers::generate_headers;
use fingerprint::*;

fn main() {
    println!("╔══════════════════════════════════════════════════════════╗");
    println!("║        HTTP Headers 和 User-Agent 示例                  ║");
    println!("╚══════════════════════════════════════════════════════════╝\n");

    // ========================================================================
    // 1. User-Agent 生成
    // ========================================================================
    println!("1️⃣  User-Agent 生成\n");

    println!("   根据 profile 名称获取 User-Agent:");
    let profiles = vec!["chrome_120", "firefox_133", "safari_16_0", "opera_91"];
    for profile in profiles {
        match get_user_agent_by_profile_name(profile) {
            Ok(ua) => println!("     {}: {}", profile, ua),
            Err(e) => println!("     {}: 错误 - {}", profile, e),
        }
    }

    println!("\n   指定操作系统获取 User-Agent:");
    match get_user_agent_by_profile_name_with_os("chrome_120", OperatingSystem::Windows11) {
        Ok(ua) => println!("     Chrome 120 (Windows 11): {}", ua),
        Err(e) => println!("     错误: {}", e),
    }

    match get_user_agent_by_profile_name_with_os("firefox_133", OperatingSystem::MacOS14) {
        Ok(ua) => println!("     Firefox 133 (macOS 14): {}", ua),
        Err(e) => println!("     错误: {}", e),
    }

    println!("\n   移动端 User-Agent:");
    let mobile_profiles = vec!["safari_ios_17_0", "okhttp4_android_13"];
    for profile in mobile_profiles {
        match get_user_agent_by_profile_name(profile) {
            Ok(ua) => println!("     {}: {}", profile, ua),
            Err(e) => println!("     {}: 错误 - {}", profile, e),
        }
    }

    println!("\n   随机操作系统:");
    for _ in 0..3 {
        let os = random_os();
        println!("     随机操作系统: {}", os.as_str());
    }

    // ========================================================================
    // 2. HTTP Headers 生成
    // ========================================================================
    println!("\n2️⃣  HTTP Headers 生成\n");

    println!("   Chrome Headers:");
    let chrome_ua = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";
    let chrome_headers = generate_headers(BrowserType::Chrome, chrome_ua, false);
    println!("     Accept: {}", chrome_headers.accept);
    println!("     Accept-Encoding: {}", chrome_headers.accept_encoding);
    println!("     Sec-CH-UA: {}", chrome_headers.sec_ch_ua);

    println!("\n   Firefox Headers:");
    let firefox_ua = "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:133.0) Gecko/20100101 Firefox/133.0";
    let firefox_headers = generate_headers(BrowserType::Firefox, firefox_ua, false);
    println!("     Accept: {}", firefox_headers.accept);
    println!("     Accept-Encoding: {}", firefox_headers.accept_encoding);

    println!("\n   移动端 Headers:");
    let mobile_ua = "Mozilla/5.0 (Linux; Android 13; SM-G991B) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Mobile Safari/537.36";
    let mobile_headers = generate_headers(BrowserType::Chrome, mobile_ua, true);
    println!("     Sec-CH-UA-Mobile: {}", mobile_headers.sec_ch_ua_mobile);
    println!("     Sec-CH-UA-Platform: {}", mobile_headers.sec_ch_ua_platform);

    // ========================================================================
    // 3. 随机语言
    // ========================================================================
    println!("\n3️⃣  随机语言\n");
    for _ in 0..5 {
        let lang = random_language();
        println!("     {}", lang);
    }

    // ========================================================================
    // 4. Headers 转换为 Map
    // ========================================================================
    println!("\n4️⃣  Headers 转换为 Map\n");
    let headers_map = chrome_headers.to_map();
    println!("   Headers 数量: {}", headers_map.len());
    for (key, value) in headers_map.iter().take(5) {
        println!("     {}: {}", key, value);
    }

    // ========================================================================
    // 5. 自定义 Headers
    // ========================================================================
    println!("\n5️⃣  自定义 Headers\n");
    let mut headers = HTTPHeaders::new();
    headers.user_agent = chrome_ua.to_string();
    headers.set("Cookie", "session_id=abc123");
    headers.set("X-API-Key", "your-api-key");
    headers.set_headers(&[("Custom-Header-1", "value1"), ("Custom-Header-2", "value2")]);

    let custom_map = headers.to_map();
    println!("   Cookie: {}", custom_map.get("Cookie").unwrap());
    println!("   X-API-Key: {}", custom_map.get("X-API-Key").unwrap());
    println!("   Custom-Header-1: {}", custom_map.get("Custom-Header-1").unwrap());

    println!("\n✅ 所有示例执行完成！\n");
}
