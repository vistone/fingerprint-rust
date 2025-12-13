//! 类型定义模块
//!
//! 定义了浏览器类型、操作系统类型等核心类型

/// 浏览器类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BrowserType {
    Chrome,
    Firefox,
    Safari,
    Opera,
    Edge,
}

impl BrowserType {
    /// 从字符串转换为浏览器类型
    ///
    /// 注意：此方法名称与标准库的 `FromStr::from_str` 不同，以避免命名冲突
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "chrome" => Some(Self::Chrome),
            "firefox" => Some(Self::Firefox),
            "safari" => Some(Self::Safari),
            "opera" => Some(Self::Opera),
            "edge" => Some(Self::Edge),
            _ => None,
        }
    }

    /// 转换为字符串
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Chrome => "chrome",
            Self::Firefox => "firefox",
            Self::Safari => "safari",
            Self::Opera => "opera",
            Self::Edge => "edge",
        }
    }
}

impl std::fmt::Display for BrowserType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// 操作系统类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OperatingSystem {
    Windows10,
    Windows11,
    MacOS13,
    MacOS14,
    MacOS15,
    Linux,
    LinuxUbuntu,
    LinuxDebian,
}

impl OperatingSystem {
    /// 获取操作系统的字符串表示
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Windows10 => "Windows NT 10.0; Win64; x64",
            Self::Windows11 => "Windows NT 10.0; Win64; x64",
            Self::MacOS13 => "Macintosh; Intel Mac OS X 13_0_0",
            Self::MacOS14 => "Macintosh; Intel Mac OS X 14_0_0",
            Self::MacOS15 => "Macintosh; Intel Mac OS X 15_0_0",
            Self::Linux => "X11; Linux x86_64",
            Self::LinuxUbuntu => "X11; Linux x86_64",
            Self::LinuxDebian => "X11; Linux x86_64",
        }
    }
}

impl std::fmt::Display for OperatingSystem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// 操作系统列表（用于随机选择）
pub static OPERATING_SYSTEMS: &[OperatingSystem] = &[
    OperatingSystem::Windows10,
    OperatingSystem::Windows11,
    OperatingSystem::MacOS13,
    OperatingSystem::MacOS14,
    OperatingSystem::MacOS15,
    OperatingSystem::Linux,
    OperatingSystem::LinuxUbuntu,
    OperatingSystem::LinuxDebian,
];

/// 为了保持与 Go 版本的兼容性，提供别名
pub type OperatingSystems = [OperatingSystem; 8];

/// User-Agent 模板
#[derive(Debug, Clone)]
pub struct UserAgentTemplate {
    pub browser: BrowserType,
    pub version: String,
    pub template: String,
    pub mobile: bool,
    pub os_required: bool,
}

impl UserAgentTemplate {
    pub fn new(
        browser: BrowserType,
        version: String,
        template: String,
        mobile: bool,
        os_required: bool,
    ) -> Self {
        Self {
            browser,
            version,
            template,
            mobile,
            os_required,
        }
    }
}
