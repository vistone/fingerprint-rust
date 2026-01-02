//! DNS moduletypedefine

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// IP addressdetailedinfo（Corresponds to Go version's IPInfo struct）
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct IPInfo {
 /// IP address
 pub ip: String,
 /// host名（optional）
 #[serde(skip_serializing_if = "Option::is_none")]
 pub hostname: Option<String>,
 /// 城市（optional）
 #[serde(skip_serializing_if = "Option::is_none")]
 pub city: Option<String>,
 /// 地区（optional）
 #[serde(skip_serializing_if = "Option::is_none")]
 pub region: Option<String>,
 /// 国家code（optional）
 #[serde(skip_serializing_if = "Option::is_none")]
 pub country: Option<String>,
 /// geographic坐标（optional）
 #[serde(skip_serializing_if = "Option::is_none")]
 pub loc: Option<String>,
 /// group织/ISP（optional）
 #[serde(skip_serializing_if = "Option::is_none")]
 pub org: Option<String>,
 /// when 区（optional）
 #[serde(skip_serializing_if = "Option::is_none")]
 pub timezone: Option<String>,
}

impl IPInfo {
 /// from IP addressCreateempty IPInfo
 pub fn new(ip: String) -> Self {
 Self {
 ip,
 hostname: None,
 city: None,
 region: None,
 country: None,
 loc: None,
 org: None,
 timezone: None,
 }
 }
}

/// domain IP addressinfo（Corresponds to Go version's DomainIPs struct）
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DomainIPs {
 /// IPv4 addresslist
 #[serde(default)]
 pub ipv4: Vec<IPInfo>,
 /// IPv6 addresslist
 #[serde(default)]
 pub ipv6: Vec<IPInfo>,
}

impl DomainIPs {
 /// Createempty DomainIPs
 pub fn new() -> Self {
 Self {
 ipv4: Vec::new(),
 ipv6: Vec::new(),
 }
 }

 /// Getall IP address（IPv4 + IPv6）
 pub fn all_ips(&self) -> Vec<String> {
 let mut ips = Vec::new();
 for info in &self.ipv4 {
 ips.push(info.ip.clone());
 }
 for info in &self.ipv6 {
 ips.push(info.ip.clone());
 }
 ips
 }

 /// Checkwhether有new IP address（ and 另an DomainIPs compare）
 ///
 /// `self` is 新Parse IP set，`other` is beforesave IP set
 /// If `self` in 有 `other` no IP, return true（discovernew IP）
 pub fn has_new_ips(&self, other: &DomainIPs) -> bool {
 let self_ips: HashSet<String> = self.all_ips().into_iter().collect();
 let other_ips: HashSet<String> = other.all_ips().into_iter().collect();

 // Check self is否有 other no IP
 self_ips.difference(&other_ips).next().is_some()
 }
}

impl Default for DomainIPs {
 fn default() -> Self {
 Self::new()
 }
}

/// DNS Parseresult
#[derive(Debug, Clone)]
pub struct DNSResult {
 /// domain
 pub domain: String,
 /// Parse to IP address（IPv4 and IPv6）
 pub ips: DomainIPs,
}

/// DNS configurationValidateerror
#[derive(Debug, thiserror::Error)]
pub enum DNSError {
 #[error("configurationerror: {0}")]
 Config(String),
 #[error("DNS Parseerror: {0}")]
 Resolver(String),
 #[error("IPInfo error: {0}")]
 IPInfo(String),
 #[error("storeerror: {0}")]
 Storage(String),
 #[error("IO error: {0}")]
 IO(#[from] std::io::Error),
 #[error("JSON error: {0}")]
 Json(#[from] serde_json::Error),
 #[error("YAML error: {0}")]
 Yaml(String),
 #[error("TOML Parseerror: {0}")]
 Toml(#[from] toml::de::Error),
 #[error("HTTP error: {0}")]
 Http(String),
 #[error("TOML serializeerror: {0}")]
 TomlSerialize(String),
 #[error("inside部error: {0}")]
 Internal(String),
}

/// DNS configurationstruct（Corresponds to Go version's Config struct）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DNSConfig {
 /// IPInfo.io API Token（必填）
 pub ipinfo_token: String,
 /// domainlist（必填）
 pub domain_list: Vec<String>,
 /// storedirectory（optional，defaultcurrentdirectory）
 #[serde(default = "default_domain_ips_dir")]
 pub domain_ips_dir: String,
 /// Checkinterval（optional，default "2m"）
 #[serde(default = "default_interval")]
 pub interval: String,
 /// DNS querymaximumconcurrentcount（optional，default 500）
 #[serde(default = "default_max_concurrency")]
 pub max_concurrency: usize,
 /// DNS querytimeout（optional，default "4s"）
 #[serde(default = "default_dns_timeout")]
 pub dns_timeout: String,
 /// HTTP Request timeout（optional，default "20s"）
 #[serde(default = "default_http_timeout")]
 pub http_timeout: String,
 /// IP infoGetmaximumconcurrentcount（optional，default 50）
 #[serde(default = "default_max_ip_fetch_conc")]
 pub max_ip_fetch_conc: usize,
}

fn default_domain_ips_dir() -> String {
 ".".to_string()
}

fn default_interval() -> String {
 "2m".to_string()
}

fn default_max_concurrency() -> usize {
 500
}

fn default_dns_timeout() -> String {
 "4s".to_string()
}

fn default_http_timeout() -> String {
 "20s".to_string()
}

fn default_max_ip_fetch_conc() -> usize {
 50
}

impl DNSConfig {
 /// Create a new DNS configuration（便利method，candirectlyusestring字面量）
 ///
 /// # Examples
 /// ```
 /// use fingerprint_dns::DNSConfig;
 ///
 /// let config = DNSConfig::new(
 /// "your-token",
 /// &["google.com", "github.com"], // candirectlyuse &str
 /// );
 /// ```
 pub fn new<S: AsRef<str>>(ipinfo_token: &str, domain_list: &[S]) -> Self {
 Self {
 ipinfo_token: ipinfo_token.to_string(),
 domain_list: domain_list.iter().map(|s| s.as_ref().to_string()).collect(),
 domain_ips_dir: default_domain_ips_dir(),
 interval: default_interval(),
 max_concurrency: default_max_concurrency(),
 dns_timeout: default_dns_timeout(),
 http_timeout: default_http_timeout(),
 max_ip_fetch_conc: default_max_ip_fetch_conc(),
 }
 }

 /// Validateconfiguration
 pub fn validate(&self) -> Result<(), DNSError> {
 if self.ipinfo_token.is_empty() {
 return Err(DNSError::Config("ipinfoToken is required".to_string()));
 }
 if self.domain_list.is_empty() {
 return Err(DNSError::Config("domainList cannot be empty".to_string()));
 }
 Ok(())
 }
}

#[cfg(test)]
mod tests {
 use super::*;

 #[test]
 fn test_has_new_ips() {
 let mut old_ips = DomainIPs::new();
 old_ips.ipv4.push(IPInfo::new("8.8.8.8".to_string()));
 old_ips.ipv4.push(IPInfo::new("8.8.4.4".to_string()));

 let mut new_ips = DomainIPs::new();
 new_ips.ipv4.push(IPInfo::new("8.8.8.8".to_string()));
 new_ips.ipv4.push(IPInfo::new("8.8.4.4".to_string()));
 new_ips.ipv4.push(IPInfo::new("1.1.1.1".to_string())); // new IP

 assert!(new_ips.has_new_ips(&old_ips), "shoulddetect to new IP 1.1.1.1");

 let mut same_ips = DomainIPs::new();
 same_ips.ipv4.push(IPInfo::new("8.8.8.8".to_string()));
 same_ips.ipv4.push(IPInfo::new("8.8.4.4".to_string()));

 assert!(!same_ips.has_new_ips(&old_ips), "same IP shouldreturn false");
 }
}
