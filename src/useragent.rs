//! User-Agent 生成模块
//!
//! 根据指纹配置生成对应的 User-Agent

use crate::types::{BrowserType, OperatingSystem, UserAgentTemplate, OPERATING_SYSTEMS};
use crate::utils::random_choice;
use std::collections::HashMap;
use std::sync::OnceLock;

/// User-Agent 生成器
pub struct UserAgentGenerator {
    templates: HashMap<String, UserAgentTemplate>,
}

impl UserAgentGenerator {
    /// 创建新的 User-Agent 生成器
    pub fn new() -> Self {
        let mut gen = Self {
            templates: HashMap::new(),
        };
        gen.init_templates();
        gen
    }

    /// 初始化 User-Agent 模板
    fn init_templates(&mut self) {
        // Chrome User-Agent 模板
        let chrome_templates: &[(&str, &str)] = &[
            ("103", "Mozilla/5.0 (%s) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/103.0.0.0 Safari/537.36"),
            ("104", "Mozilla/5.0 (%s) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/104.0.0.0 Safari/537.36"),
            ("105", "Mozilla/5.0 (%s) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/105.0.0.0 Safari/537.36"),
            ("106", "Mozilla/5.0 (%s) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/106.0.0.0 Safari/537.36"),
            ("107", "Mozilla/5.0 (%s) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/107.0.0.0 Safari/537.36"),
            ("108", "Mozilla/5.0 (%s) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/108.0.0.0 Safari/537.36"),
            ("109", "Mozilla/5.0 (%s) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/109.0.0.0 Safari/537.36"),
            ("110", "Mozilla/5.0 (%s) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/110.0.0.0 Safari/537.36"),
            ("111", "Mozilla/5.0 (%s) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/111.0.0.0 Safari/537.36"),
            ("112", "Mozilla/5.0 (%s) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/112.0.0.0 Safari/537.36"),
            ("116", "Mozilla/5.0 (%s) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/116.0.0.0 Safari/537.36"),
            ("117", "Mozilla/5.0 (%s) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/117.0.0.0 Safari/537.36"),
            ("120", "Mozilla/5.0 (%s) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36"),
            ("124", "Mozilla/5.0 (%s) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36"),
            ("130", "Mozilla/5.0 (%s) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0.0.0 Safari/537.36"),
            ("131", "Mozilla/5.0 (%s) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36"),
            ("133", "Mozilla/5.0 (%s) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/133.0.0.0 Safari/537.36"),
        ];

        for (version, template) in chrome_templates {
            self.templates.insert(
                format!("chrome_{}", version),
                UserAgentTemplate::new(
                    BrowserType::Chrome,
                    version.to_string(),
                    template.to_string(),
                    false,
                    true,
                ),
            );
        }

        // Firefox User-Agent 模板
        let firefox_templates: &[(&str, &str)] = &[
            ("102", "Mozilla/5.0 (%s; rv:102.0) Gecko/20100101 Firefox/102.0"),
            ("104", "Mozilla/5.0 (%s; rv:104.0) Gecko/20100101 Firefox/104.0"),
            ("105", "Mozilla/5.0 (%s; rv:105.0) Gecko/20100101 Firefox/105.0"),
            ("106", "Mozilla/5.0 (%s; rv:106.0) Gecko/20100101 Firefox/106.0"),
            ("108", "Mozilla/5.0 (%s; rv:108.0) Gecko/20100101 Firefox/108.0"),
            ("110", "Mozilla/5.0 (%s; rv:110.0) Gecko/20100101 Firefox/110.0"),
            ("117", "Mozilla/5.0 (%s; rv:117.0) Gecko/20100101 Firefox/117.0"),
            ("120", "Mozilla/5.0 (%s; rv:120.0) Gecko/20100101 Firefox/120.0"),
            ("123", "Mozilla/5.0 (%s; rv:123.0) Gecko/20100101 Firefox/123.0"),
            ("132", "Mozilla/5.0 (%s; rv:132.0) Gecko/20100101 Firefox/132.0"),
            ("133", "Mozilla/5.0 (%s; rv:133.0) Gecko/20100101 Firefox/133.0"),
            ("135", "Mozilla/5.0 (%s; rv:135.0) Gecko/20100101 Firefox/135.0"),
        ];

        for (version, template) in firefox_templates {
            self.templates.insert(
                format!("firefox_{}", version),
                UserAgentTemplate::new(
                    BrowserType::Firefox,
                    version.to_string(),
                    template.to_string(),
                    false,
                    true,
                ),
            );
        }

        // Safari User-Agent 模板
        let safari_templates: &[(&str, &str, bool)] = &[
            ("15_6_1", "Mozilla/5.0 (%s) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/15.6.1 Safari/605.1.15", false),
            ("16_0", "Mozilla/5.0 (%s) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/16.0 Safari/605.1.15", false),
            ("ipad_15_6", "Mozilla/5.0 (iPad; CPU OS 15_6 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/15.6 Mobile/15E148 Safari/604.1", true),
            ("ios_15_5", "Mozilla/5.0 (iPhone; CPU iPhone OS 15_5 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/15.5 Mobile/15E148 Safari/604.1", true),
            ("ios_15_6", "Mozilla/5.0 (iPhone; CPU iPhone OS 15_6 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/15.6 Mobile/15E148 Safari/604.1", true),
            ("ios_16_0", "Mozilla/5.0 (iPhone; CPU iPhone OS 16_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/16.0 Mobile/15E148 Safari/604.1", true),
            ("ios_17_0", "Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.0 Mobile/15E148 Safari/604.1", true),
            ("ios_18_0", "Mozilla/5.0 (iPhone; CPU iPhone OS 18_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/18.0 Mobile/15E148 Safari/604.1", true),
            ("ios_18_5", "Mozilla/5.0 (iPhone; CPU iPhone OS 18_5 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/18.5 Mobile/15E148 Safari/604.1", true),
        ];

        for (key, template, mobile) in safari_templates {
            self.templates.insert(
                format!("safari_{}", key),
                UserAgentTemplate::new(
                    BrowserType::Safari,
                    key.to_string(),
                    template.to_string(),
                    *mobile,
                    !mobile, // 移动端不需要操作系统信息
                ),
            );
        }

        // Opera User-Agent 模板
        let opera_templates: &[(&str, &str)] = &[
            ("89", "Mozilla/5.0 (%s) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/103.0.0.0 Safari/537.36 OPR/89.0.0.0"),
            ("90", "Mozilla/5.0 (%s) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/104.0.0.0 Safari/537.36 OPR/90.0.0.0"),
            ("91", "Mozilla/5.0 (%s) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/105.0.0.0 Safari/537.36 OPR/91.0.0.0"),
        ];

        for (version, template) in opera_templates {
            self.templates.insert(
                format!("opera_{}", version),
                UserAgentTemplate::new(
                    BrowserType::Opera,
                    version.to_string(),
                    template.to_string(),
                    false,
                    true,
                ),
            );
        }

        // 移动端和自定义指纹的 User-Agent 模板
        // iOS 应用指纹
        let ios_app_templates: &[(&str, &str)] = &[
            ("zalando_ios_mobile", "Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.0 Mobile/15E148 Safari/604.1"),
            ("nike_ios_mobile", "Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.0 Mobile/15E148 Safari/604.1"),
            ("mms_ios", "Mozilla/5.0 (iPhone; CPU iPhone OS 16_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/16.0 Mobile/15E148 Safari/604.1"),
            ("mms_ios_2", "Mozilla/5.0 (iPhone; CPU iPhone OS 16_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/16.0 Mobile/15E148 Safari/604.1"),
            ("mms_ios_3", "Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.0 Mobile/15E148 Safari/604.1"),
            ("mesh_ios", "Mozilla/5.0 (iPhone; CPU iPhone OS 16_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/16.0 Mobile/15E148 Safari/604.1"),
            ("mesh_ios_2", "Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.0 Mobile/15E148 Safari/604.1"),
            ("confirmed_ios", "Mozilla/5.0 (iPhone; CPU iPhone OS 16_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/16.0 Mobile/15E148 Safari/604.1"),
        ];

        for (key, template) in ios_app_templates {
            self.templates.insert(
                key.to_string(),
                UserAgentTemplate::new(
                    BrowserType::Safari,
                    "ios".to_string(),
                    template.to_string(),
                    true,
                    false, // iOS 移动端不需要操作系统占位符
                ),
            );
        }

        // Android 应用指纹
        let android_app_templates: &[(&str, &str)] = &[
            ("zalando_android_mobile", "Mozilla/5.0 (Linux; Android 13; SM-G991B) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Mobile Safari/537.36"),
            ("nike_android_mobile", "Mozilla/5.0 (Linux; Android 13; Pixel 7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Mobile Safari/537.36"),
            ("mesh_android", "Mozilla/5.0 (Linux; Android 12; SM-G998B) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Mobile Safari/537.36"),
            ("mesh_android_2", "Mozilla/5.0 (Linux; Android 13; Pixel 6) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Mobile Safari/537.36"),
            ("confirmed_android", "Mozilla/5.0 (Linux; Android 12; SM-G998B) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Mobile Safari/537.36"),
            ("confirmed_android_2", "Mozilla/5.0 (Linux; Android 13; Pixel 7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Mobile Safari/537.36"),
        ];

        for (key, template) in android_app_templates {
            self.templates.insert(
                key.to_string(),
                UserAgentTemplate::new(
                    BrowserType::Chrome,
                    "android".to_string(),
                    template.to_string(),
                    true,
                    false, // Android 移动端不需要操作系统占位符
                ),
            );
        }

        // OkHttp4 Android 指纹
        let okhttp_templates: &[(&str, &str)] = &[
            ("okhttp4_android_7", "Mozilla/5.0 (Linux; Android 7.0; SM-G930F) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Mobile Safari/537.36"),
            ("okhttp4_android_8", "Mozilla/5.0 (Linux; Android 8.0; SM-G950F) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Mobile Safari/537.36"),
            ("okhttp4_android_9", "Mozilla/5.0 (Linux; Android 9; SM-G960F) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Mobile Safari/537.36"),
            ("okhttp4_android_10", "Mozilla/5.0 (Linux; Android 10; SM-G970F) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Mobile Safari/537.36"),
            ("okhttp4_android_11", "Mozilla/5.0 (Linux; Android 11; SM-G991B) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Mobile Safari/537.36"),
            ("okhttp4_android_12", "Mozilla/5.0 (Linux; Android 12; SM-G998B) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Mobile Safari/537.36"),
            ("okhttp4_android_13", "Mozilla/5.0 (Linux; Android 13; Pixel 7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Mobile Safari/537.36"),
        ];

        for (key, template) in okhttp_templates {
            self.templates.insert(
                key.to_string(),
                UserAgentTemplate::new(
                    BrowserType::Chrome,
                    "okhttp4".to_string(),
                    template.to_string(),
                    true,
                    false, // Android 移动端不需要操作系统占位符
                ),
            );
        }

        // Cloudflare Custom
        self.templates.insert(
            "cloudflare_custom".to_string(),
            UserAgentTemplate::new(
                BrowserType::Chrome,
                "custom".to_string(),
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36".to_string(),
                false,
                false, // 固定 User-Agent，不需要操作系统占位符
            ),
        );
    }

    /// 根据指纹名称获取 User-Agent
    /// 如果指纹需要操作系统信息，会随机选择一个操作系统
    pub fn get_user_agent(&self, profile_name: &str) -> Result<String, String> {
        self.get_user_agent_with_os(profile_name, None)
    }

    /// 根据指纹名称和指定操作系统获取 User-Agent
    /// 如果 os 为 None，且需要操作系统信息，会随机选择一个操作系统
    pub fn get_user_agent_with_os(
        &self,
        profile_name: &str,
        os: Option<OperatingSystem>,
    ) -> Result<String, String> {
        if profile_name.is_empty() {
            return Err("profile name cannot be empty".to_string());
        }

        if let Some(template) = self.templates.get(profile_name) {
            // 如果不需要操作系统信息，直接返回模板
            if !template.os_required {
                return Ok(template.template.clone());
            }

            // 如果需要操作系统信息
            let os_str = match os {
                Some(os) => os.as_str(),
                None => {
                    // 随机选择操作系统
                    random_os().as_str()
                }
            };

            return Ok(template.template.replace("%s", os_str));
        }

        // 尝试从 profileName 中提取浏览器类型和版本
        self.generate_from_profile_name(profile_name, os)
    }

    /// 从 profile 名称生成 User-Agent
    fn generate_from_profile_name(
        &self,
        profile_name: &str,
        os: Option<OperatingSystem>,
    ) -> Result<String, String> {
        let profile_name_lower = profile_name.to_lowercase();

        // 解析浏览器类型和版本
        let (browser, version) = if profile_name_lower.starts_with("chrome_") {
            let version = profile_name_lower
                .strip_prefix("chrome_")
                .unwrap()
                .split('_')
                .next()
                .unwrap_or("133");
            (BrowserType::Chrome, version)
        } else if profile_name_lower.starts_with("firefox_") {
            let version = profile_name_lower.strip_prefix("firefox_").unwrap_or("135");
            (BrowserType::Firefox, version)
        } else if profile_name_lower.starts_with("safari_") {
            let version = profile_name_lower.strip_prefix("safari_").unwrap_or("16_0");
            (BrowserType::Safari, version)
        } else if profile_name_lower.starts_with("opera_") {
            let version = profile_name_lower.strip_prefix("opera_").unwrap_or("91");
            (BrowserType::Opera, version)
        } else {
            // 默认使用 Chrome 133
            return self.get_user_agent_with_os("chrome_133", os);
        };

        // 生成 User-Agent
        let os_str = match os {
            Some(os) => os.as_str(),
            None => random_os().as_str(),
        };

        match browser {
            BrowserType::Chrome => Ok(format!(
                "Mozilla/5.0 ({}) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/{}.0.0.0 Safari/537.36",
                os_str, version
            )),
            BrowserType::Firefox => Ok(format!(
                "Mozilla/5.0 ({}; rv:{}.0) Gecko/20100101 Firefox/{}.0",
                os_str, version, version
            )),
            BrowserType::Safari => Ok(format!(
                "Mozilla/5.0 ({}) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/{} Safari/605.1.15",
                os_str, version
            )),
            BrowserType::Opera => Ok(format!(
                "Mozilla/5.0 ({}) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/{}.0.0.0 Safari/537.36 OPR/{}.0.0.0",
                os_str, version, version
            )),
            BrowserType::Edge => Ok(format!(
                "Mozilla/5.0 ({}) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/{}.0.0.0 Safari/537.36 Edg/{}.0.0.0",
                os_str, version, version
            )),
        }
    }
}

impl Default for UserAgentGenerator {
    fn default() -> Self {
        Self::new()
    }
}

/// 全局默认生成器（线程安全）
static DEFAULT_GENERATOR: OnceLock<UserAgentGenerator> = OnceLock::new();

fn get_default_generator() -> &'static UserAgentGenerator {
    DEFAULT_GENERATOR.get_or_init(|| UserAgentGenerator::new())
}

/// 随机选择一个操作系统
pub fn random_os() -> OperatingSystem {
    random_choice(OPERATING_SYSTEMS).unwrap_or(OperatingSystem::Windows10)
}

/// 为指定的 ClientProfile 获取 User-Agent
pub fn get_user_agent_by_profile_name(profile_name: &str) -> Result<String, String> {
    get_default_generator().get_user_agent(profile_name)
}

/// 为指定的 ClientProfile 和操作系统获取 User-Agent
pub fn get_user_agent_by_profile_name_with_os(
    profile_name: &str,
    os: OperatingSystem,
) -> Result<String, String> {
    get_default_generator().get_user_agent_with_os(profile_name, Some(os))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_user_agent_chrome() {
        let gen = UserAgentGenerator::new();
        let ua = gen.get_user_agent("chrome_120").unwrap();
        assert!(ua.contains("Chrome/120"));
        assert!(ua.contains("AppleWebKit"));
    }

    #[test]
    fn test_get_user_agent_firefox() {
        let gen = UserAgentGenerator::new();
        let ua = gen.get_user_agent("firefox_133").unwrap();
        assert!(ua.contains("Firefox/133"));
    }

    #[test]
    fn test_get_user_agent_with_os() {
        let gen = UserAgentGenerator::new();
        let ua = gen
            .get_user_agent_with_os("chrome_120", Some(OperatingSystem::MacOS14))
            .unwrap();
        assert!(ua.contains("Macintosh"));
    }

    #[test]
    fn test_random_os() {
        let os = random_os();
        assert!(OPERATING_SYSTEMS.contains(&os));
    }
}
