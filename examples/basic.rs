//! 基础使用示例
//!
//! 展示如何使用 fingerprint 库获取随机指纹和 HTTP Headers

use fingerprint::*;

fn main() {
    println!("=== fingerprint-rust 基础使用示例 ===\n");

    // 1. 最简单的方式：随机获取指纹和完整的 HTTP Headers
    println!("1. 随机获取指纹：");
    match get_random_fingerprint() {
        Ok(result) => {
            println!("   Profile: {}", result.hello_client_id);
            println!("   User-Agent: {}", result.user_agent);
            println!("   Accept-Language: {}", result.headers.accept_language);
        }
        Err(e) => println!("   错误: {}", e),
    }

    println!("\n2. 指定操作系统获取指纹：");
    match get_random_fingerprint_with_os(Some(OperatingSystem::MacOS14)) {
        Ok(result) => {
            println!("   Profile: {}", result.hello_client_id);
            println!("   User-Agent: {}", result.user_agent);
        }
        Err(e) => println!("   错误: {}", e),
    }

    println!("\n3. 指定浏览器类型获取指纹：");
    match get_random_fingerprint_by_browser("chrome") {
        Ok(result) => {
            println!("   Profile: {}", result.hello_client_id);
            println!("   User-Agent: {}", result.user_agent);
        }
        Err(e) => println!("   错误: {}", e),
    }

    println!("\n4. 获取 HTTP Headers Map：");
    match get_random_fingerprint() {
        Ok(result) => {
            let headers_map = result.headers.to_map();
            println!("   Headers 数量: {}", headers_map.len());
            for (key, value) in headers_map.iter().take(5) {
                println!("   {}: {}", key, value);
            }
        }
        Err(e) => println!("   错误: {}", e),
    }

    println!("\n5. 设置自定义 Headers：");
    match get_random_fingerprint() {
        Ok(mut result) => {
            result.headers.set("Cookie", "session_id=abc123");
            result.headers.set("Authorization", "Bearer token123");
            let headers_map = result.headers.to_map();
            println!("   Cookie: {}", headers_map.get("Cookie").unwrap_or(&"未设置".to_string()));
            println!("   Authorization: {}", headers_map.get("Authorization").unwrap_or(&"未设置".to_string()));
        }
        Err(e) => println!("   错误: {}", e),
    }
}
