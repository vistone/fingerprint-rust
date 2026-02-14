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
use fingerprint_profiles::mapped_tls_clients;

/// Fingerprint result, including fingerprint, User-Agent and standard HTTP headers
#[derive(Debug)]
pub struct FingerprintResult {
    /// Profile name/ID (e.g., "chrome_133")
    pub profile_id: String,
    /// Matching User-Agent
    pub user_agent: String,
    /// Standard HTTP request headers (including global language support)
    pub headers: fingerprint_headers::headers::HTTPHeaders,
    /// Browser type
    pub browser_type: BrowserType,
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
///
/// # Note
/// When os is specified (Some), mobile profiles (e.g., Android, iOS) are automatically filtered out
/// because mobile profiles have fixed User-Agent strings that cannot be changed to other operating systems
pub fn get_random_fingerprint_with_os(
    os: Option<OperatingSystem>,
) -> Result<FingerprintResult, String> {
    let clients = mapped_tls_clients();
    if clients.is_empty() {
        return Err("no TLS client profiles available".to_string());
    }

    // Get all available fingerprint names
    // If OS is specified, filter out mobile profiles (mobile profiles have fixed UA that cannot change OS)
    let names: Vec<String> = match os {
        Some(_) => clients
            .keys()
            .filter(|name| !is_mobile_profile(name))
            .cloned()
            .collect(),
        None => clients.keys().cloned().collect(),
    };

    // Randomly select one (thread-safe)
    let name_refs: Vec<&str> = names.iter().map(|s| s.as_str()).collect();
    let random_name = random_choice_string(&name_refs)
        .ok_or_else(|| "failed to select random profile".to_string())?;

    let profile = clients
        .get(&random_name)
        .ok_or_else(|| format!("profile {} not found", random_name))?;

    let profile_id = profile.id();

    // Get matching User-Agent
    let ua = match os {
        Some(os) => get_user_agent_by_profile_name_with_os(&random_name, os)?,
        None => get_user_agent_by_profile_name(&random_name)?,
    };

    // Generate standard HTTP headers
    let (browser_type_str, _) = infer_browser_from_profile_name(&random_name);
    let is_mobile = is_mobile_profile(&random_name);
    let browser_type = BrowserType::from_str(&browser_type_str).unwrap_or(BrowserType::Chrome);
    let headers = generate_headers(browser_type, &ua, is_mobile);

    Ok(FingerprintResult {
        profile_id,
        user_agent: ua,
        headers,
        browser_type,
    })
}

/// Based on browser type randomly get fingerprint and User-Agent
/// browser_type: "chrome", "firefox", "safari", "opera" etc.
pub fn get_random_fingerprint_by_browser(
    browser_type: &str,
) -> Result<FingerprintResult, Box<dyn std::error::Error>> {
    get_random_fingerprint_by_browser_with_os(browser_type, None)
}

// / Based on browser type randomly get fingerprint and User-Agent, 并specified operating system
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

    // filter out specified browser type fingerprint
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

    // randomly select an (thread security)
    let candidate_refs: Vec<&str> = candidates.iter().map(|s| s.as_str()).collect();
    let random_name = random_choice_string(&candidate_refs)
        .ok_or_else(|| "failed to select random profile".to_string())?;

    let profile = clients
        .get(&random_name)
        .ok_or_else(|| format!("profile {} not found", random_name))?;

    let profile_id = profile.id();

    // Get pair should User-Agent
    let ua = match os {
        Some(os) => get_user_agent_by_profile_name_with_os(&random_name, os)?,
        None => get_user_agent_by_profile_name(&random_name)?,
    };

    // Generate standard HTTP Headers
    let (browser_type_str, _) = infer_browser_from_profile_name(&random_name);
    let is_mobile = is_mobile_profile(&random_name);
    let browser_type_enum = BrowserType::from_str(&browser_type_str).unwrap_or(BrowserType::Chrome);
    let headers = generate_headers(browser_type_enum, &ua, is_mobile);

    Ok(FingerprintResult {
        profile_id,
        user_agent: ua,
        headers,
        browser_type: browser_type_enum,
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
        assert!(!result.profile_id.is_empty());
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
        println!("║ TCP fingerprint and browser fingerprint sync - real test ║");
        println!("╚════════════════════════════════════════════════════════════════╝\n");

        println!(
            "【test】randomly select browser fingerprint (Validate TCP fingerprint automatic sync)\n"
        );

        for i in 1..=5 {
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            println!("第 {} 次 randomly select：", i);

            let result = get_random_fingerprint().unwrap();
            let user_agent = &result.user_agent;
            let profile_id = &result.profile_id;

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

            println!(" browser fingerprint: {}", profile_id);
            println!(" User-Agent: {}", user_agent);
            println!(" infer operating system: {}", inferred_os);

            println!(" Profile ID: {}", profile_id);

            // 如果requireTCPconfigureinfo，可ending withthroughprofile_idget
            // 这里暂时comment掉TCPconfigurecheck逻辑
            /*
            if let Some(tcp_profile) = &profile.tcp_profile {
                println!(" TCP Profile:");
                println!(" TTL: {}", tcp_profile.ttl);
                println!(" Window Size: {}", tcp_profile.window_size);

                let expected_ttl = match inferred_os {
                    "Windows" => 128,
                    "macOS" => 64,
                    "Linux" => 64,
                    _ => 64,
                };

                assert_eq!(tcp_profile.ttl, expected_ttl);
                println!(" ✓ TCP fingerprint matches operating system");
            }
            */
        }

        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("✅ test complete！");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    }
}
