//! Cookie manageer
//!
//! for manage HTTP Cookie store、send and receive

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

/// Cookie
#[derive(Debug, Clone)]
pub struct Cookie {
 pub name: String,
 pub value: String,
 pub domain: String,
 pub path: String,
 pub expires: Option<SystemTime>,
 pub max_age: Option<Duration>,
 pub secure: bool,
 pub http_only: bool,
 pub same_site: Option<SameSite>,
}

/// SameSite property
#[derive(Debug, Clone, PartialEq)]
pub enum SameSite {
 Strict,
 Lax,
 None,
}

impl Cookie {
 /// Create a new Cookie
 pub fn new(name: String, value: String, domain: String) -> Self {
 Self {
 name,
 value,
 domain,
 path: "/".to_string(),
 expires: None,
 max_age: None,
 secure: false,
 http_only: false,
 same_site: None,
 }
 }

 /// Check Cookie whetherexpire
 pub fn is_expired(&self) -> bool {
 if let Some(expires) = self.expires {
 return SystemTime::now() > expires;
 }
 false
 }

 /// convert to HTTP Cookie headerformat
 pub fn to_header_value(&self) -> String {
 format!("{}={}", self.name, self.value)
 }

 /// from Set-Cookie headerParse
 pub fn parse_set_cookie(header: &str, domain: String) -> Option<Self> {
 let parts: Vec<&str> = header.split(';').collect();
 if parts.is_empty() {
 return None;
 }

 // Parse name=value
 let name_value: Vec<&str> = parts[0].split('=').collect();
 if name_value.len() != 2 {
 return None;
 }

 let mut cookie = Cookie::new(
 name_value[0].trim().to_string(),
 name_value[1].trim().to_string(),
 domain,
 );

 // Parseotherproperty
 for part in &parts[1..] {
 let part = part.trim();
 if part.to_lowercase().starts_with("domain=") {
 cookie.domain = part[7..].to_string();
 } else if part.to_lowercase().starts_with("path=") {
 cookie.path = part[5..].to_string();
 } else if part.to_lowercase().starts_with("max-age=") {
 if let Ok(secs) = part[8..].parse::<u64>() {
 cookie.max_age = Some(Duration::from_secs(secs));
 // let Max-Age true生effect：convert to绝pair expires 以reuse is_expired()
 cookie.expires = Some(SystemTime::now() + Duration::from_secs(secs));
 }
 } else if part.to_lowercase() == "secure" {
 cookie.secure = true;
 } else if part.to_lowercase() == "httponly" {
 cookie.http_only = true;
 } else if part.to_lowercase().starts_with("samesite=") {
 let value = part[9..].to_lowercase();
 cookie.same_site = match value.as_str() {
 "strict" => Some(SameSite::Strict),
 "lax" => Some(SameSite::Lax),
 "none" => Some(SameSite::None),
 _ => None,
 };
 }
 }

 Some(cookie)
 }
}

/// Cookie store
#[derive(Debug, Clone)]
pub struct CookieStore {
 ///  by domainstore Cookie
 cookies: Arc<Mutex<HashMap<String, Vec<Cookie>>>>,
}

impl CookieStore {
 /// Create a new Cookie store
 pub fn new() -> Self {
 Self {
 cookies: Arc::new(Mutex::new(HashMap::new())),
 }
 }

 /// Add Cookie
 pub fn add_cookie(&self, cookie: Cookie) {
 if let Ok(mut cookies) = self.cookies.lock() {
 let domain_cookies = cookies.entry(cookie.domain.clone()).or_default();

 // Checkwhetheralready existssame name Cookie， if existsthenUpdate
 if let Some(pos) = domain_cookies.iter().position(|c| c.name == cookie.name) {
 domain_cookies[pos] = cookie;
 } else {
 domain_cookies.push(cookie);
 }
 } else {
 eprintln!("warning: Cookie storelockfailure，unable toAdd Cookie");
 }
 }

 /// from Set-Cookie headerAdd Cookie
 pub fn add_from_response(&self, set_cookie_header: &str, domain: String) {
 if let Some(cookie) = Cookie::parse_set_cookie(set_cookie_header, domain) {
 self.add_cookie(cookie);
 }
 }

 /// Getspecifieddomainallvalid Cookie
 ///
 /// Based on RFC 6265 specificationperformdomainmatch：
 /// - Cookie domain property (如 `.example.com`)shouldmatch `example.com` and其allchilddomain
 /// - `example.com` Cookie shouldmatch `example.com` and `*.example.com`
 pub fn get_cookies_for_domain(&self, domain: &str) -> Vec<Cookie> {
 let cookies = match self.cookies.lock() {
 Ok(c) => c,
 Err(e) => {
 eprintln!("warning: Cookie storelockfailure: {}", e);
 return Vec::new();
 }
 };
 let mut result = Vec::new();

 let domain_lower = domain.to_lowercase();
 for (cookie_domain, domain_cookies) in cookies.iter() {
 let cookie_domain_lower = cookie_domain.to_lowercase();

 // Fix: correctdomainmatchlogic (RFC 6265)
 let matches = if cookie_domain_lower == domain_lower {
 // completelymatch
 true
 } else if let Some(base) = cookie_domain_lower.strip_prefix('.') {
 // Cookie domain 以. openheader (如.example.com)
 // shouldmatch example.com and all *.example.com
 domain_lower == base || domain_lower.ends_with(&format!(".{}", base))
 } else {
 // Cookie domain not. openheader (如 example.com)
 // shouldmatch example.com and all *.example.com
 domain_lower == cookie_domain_lower
 || domain_lower.ends_with(&format!(".{}", cookie_domain_lower))
 };

 if matches {
 for cookie in domain_cookies {
 if !cookie.is_expired() {
 result.push(cookie.clone());
 }
 }
 }
 }

 result
 }

 /// Generate Cookie header
 ///
 /// # Parameters
 /// - `domain`: requestdomain
 /// - `path`: requestpath
 /// - `is_secure`: whether as HTTPS connection ( for Secure Cookie Check)
 pub fn generate_cookie_header(
 &self,
 domain: &str,
 path: &str,
 is_secure: bool,
 ) -> Option<String> {
 let cookies = self.get_cookies_for_domain(domain);
 if cookies.is_empty() {
 return None;
 }

 // filterpathmatch Cookie，并Check Secure property
 let matching_cookies: Vec<String> = cookies
.iter()
.filter(|c| {
 // pathmatch
 if !path.starts_with(&c.path) {
 return false;
 }
 // securityFix: Secure Cookie can only in HTTPS connectionupsend
 if c.secure && !is_secure {
 return false;
 }
 true
 })
.map(Cookie::to_header_value)
.collect();

 if matching_cookies.is_empty() {
 None
 } else {
 Some(matching_cookies.join("; "))
 }
 }

 /// clearspecifieddomain Cookie
 pub fn clear_domain(&self, domain: &str) {
 if let Ok(mut cookies) = self.cookies.lock() {
 cookies.remove(domain);
 }
 }

 /// clearall Cookie
 pub fn clear_all(&self) {
 if let Ok(mut cookies) = self.cookies.lock() {
 cookies.clear();
 }
 }

 /// clearexpire Cookie
 pub fn cleanup_expired(&self) {
 if let Ok(mut cookies) = self.cookies.lock() {
 for domain_cookies in cookies.values_mut() {
 domain_cookies.retain(|c| !c.is_expired());
 }
 }
 }

 /// Getall Cookie count
 pub fn count(&self) -> usize {
 self.cookies
.lock()
.map(|cookies| cookies.values().map(|v| v.len()).sum())
.unwrap_or(0)
 }
}

impl Default for CookieStore {
 fn default() -> Self {
 Self::new()
 }
}

#[cfg(test)]
mod tests {
 use super::*;

 #[test]
 fn test_cookie_creation() {
 let cookie = Cookie::new(
 "session".to_string(),
 "abc123".to_string(),
 "example.com".to_string(),
 );
 assert_eq!(cookie.name, "session");
 assert_eq!(cookie.value, "abc123");
 assert_eq!(cookie.domain, "example.com");
 }

 #[test]
 fn test_cookie_to_header() {
 let cookie = Cookie::new(
 "session".to_string(),
 "abc123".to_string(),
 "example.com".to_string(),
 );
 assert_eq!(cookie.to_header_value(), "session=abc123");
 }

 #[test]
 fn test_parse_set_cookie() {
 let header = "session=abc123; Path=/; HttpOnly";
 let cookie = Cookie::parse_set_cookie(header, "example.com".to_string()).unwrap();
 assert_eq!(cookie.name, "session");
 assert_eq!(cookie.value, "abc123");
 assert_eq!(cookie.path, "/");
 assert!(cookie.http_only);
 }

 #[test]
 fn test_cookie_store() {
 let store = CookieStore::new();
 let cookie = Cookie::new(
 "session".to_string(),
 "abc123".to_string(),
 "example.com".to_string(),
 );
 store.add_cookie(cookie);

 let cookies = store.get_cookies_for_domain("example.com");
 assert_eq!(cookies.len(), 1);
 assert_eq!(cookies[0].name, "session");
 }

 #[test]
 fn test_generate_cookie_header() {
 let store = CookieStore::new();
 store.add_cookie(Cookie::new(
 "session".to_string(),
 "abc123".to_string(),
 "example.com".to_string(),
 ));
 store.add_cookie(Cookie::new(
 "token".to_string(),
 "xyz789".to_string(),
 "example.com".to_string(),
 ));

 let header = store
.generate_cookie_header("example.com", "/", true)
.unwrap();
 assert!(header.contains("session=abc123"));
 assert!(header.contains("token=xyz789"));
 }
}
