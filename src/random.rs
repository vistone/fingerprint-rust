//! 随机指纹生成模块
//!
//! 提供随机获取指纹和 User-Agent 的功能

use crate::headers::generate_headers;
use crate::profiles::{mapped_tls_clients, ClientProfile};
use crate::types::{BrowserType, OperatingSystem};
use crate::useragent::{get_user_agent_by_profile_name, get_user_agent_by_profile_name_with_os};
use crate::utils::{infer_browser_from_profile_name, is_mobile_profile, random_choice_string};

/// 指纹结果，包含指纹、User-Agent 和标准 HTTP Headers
#[derive(Debug, Clone)]
pub struct FingerprintResult {
    /// TLS 指纹配置
    pub profile: ClientProfile,
    /// 对应的 User-Agent
    pub user_agent: String,
    /// Client Hello ID（与 tls-client 保持一致）
    pub hello_client_id: String,
    /// 标准 HTTP 请求头（包含全球语言支持）
    pub headers: crate::headers::HTTPHeaders,
}

/// 浏览器类型未找到错误
#[derive(Debug, Clone)]
pub struct BrowserNotFoundError {
    pub browser: String,
}

impl std::fmt::Display for BrowserNotFoundError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "browser type not found: {}", self.browser)
    }
}

impl std::error::Error for BrowserNotFoundError {}

/// 随机获取一个指纹和对应的 User-Agent
/// 操作系统会随机选择
pub fn get_random_fingerprint() -> Result<FingerprintResult, String> {
    get_random_fingerprint_with_os(None)
}

/// 随机获取一个指纹和对应的 User-Agent，并指定操作系统
/// 如果 os 为 None，则随机选择操作系统
pub fn get_random_fingerprint_with_os(os: Option<OperatingSystem>) -> Result<FingerprintResult, String> {
    let clients = mapped_tls_clients();
    if clients.is_empty() {
        return Err("no TLS client profiles available".to_string());
    }

    // 获取所有可用的指纹名称
    let names: Vec<String> = clients.keys().cloned().collect();

    // 随机选择一个（线程安全）
    let random_name = random_choice_string(&names.iter().map(|s| s.as_str()).collect::<Vec<_>>())
        .ok_or_else(|| "failed to select random profile".to_string())?;

    let profile = clients
        .get(&random_name)
        .ok_or_else(|| format!("profile {} not found", random_name))?
        .clone();

    if profile.get_client_hello_str().is_empty() {
        return Err(format!("profile {} is invalid (empty ClientHelloStr)", random_name));
    }

    // 获取对应的 User-Agent
    let ua = match os {
        Some(os) => get_user_agent_by_profile_name_with_os(&random_name, os)?,
        None => get_user_agent_by_profile_name(&random_name)?,
    };

    // 生成标准 HTTP Headers
    let (browser_type_str, _) = infer_browser_from_profile_name(&random_name);
    let is_mobile = is_mobile_profile(&random_name);
    let browser_type = BrowserType::from_str(&browser_type_str)
        .unwrap_or(BrowserType::Chrome);
    let headers = generate_headers(browser_type, &ua, is_mobile);

    let hello_client_id = profile.get_client_hello_str();
    Ok(FingerprintResult {
        profile,
        user_agent: ua,
        hello_client_id,
        headers,
    })
}

/// 根据浏览器类型随机获取指纹和 User-Agent
/// browser_type: "chrome", "firefox", "safari", "opera" 等
pub fn get_random_fingerprint_by_browser(browser_type: &str) -> Result<FingerprintResult, Box<dyn std::error::Error>> {
    get_random_fingerprint_by_browser_with_os(browser_type, None)
}

/// 根据浏览器类型随机获取指纹和 User-Agent，并指定操作系统
pub fn get_random_fingerprint_by_browser_with_os(
    browser_type: &str,
    os: Option<OperatingSystem>,
) -> Result<FingerprintResult, Box<dyn std::error::Error>> {
    if browser_type.is_empty() {
        return Err("browser type cannot be empty".into());
    }

    let clients = mapped_tls_clients();
    if clients.is_empty() {
        return Err("no TLS client profiles available".into());
    }

    let browser_type_lower = browser_type.to_lowercase();

    // 筛选出指定浏览器类型的指纹
    let candidates: Vec<String> = clients
        .keys()
        .filter(|name| {
            let name_lower = name.to_lowercase();
            name_lower.starts_with(&format!("{}_", browser_type_lower))
        })
        .cloned()
        .collect();

    if candidates.is_empty() {
        return Err(Box::new(BrowserNotFoundError {
            browser: browser_type.to_string(),
        }));
    }

    // 随机选择一个（线程安全）
    let random_name = random_choice_string(&candidates.iter().map(|s| s.as_str()).collect::<Vec<_>>())
        .ok_or_else(|| "failed to select random profile".to_string())?;

    let profile = clients
        .get(&random_name)
        .ok_or_else(|| format!("profile {} not found", random_name))?
        .clone();

    if profile.get_client_hello_str().is_empty() {
        return Err(format!("profile {} is invalid (empty ClientHelloStr)", random_name).into());
    }

    // 获取对应的 User-Agent
    let ua = match os {
        Some(os) => get_user_agent_by_profile_name_with_os(&random_name, os)?,
        None => get_user_agent_by_profile_name(&random_name)?,
    };

    // 生成标准 HTTP Headers
    let (browser_type_str, _) = infer_browser_from_profile_name(&random_name);
    let is_mobile = is_mobile_profile(&random_name);
    let browser_type_enum = BrowserType::from_str(&browser_type_str)
        .unwrap_or(BrowserType::Chrome);
    let headers = generate_headers(browser_type_enum, &ua, is_mobile);

    let hello_client_id = profile.get_client_hello_str();
    Ok(FingerprintResult {
        profile,
        user_agent: ua,
        hello_client_id,
        headers,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_random_fingerprint() {
        let result = get_random_fingerprint();
        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(!result.user_agent.is_empty());
        assert!(!result.hello_client_id.is_empty());
    }

    #[test]
    fn test_get_random_fingerprint_by_browser() {
        let result = get_random_fingerprint_by_browser("chrome");
        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.user_agent.contains("Chrome"));
    }

    #[test]
    fn test_get_random_fingerprint_by_browser_not_found() {
        let result = get_random_fingerprint_by_browser("unknown");
        assert!(result.is_err());
    }
}
