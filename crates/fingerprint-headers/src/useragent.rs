//! User-Agent generation module
//!
//! Generates corresponding User-Agent based on fingerprint configuration

use fingerprint_core::types::{BrowserType, OperatingSystem, UserAgentTemplate, OPERATING_SYSTEMS};
use fingerprint_core::utils::random_choice;
use std::collections::HashMap;
use std::sync::OnceLock;

/// User-Agent Generator
pub struct UserAgentGenerator {
 templates: HashMap<String, UserAgentTemplate>,
}

impl UserAgentGenerator {
 /// Create a new User-Agent Generator
 pub fn new() -> Self {
 let mut gen = Self {
 templates: HashMap::new(),
 };
 gen.init_templates();
 gen
 }

 /// Initialize User-Agent templates
 fn init_templates(&mut self) {
 // Chrome User-Agent templates
 let chrome_templates: &[(&str, &str)] = &[
 ("103", "Mozilla/5.0 (%s) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/103.0.0.0 Safari/537.36"),
 ("104", "Mozilla/5.0 (%s) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/104.0.0.0 Safari/537.36"),
 ("105", "Mozilla/5.0 (%s) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/105.0.0.0 Safari/537.36"),
 ("106", "Mozilla/5.0 (%s) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/106.0.0.0 Safari/537.36"),
 ("107", "Mozilla/5.0 (%s) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/107.0.0.0 Safari/537.36"),
 ("108", "Mozilla/5.0 (%s) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/108.0.0.0 Safari/537.36"),
 ("109", "Mozilla/5.0 (%s) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/109.0.0.0 Safari/537.36"),
 ("110", "Mozilla/5.0 (%s) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/110.0.0.0 Safari/537.36"),
 ("111", "Mozilla/5.0 (%s) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/111.0.0.0 Safari/537.36"),
 ("112", "Mozilla/5.0 (%s) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/112.0.0.0 Safari/537.36"),
 ("116", "Mozilla/5.0 (%s) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/116.0.0.0 Safari/537.36"),
 ("117", "Mozilla/5.0 (%s) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/117.0.0.0 Safari/537.36"),
 ("120", "Mozilla/5.0 (%s) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36"),
 ("124", "Mozilla/5.0 (%s) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36"),
 ("130", "Mozilla/5.0 (%s) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0.0.0 Safari/537.36"),
 ("131", "Mozilla/5.0 (%s) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36"),
 ("133", "Mozilla/5.0 (%s) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/133.0.0.0 Safari/537.36"),
 ];

 for (version, template) in chrome_templates {
 self.templates.insert(
 format!("chrome_{}", version),
 UserAgentTemplate::new(
 BrowserType::Chrome,
 version.to_string(),
 template.to_string(),
 false,
 true,
 ),
 );
 }

 // Firefox User-Agent templates
 let firefox_templates: &[(&str, &str)] = &[
 (
 "102",
 "Mozilla/5.0 (%s; rv:102.0) Gecko/20100101 Firefox/102.0",
 ),
 (
 "104",
 "Mozilla/5.0 (%s; rv:104.0) Gecko/20100101 Firefox/104.0",
 ),
 (
 "105",
 "Mozilla/5.0 (%s; rv:105.0) Gecko/20100101 Firefox/105.0",
 ),
 (
 "106",
 "Mozilla/5.0 (%s; rv:106.0) Gecko/20100101 Firefox/106.0",
 ),
 (
 "108",
 "Mozilla/5.0 (%s; rv:108.0) Gecko/20100101 Firefox/108.0",
 ),
 (
 "110",
 "Mozilla/5.0 (%s; rv:110.0) Gecko/20100101 Firefox/110.0",
 ),
 (
 "117",
 "Mozilla/5.0 (%s; rv:117.0) Gecko/20100101 Firefox/117.0",
 ),
 (
 "120",
 "Mozilla/5.0 (%s; rv:120.0) Gecko/20100101 Firefox/120.0",
 ),
 (
 "123",
 "Mozilla/5.0 (%s; rv:123.0) Gecko/20100101 Firefox/123.0",
 ),
 (
 "132",
 "Mozilla/5.0 (%s; rv:132.0) Gecko/20100101 Firefox/132.0",
 ),
 (
 "133",
 "Mozilla/5.0 (%s; rv:133.0) Gecko/20100101 Firefox/133.0",
 ),
 (
 "135",
 "Mozilla/5.0 (%s; rv:135.0) Gecko/20100101 Firefox/135.0",
 ),
 ];

 for (version, template) in firefox_templates {
 self.templates.insert(
 format!("firefox_{}", version),
 UserAgentTemplate::new(
 BrowserType::Firefox,
 version.to_string(),
 template.to_string(),
 false,
 true,
 ),
 );
 }

 // Safari User-Agent templates
 let safari_templates: &[(&str, &str, bool)] = &[
 ("15_6_1", "Mozilla/5.0 (%s) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/15.6.1 Safari/605.1.15", false),
 ("16_0", "Mozilla/5.0 (%s) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/16.0 Safari/605.1.15", false),
 ("ipad_15_6", "Mozilla/5.0 (iPad; CPU OS 15_6 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/15.6 Mobile/15E148 Safari/604.1", true),
 ("ios_15_5", "Mozilla/5.0 (iPhone; CPU iPhone OS 15_5 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/15.5 Mobile/15E148 Safari/604.1", true),
 ("ios_15_6", "Mozilla/5.0 (iPhone; CPU iPhone OS 15_6 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/15.6 Mobile/15E148 Safari/604.1", true),
 ("ios_16_0", "Mozilla/5.0 (iPhone; CPU iPhone OS 16_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/16.0 Mobile/15E148 Safari/604.1", true),
 ("ios_17_0", "Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.0 Mobile/15E148 Safari/604.1", true),
 ("ios_18_0", "Mozilla/5.0 (iPhone; CPU iPhone OS 18_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/18.0 Mobile/15E148 Safari/604.1", true),
 ("ios_18_5", "Mozilla/5.0 (iPhone; CPU iPhone OS 18_5 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/18.5 Mobile/15E148 Safari/604.1", true),
 ];

 for (key, template, mobile) in safari_templates {
 self.templates.insert(
 format!("safari_{}", key),
 UserAgentTemplate::new(
 BrowserType::Safari,
 key.to_string(),
 template.to_string(),
 *mobile,
 !mobile, // mobile does not need OS information
 ),
 );
 }

 // Opera User-Agent templates
 let opera_templates: &[(&str, &str)] = &[
 ("89", "Mozilla/5.0 (%s) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/103.0.0.0 Safari/537.36 OPR/89.0.0.0"),
 ("90", "Mozilla/5.0 (%s) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/104.0.0.0 Safari/537.36 OPR/90.0.0.0"),
 ("91", "Mozilla/5.0 (%s) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/105.0.0.0 Safari/537.36 OPR/91.0.0.0"),
 ];

 for (version, template) in opera_templates {
 self.templates.insert(
 format!("opera_{}", version),
 UserAgentTemplate::new(
 BrowserType::Opera,
 version.to_string(),
 template.to_string(),
 false,
 true,
 ),
 );
 }

 // mobile and custom fingerprint User-Agent templates
 // iOS applicationfingerprint
 let ios_app_templates: &[(&str, &str)] = &[
 ("zalando_ios_mobile", "Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.0 Mobile/15E148 Safari/604.1"),
 ("nike_ios_mobile", "Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.0 Mobile/15E148 Safari/604.1"),
 ("mms_ios", "Mozilla/5.0 (iPhone; CPU iPhone OS 16_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/16.0 Mobile/15E148 Safari/604.1"),
 ("mms_ios_2", "Mozilla/5.0 (iPhone; CPU iPhone OS 16_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/16.0 Mobile/15E148 Safari/604.1"),
 ("mms_ios_3", "Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.0 Mobile/15E148 Safari/604.1"),
 ("mesh_ios", "Mozilla/5.0 (iPhone; CPU iPhone OS 16_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/16.0 Mobile/15E148 Safari/604.1"),
 ("mesh_ios_2", "Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.0 Mobile/15E148 Safari/604.1"),
 ("confirmed_ios", "Mozilla/5.0 (iPhone; CPU iPhone OS 16_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/16.0 Mobile/15E148 Safari/604.1"),
 ];

 for (key, template) in ios_app_templates {
 self.templates.insert(
 key.to_string(),
 UserAgentTemplate::new(
 BrowserType::Safari,
 "ios".to_string(),
 template.to_string(),
 true,
 false, // iOS mobile does not need OS placeholder
 ),
 );
 }

 // Android applicationfingerprint
 let android_app_templates: &[(&str, &str)] = &[
 ("zalando_android_mobile", "Mozilla/5.0 (Linux; Android 13; SM-G991B) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Mobile Safari/537.36"),
 ("nike_android_mobile", "Mozilla/5.0 (Linux; Android 13; Pixel 7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Mobile Safari/537.36"),
 ("mesh_android", "Mozilla/5.0 (Linux; Android 12; SM-G998B) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Mobile Safari/537.36"),
 ("mesh_android_2", "Mozilla/5.0 (Linux; Android 13; Pixel 6) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Mobile Safari/537.36"),
 ("confirmed_android", "Mozilla/5.0 (Linux; Android 12; SM-G998B) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Mobile Safari/537.36"),
 ("confirmed_android_2", "Mozilla/5.0 (Linux; Android 13; Pixel 7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Mobile Safari/537.36"),
 ];

 for (key, template) in android_app_templates {
 self.templates.insert(
 key.to_string(),
 UserAgentTemplate::new(
 BrowserType::Chrome,
 "android".to_string(),
 template.to_string(),
 true,
 false, // Android mobile does not need OS placeholder
 ),
 );
 }

 // OkHttp4 Android fingerprint
 let okhttp_templates: &[(&str, &str)] = &[
 ("okhttp4_android_7", "Mozilla/5.0 (Linux; Android 7.0; SM-G930F) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Mobile Safari/537.36"),
 ("okhttp4_android_8", "Mozilla/5.0 (Linux; Android 8.0; SM-G950F) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Mobile Safari/537.36"),
 ("okhttp4_android_9", "Mozilla/5.0 (Linux; Android 9; SM-G960F) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Mobile Safari/537.36"),
 ("okhttp4_android_10", "Mozilla/5.0 (Linux; Android 10; SM-G970F) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Mobile Safari/537.36"),
 ("okhttp4_android_11", "Mozilla/5.0 (Linux; Android 11; SM-G991B) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Mobile Safari/537.36"),
 ("okhttp4_android_12", "Mozilla/5.0 (Linux; Android 12; SM-G998B) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Mobile Safari/537.36"),
 ("okhttp4_android_13", "Mozilla/5.0 (Linux; Android 13; Pixel 7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Mobile Safari/537.36"),
 ];

 for (key, template) in okhttp_templates {
 self.templates.insert(
 key.to_string(),
 UserAgentTemplate::new(
 BrowserType::Chrome,
 "okhttp4".to_string(),
 template.to_string(),
 true,
 false, // Android mobile does not need OS placeholder
 ),
 );
 }

 // Cloudflare Custom
 self.templates.insert(
 "cloudflare_custom".to_string(),
 UserAgentTemplate::new(
 BrowserType::Chrome,
 "custom".to_string(),
 "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36".to_string(),
 false,
 false, // fixed User-Agent, does not need OS placeholder
 ),
 );
 }

 /// Based on fingerprint nameGet User-Agent
 /// If fingerprintneed operating system information, will randomly select anoperating system
 pub fn get_user_agent(&self, profile_name: &str) -> Result<String, String> {
 self.get_user_agent_with_os(profile_name, None)
 }

 /// Based on fingerprint name and specifiedoperating systemGet User-Agent
 /// If os as None, and need operating system information，will randomly select anoperating system
 pub fn get_user_agent_with_os(
 &self,
 profile_name: &str,
 os: Option<OperatingSystem>,
 ) -> Result<String, String> {
 if profile_name.is_empty() {
 return Err("profile name cannot be empty".to_string());
 }

 if let Some(template) = self.templates.get(profile_name) {
 // Ifdoes not need OS information, directlyreturn templates
 if !template.os_required {
 return Ok(template.template.clone());
 }

 // if need operating system information
 let os_str = match os {
 Some(os) => os.as_str(),
 None => {
 // randomly selectoperating system
 random_os().as_str()
 }
 };

 return Ok(template.template.replace("%s", os_str));
 }

 // try from profileName in Extractbrowsertype and version
 self.generate_from_profile_name(profile_name, os)
 }

 /// from profile nameGenerate User-Agent
 fn generate_from_profile_name(
 &self,
 profile_name: &str,
 os: Option<OperatingSystem>,
 ) -> Result<String, String> {
 let profile_name_lower = profile_name.to_lowercase();

 // Parsebrowsertype and version
 let (browser, version) = if profile_name_lower.starts_with("chrome_") {
 let version = profile_name_lower
.strip_prefix("chrome_")
.unwrap_or("")
.split('_')
.next()
.unwrap_or("133");
 (BrowserType::Chrome, version)
 } else if profile_name_lower.starts_with("firefox_") {
 let version = profile_name_lower.strip_prefix("firefox_").unwrap_or("135");
 (BrowserType::Firefox, version)
 } else if profile_name_lower.starts_with("safari_") {
 let version = profile_name_lower.strip_prefix("safari_").unwrap_or("16_0");
 (BrowserType::Safari, version)
 } else if profile_name_lower.starts_with("opera_") {
 let version = profile_name_lower.strip_prefix("opera_").unwrap_or("91");
 (BrowserType::Opera, version)
 } else if profile_name_lower.starts_with("edge_") {
 let version = profile_name_lower.strip_prefix("edge_").unwrap_or("133");
 (BrowserType::Edge, version)
 } else {
 // defaultuse Chrome 133
 return self.get_user_agent_with_os("chrome_133", os);
 };

 // Generate User-Agent
 let os_str = match os {
 Some(os) => os.as_str(),
 None => random_os().as_str(),
 };

 match browser {
 BrowserType::Chrome => Ok(format!(
 "Mozilla/5.0 ({}) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/{}.0.0.0 Safari/537.36",
 os_str, version
 )),
 BrowserType::Firefox => Ok(format!(
 "Mozilla/5.0 ({}; rv:{}.0) Gecko/20100101 Firefox/{}.0",
 os_str, version, version
 )),
 BrowserType::Safari => Ok(format!(
 "Mozilla/5.0 ({}) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/{} Safari/605.1.15",
 os_str, version
 )),
 BrowserType::Opera => Ok(format!(
 "Mozilla/5.0 ({}) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/{}.0.0.0 Safari/537.36 OPR/{}.0.0.0",
 os_str, version, version
 )),
 BrowserType::Edge => Ok(format!(
 "Mozilla/5.0 ({}) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/{}.0.0.0 Safari/537.36 Edg/{}.0.0.0",
 os_str, version, version
 )),
 }
 }
}

impl Default for UserAgentGenerator {
 fn default() -> Self {
 Self::new()
 }
}

/// globaldefaultGenerator (threadsecurity)
static DEFAULT_GENERATOR: OnceLock<UserAgentGenerator> = OnceLock::new();

fn get_default_generator() -> &'static UserAgentGenerator {
 DEFAULT_GENERATOR.get_or_init(UserAgentGenerator::new)
}

/// randomly select anoperating system
pub fn random_os() -> OperatingSystem {
 random_choice(OPERATING_SYSTEMS).unwrap_or(OperatingSystem::Windows10)
}

/// as specified ClientProfile Get User-Agent
pub fn get_user_agent_by_profile_name(profile_name: &str) -> Result<String, String> {
 get_default_generator().get_user_agent(profile_name)
}

/// as specified ClientProfile and operating systemGet User-Agent
pub fn get_user_agent_by_profile_name_with_os(
 profile_name: &str,
 os: OperatingSystem,
) -> Result<String, String> {
 get_default_generator().get_user_agent_with_os(profile_name, Some(os))
}

#[cfg(test)]
mod tests {
 use super::*;

 #[test]
 fn test_get_user_agent_chrome() {
 let gen = UserAgentGenerator::new();
 let ua = gen.get_user_agent("chrome_120").unwrap();
 assert!(ua.contains("Chrome/120"));
 assert!(ua.contains("AppleWebKit"));
 }

 #[test]
 fn test_get_user_agent_firefox() {
 let gen = UserAgentGenerator::new();
 let ua = gen.get_user_agent("firefox_133").unwrap();
 assert!(ua.contains("Firefox/133"));
 }

 #[test]
 fn test_get_user_agent_with_os() {
 let gen = UserAgentGenerator::new();
 let ua = gen
.get_user_agent_with_os("chrome_120", Some(OperatingSystem::MacOS14))
.unwrap();
 assert!(ua.contains("Macintosh"));
 }

 #[test]
 fn test_random_os() {
 let os = random_os();
 assert!(OPERATING_SYSTEMS.contains(&os));
 }
}
