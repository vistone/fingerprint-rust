//! Random fingerprint generation module
//!
//! Provides random fingerprint selection and User-Agent features

use fingerprint_core::types::{BrowserType, OperatingSystem};
use fingerprint_core::utils::{
    infer_browser_from_profile_name, is_mobile_profile, random_choice_string,
};
use fingerprint_headers::headers::generate_headers;
use fingerprint_headers::useragent::{
    get_user_agent_by_profile_name, get_user_agent_by_profile_name_with_os,
};
use fingerprint_profiles::profiles::{mapped_tls_clients, ClientProfile};

/// Fingerprint result, including fingerprint, User-Agent and standard HTTP headers
#[derive(Debug, Clone)]
pub struct FingerprintResult {
    /// TLS fingerprint configuration
    pub profile: ClientProfile,
    /// Matching User-Agent
    pub user_agent: String,
    /// Client Hello ID (consistent with tls-client)
    pub hello_client_id: String,
    /// Standard HTTP request headers (including global language support)
    pub headers: fingerprint_headers::headers::HTTPHeaders,
}

/// Browser type not found error
#[derive(Debug, Clone)]
pub struct BrowserNotFoundError {
    pub browser: String,
}

impl std::fmt::Display for BrowserNotFoundError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "browser type not found: {}", self.browser)
    }
}

impl std::error::Error for BrowserNotFoundError {}

/// Randomly get a fingerprint and matching User-Agent
/// Operating system will be randomly selected
pub fn get_random_fingerprint() -> Result<FingerprintResult, String> {
    get_random_fingerprint_with_os(None)
}

/// Randomly get a fingerprint and matching User-Agent, with specified operating system
/// If os is None, then operating system is randomly selected
pub fn get_random_fingerprint_with_os(
    os: Option<OperatingSystem>,
) -> Result<FingerprintResult, String> {
    let clients = mapped_tls_clients();
    if clients.is_empty() {
        return Err("no TLS client profiles available".to_string());
    }

    // Get all available fingerprint names
    let names: Vec<String> = clients.keys().cloned().collect();

    // Randomly select one (thread-safe)
    let name_refs: Vec<&str> = names.iter().map(|s| s.as_str()).collect();
    let random_name = random_choice_string(&name_refs)
        .ok_or_else(|| "failed to select random profile".to_string())?;

    let mut profile = clients
        .get(&random_name)
        .ok_or_else(|| format!("profile {} not found", random_name))?
        .clone();

    if profile.get_client_hello_str().is_empty() {
        return Err(format!(
            "profile {} is invalid (empty ClientHelloStr)",
            random_name
        ));
    }

    // Get matching User-Agent
    let ua = match os {
        Some(os) => get_user_agent_by_profile_name_with_os(&random_name, os)?,
        None => get_user_agent_by_profile_name(&random_name)?,
    };

    // 🔥 Critical fix: Based on User-Agent sync TCP profile
    // Ensure browser fingerprint and TCP fingerprint are completely consistent
    profile = profile.with_synced_tcp_profile(&ua);

    // Generate standard HTTP headers
    let (browser_type_str, _) = infer_browser_from_profile_name(&random_name);
    let is_mobile = is_mobile_profile(&random_name);
    let browser_type = BrowserType::from_str(&browser_type_str).unwrap_or(BrowserType::Chrome);
    let headers = generate_headers(browser_type, &ua, is_mobile);

    let hello_client_id = profile.get_client_hello_str();
    Ok(FingerprintResult {
        profile,
        user_agent: ua,
        hello_client_id,
        headers,
    })
}

/// Based onbrowsertyperandomGetfingerprint and User-Agent
/// browser_type: "chrome", "firefox", "safari", "opera" etc.
pub fn get_random_fingerprint_by_browser(
    browser_type: &str,
) -> Result<FingerprintResult, Box<dyn std::error::Error>> {
    get_random_fingerprint_by_browser_with_os(browser_type, None)
}

/// Based onbrowsertyperandomGetfingerprint and User-Agent, 并specifiedoperating system
pub fn get_random_fingerprint_by_browser_with_os(
    browser_type: &str,
    os: Option<OperatingSystem>,
) -> Result<FingerprintResult, Box<dyn std::error::Error>> {
    if browser_type.is_empty() {
        return Err("browser type cannot be empty".into());
    }

    let clients = mapped_tls_clients();
    if clients.is_empty() {
        return Err("no TLS client profiles available".into());
    }

    let browser_type_lower = browser_type.to_lowercase();

    // filteroutspecifiedbrowsertypefingerprint
    let candidates: Vec<String> = clients
        .keys()
        .filter(|name| {
            let name_lower = name.to_lowercase();
            name_lower.starts_with(&format!("{}_", browser_type_lower))
        })
        .cloned()
        .collect();

    if candidates.is_empty() {
        return Err(Box::new(BrowserNotFoundError {
            browser: browser_type.to_string(),
        }));
    }

    // randomly select an (threadsecurity)
    let candidate_refs: Vec<&str> = candidates.iter().map(|s| s.as_str()).collect();
    let random_name = random_choice_string(&candidate_refs)
        .ok_or_else(|| "failed to select random profile".to_string())?;

    let mut profile = clients
        .get(&random_name)
        .ok_or_else(|| format!("profile {} not found", random_name))?
        .clone();

    if profile.get_client_hello_str().is_empty() {
        return Err(format!("profile {} is invalid (empty ClientHelloStr)", random_name).into());
    }

    // Getpairshould User-Agent
    let ua = match os {
        Some(os) => get_user_agent_by_profile_name_with_os(&random_name, os)?,
        None => get_user_agent_by_profile_name(&random_name)?,
    };

    // 🔥 closekeyFix: Based on User-Agent sync TCP Profile
    // ensurebrowserfingerprint and TCP fingerprintcompletelyconsistent
    profile = profile.with_synced_tcp_profile(&ua);

    // Generatestandard HTTP Headers
    let (browser_type_str, _) = infer_browser_from_profile_name(&random_name);
    let is_mobile = is_mobile_profile(&random_name);
    let browser_type_enum = BrowserType::from_str(&browser_type_str).unwrap_or(BrowserType::Chrome);
    let headers = generate_headers(browser_type_enum, &ua, is_mobile);

    let hello_client_id = profile.get_client_hello_str();
    Ok(FingerprintResult {
        profile,
        user_agent: ua,
        hello_client_id,
        headers,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_random_fingerprint() {
        let result = get_random_fingerprint();
        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(!result.user_agent.is_empty());
        assert!(!result.hello_client_id.is_empty());
    }

    #[test]
    fn test_get_random_fingerprint_by_browser() {
        let result = get_random_fingerprint_by_browser("chrome");
        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.user_agent.contains("Chrome"));
    }

    #[test]
    fn test_get_random_fingerprint_by_browser_not_found() {
        let result = get_random_fingerprint_by_browser("unknown");
        assert!(result.is_err());
    }

    #[test]
    fn test_tcp_sync_real_demo() {
        println!("\n╔════════════════════════════════════════════════════════════════╗");
        println!("║ TCP fingerprint and browserfingerprintsync - realtest ║");
        println!("╚════════════════════════════════════════════════════════════════╝\n");

        println!(
            "【test】randomly selectbrowserfingerprint (Validate TCP fingerprintautomaticsync)\n"
        );

        for i in 1..=5 {
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            println!("第 {} 次randomly select：", i);

            let result = get_random_fingerprint().unwrap();
            let user_agent = &result.user_agent;
            let profile = &result.profile;

            let inferred_os = if user_agent.contains("Windows NT 10.0")
                || user_agent.contains("Windows NT 11.0")
            {
                "Windows"
            } else if user_agent.contains("Macintosh") || user_agent.contains("Mac OS X") {
                "macOS"
            } else if user_agent.contains("Linux") || user_agent.contains("X11") {
                "Linux"
            } else {
                "Unknown"
            };

            println!(" browserfingerprint: {}", result.hello_client_id);
            println!(" User-Agent: {}", user_agent);
            println!(" inferoperating system: {}", inferred_os);

            if let Some(tcp_profile) = &profile.tcp_profile {
                println!(" TCP Profile:");
                println!(" TTL: {}", tcp_profile.ttl);
                println!(" Window Size: {}", tcp_profile.window_size);

                let expected_ttl = match inferred_os {
                    "Windows" => 128,
                    "macOS" | "Linux" => 64,
                    _ => {
                        println!(" ⚠️ unable toValidate (not知operating system)");
                        continue;
                    }
                };

                if tcp_profile.ttl == expected_ttl {
                    println!(
                        " ✅ syncValidatethrough！TTL ({}) and operating system ({}) match",
                        tcp_profile.ttl, inferred_os
                    );
                } else {
                    println!(
 " ❌ syncfailure！TTL ({}) and operating system ({}) does not match (expected: {})",
 tcp_profile.ttl, inferred_os, expected_ttl
 );
                }
            } else {
                println!(" ❌ TCP Profile 不 exists - syncfailure！");
            }
            println!();
        }

        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("✅ testcomplete！");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    }
}
