//! DNS 模块类型定义

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// IP 地址详细信息（对应 Go 版本的 IPInfo 结构）
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct IPInfo {
    /// IP 地址
    pub ip: String,
    /// 主机名（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hostname: Option<String>,
    /// 城市（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub city: Option<String>,
    /// 地区（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub region: Option<String>,
    /// 国家代码（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country: Option<String>,
    /// 地理坐标（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub loc: Option<String>,
    /// 组织/ISP（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub org: Option<String>,
    /// 时区（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timezone: Option<String>,
}

impl IPInfo {
    /// 从 IP 地址创建空的 IPInfo
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

/// 域名 IP 地址信息（对应 Go 版本的 DomainIPs 结构）
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DomainIPs {
    /// IPv4 地址列表
    #[serde(default)]
    pub ipv4: Vec<IPInfo>,
    /// IPv6 地址列表
    #[serde(default)]
    pub ipv6: Vec<IPInfo>,
}

impl DomainIPs {
    /// 创建空的 DomainIPs
    pub fn new() -> Self {
        Self {
            ipv4: Vec::new(),
            ipv6: Vec::new(),
        }
    }

    /// 获取所有 IP 地址（IPv4 + IPv6）
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

    /// 检查是否有新的 IP 地址（与另一个 DomainIPs 比较）
    /// 
    /// `self` 是新解析的 IP 集合，`other` 是之前保存的 IP 集合
    /// 如果 `self` 中有 `other` 没有的 IP，返回 true（发现新 IP）
    pub fn has_new_ips(&self, other: &DomainIPs) -> bool {
        let self_ips: HashSet<String> = self.all_ips().into_iter().collect();
        let other_ips: HashSet<String> = other.all_ips().into_iter().collect();

        // 检查 self 中是否有 other 没有的 IP
        self_ips.difference(&other_ips).next().is_some()
    }
}

impl Default for DomainIPs {
    fn default() -> Self {
        Self::new()
    }
}

/// DNS 解析结果
#[derive(Debug, Clone)]
pub struct DNSResult {
    /// 域名
    pub domain: String,
    /// 解析到的 IP 地址（IPv4 和 IPv6）
    pub ips: DomainIPs,
}

/// DNS 配置验证错误
#[derive(Debug, thiserror::Error)]
pub enum DNSError {
    #[error("配置错误: {0}")]
    Config(String),
    #[error("DNS 解析错误: {0}")]
    Resolver(String),
    #[error("IPInfo 错误: {0}")]
    IPInfo(String),
    #[error("存储错误: {0}")]
    Storage(String),
    #[error("IO 错误: {0}")]
    IO(#[from] std::io::Error),
    #[error("JSON 错误: {0}")]
    Json(#[from] serde_json::Error),
    #[error("YAML 错误: {0}")]
    Yaml(String),
    #[error("TOML 解析错误: {0}")]
    Toml(#[from] toml::de::Error),
    #[error("HTTP 错误: {0}")]
    Http(String),
    #[error("TOML 序列化错误: {0}")]
    TomlSerialize(String),
}

/// DNS 配置结构（对应 Go 版本的 Config 结构）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DNSConfig {
    /// IPInfo.io API Token（必填）
    pub ipinfo_token: String,
    /// 域名列表（必填）
    pub domain_list: Vec<String>,
    /// 存储目录（可选，默认当前目录）
    #[serde(default = "default_domain_ips_dir")]
    pub domain_ips_dir: String,
    /// 检查间隔（可选，默认 "2m"）
    #[serde(default = "default_interval")]
    pub interval: String,
    /// DNS 查询最大并发数（可选，默认 500）
    #[serde(default = "default_max_concurrency")]
    pub max_concurrency: usize,
    /// DNS 查询超时（可选，默认 "4s"）
    #[serde(default = "default_dns_timeout")]
    pub dns_timeout: String,
    /// HTTP 请求超时（可选，默认 "20s"）
    #[serde(default = "default_http_timeout")]
    pub http_timeout: String,
    /// IP 信息获取最大并发数（可选，默认 50）
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
    /// 创建新的 DNS 配置（便利方法，可以直接使用字符串字面量）
    /// 
    /// # 示例
    /// ```
    /// let config = DNSConfig::new(
    ///     "your-token",
    ///     &["google.com", "github.com"],  // 可以直接使用 &str
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

    /// 验证配置
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
        new_ips.ipv4.push(IPInfo::new("1.1.1.1".to_string())); // 新 IP

        assert!(new_ips.has_new_ips(&old_ips), "应该检测到新 IP 1.1.1.1");

        let mut same_ips = DomainIPs::new();
        same_ips.ipv4.push(IPInfo::new("8.8.8.8".to_string()));
        same_ips.ipv4.push(IPInfo::new("8.8.4.4".to_string()));

        assert!(!same_ips.has_new_ips(&old_ips), "相同 IP 应该返回 false");
    }
}
