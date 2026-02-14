//! DNS Module and HTTP Client Full Integration Example
//!
//! Shows how to combine DNS pre-resolution service and HTTP client to achieve intelligent domain resolution and request optimization
//!
//! Usage:
//!   cargo run --example dns_full_integration --features dns,rustls-tls,http2

#[cfg(feature = "dns")]
use fingerprint::{
    chrome_133, DNSCache, DNSConfig, DNSResolver, DNSService, DomainIPs, HttpClient,
    HttpClientConfig, IPInfo,
};
#[cfg(feature = "dns")]
use std::sync::Arc;
#[cfg(feature = "dns")]
use std::time::Duration;

#[cfg(feature = "dns")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 DNS Module and HTTP Client Full Integration Example");
    println!("=" .repeat(70));
    println!();

    // === Scenario 1: Using DNS Cache to Accelerate HTTP Requests ===
    println!("📦 Scenario 1: DNS Cache Acceleration");
    println!("-" .repeat(70));

    // Create DNS cache
    let dns_cache = DNSCache::new(Duration::from_secs(300));

    // Create domain list
    let domains = vec!["www.google.com", "www.github.com"];

    // Pre-resolve domains and populate cache
    let resolver = DNSResolver::new(Duration::from_secs(4));
    println!("🔍 Pre-resolving domains...");
    for domain in &domains {
        match resolver.resolve(domain).await {
            Ok(result) => {
                println!(
                    "   ✅ {}: {} IPv4, {} IPv6",
                    domain,
                    result.ips.ipv4.len(),
                    result.ips.ipv6.len()
                );
                dns_cache.put(domain, result.ips);
            }
            Err(e) => {
                println!("   ❌ {} resolution failed: {}", domain, e);
            }
        }
    }

    // Show cache statistics
    let (total, expired) = dns_cache.stats();
    println!("   📊 DNS cache statistics: {} domains, {} expired", total, expired);
    println!();

    // Create HTTP client
    let profile = chrome_133();
    let config = HttpClientConfig {
        user_agent: "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36".to_string(),
        prefer_http2: true,
        profile: Some(profile),
        ..Default::default()
    };
    let client = HttpClient::new(config);

    // Send HTTP requests (DNS is already cached)
    println!("🌐 Sending HTTP requests (using pre-resolved DNS cache)...");
    for domain in &domains {
        let url = format!("https://{}/", domain);
        println!("   Request: {}", url);

        match client.get(&url) {
            Ok(response) => {
                println!("      ✅ Status code: {}", response.status_code);
                println!("      ✅ HTTP version: {}", response.http_version);
                println!("      ✅ Response size: {} bytes", response.body.len());
            }
            Err(e) => {
                println!("      ❌ Request failed: {}", e);
            }
        }
        println!();
    }

    // === Scenario 2: DNS Pre-resolution Service Automatic Maintenance ===
    println!("📦 Scenario 2: DNS Pre-resolution Service (Automatic Background Maintenance)");
    println!("-" .repeat(70));
    println!("💡 Note: This scenario requires IPInfo token and longer runtime, demonstrating configuration only");
    println!();

    // Create DNS service configuration
    let dns_config = DNSConfig::new(
        "your-ipinfo-token", // Requires real IPInfo token
        &["google.com", "github.com"],
    );

    println!("⚙️  DNS Service Configuration:");
    println!("   - IPInfo Token: {} (needs to be replaced with real token)", dns_config.ipinfo_token);
    println!("   - Domain List: {:?}", dns_config.domain_list);
    println!("   - Check Interval: {}", dns_config.interval);
    println!("   - Max Concurrency: {}", dns_config.max_concurrency);
    println!("   - DNS Timeout: {}", dns_config.dns_timeout);
    println!();

    // Note: For actual use:
    // 1. Obtain real IPInfo token
    // 2. Start DNS service: service.start().await?
    // 3. Regularly read resolution results from dns_output directory
    // 4. Use these pre-resolved IPs in HTTP requests

    println!("📝 Actual Usage Steps:");
    println!("   1. Get IPInfo token: https://ipinfo.io/");
    println!("   2. Configure DNS service (see examples/dns_service.rs)");
    println!("   3. Start service: service.start().await");
    println!("   4. Service automatically maintains domain IP list");
    println!("   5. Read latest IPs from dns_output/*.json");
    println!("   6. Prioritize using these IPs in HTTP requests");
    println!();

    // === Scenario 3: Intelligent IP Selection (Based on Geographic Location) ===
    println!("📦 Scenario 3: Intelligent IP Selection (Example)");
    println!("-" .repeat(70));

    // Simulate domain IP information obtained from DNS service
    let mut domain_ips = DomainIPs::new();

    // Add some example IP information (should actually come from DNS service)
    domain_ips.ipv4.push(IPInfo {
        ip: "142.250.191.14".to_string(),
        hostname: None,
        city: Some("Mountain View".to_string()),
        region: Some("California".to_string()),
        country: Some("US".to_string()),
        loc: Some("37.4056,-122.0775".to_string()),
        org: Some("Google LLC".to_string()),
        timezone: Some("America/Los_Angeles".to_string()),
    });

    domain_ips.ipv4.push(IPInfo {
        ip: "172.217.14.206".to_string(),
        hostname: None,
        city: Some("Tokyo".to_string()),
        region: Some("Tokyo".to_string()),
        country: Some("JP".to_string()),
        loc: Some("35.6895,139.6917".to_string()),
        org: Some("Google LLC".to_string()),
        timezone: Some("Asia/Tokyo".to_string()),
    });

    println!("🌍 Available IP Addresses:");
    for (i, ip_info) in domain_ips.ipv4.iter().enumerate() {
        println!("   {}. {}", i + 1, ip_info.ip);
        if let Some(city) = &ip_info.city {
            println!("      City: {}", city);
        }
        if let Some(country) = &ip_info.country {
            println!("      Country: {}", country);
        }
        if let Some(org) = &ip_info.org {
            println!("      Organization: {}", org);
        }
        println!();
    }

    println!("💡 Intelligent Selection Strategy:");
    println!("   - Select nearest IP based on geographic location");
    println!("   - Select fastest IP based on network latency");
    println!("   - Dynamically switch IPs based on load conditions");
    println!("   - Implement failover and high availability");
    println!();

    // === Summary ===
    println!("=" .repeat(70));
    println!("🎉 Integration Completed!");
    println!();
    println!("📚 DNS Module Enhanced Features Summary:");
    println!("   ✅ DNS Cache (DNSCache) - Reduce duplicate resolution");
    println!("   ✅ DNS Pre-resolution (DNSResolver) - Prepare IPs in advance");
    println!("   ✅ DNS Service (DNSService) - Automatically maintain domain IPs");
    println!("   ✅ IP Geographic Information (IPInfo) - Intelligent IP selection");
    println!("   ✅ HTTP Client Integration - Seamless cooperation");
    println!();
    println!("🔗 More Examples:");
    println!("   - examples/dns_service.rs - DNS service usage");
    println!("   - examples/resolve_domains.rs - Domain resolution");
    println!("   - examples/dns_cache_integration.rs - Cache integration");
    println!();

    Ok(())
}

#[cfg(not(feature = "dns"))]
fn main() {
    println!("This example requires enabling 'dns' feature");
    println!("Usage: cargo run --example dns_full_integration --features dns,rustls-tls,http2");
}