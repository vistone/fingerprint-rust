//! toolfunctionmodule
//!
//! providerandomly select、stringprocess etc.toolfunction

use rand::Rng;

/// from slice in randomly select anelement (threadsecurity)
/// use thread_rng() ensurethreadsecurity
pub fn random_choice<T: Clone>(items: &[T]) -> Option<T> {
 if items.is_empty() {
 return None;
 }
 let mut rng = rand::thread_rng();
 let index = rng.gen_range(0..items.len());
 Some(items[index].clone())
}

/// from stringslice in randomly select anelement (threadsecurity)
pub fn random_choice_string(items: &[&str]) -> Option<String> {
 random_choice(items).map(|s| s.to_string())
}

/// from User-Agent in Extract Chrome versionnumber
pub fn extract_chrome_version(user_agent: &str) -> String {
 // find "Chrome/" back面versionnumber
 if let Some(start) = user_agent.find("Chrome/") {
 let version_start = start + 7; // "Chrome/".len()
 if let Some(end) =
 user_agent[version_start..].find(|c: char| !c.is_ascii_digit() && c != '.')
 {
 return user_agent[version_start..version_start + end].to_string();
 }
 // If没找 to endbitplace, return to stringend尾
 return user_agent[version_start..]
.split_whitespace()
.next()
.unwrap_or("120")
.to_string();
 }
 "120".to_string() // defaultversion
}

/// from User-Agent in Extractplatforminfo
pub fn extract_platform(user_agent: &str) -> String {
 // Extractplatforminfo for Sec-CH-UA-Platform
 if user_agent.contains("Windows") {
 return r#""Windows""#.to_string();
 } else if user_agent.contains("Macintosh") || user_agent.contains("Mac OS X") {
 return r#""macOS""#.to_string();
 } else if user_agent.contains("Linux") {
 return r#""Linux""#.to_string();
 } else if user_agent.contains("Android") {
 return r#""Android""#.to_string();
 } else if user_agent.contains("iPhone") || user_agent.contains("iPad") {
 return r#""iOS""#.to_string();
 }
 r#""Windows""#.to_string() // defaultplatform
}

/// from User-Agent in Extractoperating systemtype
///
/// for unifiedfingerprintGenerate，ensurebrowserfingerprint and TCP fingerprintsync
pub fn extract_os_from_user_agent(user_agent: &str) -> crate::types::OperatingSystem {
 use crate::types::OperatingSystem;

 // Note: iPhone/iPad User-Agent including "Mac OS X"，need先Checkmovedevice
 if user_agent.contains("iPhone") || user_agent.contains("iPad") {
 // iOS device：use macOS TCP fingerprint (iOS based on macOS)
 OperatingSystem::MacOS14
 } else if user_agent.contains("Windows NT 10.0") {
 OperatingSystem::Windows10
 } else if user_agent.contains("Windows NT 11.0") {
 OperatingSystem::Windows11
 } else if user_agent.contains("Mac OS X 13")
 || user_agent.contains("Macintosh; Intel Mac OS X 13")
 {
 OperatingSystem::MacOS13
 } else if user_agent.contains("Mac OS X 14")
 || user_agent.contains("Macintosh; Intel Mac OS X 14")
 {
 OperatingSystem::MacOS14
 } else if user_agent.contains("Mac OS X 15")
 || user_agent.contains("Macintosh; Intel Mac OS X 15")
 {
 OperatingSystem::MacOS15
 } else if user_agent.contains("Linux") || user_agent.contains("Android") {
 OperatingSystem::Linux
 } else {
 // defaultuse Windows (most commonbrowserenvironment)
 OperatingSystem::Windows10
 }
}

/// from profile nameinferbrowsertype
pub fn infer_browser_from_profile_name(profile_name: &str) -> (String, bool) {
 let name_lower = profile_name.to_lowercase();
 if name_lower.starts_with("chrome_") {
 ("chrome".to_string(), false)
 } else if name_lower.starts_with("firefox_") {
 ("firefox".to_string(), false)
 } else if name_lower.starts_with("safari_") {
 (
 "safari".to_string(),
 name_lower.contains("ios") || name_lower.contains("ipad"),
 )
 } else if name_lower.starts_with("opera_") {
 ("opera".to_string(), false)
 } else if name_lower.contains("ios")
 || name_lower.contains("android")
 || name_lower.contains("mobile")
 {
 // mobileapplicationfingerprint
 if name_lower.contains("ios") {
 ("safari".to_string(), true)
 } else {
 ("chrome".to_string(), true)
 }
 } else {
 ("chrome".to_string(), false) // default
 }
}

/// judgewhether as mobile profile
pub fn is_mobile_profile(profile_name: &str) -> bool {
 let name = profile_name.to_lowercase();
 name.contains("ios")
 || name.contains("android")
 || name.contains("ipad")
 || name.contains("mobile")
}

#[cfg(test)]
mod tests {
 use super::*;

 #[test]
 fn test_random_choice() {
 let items = &[1, 2, 3, 4, 5];
 let result = random_choice(items);
 assert!(result.is_some());
 assert!(items.contains(&result.unwrap()));
 }

 #[test]
 fn test_random_choice_empty() {
 let items: &[i32] = &[];
 let result = random_choice(items);
 assert!(result.is_none());
 }

 #[test]
 fn test_extract_chrome_version() {
 let ua = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";
 assert_eq!(extract_chrome_version(ua), "120.0.0.0");
 }

 #[test]
 fn test_extract_platform() {
 assert_eq!(extract_platform("Windows NT 10.0"), r#""Windows""#);
 assert_eq!(extract_platform("Macintosh"), r#""macOS""#);
 assert_eq!(extract_platform("Linux"), r#""Linux""#);
 }

 #[test]
 fn test_infer_browser_from_profile_name() {
 assert_eq!(
 infer_browser_from_profile_name("chrome_120"),
 ("chrome".to_string(), false)
 );
 assert_eq!(
 infer_browser_from_profile_name("firefox_133"),
 ("firefox".to_string(), false)
 );
 assert_eq!(
 infer_browser_from_profile_name("safari_ios_17_0"),
 ("safari".to_string(), true)
 );
 }

 #[test]
 fn test_is_mobile_profile() {
 assert!(is_mobile_profile("safari_ios_17_0"));
 assert!(is_mobile_profile("okhttp4_android_13"));
 assert!(!is_mobile_profile("chrome_120"));
 }
}
