//! 基础使用示例
//!
//! 展示 TCP 指纹与浏览器指纹的自动同步
//!
//! 展示如何使用 fingerprint 库获取随机指纹和 HTTP Headers

use fingerprint::*;

fn main() {
    println!("=== fingerprint-rust 基础使用示例 ===\n");

    // 1. 最简单的方式：随机获取指纹和完整的 HTTP Headers
    println!("1. 随机获取指纹（TCP 指纹自动同步）：");
    match get_random_fingerprint() {
        Ok(result) => {
            println!("   Profile: {}", result.hello_client_id);
            println!("   User-Agent: {}", result.user_agent);
            println!("   Accept-Language: {}", result.headers.accept_language);
            
            // 展示 TCP 指纹同步
            if let Some(tcp_profile) = &result.profile.tcp_profile {
                let inferred_os = if result.user_agent.contains("Windows") {
                    "Windows"
                } else if result.user_agent.contains("Macintosh") || result.user_agent.contains("Mac OS X") {
                    "macOS"
                } else if result.user_agent.contains("Linux") || result.user_agent.contains("X11") {
                    "Linux"
                } else {
                    "Unknown"
                };
                
                println!("   TCP Profile (已自动同步):");
                println!("     TTL: {} (操作系统: {})", tcp_profile.ttl, inferred_os);
                println!("     Window Size: {}", tcp_profile.window_size);
            }
        }
        Err(e) => println!("   错误: {}", e),
    }

    println!("\n2. 指定操作系统获取指纹（TCP 指纹自动同步）：");
    match get_random_fingerprint_with_os(Some(OperatingSystem::MacOS14)) {
        Ok(result) => {
            println!("   Profile: {}", result.hello_client_id);
            println!("   User-Agent: {}", result.user_agent);
            if let Some(tcp_profile) = &result.profile.tcp_profile {
                println!("   TCP TTL: {} (macOS 应为 64)", tcp_profile.ttl);
            }
        }
        Err(e) => println!("   错误: {}", e),
    }

    println!("\n3. 指定浏览器类型获取指纹（TCP 指纹自动同步）：");
    match get_random_fingerprint_by_browser("chrome") {
        Ok(result) => {
            println!("   Profile: {}", result.hello_client_id);
            println!("   User-Agent: {}", result.user_agent);
            if let Some(tcp_profile) = &result.profile.tcp_profile {
                let os = if result.user_agent.contains("Windows") { "Windows (TTL=128)" }
                    else if result.user_agent.contains("Macintosh") { "macOS (TTL=64)" }
                    else { "Linux (TTL=64)" };
                println!("   TCP TTL: {} ({})", tcp_profile.ttl, os);
            }
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
            println!(
                "   Cookie: {}",
                headers_map.get("Cookie").unwrap_or(&"未设置".to_string())
            );
            println!(
                "   Authorization: {}",
                headers_map
                    .get("Authorization")
                    .unwrap_or(&"未设置".to_string())
            );
        }
        Err(e) => println!("   错误: {}", e),
    }
}
