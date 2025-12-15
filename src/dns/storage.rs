//! DNS 存储管理模块
//!
//! 提供原子性文件写入和多格式输出（JSON、YAML、TOML）

use crate::dns::types::{DomainIPs, DNSError};
use std::fs;
use std::io::Write;
use std::path::Path;

/// 将域名 IP 信息保存到文件（原子性写入）
/// 支持 JSON、YAML、TOML 三种格式
pub fn save_domain_ips<P: AsRef<Path>>(
    domain: &str,
    domain_ips: &DomainIPs,
    base_dir: P,
) -> Result<(), DNSError> {
    let base_dir = base_dir.as_ref();
    
    // 确保目录存在
    fs::create_dir_all(base_dir)?;

    // 保存为 JSON
    let json_path = base_dir.join(format!("{}.json", domain));
    save_as_json(&json_path, domain_ips)?;

    // 保存为 YAML
    let yaml_path = base_dir.join(format!("{}.yaml", domain));
    save_as_yaml(&yaml_path, domain_ips)?;

    // 保存为 TOML
    let toml_path = base_dir.join(format!("{}.toml", domain));
    save_as_toml(&toml_path, domain_ips)?;

    Ok(())
}

/// 从文件加载域名 IP 信息
/// 自动尝试 JSON、YAML、TOML 格式
pub fn load_domain_ips<P: AsRef<Path>>(
    domain: &str,
    base_dir: P,
) -> Result<Option<DomainIPs>, DNSError> {
    let base_dir = base_dir.as_ref();

    // 按优先级尝试：JSON -> YAML -> TOML
    let json_path = base_dir.join(format!("{}.json", domain));
    if json_path.exists() {
        return Ok(Some(load_from_json(&json_path)?));
    }

    let yaml_path = base_dir.join(format!("{}.yaml", domain));
    if yaml_path.exists() {
        return Ok(Some(load_from_yaml(&yaml_path)?));
    }

    let toml_path = base_dir.join(format!("{}.toml", domain));
    if toml_path.exists() {
        return Ok(Some(load_from_toml(&toml_path)?));
    }

    Ok(None)
}

/// 保存为 JSON（原子性写入）
fn save_as_json(path: &Path, domain_ips: &DomainIPs) -> Result<(), DNSError> {
    let json_content = serde_json::to_string_pretty(domain_ips)?;
    atomic_write(path, json_content.as_bytes())?;
    Ok(())
}

/// 从 JSON 加载
fn load_from_json(path: &Path) -> Result<DomainIPs, DNSError> {
    let content = fs::read_to_string(path)?;
    let domain_ips: DomainIPs = serde_json::from_str(&content)?;
    Ok(domain_ips)
}

/// 保存为 YAML（原子性写入）
fn save_as_yaml(path: &Path, domain_ips: &DomainIPs) -> Result<(), DNSError> {
    #[cfg(feature = "dns")]
    {
        // 使用 serde_yaml 直接序列化
        let yaml_string = serde_yaml::to_string(domain_ips)
            .map_err(|e| DNSError::Yaml(e.to_string()))?;
        atomic_write(path, yaml_string.as_bytes())?;
        Ok(())
    }
    #[cfg(not(feature = "dns"))]
    {
        Err(DNSError::Yaml("YAML support not enabled".to_string()))
    }
}

/// 从 YAML 加载
fn load_from_yaml(path: &Path) -> Result<DomainIPs, DNSError> {
    #[cfg(feature = "dns")]
    {
        let content = fs::read_to_string(path)?;
        // 使用 serde_yaml 直接反序列化
        let domain_ips: DomainIPs = serde_yaml::from_str(&content)
            .map_err(|e| DNSError::Yaml(e.to_string()))?;
        Ok(domain_ips)
    }
    #[cfg(not(feature = "dns"))]
    {
        Err(DNSError::Yaml("YAML support not enabled".to_string()))
    }
}

/// 保存为 TOML（原子性写入）
fn save_as_toml(path: &Path, domain_ips: &DomainIPs) -> Result<(), DNSError> {
        let toml_content = toml::to_string_pretty(domain_ips)
            .map_err(|e| DNSError::TomlSerialize(e.to_string()))?;
    atomic_write(path, toml_content.as_bytes())?;
    Ok(())
}

/// 从 TOML 加载
fn load_from_toml(path: &Path) -> Result<DomainIPs, DNSError> {
    let content = fs::read_to_string(path)?;
    let domain_ips: DomainIPs = toml::from_str(&content)?;
    Ok(domain_ips)
}

/// 原子性写入文件
/// 先写入临时文件，然后重命名，确保数据安全
fn atomic_write(path: &Path, content: &[u8]) -> Result<(), DNSError> {
    let parent = path.parent()
        .ok_or_else(|| DNSError::IO(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "path has no parent directory"
        )))?;
    
    fs::create_dir_all(parent)?;
    
    // 创建临时文件
    let temp_path = path.with_extension(".tmp");
    let mut temp_file = fs::File::create(&temp_path)?;
    temp_file.write_all(content)?;
    temp_file.sync_all()?;
    drop(temp_file);
    
    // 原子性重命名
    fs::rename(&temp_path, path)?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dns::types::IPInfo;

    #[test]
    fn test_save_and_load_domain_ips() {
        use std::fs;
        use std::path::PathBuf;
        
        let temp_dir = PathBuf::from("/tmp/test_dns_storage");
        fs::create_dir_all(&temp_dir).ok();
        let domain = "test.com";
        
        let mut domain_ips = DomainIPs::new();
        domain_ips.ipv4.push(IPInfo::new("8.8.8.8".to_string()));
        domain_ips.ipv6.push(IPInfo::new("2001:4860:4860::8888".to_string()));

        save_domain_ips(domain, &domain_ips, &temp_dir).unwrap();
        
        let loaded = load_domain_ips(domain, &temp_dir).unwrap().unwrap();
        assert_eq!(loaded.ipv4.len(), 1);
        assert_eq!(loaded.ipv4[0].ip, "8.8.8.8");
    }
}

