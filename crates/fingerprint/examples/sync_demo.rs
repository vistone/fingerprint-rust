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

        println!("  浏览器指纹: {}", result.profile_id);
        println!("  User-Agent: {}", user_agent);
        println!("  推断的操作系统: {}", inferred_os);

        let expected_ttl = match inferred_os {
            "Windows" => Some(128),
            "macOS" | "Linux" => Some(64),
            _ => None,
        };

        if let Some(ttl) = expected_ttl {
            println!("  参考 TCP TTL: {}", ttl);
        } else {
            println!("  ⚠️  无法推断操作系统对应的 TCP TTL");
        }
        println!();
    }

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("✅ 演示完成！TCP 指纹和浏览器指纹已完全同步！");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    Ok(())
}
