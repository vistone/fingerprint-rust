//! HTTP Headers module
//!
//! providestandard HTTP requestheaderGenerate and manageFeatures

use fingerprint_core::types::BrowserType;
use fingerprint_core::utils::{extract_chrome_version, extract_platform, random_choice_string};

/// globallanguagelist（按usefrequencysort）
pub static LANGUAGES: &[&str] = &[
 "en-US,en;q=0.9", // 英语（美国）
 "zh-CN,zh;q=0.9,en;q=0.8", // in 文（简体）
 "es-ES,es;q=0.9,en;q=0.8", // 西班牙语
 "fr-FR,fr;q=0.9,en;q=0.8", // 法语
 "de-DE,de;q=0.9,en;q=0.8", // 德语
 "ja-JP,ja;q=0.9,en;q=0.8", // 日语
 "pt-BR,pt;q=0.9,en;q=0.8", // Portuguese（巴西）
 "ru-RU,ru;q=0.9,en;q=0.8", // 俄语
 "ar-SA,ar;q=0.9,en;q=0.8", // 阿拉伯语
 "ko-KR,ko;q=0.9,en;q=0.8", // 韩语
 "it-IT,it;q=0.9,en;q=0.8", // 意大利语
 "tr-TR,tr;q=0.9,en;q=0.8", // 土耳其语
 "pl-PL,pl;q=0.9,en;q=0.8", // 波兰语
 "nl-NL,nl;q=0.9,en;q=0.8", // 荷兰语
 "sv-SE,sv;q=0.9,en;q=0.8", // 瑞典语
 "vi-VN,vi;q=0.9,en;q=0.8", // 越南语
 "th-TH,th;q=0.9,en;q=0.8", // 泰语
 "id-ID,id;q=0.9,en;q=0.8", // 印尼语
 "hi-IN,hi;q=0.9,en;q=0.8", // 印地语
 "cs-CZ,cs;q=0.9,en;q=0.8", // 捷克语
 "ro-RO,ro;q=0.9,en;q=0.8", // 罗马尼亚语
 "hu-HU,hu;q=0.9,en;q=0.8", // 匈牙利语
 "el-GR,el;q=0.9,en;q=0.8", // 希腊语
 "da-DK,da;q=0.9,en;q=0.8", // 丹麦语
 "fi-FI,fi;q=0.9,en;q=0.8", // 芬兰语
 "no-NO,no;q=0.9,en;q=0.8", // 挪威语
 "he-IL,he;q=0.9,en;q=0.8", // 希伯来语
 "uk-UA,uk;q=0.9,en;q=0.8", // 乌克兰语
 "pt-PT,pt;q=0.9,en;q=0.8", // Portuguese（葡萄牙）
 "zh-TW,zh;q=0.9,en;q=0.8", // in 文（繁体）
];

/// standard HTTP requestheader
#[derive(Debug, Clone)]
pub struct HTTPHeaders {
 /// Accept header
 pub accept: String,
 /// Accept-Language header（supportgloballanguage）
 pub accept_language: String,
 /// Accept-Encoding header
 pub accept_encoding: String,
 /// User-Agent header
 pub user_agent: String,
 /// Sec-Fetch-Site header
 pub sec_fetch_site: String,
 /// Sec-Fetch-Mode header
 pub sec_fetch_mode: String,
 /// Sec-Fetch-User header
 pub sec_fetch_user: String,
 /// Sec-Fetch-Dest header
 pub sec_fetch_dest: String,
 /// Sec-CH-UA header
 pub sec_ch_ua: String,
 /// Sec-CH-UA-Mobile header
 pub sec_ch_ua_mobile: String,
 /// Sec-CH-UA-Platform header
 pub sec_ch_ua_platform: String,
 /// Upgrade-Insecure-Requests header
 pub upgrade_insecure_requests: String,
 /// usercustom headers（如 Cookie、Authorization、X-API-Key etc.）
 pub custom: std::collections::HashMap<String, String>,
}

impl HTTPHeaders {
 /// Create a new HTTPHeaders
 pub fn new() -> Self {
 Self {
 accept: String::new(),
 accept_language: String::new(),
 accept_encoding: String::new(),
 user_agent: String::new(),
 sec_fetch_site: String::new(),
 sec_fetch_mode: String::new(),
 sec_fetch_user: String::new(),
 sec_fetch_dest: String::new(),
 sec_ch_ua: String::new(),
 sec_ch_ua_mobile: String::new(),
 sec_ch_ua_platform: String::new(),
 upgrade_insecure_requests: String::new(),
 custom: std::collections::HashMap::new(),
 }
 }

 /// clone HTTPHeaders pair象，returnannew副本
 ///
 /// Note: 此methodname and standardlibrary `Clone::clone` different，以avoidnamingconflict
 #[allow(clippy::should_implement_trait)]
 pub fn clone(&self) -> Self {
 Self {
 accept: self.accept.clone(),
 accept_language: self.accept_language.clone(),
 accept_encoding: self.accept_encoding.clone(),
 user_agent: self.user_agent.clone(),
 sec_fetch_site: self.sec_fetch_site.clone(),
 sec_fetch_mode: self.sec_fetch_mode.clone(),
 sec_fetch_user: self.sec_fetch_user.clone(),
 sec_fetch_dest: self.sec_fetch_dest.clone(),
 sec_ch_ua: self.sec_ch_ua.clone(),
 sec_ch_ua_mobile: self.sec_ch_ua_mobile.clone(),
 sec_ch_ua_platform: self.sec_ch_ua_platform.clone(),
 upgrade_insecure_requests: self.upgrade_insecure_requests.clone(),
 custom: self.custom.clone(),
 }
 }

 /// settingsusercustom header（systemwillautomaticmerge to to_map() in ）
 /// this isrecommendmethod，settingsbackcall to_map() canautomaticincludingcustom headers
 /// Examples：result.headers.set("Cookie", "session_id=abc123")
 pub fn set(&mut self, key: &str, value: &str) {
 if value.is_empty() {
 self.custom.remove(key);
 } else {
 self.custom.insert(key.to_string(), value.to_string());
 }
 }

 /// bulksettingsusercustom headers（systemwillautomaticmerge to to_map() in ）
 /// Examples：result.headers.set_headers(&[("Cookie", "session_id=abc123"), ("X-API-Key", "key")])
 pub fn set_headers(&mut self, custom_headers: &[(&str, &str)]) {
 for (key, value) in custom_headers {
 self.set(key, value);
 }
 }

 /// will HTTPHeaders convert to HashMap
 /// systemwillautomaticmerge Custom inusercustom headers（如 Cookie、Authorization、X-API-Key etc.）
 pub fn to_map(&self) -> std::collections::HashMap<String, String> {
 self.to_map_with_custom(&[])
 }

 /// will HTTPHeaders convert to HashMap，并mergeusercustom headers
 /// custom_headers: usercustom headers（如 session、cookie、apikey etc.）
 /// usercustom headers priority更high，willcoversystemGenerate headers
 pub fn to_map_with_custom(
 &self,
 custom_headers: &[(&str, &str)],
 ) -> std::collections::HashMap<String, String> {
 let mut headers = std::collections::HashMap::new();

 // 先AddsystemGenerate's standard headers
 if !self.accept.is_empty() {
 headers.insert("Accept".to_string(), self.accept.clone());
 }
 if !self.accept_language.is_empty() {
 headers.insert("Accept-Language".to_string(), self.accept_language.clone());
 }
 if !self.accept_encoding.is_empty() {
 headers.insert("Accept-Encoding".to_string(), self.accept_encoding.clone());
 }
 if !self.user_agent.is_empty() {
 headers.insert("User-Agent".to_string(), self.user_agent.clone());
 }
 if !self.sec_fetch_site.is_empty() {
 headers.insert("Sec-Fetch-Site".to_string(), self.sec_fetch_site.clone());
 }
 if !self.sec_fetch_mode.is_empty() {
 headers.insert("Sec-Fetch-Mode".to_string(), self.sec_fetch_mode.clone());
 }
 if !self.sec_fetch_user.is_empty() {
 headers.insert("Sec-Fetch-User".to_string(), self.sec_fetch_user.clone());
 }
 if !self.sec_fetch_dest.is_empty() {
 headers.insert("Sec-Fetch-Dest".to_string(), self.sec_fetch_dest.clone());
 }
 if !self.sec_ch_ua.is_empty() {
 headers.insert("Sec-CH-UA".to_string(), self.sec_ch_ua.clone());
 }
 if !self.sec_ch_ua_mobile.is_empty() {
 headers.insert(
 "Sec-CH-UA-Mobile".to_string(),
 self.sec_ch_ua_mobile.clone(),
 );
 }
 if !self.sec_ch_ua_platform.is_empty() {
 headers.insert(
 "Sec-CH-UA-Platform".to_string(),
 self.sec_ch_ua_platform.clone(),
 );
 }
 if !self.upgrade_insecure_requests.is_empty() {
 headers.insert(
 "Upgrade-Insecure-Requests".to_string(),
 self.upgrade_insecure_requests.clone(),
 );
 }

 // merge HTTPHeaders in Custom headers
 for (key, value) in &self.custom {
 if !value.is_empty() {
 headers.insert(key.clone(), value.clone());
 }
 }

 // merge传入 custom_headers（priority最high，willcoverallalready有 headers）
 for (key, value) in custom_headers {
 if !value.is_empty() {
 headers.insert((*key).to_string(), (*value).to_string());
 }
 }

 headers
 }

 /// will HTTPHeaders convert toordered Vec，followspecified header_order
 pub fn to_ordered_vec(&self, order: &[String]) -> Vec<(String, String)> {
 let map = self.to_map();
 let mut result = Vec::with_capacity(map.len());
 let mut used = std::collections::HashSet::new();

 // 1. 先按specified order orderAdd
 for key in order {
 // find map is否 existsmatch key (ignoresize写performmatch，butpreserve order insize写)
 for (m_key, m_val) in &map {
 if m_key.eq_ignore_ascii_case(key) && !used.contains(m_key) {
 result.push((key.clone(), m_val.clone()));
 used.insert(m_key.clone());
 break;
 }
 }
 }

 // 2. Add剩down且不再 order in headers
 for (m_key, m_val) in map {
 if !used.contains(&m_key) {
 result.push((m_key, m_val));
 }
 }

 result
 }
}

impl Default for HTTPHeaders {
 fn default() -> Self {
 Self::new()
 }
}

/// randomly selectanlanguage
pub fn random_language() -> String {
 random_choice_string(LANGUAGES).unwrap_or_else(|| "en-US,en;q=0.9".to_string())
}

/// Based onbrowsertype and User-Agent Generatestandard HTTP headers
pub fn generate_headers(
 browser_type: BrowserType,
 user_agent: &str,
 is_mobile: bool,
) -> HTTPHeaders {
 let user_agent = if user_agent.is_empty() {
 "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/135.0.0.0 Safari/537.36"
 } else {
 user_agent
 };

 let mut headers = HTTPHeaders::new();
 headers.user_agent = user_agent.to_string();

 match browser_type {
 BrowserType::Chrome => {
 headers.accept = "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.7".to_string();
 headers.accept_encoding = "gzip, deflate, br, zstd".to_string();
 headers.sec_fetch_site = "none".to_string();
 headers.sec_fetch_mode = "navigate".to_string();
 headers.sec_fetch_user = "?1".to_string();
 headers.sec_fetch_dest = "document".to_string();
 headers.upgrade_insecure_requests = "1".to_string();

 if is_mobile {
 headers.sec_ch_ua =
 r#""Not(A:Brand";v="99", "Google Chrome";v="135", "Chromium";v="135""#
.to_string();
 headers.sec_ch_ua_mobile = "?1".to_string();
 headers.sec_ch_ua_platform = r#""Android""#.to_string();
 } else {
 let chrome_version = extract_chrome_version(user_agent);
 headers.sec_ch_ua = format!(
 r#""Not(A:Brand";v="99", "Google Chrome";v="{}", "Chromium";v="{}""#,
 chrome_version, chrome_version
 );
 headers.sec_ch_ua_mobile = "?0".to_string();
 headers.sec_ch_ua_platform = extract_platform(user_agent);
 }
 }
 BrowserType::Firefox => {
 headers.accept = "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,*/*;q=0.8".to_string();
 headers.accept_encoding = "gzip, deflate, br".to_string();
 // Firefox 不use Sec-Fetch-* headers（旧version）
 // 新version Firefox use，butformatdifferent
 if is_mobile {
 headers.sec_fetch_site = "none".to_string();
 headers.sec_fetch_mode = "navigate".to_string();
 headers.sec_fetch_user = "?1".to_string();
 headers.sec_fetch_dest = "document".to_string();
 }
 }
 BrowserType::Safari => {
 headers.accept =
 "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8".to_string();
 headers.accept_encoding = "gzip, deflate, br".to_string();
 if !is_mobile {
 headers.sec_fetch_site = "none".to_string();
 headers.sec_fetch_mode = "navigate".to_string();
 headers.sec_fetch_user = "?1".to_string();
 headers.sec_fetch_dest = "document".to_string();
 }
 }
 BrowserType::Opera => {
 // Opera use Chrome inside核，headers similar Chrome
 headers.accept = "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.7".to_string();
 headers.accept_encoding = "gzip, deflate, br, zstd".to_string();
 headers.sec_fetch_site = "none".to_string();
 headers.sec_fetch_mode = "navigate".to_string();
 headers.sec_fetch_user = "?1".to_string();
 headers.sec_fetch_dest = "document".to_string();
 headers.upgrade_insecure_requests = "1".to_string();

 if is_mobile {
 headers.sec_ch_ua =
 r#""Opera";v="91", "Chromium";v="105", "Not A(Brand";v="8""#.to_string();
 headers.sec_ch_ua_mobile = "?1".to_string();
 headers.sec_ch_ua_platform = r#""Android""#.to_string();
 } else {
 headers.sec_ch_ua =
 r#""Opera";v="91", "Chromium";v="105", "Not A(Brand";v="8""#.to_string();
 headers.sec_ch_ua_mobile = "?0".to_string();
 headers.sec_ch_ua_platform = extract_platform(user_agent);
 }
 }
 BrowserType::Edge => {
 // Edge use Chrome inside核
 headers.accept = "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.7".to_string();
 headers.accept_encoding = "gzip, deflate, br, zstd".to_string();
 headers.sec_fetch_site = "none".to_string();
 headers.sec_fetch_mode = "navigate".to_string();
 headers.sec_fetch_user = "?1".to_string();
 headers.sec_fetch_dest = "document".to_string();
 headers.upgrade_insecure_requests = "1".to_string();
 }
 }

 // Accept-Language userandomlanguage
 headers.accept_language = random_language();

 headers
}

#[cfg(test)]
mod tests {
 use super::*;

 #[test]
 fn test_random_language() {
 let lang = random_language();
 assert!(!lang.is_empty());
 assert!(LANGUAGES.contains(&lang.as_str()));
 }

 #[test]
 fn test_generate_headers_chrome() {
 let ua = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";
 let headers = generate_headers(BrowserType::Chrome, ua, false);
 assert_eq!(headers.user_agent, ua);
 assert!(!headers.accept.is_empty());
 assert!(!headers.accept_language.is_empty());
 }

 #[test]
 fn test_http_headers_set() {
 let mut headers = HTTPHeaders::new();
 headers.set("Cookie", "session_id=abc123");
 assert_eq!(
 headers.custom.get("Cookie"),
 Some(&"session_id=abc123".to_string())
 );
 }

 #[test]
 fn test_http_headers_to_map() {
 let mut headers = HTTPHeaders::new();
 headers.user_agent = "test".to_string();
 headers.set("Cookie", "session_id=abc123");
 let map = headers.to_map();
 assert_eq!(map.get("User-Agent"), Some(&"test".to_string()));
 assert_eq!(map.get("Cookie"), Some(&"session_id=abc123".to_string()));
 }
}
