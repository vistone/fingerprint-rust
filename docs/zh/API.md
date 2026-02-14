# API 参考

**版本**: v1.0  
**最后更新**: 2026-02-13  
**文档类型**: 技术文档

---

## 核心函数

### 随机指纹获取

```rust
// 获取随机指纹（推荐）
pub fn get_random_fingerprint() -> Result<FingerprintResult, String>

// 获取指定操作系统的随机指纹
pub fn get_random_fingerprint_with_os(os: Option<OperatingSystem>) -> Result<FingerprintResult, String>

// 获取指定浏览器的随机指纹
pub fn get_random_fingerprint_by_browser(browser_type: &str) -> Result<FingerprintResult, Box<dyn Error>>

// 获取指定浏览器和操作系统的随机指纹
pub fn get_random_fingerprint_by_browser_with_os(
    browser_type: &str,
    os: Option<OperatingSystem>,
) -> Result<FingerprintResult, Box<dyn Error>>
```

### User-Agent 生成

```rust
// 根据 profile 名称获取 User-Agent
pub fn get_user_agent_by_profile_name(profile_name: &str) -> Result<String, String>

// 根据 profile 名称和操作系统获取 User-Agent
pub fn get_user_agent_by_profile_name_with_os(
    profile_name: &str,
    os: OperatingSystem,
) -> Result<String, String>

// 获取随机操作系统
pub fn random_os() -> OperatingSystem

// 获取随机语言
pub fn random_language() -> String
```

### HTTP 请求头生成

```rust
// 根据浏览器类型、User-Agent 和是否为移动终端生成 HTTP 请求头
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
    pub accept: String,                   // Accept 请求头
    pub accept_language: String,          // Accept-Language 请求头
    pub accept_encoding: String,          // Accept-Encoding 请求头
    pub user_agent: String,               // User-Agent 请求头
    pub sec_fetch_site: String,           // Sec-Fetch-Site 请求头
    pub sec_fetch_mode: String,           // Sec-Fetch-Mode 请求头
    pub sec_fetch_user: String,           // Sec-Fetch-User 请求头
    pub sec_fetch_dest: String,           // Sec-Fetch-Dest 请求头
    pub sec_ch_ua: String,                // Sec-CH-UA 请求头
    pub sec_ch_ua_mobile: String,         // Sec-CH-UA-Mobile 请求头
    pub sec_ch_ua_platform: String,       // Sec-CH-UA-Platform 请求头
    pub upgrade_insecure_requests: String,// Upgrade-Insecure-Requests 请求头
    pub custom: HashMap<String, String>,  // 用户自定义的请求头
}

impl HTTPHeaders {
    // 创建新的 HTTPHeaders 实例
    pub fn new() -> Self
    
    // 克隆 HTTPHeaders
    pub fn clone(&self) -> Self
    
    // 设置单个请求头
    pub fn set(&mut self, key: &str, value: &str)
    
    // 设置多个请求头
    pub fn set_headers(&mut self, custom_headers: &[(&str, &str)])
    
    // 转换为 HashMap
    pub fn to_map(&self) -> HashMap<String, String>
    
    // 转换为 HashMap 并包含自定义请求头
    pub fn to_map_with_custom(&self, custom_headers: &[(&str, &str)]) -> HashMap<String, String>
}
```

### BrowserType

```rust
pub enum BrowserType {
    Chrome,    // Google Chrome
    Firefox,   // Mozilla Firefox
    Safari,    // Apple Safari
    Opera,     // Opera 浏览器
    Edge,      // Microsoft Edge
}

impl BrowserType {
    // 从字符串转换为 BrowserType
    pub fn from_str(s: &str) -> Option<Self>
    
    // 转换为字符串
    pub fn as_str(&self) -> &'static str
}
```

### OperatingSystem

```rust
pub enum OperatingSystem {
    Windows10,        // Windows 10
    Windows11,        // Windows 11
    MacOS13,          // macOS 13
    MacOS14,          // macOS 14
    MacOS15,          // macOS 15
    Linux,            // Linux 通用
    LinuxUbuntu,      // Ubuntu
    LinuxDebian,      // Debian
}

impl OperatingSystem {
    // 转换为字符串
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

// 获取 HTTP 请求头的 Map 形式
let headers_map = result.headers.to_map();

// 设置自定义请求头
result.headers.set("Cookie", "session_id=abc123");
```

### 指定浏览器类型

```rust
// 获取 Chrome 浏览器的随机指纹
let result = get_random_fingerprint_by_browser("chrome")?;

// 指定浏览器和操作系统获取指纹
let result = get_random_fingerprint_by_browser_with_os(
    "firefox",
    Some(OperatingSystem::Windows10),
)?;
```

### User-Agent 生成

```rust
// 根据 profile 名称获取 User-Agent
let ua = get_user_agent_by_profile_name("chrome_120")?;

// 指定操作系统获取 User-Agent
let ua = get_user_agent_by_profile_name_with_os(
    "chrome_120",
    OperatingSystem::MacOS14,
)?;
```

### HTTP 请求头管理

```rust
use fingerprint::headers::generate_headers;

// 生成 HTTP 请求头
let headers = generate_headers(
    BrowserType::Chrome,
    user_agent,
    false, // 是否为移动终端
);

// 设置单个自定义请求头
headers.set("Cookie", "session_id=abc123");

// 设置多个自定义请求头
headers.set_headers(&[
    ("Authorization", "Bearer token"),
    ("X-API-Key", "key"),
]);

// 转换为 Map
let headers_map = headers.to_map();
```

### HTTP 客户端

```rust
use fingerprint::{HttpClient, HttpClientConfig, chrome_133};

// 创建客户端配置
let config = HttpClientConfig {
    profile: Some(chrome_133()),
    max_redirects: 10,  // 最大重定向次数
    verify_tls: true,    // 验证 TLS 证书
    prefer_http2: true, // 优先使用 HTTP/2
    ..Default::default()
};

// 创建 HTTP 客户端
let client = HttpClient::new(config);

// 发送 GET 请求（自动处理重定向）
let response = client.get("https://example.com")?;

// 发送 POST 请求
let response = client.post("https://example.com/api", b"data")?;

// 查看响应
println!("HTTP 状态码: {}", response.status_code);
println!("响应体: {}", response.body_as_string()?);
```

### 连接池支持

```rust
use fingerprint::{HttpClient, HttpClientConfig};
use fingerprint::http_client::PoolManagerConfig;

// 创建连接池配置
let pool_config = PoolManagerConfig {
    max_connections: 100,       // 最大连接数
    min_idle: 10,               // 最小空闲连接数
    enable_reuse: true,         // 启用连接复用
    ..Default::default()
};

// 创建带连接池的 HTTP 客户端
let client = HttpClient::with_pool(config, pool_config);

// 自动使用连接池发送请求
let response = client.get("http://example.com/")?;

// 查看连接池统计信息
if let Some(stats) = client.pool_stats() {
    for stat in stats {
        stat.print();
    }
}
```
