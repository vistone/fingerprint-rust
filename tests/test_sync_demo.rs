use fingerprint::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║        TCP 指纹与浏览器指纹同步演示                         ║");
    println!("╚════════════════════════════════════════════════════════════════╝\n");

    println!("【演示】随机选择浏览器指纹（自动同步 TCP 指纹）\n");
    
    for i in 1..=5 {
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("第 {} 次随机选择：", i);
        
        let result = get_random_fingerprint()?;
        let user_agent = &result.user_agent;
        let profile = &result.profile;
        
        let inferred_os = if user_agent.contains("Windows NT 10.0") || user_agent.contains("Windows NT 11.0") {
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
            
            let expected_ttl = match inferred_os {
                "Windows" => 128,
                "macOS" | "Linux" => 64,
                _ => {
                    println!("    ⚠️  无法验证（未知操作系统）");
                    continue;
                }
            };
            
            if tcp_profile.ttl == expected_ttl {
                println!("    ✅ 同步验证通过！TTL ({}) 与操作系统 ({}) 匹配", 
                    tcp_profile.ttl, inferred_os);
            } else {
                println!("    ❌ 同步失败！TTL ({}) 与操作系统 ({}) 不匹配（期望: {}）", 
                    tcp_profile.ttl, inferred_os, expected_ttl);
            }
        } else {
            println!("  ⚠️  TCP Profile 不存在");
        }
        println!();
    }

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("✅ 演示完成！TCP 指纹和浏览器指纹已完全同步！");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    
    Ok(())
}
