//! HTTP Headers 示例
//!
//! 展示如何生成和使用 HTTP Headers

use fingerprint::headers::generate_headers;
use fingerprint::*;

fn main() {
    println!("=== HTTP Headers 示例 ===\n");

    // 1. 生成不同浏览器的 Headers
    println!("1. Chrome Headers：");
    let chrome_ua = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";
    let chrome_headers = generate_headers(BrowserType::Chrome, chrome_ua, false);
    println!("   Accept: {}", chrome_headers.accept);
    println!("   Accept-Encoding: {}", chrome_headers.accept_encoding);
    println!("   Sec-CH-UA: {}", chrome_headers.sec_ch_ua);

    println!("\n2. Firefox Headers：");
    let firefox_ua =
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:133.0) Gecko/20100101 Firefox/133.0";
    let firefox_headers = generate_headers(BrowserType::Firefox, firefox_ua, false);
    println!("   Accept: {}", firefox_headers.accept);
    println!("   Accept-Encoding: {}", firefox_headers.accept_encoding);

    println!("\n3. 移动端 Headers：");
    let mobile_ua = "Mozilla/5.0 (Linux; Android 13; SM-G991B) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Mobile Safari/537.36";
    let mobile_headers = generate_headers(BrowserType::Chrome, mobile_ua, true);
    println!("   Sec-CH-UA-Mobile: {}", mobile_headers.sec_ch_ua_mobile);
    println!(
        "   Sec-CH-UA-Platform: {}",
        mobile_headers.sec_ch_ua_platform
    );

    println!("\n4. 随机语言：");
    for _ in 0..5 {
        let lang = random_language();
        println!("   {}", lang);
    }

    println!("\n5. Headers 转换为 Map：");
    let headers_map = chrome_headers.to_map();
    println!("   Headers 数量: {}", headers_map.len());
    for (key, value) in headers_map.iter() {
        println!("   {}: {}", key, value);
    }

    println!("\n6. 自定义 Headers：");
    let mut headers = HTTPHeaders::new();
    headers.user_agent = chrome_ua.to_string();
    headers.set("Cookie", "session_id=abc123");
    headers.set("X-API-Key", "your-api-key");
    headers.set_headers(&[("Custom-Header-1", "value1"), ("Custom-Header-2", "value2")]);

    let custom_map = headers.to_map();
    println!("   Cookie: {}", custom_map.get("Cookie").unwrap());
    println!("   X-API-Key: {}", custom_map.get("X-API-Key").unwrap());
    println!(
        "   Custom-Header-1: {}",
        custom_map.get("Custom-Header-1").unwrap()
    );
}
