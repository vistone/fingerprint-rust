//! type definitionsmodule
//!
//! define了browsertype, operating systemtype etc.coretype

/// browsertype
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum BrowserType {
 Chrome,
 Firefox,
 Safari,
 Opera,
 Edge,
}

impl BrowserType {
 /// from stringconvert tobrowsertype
 ///
 /// Note: 此methodname and standardlibrary `FromStr::from_str` different, 以avoidnamingconflict
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

 /// convert tostring
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

/// operating systemtype
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
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
 /// Getoperating systemstringrepresent
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

/// operating systemlist ( for randomly select)
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

/// in order tokeep and Go versioncompatibleproperty, providealias
pub type OperatingSystems = [OperatingSystem; 8];

/// User-Agent templates
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
