//! 统一指纹生成演示
//!
//! 展示如何生成浏览器指纹和 HTTP 头

use fingerprint::{get_random_fingerprint, get_random_fingerprint_with_os, mapped_tls_clients, OperatingSystem};
use fingerprint_core::tcp::TcpProfile;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== 统一指纹生成演示 ===\n");

    // 演示 1: 使用随机指纹生成函数（推荐方式）
    println!("【演示 1】使用随机指纹生成\n");
    
    for i in 1..=3 {
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("随机选择 #{}", i);
        
        let result = get_random_fingerprint()?;
        let user_agent = &result.user_agent;
        
        // 从 User-Agent 推断操作系统
        let inferred_os = if user_agent.contains("Windows") {
            "Windows"
        } else if user_agent.contains("Macintosh") || user_agent.contains("Mac OS X") {
            "macOS"
        } else if user_agent.contains("Linux") {
            "Linux"
        } else {
            "Unknown"
        };
        
        println!("  Profile ID: {}", result.profile_id);
        println!("  浏览器类型: {:?}", result.browser_type);
        println!("  User-Agent: {}", user_agent);
        println!("  推断的操作系统: {}", inferred_os);
        println!("  Accept-Language: {}", result.headers.accept_language);
        println!();
    }

    // 演示 2: 直接使用浏览器配置
    println!("【演示 2】直接使用浏览器配置\n");
    
    let profiles = mapped_tls_clients();
    
    for (name, profile) in profiles.iter() {
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("配置: {}", name);
        println!("  浏览器: {}", profile.metadata.browser_name);
        println!("  版本: {}", profile.metadata.browser_version);
        println!("  平台: {}", profile.metadata.platform);
        println!("  移动端: {}", profile.metadata.is_mobile);
        
        if let Ok(spec) = profile.get_client_hello_spec() {
            println!("  TLS 密码套件: {}", spec.cipher_suites.len());
            println!("  TLS 扩展: {}", spec.extensions.len());
        }
        println!();
    }

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

    // 演示 5: 完整流程 - 指定操作系统生成指纹
    println!("【演示 5】完整流程 - 指定操作系统生成指纹\n");
    
    let result = get_random_fingerprint_with_os(Some(OperatingSystem::Windows10))?;
    println!("生成的 User-Agent: {}", result.user_agent);
    println!("Profile ID: {}", result.profile_id);
    println!("浏览器类型: {:?}", result.browser_type);
    
    // 验证一致性
    let inferred_os = if result.user_agent.contains("Windows") {
        "Windows"
    } else if result.user_agent.contains("Macintosh") || result.user_agent.contains("Mac OS X") {
        "macOS"
    } else if result.user_agent.contains("Linux") {
        "Linux"
    } else {
        "Unknown"
    };
    
    println!("\n✅ 指纹一致性验证:");
    println!("  User-Agent 操作系统: {}", inferred_os);

    println!("\n=== 演示完成 ===");
    Ok(())
}
