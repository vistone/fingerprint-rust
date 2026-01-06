//! DNS storemanagemodule
//!
//! provideoriginalchildpropertyfilewrite and multipleformatoutput (JSON, YAML, TOML)

use crate::dns::types::{DNSError, DomainIPs};
use std::fs;
use std::io::Write;
use std::path::Path;

/// willdomain IP infosave to file (originalchildpropertywrite)
/// support JSON, YAML, TOML threeformat
pub fn save_domain_ips<P: AsRef<Path>>(
    domain: &str,
    domain_ips: &DomainIPs,
    base_dir: P,
) -> Result<(), DNSError> {
    let base_dir = base_dir.as_ref();

    // ensuredirectory exists
    fs::create_dir_all(base_dir)?;

    // save as JSON
    let json_path = base_dir.join(format!("{}.json", domain));
    save_as_json(&json_path, domain_ips)?;

    // save as YAML
    let yaml_path = base_dir.join(format!("{}.yaml", domain));
    save_as_yaml(&yaml_path, domain_ips)?;

    // save as TOML
    let toml_path = base_dir.join(format!("{}.toml", domain));
    save_as_toml(&toml_path, domain_ips)?;

    Ok(())
}

/// from fileloaddomain IP info
/// automatictry JSON, YAML, TOML format
pub fn load_domain_ips<P: AsRef<Path>>(
    domain: &str,
    base_dir: P,
) -> Result<Option<DomainIPs>, DNSError> {
    let base_dir = base_dir.as_ref();

    //  by prioritytry：JSON -> YAML -> TOML
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

/// save as JSON (originalchildpropertywrite)
fn save_as_json(path: &Path, domain_ips: &DomainIPs) -> Result<(), DNSError> {
    let json_content = serde_json::to_string_pretty(domain_ips)?;
    atomic_write(path, json_content.as_bytes())?;
    Ok(())
}

/// from JSON load
fn load_from_json(path: &Path) -> Result<DomainIPs, DNSError> {
    let content = fs::read_to_string(path)?;
    let domain_ips: DomainIPs = serde_json::from_str(&content)?;
    Ok(domain_ips)
}

/// save as YAML (originalchildpropertywrite)
fn save_as_yaml(path: &Path, domain_ips: &DomainIPs) -> Result<(), DNSError> {
    // use serde_yaml directlyserialize
    let yaml_string =
        serde_yaml::to_string(domain_ips).map_err(|e| DNSError::Yaml(e.to_string()))?;
    atomic_write(path, yaml_string.as_bytes())?;
    Ok(())
}

/// from YAML load
fn load_from_yaml(path: &Path) -> Result<DomainIPs, DNSError> {
    let content = fs::read_to_string(path)?;
    // use serde_yaml directlyreverseserialize
    let domain_ips: DomainIPs =
        serde_yaml::from_str(&content).map_err(|e| DNSError::Yaml(e.to_string()))?;
    Ok(domain_ips)
}

/// save as TOML (originalchildpropertywrite)
fn save_as_toml(path: &Path, domain_ips: &DomainIPs) -> Result<(), DNSError> {
    let toml_content =
        toml::to_string_pretty(domain_ips).map_err(|e| DNSError::TomlSerialize(e.to_string()))?;
    atomic_write(path, toml_content.as_bytes())?;
    Ok(())
}

/// from TOML load
fn load_from_toml(path: &Path) -> Result<DomainIPs, DNSError> {
    let content = fs::read_to_string(path)?;
    let domain_ips: DomainIPs = toml::from_str(&content)?;
    Ok(domain_ips)
}

/// originalchildpropertywritefile
/// 先writetemporaryfile, thenrename, ensurecountdatasecurity
fn atomic_write(path: &Path, content: &[u8]) -> Result<(), DNSError> {
    let parent = path.parent().ok_or_else(|| {
        DNSError::IO(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "path has no parent directory",
        ))
    })?;

    fs::create_dir_all(parent)?;

    // Createtemporaryfile
    let temp_path = path.with_extension(".tmp");
    let mut temp_file = fs::File::create(&temp_path)?;
    temp_file.write_all(content)?;
    temp_file.sync_all()?;
    drop(temp_file);

    // originalchildpropertyrename
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
        domain_ips
            .ipv6
            .push(IPInfo::new("2001:4860:4860::8888".to_string()));

        save_domain_ips(domain, &domain_ips, &temp_dir).unwrap();

        let loaded = load_domain_ips(domain, &temp_dir).unwrap().unwrap();
        assert_eq!(loaded.ipv4.len(), 1);
        assert_eq!(loaded.ipv4[0].ip, "8.8.8.8");
    }
}
