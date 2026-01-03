//! p0f signaturedatabase parsed 
//!
//! parsed p0f.fp formatsignaturedatabasefile。

use crate::passive::p0f_parser;
use crate::passive::tcp::TcpSignature;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use thiserror::Error;

/// p0f signaturedatabase
pub struct P0fDatabase {
 /// TCP requestsignature
 tcp_request: HashMap<String, TcpSignature>,

 /// TCP responsesignature
 tcp_response: HashMap<String, TcpSignature>,

 /// HTTP requestsignature
 http_request: HashMap<String, P0fHttpSignature>,

 /// HTTP responsesignature
 http_response: HashMap<String, P0fHttpSignature>,
}

/// HTTP signature (p0f format)
#[derive(Debug, Clone)]
pub struct P0fHttpSignature {
 pub id: String,
 pub label: String,
 pub user_agent_pattern: Option<String>,
 pub headers: Vec<String>,
}

impl P0fDatabase {
 /// from fileload p0f database
 pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, P0fError> {
 let content = fs::read_to_string(path)?;
 Self::parse(&content)
 }

 /// parsed p0f databaseinside容
 pub fn parse(content: &str) -> Result<Self, P0fError> {
 let mut db = Self {
 tcp_request: HashMap::new(),
 tcp_response: HashMap::new(),
 http_request: HashMap::new(),
 http_response: HashMap::new(),
 };

 let mut current_section: Option<&str> = None;
 let mut current_label: Option<String> = None;

 for line in content.lines() {
 let line = line.trim();

 // skipempty row and comment
 if line.is_empty() || line.starts_with('#') {
 continue;
 }

 // Checkwhether is newpartial
 if line.starts_with('[') && line.ends_with(']') {
 current_section = Some(&line[1..line.len() - 1]);
 current_label = None;
 continue;
 }

 // parsed label
 if let Some(stripped) = line.strip_prefix("label = ") {
 current_label = Some(stripped.trim().to_string());
 continue;
 }

 // parsed sig
 if let Some(stripped) = line.strip_prefix("sig = ") {
 let sig_value = stripped.trim().to_string();

 // If have label and sig, try parsed 
 if let Some(label) = &current_label {
 if let Some(section) = current_section {
 match section {
 "tcp:request" => {
 if let Ok(tcp_sig) = Self::parse_tcp_signature(label, &sig_value) {
 db.tcp_request.insert(tcp_sig.id.clone(), tcp_sig);
 }
 }
 "tcp:response" => {
 if let Ok(tcp_sig) = Self::parse_tcp_signature(label, &sig_value) {
 db.tcp_response.insert(tcp_sig.id.clone(), tcp_sig);
 }
 }
 "http:request" => {
 if let Ok(http_sig) = Self::parse_http_signature(label, &sig_value)
 {
 db.http_request.insert(http_sig.id.clone(), http_sig);
 }
 }
 "http:response" => {
 if let Ok(http_sig) = Self::parse_http_signature(label, &sig_value)
 {
 db.http_response.insert(http_sig.id.clone(), http_sig);
 }
 }
 _ => {}
 }
 }
 }
 continue;
 }
 }

 Ok(db)
 }

 /// parsed TCP signature (usedetailed parsed er)
 fn parse_tcp_signature(label: &str, sig: &str) -> Result<TcpSignature, P0fError> {
 // usedetailed parsed er
 let p0f_sig = p0f_parser::parse_tcp_signature(label, sig)
.map_err(|e| P0fError:: parsed (e.to_string()))?;

 // convert to TcpSignature
 Ok(p0f_sig.into())
 }

 /// parsed HTTP signature
 fn parse_http_signature(label: &str, _sig: &str) -> Result<P0fHttpSignature, P0fError> {
 // simplify parsed 
 Ok(P0fHttpSignature {
 id: format!("http-{}", label.replace(':', "-")),
 label: label.to_string(),
 user_agent_pattern: None,
 headers: Vec::new(),
 })
 }

 /// Get TCP requestsignature
 pub fn get_tcp_request(&self, id: &str) -> Option<&TcpSignature> {
 self.tcp_request.get(id)
 }

 /// Get all TCP requestsignature
 pub fn get_ all _tcp_request(&self) -> Vec<&TcpSignature> {
 self.tcp_request.values().collect()
 }

 /// Get all TCP responsesignature
 pub fn get_ all _tcp_response(&self) -> Vec<&TcpSignature> {
 self.tcp_response.values().collect()
 }

 /// Get all HTTP requestsignature
 pub fn get_ all _http_request(&self) -> Vec<&P0fHttpSignature> {
 self.http_request.values().collect()
 }

 /// Get all HTTP responsesignature
 pub fn get_ all _http_response(&self) -> Vec<&P0fHttpSignature> {
 self.http_response.values().collect()
 }

 /// Getstatisticsinfo
 pub fn stats(&self) -> P0fStats {
 P0fStats {
 tcp_request_count: self.tcp_request.len(),
 tcp_response_count: self.tcp_response.len(),
 http_request_count: self.http_request.len(),
 http_response_count: self.http_response.len(),
 }
 }
}

/// p0f databasestatisticsinfo
#[derive(Debug)]
pub struct P0fStats {
 pub tcp_request_count: usize,
 pub tcp_response_count: usize,
 pub http_request_count: usize,
 pub http_response_count: usize,
}

/// p0f error
#[derive(Debug, Error)]
pub enum P0fError {
 #[error("IO error: {0}")]
 Io(#[from] std::io::Error),

 #[error("invalidformat")]
 InvalidFormat,

 #[error(" parsed error: {0}")]
 parsed (String),
}

#[cfg(test)]
mod tests {
 use super::*;

 #[test]
 fn test_print_ all _p0f_data() {
 println!("\n╔════════════════════════════════════════════════════════════════╗");
 println!("║ print p0f all countdata ║");
 println!("╚════════════════════════════════════════════════════════════════╝\n");

 // try from commonbit置load p0f database
 let p0f_paths = vec![
 "p0f.fp",
 "/etc/p0f/p0f.fp",
 "/usr/share/p0f/p0f.fp",
 "crates/fingerprint-defense/p0f.fp",
 "fingerprint-defense/p0f.fp",
 ];

 let mut db: Option<P0fDatabase> = None;

 for path in &p0f_paths {
 if Path::new(path).exists() {
 println!("📂 找 to p0f databasefile: {}", path);
 match P0fDatabase::load_from_file(path) {
 Ok(database) => {
 db = Some(database);
 println!("✅ successload p0f database (path: {})\n", path);
 break;
 }
 Err(e) => {
 println!("❌ load failure: {}\n", e);
 }
 }
 }
 }

 if db.is_none() {
 println!("⚠️ not找 to p0f databasefile");
 println!(" please ensure p0f.fp file exists于belowbit置之一：");
 for path in &p0f_paths {
 println!(" - {}", path);
 }
 println!("\n or 者CreateanExamplesdatabase perform test");

 // CreateanExamplesdatabase for demo
 println!("\n【CreateExamplesdatabase for demo】\n");
 let example_content = r#"
[tcp:request]
label = s:unix:Linux:3.x
sig = *:64:0:*:mss*20,10:mss,sok,ts,nop,ws:df,id+:0

[tcp:response]
label = s:unix:Linux:3.x
sig = *:64:0:*:mss*20,10:mss,sok,ts,nop,ws:df,id+:0
"#;

 match P0fDatabase::parse(example_content) {
 Ok(database) => {
 db = Some(database);
 println!("✅ Usage Exampledatabase\n");
 }
 Err(e) => {
 println!("❌ parsed Examplesdatabase failure: {}\n", e);
 return;
 }
 }
 }

 let db = db.unwrap();

 // printstatisticsinfo
 let stats = db.stats();
 println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
 println!("【p0f databasestatistics】");
 println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
 println!(" TCP requestsignature: {} ", stats.tcp_request_count);
 println!(" TCP responsesignature: {} ", stats.tcp_response_count);
 println!(" HTTP requestsignature: {} ", stats.http_request_count);
 println!(" HTTP responsesignature: {} ", stats.http_response_count);
 println!();

 // print all TCP requestsignature
 println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
 println!("【TCP requestsignature】");
 println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

 let tcp_requests = db.get_ all _tcp_request();
 println!("total: {} signature\n", tcp_requests.len());

 for (i, sig) in tcp_requests.iter().enumerate() {
 println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
 println!("signature #{}: {}", i + 1, sig.id);
 println!(" operating system : {:?}", sig.os_type);
 println!(" TTL: {}", sig.ttl);
 println!(" window size: {}", sig. window _size);
 println!(" MSS: {:?}", sig.mss);
 println!(" Window Scale: {:?}", sig. window _scale);
 println!(" confidence: {:.2}", sig.confidence);
 println!(" samplecount: {}", sig.sample_count);
 println!();
 }

 // print all TCP responsesignature
 println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
 println!("【TCP responsesignature】");
 println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

 let tcp_responses = db.get_ all _tcp_response();
 println!("total: {} signature\n", tcp_responses.len());

 for (i, sig) in tcp_responses.iter().enumerate() {
 println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
 println!("signature #{}: {}", i + 1, sig.id);
 println!(" operating system : {:?}", sig.os_type);
 println!(" TTL: {}", sig.ttl);
 println!(" window size: {}", sig. window _size);
 println!(" MSS: {:?}", sig.mss);
 println!(" Window Scale: {:?}", sig. window _scale);
 println!(" confidence: {:.2}", sig.confidence);
 println!(" samplecount: {}", sig.sample_count);
 println!();
 }

 // print all HTTP requestsignature
 println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
 println!("【HTTP requestsignature】");
 println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

 let http_requests = db.get_ all _http_request();
 println!("total: {} signature\n", http_requests.len());

 for (i, sig) in http_requests.iter().enumerate() {
 println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
 println!("signature #{}: {}", i + 1, sig.id);
 println!(" tag: {}", sig.label);
 println!(" User-Agent pattern: {:?}", sig.user_agent_pattern);
 println!(" Headers: {:?}", sig.headers);
 println!();
 }

 // print all HTTP responsesignature
 println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
 println!("【HTTP responsesignature】");
 println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

 let http_responses = db.get_ all _http_response();
 println!("total: {} signature\n", http_responses.len());

 for (i, sig) in http_responses.iter().enumerate() {
 println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
 println!("signature #{}: {}", i + 1, sig.id);
 println!(" tag: {}", sig.label);
 println!(" User-Agent pattern: {:?}", sig.user_agent_pattern);
 println!(" Headers: {:?}", sig.headers);
 println!();
 }

 println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
 println!("✅ all p0f countdataprintcomplete！");
 println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
 }
}
