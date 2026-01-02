//! HTTP requestBuilder

use fingerprint_headers::headers::HTTPHeaders;
use std::collections::HashMap;

/// HTTP method
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpMethod {
 Get,
 Post,
 Put,
 Delete,
 Head,
 Options,
 Patch,
}

impl HttpMethod {
 pub fn as_str(&self) -> &str {
 match self {
 HttpMethod::Get => "GET",
 HttpMethod::Post => "POST",
 HttpMethod::Put => "PUT",
 HttpMethod::Delete => "DELETE",
 HttpMethod::Head => "HEAD",
 HttpMethod::Options => "OPTIONS",
 HttpMethod::Patch => "PATCH",
 }
 }
}

/// HTTP request
#[derive(Debug, Clone)]
pub struct HttpRequest {
 pub method: HttpMethod,
 pub url: String,
 pub headers: HashMap<String, String>,
 pub body: Option<Vec<u8>>,
}

/// auxiliaryfunction： as requestAdd Cookie（ if exists）
pub fn add_cookies_to_request(
 request: &mut HttpRequest,
 cookie_store: &super::cookie::CookieStore,
 host: &str,
 path: &str,
 is_secure: bool,
) {
 if let Some(cookie_header) = cookie_store.generate_cookie_header(host, path, is_secure) {
 request.headers.insert("Cookie".to_string(), cookie_header);
 }
}

impl HttpRequest {
 /// Create a newrequest
 pub fn new(method: HttpMethod, url: &str) -> Self {
 Self {
 method,
 url: url.to_string(),
 headers: HashMap::new(),
 body: None,
 }
 }

 /// Add User-Agent
 pub fn with_user_agent(mut self, user_agent: &str) -> Self {
 self.headers
.insert("User-Agent".to_string(), user_agent.to_string());
 self
 }

 /// Addcustom header
 pub fn with_header(mut self, key: &str, value: &str) -> Self {
 self.headers.insert(key.to_string(), value.to_string());
 self
 }

 /// bulkAdd headers
 pub fn with_headers(mut self, headers: &HTTPHeaders) -> Self {
 let headers_map = headers.to_map();
 for (key, value) in headers_map {
 self.headers.insert(key, value);
 }
 self
 }

 /// settingsrequest体
 pub fn with_body(mut self, body: Vec<u8>) -> Self {
 self.body = Some(body);
 self
 }

 /// settings JSON request体
 pub fn with_json_body(mut self, json: &str) -> Self {
 self.headers
.insert("Content-Type".to_string(), "application/json".to_string());
 self.body = Some(json.as_bytes().to_vec());
 self
 }

 /// Build HTTP/1.1 requeststring
 ///
 /// Note: 该methodwill把 body when作 UTF-8 text拼接 to string in ，**不适 for 二进制 body**。
 /// 如需send二进制countdata，请use `build_http1_request_bytes`。
 pub fn build_http1_request(&self, host: &str, path: &str) -> String {
 // security清洗：prevent CRLF 注入
 let safe_method = self.method.as_str().replace(['\r', '\n'], "");
 let safe_path = path.replace(['\r', '\n'], "");
 let safe_host = host.replace(['\r', '\n'], "");

 let mut request = format!("{} {} HTTP/1.1\r\n", safe_method, safe_path);

 // Host header (required)
 request.push_str(&format!("Host: {}\r\n", safe_host));

 // Addother headers
 for (key, value) in &self.headers {
 if key.to_lowercase() != "host" {
 // securitycleanup Key and Value
 let safe_key = key.replace(['\r', '\n'], "");
 let safe_value = value.replace(['\r', '\n'], "");
 request.push_str(&format!("{}: {}\r\n", safe_key, safe_value));
 }
 }

 // Content-Length ( if 有 body)
 if let Some(ref body) = self.body {
 request.push_str(&format!("Content-Length: {}\r\n", body.len()));
 }

 // Connection: close (HTTP/1.1)
 if !self
.headers
.keys()
.any(|k| k.eq_ignore_ascii_case("connection"))
 {
 request.push_str("Connection: close\r\n");
 }

 // end headers
 request.push_str("\r\n");

 // Add body
 if let Some(ref body) = self.body {
 request.push_str(&String::from_utf8_lossy(body));
 }

 request
 }

 /// Build HTTP/1.1 requestbytes（推荐）
 pub fn build_http1_request_bytes(
 &self,
 host: &str,
 path: &str,
 header_order: Option<&[String]>,
 ) -> Vec<u8> {
 // security清洗
 let safe_method = self.method.as_str().replace(['\r', '\n'], "");
 let safe_path = path.replace(['\r', '\n'], "");
 let safe_host = host.replace(['\r', '\n'], "");

 let mut head = format!("{} {} HTTP/1.1\r\n", safe_method, safe_path);

 // use有序list（ if provide）
 let ordered_headers = if let Some(order) = header_order {
 let mut h = HTTPHeaders::new();
 for (k, v) in &self.headers {
 h.set(k, v);
 }
 h.to_ordered_vec(order)
 } else {
 self.headers
.iter()
.map(|(k, v)| (k.clone(), v.clone()))
.collect()
 };

 // Host header
 if !ordered_headers
.iter()
.any(|(k, _)| k.eq_ignore_ascii_case("host"))
 {
 head.push_str(&format!("Host: {}\r\n", safe_host));
 }

 // Addother headers
 for (key, value) in ordered_headers {
 let safe_key = key.replace(['\r', '\n'], "");
 let safe_value = value.replace(['\r', '\n'], "");
 head.push_str(&format!("{}: {}\r\n", safe_key, safe_value));
 }

 // Content-Length
 let body_len = self.body.as_ref().map(|b| b.len()).unwrap_or(0);
 if body_len > 0 {
 head.push_str(&format!("Content-Length: {}\r\n", body_len));
 }

 // Connection: close
 if !self
.headers
.keys()
.any(|k| k.eq_ignore_ascii_case("connection"))
 {
 head.push_str("Connection: close\r\n");
 }

 // end headers
 head.push_str("\r\n");

 let mut out = head.into_bytes();
 if let Some(ref body) = self.body {
 out.extend_from_slice(body);
 }
 out
 }

 /// random化 Header size写（simulate某些specificfingerprint or avoid WAF trait）
 pub fn with_randomized_header_case(&mut self) {
 let mut new_headers = HashMap::new();
 for (key, value) in self.headers.drain() {
 let mut randomized_key = String::new();
 for (i, c) in key.chars().enumerate() {
 if i % 2 == 0 {
 randomized_key.push(c.to_ascii_uppercase());
 } else {
 randomized_key.push(c.to_ascii_lowercase());
 }
 }
 new_headers.insert(randomized_key, value);
 }
 self.headers = new_headers;
 }
}

#[cfg(test)]
mod tests {
 use super::*;

 #[test]
 fn test_build_http1_request() {
 let request = HttpRequest::new(HttpMethod::Get, "http://example.com/test")
.with_user_agent("TestAgent/1.0")
.with_header("Accept", "text/html");

 let http1_request = request.build_http1_request("example.com", "/test");

 assert!(http1_request.contains("GET /test HTTP/1.1"));
 assert!(http1_request.contains("Host: example.com"));
 assert!(http1_request.contains("User-Agent: TestAgent/1.0"));
 assert!(http1_request.contains("Accept: text/html"));
 }

 #[test]
 fn test_post_with_body() {
 let body = b"test data";
 let request =
 HttpRequest::new(HttpMethod::Post, "http://example.com/api").with_body(body.to_vec());

 let http1_request = request.build_http1_request("example.com", "/api");

 assert!(http1_request.contains("POST /api HTTP/1.1"));
 assert!(http1_request.contains("Content-Length: 9"));
 assert!(http1_request.contains("test data"));
 }
}
