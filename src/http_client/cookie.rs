//! Cookie 管理器
//!
//! 用于管理 HTTP Cookie 的存储、发送和接收

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

/// Cookie
#[derive(Debug, Clone)]
pub struct Cookie {
    pub name: String,
    pub value: String,
    pub domain: String,
    pub path: String,
    pub expires: Option<SystemTime>,
    pub max_age: Option<Duration>,
    pub secure: bool,
    pub http_only: bool,
    pub same_site: Option<SameSite>,
}

/// SameSite 属性
#[derive(Debug, Clone, PartialEq)]
pub enum SameSite {
    Strict,
    Lax,
    None,
}

impl Cookie {
    /// 创建新的 Cookie
    pub fn new(name: String, value: String, domain: String) -> Self {
        Self {
            name,
            value,
            domain,
            path: "/".to_string(),
            expires: None,
            max_age: None,
            secure: false,
            http_only: false,
            same_site: None,
        }
    }

    /// 检查 Cookie 是否过期
    pub fn is_expired(&self) -> bool {
        if let Some(expires) = self.expires {
            return SystemTime::now() > expires;
        }
        false
    }

    /// 转换为 HTTP Cookie 头格式
    pub fn to_header_value(&self) -> String {
        format!("{}={}", self.name, self.value)
    }

    /// 从 Set-Cookie 头解析
    pub fn parse_set_cookie(header: &str, domain: String) -> Option<Self> {
        let parts: Vec<&str> = header.split(';').collect();
        if parts.is_empty() {
            return None;
        }

        // 解析 name=value
        let name_value: Vec<&str> = parts[0].split('=').collect();
        if name_value.len() != 2 {
            return None;
        }

        let mut cookie = Cookie::new(
            name_value[0].trim().to_string(),
            name_value[1].trim().to_string(),
            domain,
        );

        // 解析其他属性
        for part in &parts[1..] {
            let part = part.trim();
            if part.to_lowercase().starts_with("domain=") {
                cookie.domain = part[7..].to_string();
            } else if part.to_lowercase().starts_with("path=") {
                cookie.path = part[5..].to_string();
            } else if part.to_lowercase().starts_with("max-age=") {
                if let Ok(secs) = part[8..].parse::<u64>() {
                    cookie.max_age = Some(Duration::from_secs(secs));
                }
            } else if part.to_lowercase() == "secure" {
                cookie.secure = true;
            } else if part.to_lowercase() == "httponly" {
                cookie.http_only = true;
            } else if part.to_lowercase().starts_with("samesite=") {
                let value = part[9..].to_lowercase();
                cookie.same_site = match value.as_str() {
                    "strict" => Some(SameSite::Strict),
                    "lax" => Some(SameSite::Lax),
                    "none" => Some(SameSite::None),
                    _ => None,
                };
            }
        }

        Some(cookie)
    }
}

/// Cookie 存储
#[derive(Debug, Clone)]
pub struct CookieStore {
    /// 按域名存储 Cookie
    cookies: Arc<Mutex<HashMap<String, Vec<Cookie>>>>,
}

impl CookieStore {
    /// 创建新的 Cookie 存储
    pub fn new() -> Self {
        Self {
            cookies: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// 添加 Cookie
    pub fn add_cookie(&self, cookie: Cookie) {
        if let Ok(mut cookies) = self.cookies.lock() {
            let domain_cookies = cookies.entry(cookie.domain.clone()).or_default();

            // 检查是否已存在同名 Cookie，如果存在则更新
            if let Some(pos) = domain_cookies.iter().position(|c| c.name == cookie.name) {
                domain_cookies[pos] = cookie;
            } else {
                domain_cookies.push(cookie);
            }
        } else {
            eprintln!("警告: Cookie 存储锁失败，无法添加 Cookie");
        }
    }

    /// 从 Set-Cookie 头添加 Cookie
    pub fn add_from_response(&self, set_cookie_header: &str, domain: String) {
        if let Some(cookie) = Cookie::parse_set_cookie(set_cookie_header, domain) {
            self.add_cookie(cookie);
        }
    }

    /// 获取指定域名的所有有效 Cookie
    pub fn get_cookies_for_domain(&self, domain: &str) -> Vec<Cookie> {
        let cookies = match self.cookies.lock() {
            Ok(c) => c,
            Err(e) => {
                eprintln!("警告: Cookie 存储锁失败: {}", e);
                return Vec::new();
            }
        };
        let mut result = Vec::new();

        // 检查完全匹配和父域名（更严格的匹配逻辑）
        let domain_lower = domain.to_lowercase();
        for (cookie_domain, domain_cookies) in cookies.iter() {
            let cookie_domain_lower = cookie_domain.to_lowercase();
            // 完全匹配或 domain 是 cookie_domain 的子域名（如 example.com 匹配 .example.com）
            if domain_lower == cookie_domain_lower
                || (cookie_domain_lower.starts_with('.')
                    && domain_lower.ends_with(&cookie_domain_lower))
                || (domain_lower.ends_with(&format!(".{}", cookie_domain_lower)))
            {
                for cookie in domain_cookies {
                    if !cookie.is_expired() {
                        result.push(cookie.clone());
                    }
                }
            }
        }

        result
    }

    /// 生成 Cookie 头
    pub fn generate_cookie_header(&self, domain: &str, path: &str) -> Option<String> {
        let cookies = self.get_cookies_for_domain(domain);
        if cookies.is_empty() {
            return None;
        }

        // 过滤路径匹配的 Cookie
        let matching_cookies: Vec<String> = cookies
            .iter()
            .filter(|c| path.starts_with(&c.path))
            .map(Cookie::to_header_value)
            .collect();

        if matching_cookies.is_empty() {
            None
        } else {
            Some(matching_cookies.join("; "))
        }
    }

    /// 清除指定域名的 Cookie
    pub fn clear_domain(&self, domain: &str) {
        if let Ok(mut cookies) = self.cookies.lock() {
            cookies.remove(domain);
        }
    }

    /// 清除所有 Cookie
    pub fn clear_all(&self) {
        if let Ok(mut cookies) = self.cookies.lock() {
            cookies.clear();
        }
    }

    /// 清除过期的 Cookie
    pub fn cleanup_expired(&self) {
        if let Ok(mut cookies) = self.cookies.lock() {
            for domain_cookies in cookies.values_mut() {
                domain_cookies.retain(|c| !c.is_expired());
            }
        }
    }

    /// 获取所有 Cookie 数量
    pub fn count(&self) -> usize {
        self.cookies
            .lock()
            .map(|cookies| cookies.values().map(|v| v.len()).sum())
            .unwrap_or(0)
    }
}

impl Default for CookieStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cookie_creation() {
        let cookie = Cookie::new(
            "session".to_string(),
            "abc123".to_string(),
            "example.com".to_string(),
        );
        assert_eq!(cookie.name, "session");
        assert_eq!(cookie.value, "abc123");
        assert_eq!(cookie.domain, "example.com");
    }

    #[test]
    fn test_cookie_to_header() {
        let cookie = Cookie::new(
            "session".to_string(),
            "abc123".to_string(),
            "example.com".to_string(),
        );
        assert_eq!(cookie.to_header_value(), "session=abc123");
    }

    #[test]
    fn test_parse_set_cookie() {
        let header = "session=abc123; Path=/; HttpOnly";
        let cookie = Cookie::parse_set_cookie(header, "example.com".to_string()).unwrap();
        assert_eq!(cookie.name, "session");
        assert_eq!(cookie.value, "abc123");
        assert_eq!(cookie.path, "/");
        assert!(cookie.http_only);
    }

    #[test]
    fn test_cookie_store() {
        let store = CookieStore::new();
        let cookie = Cookie::new(
            "session".to_string(),
            "abc123".to_string(),
            "example.com".to_string(),
        );
        store.add_cookie(cookie);

        let cookies = store.get_cookies_for_domain("example.com");
        assert_eq!(cookies.len(), 1);
        assert_eq!(cookies[0].name, "session");
    }

    #[test]
    fn test_generate_cookie_header() {
        let store = CookieStore::new();
        store.add_cookie(Cookie::new(
            "session".to_string(),
            "abc123".to_string(),
            "example.com".to_string(),
        ));
        store.add_cookie(Cookie::new(
            "token".to_string(),
            "xyz789".to_string(),
            "example.com".to_string(),
        ));

        let header = store.generate_cookie_header("example.com", "/").unwrap();
        assert!(header.contains("session=abc123"));
        assert!(header.contains("token=xyz789"));
    }
}
