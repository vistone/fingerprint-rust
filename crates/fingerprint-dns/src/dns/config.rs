//! DNS configuration管理module
//!
//! support from  JSON、YAML、TOML format的configurationfileload DNS configuration

use crate::dns::types::{DNSConfig, DNSError};
use std::fs;
use std::path::Path;

///  from configurationfileload DNS configuration
/// automatic识别configurationfileformat（JSON、YAML、TOML）
pub fn load_config<P: AsRef<Path>>(path: P) -> Result<DNSConfig, DNSError> {
    let path = path.as_ref();
    let content = fs::read_to_string(path)?;

    // Based onfileextension名selectParse器
    let config: DNSConfig = match path.extension().and_then(|s| s.to_str()) {
        Some("json") => serde_json::from_str(&content).map_err(DNSError::Json)?,
        Some("yaml") | Some("yml") => {
            // use serde_yaml 直接反序列化
            serde_yaml::from_str(&content).map_err(|e| DNSError::Yaml(e.to_string()))?
        }
        Some("toml") => toml::from_str(&content)?,
        _ => {
            // try按 JSON Parse
            serde_json::from_str(&content).map_err(|_| {
                DNSError::Config(format!(
                    "unsupported config format: {:?}. Supported: json, yaml, toml",
                    path.extension()
                ))
            })?
        }
    };

    // Validateconfiguration
    config.validate()?;

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
