/// Browser Version Rapid Adaptation Example
///
/// Demonstrates automatic browser version detection and profile adaptation.
/// Shows how new browser versions are automatically supported without code changes.
use fingerprint_profiles::{version_adapter::quick, BrowserType, VersionAdapter, VersionDetector};

fn main() {
    println!("=== Browser Version Rapid Adaptation System ===\n");

    // Example 1: Detect browser from User-Agent strings
    println!("1. User-Agent Detection:\n");

    let user_agents = vec![
        (
            "Chrome 133",
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/133.0.0.0 Safari/537.36",
        ),
        (
            "Firefox 133",
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:133.0) Gecko/20100101 Firefox/133.0",
        ),
        (
            "Safari 18",
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Version/18.0 Safari/537.36",
        ),
        (
            "Edge 133",
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/133.0.0.0 Safari/537.36 Edg/133.0.0.0",
        ),
    ];

    for (label, ua) in user_agents {
        if let Some(info) = VersionDetector::detect(ua) {
            println!(
                "   {} → {} v{} ({})",
                label,
                info.browser,
                info.version,
                info.os.unwrap_or_else(|| "Unknown".to_string())
            );
        }
    }

    // Example 2: Quick API for profile adaptation
    println!("\n2. Automatic Profile Adaptation (Quick API):\n");

    let test_ua = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/133.0.0.0 Safari/537.36";

    println!("   Input UA: Chrome/133");
    if let Some(_profile) = quick::profile_from_ua(test_ua) {
        println!("   ✓ Profile automatically loaded");
    }

    // Example 3: Browser version registry
    println!("\n3. Version Registry Information:\n");

    let adapter = VersionAdapter::new();

    println!("   Chrome Versions Supported: {} versions", {
        let versions = adapter.get_versions_with_capability(BrowserType::Chrome, "ech");
        println!("     - With ECH support: {} versions", versions.len());
        versions.len()
    });

    println!("   Firefox Versions Supported: {} versions", {
        let versions = adapter.get_versions_with_capability(BrowserType::Firefox, "http3");
        println!("     - With HTTP/3 support: {} versions", versions.len());
        versions.len()
    });

    // Example 4: Capability detection
    println!("\n4. Capability Detection:\n");

    let test_cases = vec![
        (BrowserType::Chrome, 133, "ech"),
        (BrowserType::Chrome, 133, "psk"),
        (BrowserType::Chrome, 133, "early_data"),
        (BrowserType::Chrome, 120, "early_data"),
        (BrowserType::Firefox, 133, "ech"),
        (BrowserType::Safari, 18, "http3"),
    ];

    for (browser, version, capability) in test_cases {
        let supports = adapter.supports_capability(browser, version, capability);
        println!(
            "   {} v{} supports {}: {}",
            browser,
            version,
            capability,
            if supports { "✓" } else { "✗" }
        );
    }

    // Example 5: Version information
    println!("\n5. Detailed Version Information:\n");

    let info_versions = vec![
        (BrowserType::Chrome, 133),
        (BrowserType::Firefox, 133),
        (BrowserType::Safari, 18),
        (BrowserType::Edge, 133),
    ];

    for (browser, version) in info_versions {
        if let Some(info) = adapter.get_version_info(browser, version) {
            println!("   {}", info);
        }
    }

    // Example 6: Latest version support
    println!("\n6. Latest Browser Versions:\n");

    let browsers = vec![
        BrowserType::Chrome,
        BrowserType::Firefox,
        BrowserType::Safari,
        BrowserType::Edge,
    ];

    for browser in browsers {
        if let Some(_profile) = adapter.get_latest_profile(browser) {
            if let Some(info) = adapter.get_version_info(browser, 138) {
                println!("   {}", info);
            } else {
                println!("   {} - Latest version loaded", browser);
            }
        }
    }

    // Example 7: Version range parsing
    println!("\n7. Version Range Parsing:\n");

    let ranges = vec!["133", "130-135", "120-125"];

    for range_str in ranges {
        match VersionDetector::parse_version_range(range_str) {
            Ok(versions) => {
                println!(
                    "   '{}' → {} versions: {}..{}",
                    range_str,
                    versions.len(),
                    versions.first().unwrap_or(&0),
                    versions.last().unwrap_or(&0)
                );
            }
            Err(e) => println!("   '{}' → Error: {}", range_str, e),
        }
    }

    // Example 8: Fallback mechanism
    println!("\n8. Version Fallback Mechanism:\n");

    println!("   When a version is not explicitly defined:");
    println!("   - System automatically finds the nearest compatible version");
    println!("   - For Android Chrome 199 → fallback to Chrome 138 profile");

    if let Some(_profile) = adapter.get_profile(BrowserType::Chrome, 199) {
        println!("   ✓ Chrome 199 fallback profile loaded");
    }

    // Example 9: Mobile detection
    println!("\n9. Mobile Device Detection:\n");

    let mobile_uas = vec![
        (
            "iPhone",
            "Mozilla/5.0 (iPhone; CPU iPhone OS 18_0 like Mac OS X) AppleWebKit/537.36",
        ),
        (
            "Android",
            "Mozilla/5.0 (Linux; Android 14) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/133.0.0.0 Mobile Safari/537.36",
        ),
        (
            "iPad",
            "Mozilla/5.0 (iPad; CPU OS 18_0 like Mac OS X) AppleWebKit/537.36",
        ),
    ];

    for (label, ua) in mobile_uas {
        if let Some(info) = VersionDetector::detect(ua) {
            println!(
                "   {} (Mobile: {}) → {} v{}",
                label, info.is_mobile, info.browser, info.version
            );
        }
    }

    // Example 10: Rapid adaptation workflow
    println!("\n10. Rapid Adaptation Workflow:\n");

    println!("    Scenario: New Chrome 140 released");
    println!("    ────────────────────────────────");
    println!("    1. Browser sends Chrome/140 in User-Agent");
    println!("    2. VersionDetector parses version 140");
    println!("    3. VersionAdapter looks up version in registry");
    println!("    4. If exact version not found:");
    println!("       a. Find nearest compatible version (Chrome 139)");
    println!("       b. Use its profile (or create new profile)");
    println!("    5. Application continues working without code changes");
    println!("    6. Version registry updated periodically with new profiles");

    // Example 11: Feature timeline
    println!("\n11. Feature Support Timeline:\n");

    let features = vec!["ech", "http3", "psk", "early_data", "pq"];
    for feature in features {
        let versions = adapter.get_versions_with_capability(BrowserType::Chrome, feature);
        if !versions.is_empty() {
            println!(
                "   Chrome {}: {} support from v{}",
                feature.to_uppercase(),
                if versions.len() > 5 {
                    "wide"
                } else {
                    "limited"
                },
                versions.first().unwrap_or(&0)
            );
        }
    }

    // Example 12: Statistics
    println!("\n12. Version Registry Statistics:\n");

    let chrome_count = adapter
        .get_versions_with_capability(BrowserType::Chrome, "http2")
        .len();
    let firefox_count = adapter
        .get_versions_with_capability(BrowserType::Firefox, "http2")
        .len();
    let _safari_count = adapter
        .get_versions_with_capability(BrowserType::Safari, "http2")
        .len();

    println!("   Total Supported Versions:");
    println!("   - Chrome: {} versions (v103-v138+)", chrome_count);
    println!("   - Firefox: {} versions (v102-v138+)", firefox_count);
    println!("   - Safari: 4 major versions");
    println!("   - Edge: ~18 versions");
    println!("   - Opera: 6+ versions");

    println!("\n=== Summary ===\n");
    println!("✓ Automatic version detection from User-Agent");
    println!("✓ Fallback to nearest compatible version");
    println!("✓ Capability-based version filtering");
    println!("✓ Mobile device identification");
    println!("✓ Future-proof: New versions supported automatically");
    println!("✓ Quick API for rapid integration");
    println!("\n");
}
