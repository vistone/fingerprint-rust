//! DNS 配置管理模块
//!
//! 支持从 JSON、YAML、TOML 格式的配置文件加载 DNS 配置

use crate::dns::types::{DNSConfig, DNSError};
use std::fs;
use std::path::Path;

/// 从配置文件加载 DNS 配置
/// 自动识别配置文件格式（JSON、YAML、TOML）
pub fn load_config<P: AsRef<Path>>(path: P) -> Result<DNSConfig, DNSError> {
    let path = path.as_ref();
    let content = fs::read_to_string(path)?;

    // 根据文件扩展名选择解析器
    let config = match path.extension().and_then(|s| s.to_str()) {
        Some("json") => serde_json::from_str(&content)
            .map_err(|e| DNSError::Json(e))?,
        Some("yaml") | Some("yml") => {
            #[cfg(feature = "dns")]
            {
                yaml_rust::YamlLoader::load_from_str(&content)
                    .map_err(|e| DNSError::Yaml(e.to_string()))?
                    .first()
                    .ok_or_else(|| DNSError::Yaml("empty YAML document".to_string()))
                    .and_then(|yaml| {
                        yaml_to_config(yaml)
                    })?
            }
            #[cfg(not(feature = "dns"))]
            {
                return Err(DNSError::Yaml("YAML support not enabled".to_string()));
            }
        }
        Some("toml") => {
            #[cfg(feature = "dns")]
            {
                toml::from_str(&content)?
            }
            #[cfg(not(feature = "dns"))]
            {
                return Err(DNSError::Toml(toml::de::Error::custom("TOML support not enabled")));
            }
        }
        _ => {
            // 尝试按 JSON 解析
            serde_json::from_str(&content)
                .map_err(|_| DNSError::Config(
                    format!("unsupported config format: {:?}. Supported: json, yaml, toml", 
                            path.extension())
                ))?
        }
    };

    // 验证配置
    config.validate()?;

    Ok(config)
}

#[cfg(feature = "dns")]
fn yaml_to_config(yaml: &yaml_rust::Yaml) -> Result<DNSConfig, DNSError> {

    let config = DNSConfig {
        ipinfo_token: yaml["ipinfoToken"].as_str()
            .or_else(|| yaml["ipinfotoken"].as_str())
            .ok_or_else(|| DNSError::Config("ipinfoToken is required".to_string()))?
            .to_string(),
        domain_list: yaml["domainList"]
            .as_vec()
            .ok_or_else(|| DNSError::Config("domainList must be an array".to_string()))?
            .iter()
            .map(|v| v.as_str().unwrap_or("").to_string())
            .filter(|s| !s.is_empty())
            .collect(),
        domain_ips_dir: yaml["domainIPsDir"]
            .as_str()
            .or_else(|| yaml["domainipsdir"].as_str())
            .unwrap_or("./data")
            .to_string(),
        interval: yaml["interval"].as_str().unwrap_or("2m").to_string(),
        max_concurrency: yaml["maxConcurrency"]
            .as_i64()
            .unwrap_or(500) as usize,
        dns_timeout: yaml["dnsTimeout"]
            .as_str()
            .unwrap_or("4s")
            .to_string(),
        http_timeout: yaml["httpTimeout"]
            .as_str()
            .unwrap_or("20s")
            .to_string(),
        max_ip_fetch_conc: yaml["maxIPFetchConc"]
            .as_i64()
            .unwrap_or(50) as usize,
    };

    if config.domain_list.is_empty() {
        return Err(DNSError::Config("domainList cannot be empty".to_string()));
    }

    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    #[test]
    fn test_load_json_config() {
        let temp_dir = PathBuf::from("/tmp/test_dns_config");
        fs::create_dir_all(&temp_dir).ok();
        let config_path = temp_dir.join("config.json");
        
        let json_content = r#"{
            "ipinfoToken": "test-token",
            "domainList": ["google.com", "github.com"],
            "domainIPsDir": "./data",
            "interval": "2m",
            "maxConcurrency": 500,
            "dnsTimeout": "4s",
            "httpTimeout": "20s",
            "maxIPFetchConc": 50
        }"#;

        fs::write(&config_path, json_content).unwrap();
        
        let config = load_config(&config_path).unwrap();
        assert_eq!(config.ipinfo_token, "test-token");
        assert_eq!(config.domain_list.len(), 2);
        assert_eq!(config.domain_list[0], "google.com");
    }
}

