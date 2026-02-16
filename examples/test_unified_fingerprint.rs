//! Unified Fingerprint Generation Demo
//!
//! Demonstrates how to generate browser fingerprints with HTTP headers

use fingerprint::{get_random_fingerprint, get_random_fingerprint_with_os, mapped_tls_clients, OperatingSystem};
use fingerprint_core::tcp::TcpProfile;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Unified Fingerprint Generation Demo ===\n");

    // Demo 1: Using random fingerprint generation function (recommended approach)
    println!("【Demo 1】Using random fingerprint generation\n");
    
    for i in 1..=3 {
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("Random selection #{}", i);
        
        let result = get_random_fingerprint()?;
        let user_agent = &result.user_agent;
        
        // Infer operating system from User-Agent
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
        println!("  Browser Type: {:?}", result.browser_type);
        println!("  User-Agent: {}", user_agent);
        println!("  Inferred OS: {}", inferred_os);
        println!("  Accept-Language: {}", result.headers.accept_language);
        println!();
    }

    // Demo 2: Using browser profiles directly
    println!("【Demo 2】Using browser profiles directly\n");
    
    let profiles = mapped_tls_clients();
    
    for (name, profile) in profiles.iter() {
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("Profile: {}", name);
        println!("  Browser: {}", profile.metadata.browser_name);
        println!("  Version: {}", profile.metadata.browser_version);
        println!("  Platform: {}", profile.metadata.platform);
        println!("  Mobile: {}", profile.metadata.is_mobile);
        
        if let Ok(spec) = profile.get_client_hello_spec() {
            println!("  TLS Cipher Suites: {}", spec.cipher_suites.len());
            println!("  TLS Extensions: {}", spec.extensions.len());
        }
        println!();
    }

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

    // Demo 5: Complete workflow with specified OS
    println!("【Demo 5】Complete workflow - Fingerprint with specified OS\n");
    
    let result = get_random_fingerprint_with_os(Some(OperatingSystem::Windows10))?;
    println!("Generated User-Agent: {}", result.user_agent);
    println!("Profile ID: {}", result.profile_id);
    println!("Browser Type: {:?}", result.browser_type);
    
    // Verify consistency
    let inferred_os = if result.user_agent.contains("Windows") {
        "Windows"
    } else if result.user_agent.contains("Macintosh") || result.user_agent.contains("Mac OS X") {
        "macOS"
    } else if result.user_agent.contains("Linux") {
        "Linux"
    } else {
        "Unknown"
    };
    
    println!("\n✅ Fingerprint Consistency Verification:");
    println!("  User-Agent Operating System: {}", inferred_os);

    println!("\n=== Demo Completed ===");
    Ok(())
}