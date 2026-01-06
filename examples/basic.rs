//! Basic usage example
//!
//! Demonstrates automatic synchronization of TCP fingerprints with browser fingerprints
//!
//! Shows how to use the fingerprint library to get random fingerprints and HTTP Headers

use fingerprint::*;

fn main() {
    println!("=== fingerprint-rust Basic Usage Example ===\n");

    // 1. Simplest way: randomly get fingerprint and complete HTTP Headers
    println!("1. Get random fingerprint (TCP fingerprint auto-synced):");
    match get_random_fingerprint() {
        Ok(result) => {
            println!("   Profile: {}", result.hello_client_id);
            println!("   User-Agent: {}", result.user_agent);
            println!("   Accept-Language: {}", result.headers.accept_language);
            
            // Show TCP fingerprint synchronization
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
                
                println!("   TCP Profile (auto-synced):");
                println!("     TTL: {} (OS: {})", tcp_profile.ttl, inferred_os);
                println!("     Window Size: {}", tcp_profile.window_size);
            }
        }
        Err(e) => println!("   Error: {}", e),
    }

    println!("\n2. Get fingerprint with specified OS (TCP fingerprint auto-synced):");
    match get_random_fingerprint_with_os(Some(OperatingSystem::MacOS14)) {
        Ok(result) => {
            println!("   Profile: {}", result.hello_client_id);
            println!("   User-Agent: {}", result.user_agent);
            if let Some(tcp_profile) = &result.profile.tcp_profile {
                println!("   TCP TTL: {} (macOS should be 64)", tcp_profile.ttl);
            }
        }
        Err(e) => println!("   Error: {}", e),
    }

    println!("\n3. Get fingerprint by browser type (TCP fingerprint auto-synced):");
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
        Err(e) => println!("   Error: {}", e),
    }

    println!("\n4. Get HTTP Headers Map:");
    match get_random_fingerprint() {
        Ok(result) => {
            let headers_map = result.headers.to_map();
            println!("   Headers count: {}", headers_map.len());
            for (key, value) in headers_map.iter().take(5) {
                println!("   {}: {}", key, value);
            }
        }
        Err(e) => println!("   Error: {}", e),
    }

    println!("\n5. Set custom Headers:");
    match get_random_fingerprint() {
        Ok(mut result) => {
            result.headers.set("Cookie", "session_id=abc123");
            result.headers.set("Authorization", "Bearer token123");
            let headers_map = result.headers.to_map();
            println!(
                "   Cookie: {}",
                headers_map.get("Cookie").unwrap_or(&"Not set".to_string())
            );
            println!(
                "   Authorization: {}",
                headers_map
                    .get("Authorization")
                    .unwrap_or(&"Not set".to_string())
            );
        }
        Err(e) => println!("   Error: {}", e),
    }
}
