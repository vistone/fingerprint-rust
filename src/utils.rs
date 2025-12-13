//! 工具函数模块
//!
//! 提供随机选择、字符串处理等工具函数

use rand::Rng;

/// 从切片中随机选择一个元素（线程安全）
/// 使用 thread_rng() 确保线程安全
pub fn random_choice<T: Clone>(items: &[T]) -> Option<T> {
    if items.is_empty() {
        return None;
    }
    let mut rng = rand::thread_rng();
    let index = rng.gen_range(0..items.len());
    Some(items[index].clone())
}

/// 从字符串切片中随机选择一个元素（线程安全）
pub fn random_choice_string(items: &[&str]) -> Option<String> {
    random_choice(items).map(|s| s.to_string())
}

/// 从 User-Agent 中提取 Chrome 版本号
pub fn extract_chrome_version(user_agent: &str) -> String {
    // 查找 "Chrome/" 后面的版本号
    if let Some(start) = user_agent.find("Chrome/") {
        let version_start = start + 7; // "Chrome/".len()
        if let Some(end) = user_agent[version_start..].find(|c: char| !c.is_ascii_digit() && c != '.') {
            return user_agent[version_start..version_start + end].to_string();
        }
        // 如果没找到结束位置，返回到字符串末尾
        return user_agent[version_start..].split_whitespace().next().unwrap_or("120").to_string();
    }
    "120".to_string() // 默认版本
}

/// 从 User-Agent 中提取平台信息
pub fn extract_platform(user_agent: &str) -> String {
    // 提取平台信息用于 Sec-CH-UA-Platform
    if user_agent.contains("Windows") {
        return r#""Windows""#.to_string();
    } else if user_agent.contains("Macintosh") || user_agent.contains("Mac OS X") {
        return r#""macOS""#.to_string();
    } else if user_agent.contains("Linux") {
        return r#""Linux""#.to_string();
    } else if user_agent.contains("Android") {
        return r#""Android""#.to_string();
    } else if user_agent.contains("iPhone") || user_agent.contains("iPad") {
        return r#""iOS""#.to_string();
    }
    r#""Windows""#.to_string() // 默认平台
}

/// 从 profile 名称推断浏览器类型
pub fn infer_browser_from_profile_name(profile_name: &str) -> (String, bool) {
    let name_lower = profile_name.to_lowercase();
    if name_lower.starts_with("chrome_") {
        ("chrome".to_string(), false)
    } else if name_lower.starts_with("firefox_") {
        ("firefox".to_string(), false)
    } else if name_lower.starts_with("safari_") {
        ("safari".to_string(), name_lower.contains("ios") || name_lower.contains("ipad"))
    } else if name_lower.starts_with("opera_") {
        ("opera".to_string(), false)
    } else if name_lower.contains("ios") || name_lower.contains("android") || name_lower.contains("mobile") {
        // 移动端应用指纹
        if name_lower.contains("ios") {
            ("safari".to_string(), true)
        } else {
            ("chrome".to_string(), true)
        }
    } else {
        ("chrome".to_string(), false) // 默认
    }
}

/// 判断是否为移动端 profile
pub fn is_mobile_profile(profile_name: &str) -> bool {
    let name = profile_name.to_lowercase();
    name.contains("ios")
        || name.contains("android")
        || name.contains("ipad")
        || name.contains("mobile")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_random_choice() {
        let items = &[1, 2, 3, 4, 5];
        let result = random_choice(items);
        assert!(result.is_some());
        assert!(items.contains(&result.unwrap()));
    }

    #[test]
    fn test_random_choice_empty() {
        let items: &[i32] = &[];
        let result = random_choice(items);
        assert!(result.is_none());
    }

    #[test]
    fn test_extract_chrome_version() {
        let ua = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";
        assert_eq!(extract_chrome_version(ua), "120.0.0.0");
    }

    #[test]
    fn test_extract_platform() {
        assert_eq!(extract_platform("Windows NT 10.0"), r#""Windows""#);
        assert_eq!(extract_platform("Macintosh"), r#""macOS""#);
        assert_eq!(extract_platform("Linux"), r#""Linux""#);
    }

    #[test]
    fn test_infer_browser_from_profile_name() {
        assert_eq!(infer_browser_from_profile_name("chrome_120"), ("chrome".to_string(), false));
        assert_eq!(infer_browser_from_profile_name("firefox_133"), ("firefox".to_string(), false));
        assert_eq!(infer_browser_from_profile_name("safari_ios_17_0"), ("safari".to_string(), true));
    }

    #[test]
    fn test_is_mobile_profile() {
        assert!(is_mobile_profile("safari_ios_17_0"));
        assert!(is_mobile_profile("okhttp4_android_13"));
        assert!(!is_mobile_profile("chrome_120"));
    }
}
