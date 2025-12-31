//! 统一指纹生成演示
//!
//! 展示如何生成同步的浏览器指纹和 TCP 指纹

use fingerprint_profiles::profiles::{generate_unified_fingerprint, get_client_profile};
use fingerprint_headers::useragent::get_user_agent_by_profile_name;
use fingerprint_core::tcp::TcpProfile;
use fingerprint_core::types::OperatingSystem;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== 统一指纹生成演示 ===\n");

    // 演示 1: 使用统一指纹生成函数（推荐方式）
    println!("【演示 1】使用统一指纹生成函数\n");
    
    let user_agents = vec![
        ("Windows", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/135.0.0.0 Safari/537.36"),
        ("Linux", "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/135.0.0.0 Safari/537.36"),
        ("macOS", "Mozilla/5.0 (Macintosh; Intel Mac OS X 14_0_0) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/135.0.0.0 Safari/537.36"),
    ];

    for (os_name, user_agent) in user_agents {
        println!("操作系统: {}", os_name);
        println!("User-Agent: {}", user_agent);
        
        let profile = generate_unified_fingerprint("chrome_135", user_agent)?;
        
        if let Some(tcp_profile) = profile.tcp_profile {
            println!("TCP Profile:");
            println!("  TTL: {}", tcp_profile.ttl);
            println!("  Window Size: {}", tcp_profile.window_size);
            println!("  MSS: {:?}", tcp_profile.mss);
            println!("  Window Scale: {:?}", tcp_profile.window_scale);
        }
        
        println!("浏览器指纹: {}", profile.get_client_hello_str());
        println!();
    }

    // 演示 2: 手动同步 TCP Profile
    println!("【演示 2】手动同步 TCP Profile\n");
    
    let profile = get_client_profile("firefox_133")?;
    println!("原始 Profile: {}", profile.get_client_hello_str());
    
    let windows_ua = "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:133.0) Gecko/20100101 Firefox/133.0";
    let synced_profile = profile.with_synced_tcp_profile(windows_ua);
    
    if let Some(tcp_profile) = synced_profile.tcp_profile {
        println!("同步后的 TCP Profile:");
        println!("  TTL: {}", tcp_profile.ttl);
        println!("  Window Size: {}", tcp_profile.window_size);
        println!("  MSS: {:?}", tcp_profile.mss);
        println!("  Window Scale: {:?}", tcp_profile.window_scale);
    }
    println!();

    // 演示 3: 根据操作系统类型生成 TCP Profile
    println!("【演示 3】根据操作系统类型生成 TCP Profile\n");
    
    let operating_systems = vec![
        OperatingSystem::Windows10,
        OperatingSystem::Linux,
        OperatingSystem::MacOS14,
    ];

    for os in operating_systems {
        let tcp_profile = TcpProfile::for_os(os);
        println!("操作系统: {:?}", os);
        println!("TCP Profile:");
        println!("  TTL: {}", tcp_profile.ttl);
        println!("  Window Size: {}", tcp_profile.window_size);
        println!("  MSS: {:?}", tcp_profile.mss);
        println!("  Window Scale: {:?}", tcp_profile.window_scale);
        println!();
    }

    // 演示 4: 从 User-Agent 推断 TCP Profile
    println!("【演示 4】从 User-Agent 推断 TCP Profile\n");
    
    let test_user_agents = vec![
        ("Windows 10", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36"),
        ("Windows 11", "Mozilla/5.0 (Windows NT 11.0; Win64; x64) AppleWebKit/537.36"),
        ("macOS 14", "Mozilla/5.0 (Macintosh; Intel Mac OS X 14_0_0) AppleWebKit/537.36"),
        ("Linux", "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36"),
    ];

    for (os_name, user_agent) in test_user_agents {
        let tcp_profile = TcpProfile::from_user_agent(user_agent);
        println!("User-Agent 包含: {}", os_name);
        println!("推断的 TCP Profile:");
        println!("  TTL: {}", tcp_profile.ttl);
        println!("  Window Size: {}", tcp_profile.window_size);
        println!("  MSS: {:?}", tcp_profile.mss);
        println!("  Window Scale: {:?}", tcp_profile.window_scale);
        println!();
    }

    // 演示 5: 完整流程 - 生成 User-Agent 并同步指纹
    println!("【演示 5】完整流程 - 生成 User-Agent 并同步指纹\n");
    
    let user_agent = get_user_agent_by_profile_name("chrome_135")?;
    println!("生成的 User-Agent: {}", user_agent);
    
    let profile = generate_unified_fingerprint("chrome_135", &user_agent)?;
    
    println!("浏览器指纹: {}", profile.get_client_hello_str());
    if let Some(tcp_profile) = profile.tcp_profile {
        println!("同步的 TCP Profile:");
        println!("  TTL: {}", tcp_profile.ttl);
        println!("  Window Size: {}", tcp_profile.window_size);
        println!("  MSS: {:?}", tcp_profile.mss);
        println!("  Window Scale: {:?}", tcp_profile.window_scale);
        
        // 验证一致性
        let inferred_os = if tcp_profile.ttl == 128 {
            "Windows"
        } else if tcp_profile.ttl == 64 && tcp_profile.window_size == 65535 {
            if user_agent.contains("Macintosh") || user_agent.contains("Mac OS X") {
                "macOS"
            } else {
                "Linux"
            }
        } else {
            "Unknown"
        };
        
        println!("\n✅ 指纹一致性验证:");
        println!("  User-Agent 操作系统: {}", 
            if user_agent.contains("Windows") { "Windows" }
            else if user_agent.contains("Macintosh") || user_agent.contains("Mac OS X") { "macOS" }
            else if user_agent.contains("Linux") { "Linux" }
            else { "Unknown" }
        );
        println!("  TCP Profile 操作系统: {}", inferred_os);
    }

    println!("\n=== 演示完成 ===");
    Ok(())
}
