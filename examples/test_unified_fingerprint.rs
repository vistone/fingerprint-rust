//! Unified Fingerprint Generation Demo
//!
//! Demonstrates how to generate synchronized browser fingerprints and TCP fingerprints

use fingerprint_profiles::profiles::{generate_unified_fingerprint, get_client_profile};
use fingerprint_headers::useragent::get_user_agent_by_profile_name;
use fingerprint_core::tcp::TcpProfile;
use fingerprint_core::types::OperatingSystem;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Unified Fingerprint Generation Demo ===\n");

    // Demo 1: Using unified fingerprint generation function (recommended approach)
    println!("【Demo 1】Using unified fingerprint generation function\n");
    
    let user_agents = vec![
        ("Windows", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/135.0.0.0 Safari/537.36"),
        ("Linux", "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/135.0.0.0 Safari/537.36"),
        ("macOS", "Mozilla/5.0 (Macintosh; Intel Mac OS X 14_0_0) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/135.0.0.0 Safari/537.36"),
    ];

    for (os_name, user_agent) in user_agents {
        println!("Operating System: {}", os_name);
        println!("User-Agent: {}", user_agent);
        
        let profile = generate_unified_fingerprint("chrome_135", user_agent)?;
        
        if let Some(tcp_profile) = profile.tcp_profile {
            println!("TCP Profile:");
            println!("  TTL: {}", tcp_profile.ttl);
            println!("  Window Size: {}", tcp_profile.window_size);
            println!("  MSS: {:?}", tcp_profile.mss);
            println!("  Window Scale: {:?}", tcp_profile.window_scale);
        }
        
        println!("Browser Fingerprint: {}", profile.get_client_hello_str());
        println!();
    }

    // Demo 2: Manual TCP Profile synchronization
    println!("【Demo 2】Manual TCP Profile synchronization\n");
    
    let profile = get_client_profile("firefox_133")?;
    println!("Original Profile: {}", profile.get_client_hello_str());
    
    let windows_ua = "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:133.0) Gecko/20100101 Firefox/133.0";
    let synced_profile = profile.with_synced_tcp_profile(windows_ua);
    
    if let Some(tcp_profile) = synced_profile.tcp_profile {
        println!("Synchronized TCP Profile:");
        println!("  TTL: {}", tcp_profile.ttl);
        println!("  Window Size: {}", tcp_profile.window_size);
        println!("  MSS: {:?}", tcp_profile.mss);
        println!("  Window Scale: {:?}", tcp_profile.window_scale);
    }
    println!();

    // Demo 3: Generate TCP Profile based on operating system type
    println!("【Demo 3】Generate TCP Profile based on operating system type\n");
    
    let operating_systems = vec![
        OperatingSystem::Windows10,
        OperatingSystem::Linux,
        OperatingSystem::MacOS14,
    ];

    for os in operating_systems {
        let tcp_profile = TcpProfile::for_os(os);
        println!("Operating System: {:?}", os);
        println!("TCP Profile:");
        println!("  TTL: {}", tcp_profile.ttl);
        println!("  Window Size: {}", tcp_profile.window_size);
        println!("  MSS: {:?}", tcp_profile.mss);
        println!("  Window Scale: {:?}", tcp_profile.window_scale);
        println!();
    }

    // Demo 4: Infer TCP Profile from User-Agent
    println!("【Demo 4】Infer TCP Profile from User-Agent\n");
    
    let test_user_agents = vec![
        ("Windows 10", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36"),
        ("Windows 11", "Mozilla/5.0 (Windows NT 11.0; Win64; x64) AppleWebKit/537.36"),
        ("macOS 14", "Mozilla/5.0 (Macintosh; Intel Mac OS X 14_0_0) AppleWebKit/537.36"),
        ("Linux", "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36"),
    ];

    for (os_name, user_agent) in test_user_agents {
        let tcp_profile = TcpProfile::from_user_agent(user_agent);
        println!("User-Agent contains: {}", os_name);
        println!("Inferred TCP Profile:");
        println!("  TTL: {}", tcp_profile.ttl);
        println!("  Window Size: {}", tcp_profile.window_size);
        println!("  MSS: {:?}", tcp_profile.mss);
        println!("  Window Scale: {:?}", tcp_profile.window_scale);
        println!();
    }

    // Demo 5: Complete workflow - Generate User-Agent and synchronize fingerprint
    println!("【Demo 5】Complete workflow - Generate User-Agent and synchronize fingerprint\n");
    
    let user_agent = get_user_agent_by_profile_name("chrome_135")?;
    println!("Generated User-Agent: {}", user_agent);
    
    let profile = generate_unified_fingerprint("chrome_135", &user_agent)?;
    
    println!("Browser Fingerprint: {}", profile.get_client_hello_str());
    if let Some(tcp_profile) = profile.tcp_profile {
        println!("Synchronized TCP Profile:");
        println!("  TTL: {}", tcp_profile.ttl);
        println!("  Window Size: {}", tcp_profile.window_size);
        println!("  MSS: {:?}", tcp_profile.mss);
        println!("  Window Scale: {:?}", tcp_profile.window_scale);
        
        // Verify consistency
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
        
        println!("\n✅ Fingerprint Consistency Verification:");
        println!("  User-Agent Operating System: {}", 
            if user_agent.contains("Windows") { "Windows" }
            else if user_agent.contains("Macintosh") || user_agent.contains("Mac OS X") { "macOS" }
            else if user_agent.contains("Linux") { "Linux" }
            else { "Unknown" }
        );
        println!("  TCP Profile Operating System: {}", inferred_os);
    }

    println!("\n=== Demo Completed ===");
    Ok(())
}