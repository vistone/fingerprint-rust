# API 参考文档

## 核心函数

### 随机指纹获取

```rust
// 随机指纹（推荐）
pub fn get_random_fingerprint() -> Result<FingerprintResult, String>
pub fn get_random_fingerprint_with_os(os: Option<OperatingSystem>) -> Result<FingerprintResult, String>
pub fn get_random_fingerprint_by_browser(browser_type: &str) -> Result<FingerprintResult, Box<dyn Error>>
pub fn get_random_fingerprint_by_browser_with_os(
    browser_type: &str,
    os: Option<OperatingSystem>,
) -> Result<FingerprintResult, Box<dyn Error>>
```

### User-Agent 生成

```rust
pub fn get_user_agent_by_profile_name(profile_name: &str) -> Result<String, String>
pub fn get_user_agent_by_profile_name_with_os(
    profile_name: &str,
    os: OperatingSystem,
) -> Result<String, String>
pub fn random_os() -> OperatingSystem
pub fn random_language() -> String
```

### Headers 生成

```rust
pub fn generate_headers(
    browser_type: BrowserType,
    user_agent: &str,
    is_mobile: bool,
) -> HTTPHeaders
```

## 数据结构

### FingerprintResult

```rust
pub struct FingerprintResult {
    pub profile: ClientProfile,           // TLS 指纹配置
    pub user_agent: String,               // 对应的 User-Agent
    pub hello_client_id: String,          // Client Hello ID
    pub headers: HTTPHeaders,             // 标准 HTTP 请求头
}
```

### HTTPHeaders

```rust
pub struct HTTPHeaders {
    pub accept: String,
    pub accept_language: String,
    pub accept_encoding: String,
    pub user_agent: String,
    pub sec_fetch_site: String,
    pub sec_fetch_mode: String,
    pub sec_fetch_user: String,
    pub sec_fetch_dest: String,
    pub sec_ch_ua: String,
    pub sec_ch_ua_mobile: String,
    pub sec_ch_ua_platform: String,
    pub upgrade_insecure_requests: String,
    pub custom: HashMap<String, String>,  // 用户自定义的 headers
}

impl HTTPHeaders {
    pub fn new() -> Self
    pub fn clone(&self) -> Self
    pub fn set(&mut self, key: &str, value: &str)
    pub fn set_headers(&mut self, custom_headers: &[(&str, &str)])
    pub fn to_map(&self) -> HashMap<String, String>
    pub fn to_map_with_custom(&self, custom_headers: &[(&str, &str)]) -> HashMap<String, String>
}
```

### BrowserType

```rust
pub enum BrowserType {
    Chrome,
    Firefox,
    Safari,
    Opera,
    Edge,
}

impl BrowserType {
    pub fn from_str(s: &str) -> Option<Self>
    pub fn as_str(&self) -> &'static str
}
```

### OperatingSystem

```rust
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
    pub fn as_str(&self) -> &'static str
}
```

## 使用示例

### 基础使用

```rust
use fingerprint::*;

// 获取随机指纹
let result = get_random_fingerprint()?;
println!("User-Agent: {}", result.user_agent);

// 获取 Headers Map
let headers_map = result.headers.to_map();

// 设置自定义 Headers
result.headers.set("Cookie", "session_id=abc123");
```

### 指定浏览器类型

```rust
// 随机获取 Chrome 指纹
let result = get_random_fingerprint_by_browser("chrome")?;

// 指定浏览器和操作系统
let result = get_random_fingerprint_by_browser_with_os(
    "firefox",
    Some(OperatingSystem::Windows10),
)?;
```

### User-Agent 生成

```rust
// 根据 profile 名称获取 User-Agent
let ua = get_user_agent_by_profile_name("chrome_120")?;

// 指定操作系统
let ua = get_user_agent_by_profile_name_with_os(
    "chrome_120",
    OperatingSystem::MacOS14,
)?;
```

### Headers 管理

```rust
use fingerprint::headers::generate_headers;

// 生成 Headers
let headers = generate_headers(
    BrowserType::Chrome,
    user_agent,
    false, // is_mobile
);

// 设置自定义 Headers
headers.set("Cookie", "session_id=abc123");
headers.set_headers(&[
    ("Authorization", "Bearer token"),
    ("X-API-Key", "key"),
]);

// 转换为 Map
let headers_map = headers.to_map();
```
