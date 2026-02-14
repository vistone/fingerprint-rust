# API 参考文档

**版本**: v1.0  
**最后更新**: 2026-02-13  
**文档类型**: 技术参考

---

## 核心函数

### 随机指纹生成

```rust
// 生成随机指纹（推荐）
pub fn get_random_fingerprint() -> Result<FingerprintResult, String>

// 使用可选的操作系统参数生成随机指纹
pub fn get_random_fingerprint_with_os(os: Option<OperatingSystem>) -> Result<FingerprintResult, String>

// 按特定浏览器类型生成随机指纹
pub fn get_random_fingerprint_by_browser(browser_type: &str) -> Result<FingerprintResult, Box<dyn Error>>

// 使用特定浏览器和操作系统生成随机指纹
pub fn get_random_fingerprint_by_browser_with_os(
    browser_type: &str,
    os: Option<OperatingSystem>,
) -> Result<FingerprintResult, Box<dyn Error>>
```

这些函数生成各种类型的随机浏览器指纹。最简单的方法是使用 `get_random_fingerprint()`，它返回一个包含完整指纹信息的结果对象。对于特定的操作系统或浏览器类型，使用其他函数。

### User-Agent 生成

```rust
// 通过配置文件名称获取 User-Agent 字符串
pub fn get_user_agent_by_profile_name(profile_name: &str) -> Result<String, String>

// 通过配置文件名称和操作系统获取 User-Agent 字符串
pub fn get_user_agent_by_profile_name_with_os(
    profile_name: &str,
    os: OperatingSystem,
) -> Result<String, String>

// 获取随机操作系统
pub fn random_os() -> OperatingSystem

// 获取随机语言
pub fn random_language() -> String
```

### HTTP 头部生成

```rust
pub fn generate_headers(
    browser_type: BrowserType,
    user_agent: &str,
    is_mobile: bool,
) -> HTTPHeaders
```

此函数基于浏览器类型、用户代理和是否为移动设备生成相应的 HTTP 头部。

## 数据结构

### FingerprintResult

```rust
pub struct FingerprintResult {
    pub profile: ClientProfile,           // TLS 指纹配置
    pub user_agent: String,               // 对应的 User-Agent 字符串
    pub hello_client_id: String,          // ClientHello ID
    pub headers: HTTPHeaders,             // 标准 HTTP 请求头部
}
```

此结构包含完整的指纹结果，包括 TLS 配置、用户代理、ClientHello ID 和 HTTP 头部。

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
    pub custom: HashMap<String, String>,  // 用户自定义头部
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

### 基本用法

```rust
use fingerprint::*;

// 获取随机指纹
let result = get_random_fingerprint()?;
println!("User-Agent: {}", result.user_agent);

// 获取头部的 Map
let headers_map = result.headers.to_map();

// 设置自定义头部
result.headers.set("Cookie", "session_id=abc123");
```

### 指定浏览器类型

```rust
// 生成随机 Chrome 指纹
let result = get_random_fingerprint_by_browser("chrome")?;

// 指定浏览器和操作系统
let result = get_random_fingerprint_by_browser_with_os(
    "firefox",
    Some(OperatingSystem::Windows10),
)?;
```

### User-Agent 生成

```rust
// 通过配置文件名称获取 User-Agent
let ua = get_user_agent_by_profile_name("chrome_120")?;

// 指定操作系统
let ua = get_user_agent_by_profile_name_with_os(
    "chrome_120",
    OperatingSystem::MacOS14,
)?;
```

### 头部管理

```rust
use fingerprint::headers::generate_headers;

// 生成头部
let headers = generate_headers(
    BrowserType::Chrome,
    user_agent,
    false, // is_mobile
);

// 设置自定义头部
headers.set("Cookie", "session_id=abc123");
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
    max_redirects: 10,  // 最大重定向跳转数
    verify_tls: true,    // 验证 TLS 证书
    prefer_http2: true, // 可用时偏好 HTTP/2
    ..Default::default()
};

// 创建 HTTP 客户端
let client = HttpClient::new(config);

// 发送 GET 请求（自动处理重定向）
let response = client.get("https://example.com")?;

// 发送 POST 请求
let response = client.post("https://example.com/api", b"data")?;

// 查看响应
println!("Status Code: {}", response.status_code);
println!("Response Body: {}", response.body_as_string()?);
```

### 连接池支持

```rust
use fingerprint::{HttpClient, HttpClientConfig};
use fingerprint::http_client::PoolManagerConfig;

// 配置连接池
let pool_config = PoolManagerConfig {
    max_connections: 100,
    min_idle: 10,
    enable_reuse: true,
    ..Default::default()
};

// 使用连接池创建 HTTP 客户端
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
