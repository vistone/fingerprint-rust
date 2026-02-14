# API 参考文档

**版本**: v1.0  
**最后更新**: 2026-02-13  
**文档类型**: 技术文档

---

## 核心函数

### 随机指纹获取

```rust
// 获取随机指纹（推荐使用）
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

这些函数用于生成各种随机浏览器指纹。最简单的方式是使用 `get_random_fingerprint()`，它会返回一个包含完整指纹信息的结果对象。如果需要特定的操作系统或浏览器类型，可以使用其他函数进行指定。

### 用户代理字符串生成

```rust
// 根据配置文件名称获取用户代理字符串
pub fn get_user_agent_by_profile_name(profile_name: &str) -> Result<String, String>

// 根据配置文件名称和操作系统获取用户代理字符串
pub fn get_user_agent_by_profile_name_with_os(
    profile_name: &str,
    os: OperatingSystem,
) -> Result<String, String>

// 获取随机操作系统
pub fn random_os() -> OperatingSystem

// 获取随机语言
pub fn random_language() -> String
```

这些函数提供了灵活的用户代理字符串生成方式。通过配置文件名称可以获取特定版本浏览器的用户代理字符串，支持为指定的操作系统生成相应的用户代理。`random_os()` 和 `random_language()` 辅助函数可帮助生成多样化的用户代理信息。

### HTTP 请求头生成

```rust
// 根据浏览器类型、用户代理字符串和是否为移动终端生成 HTTP 请求头
pub fn generate_headers(
    browser_type: BrowserType,
    user_agent: &str,
    is_mobile: bool,
) -> HTTPHeaders
```

此函数根据提供的浏览器信息和终端类型自动生成相应的 HTTP 请求头。生成的请求头包含现代浏览器的所有标准字段，确保网络请求的逼真性。

## 数据结构

### 指纹结果

```rust
pub struct FingerprintResult {
    pub profile: ClientProfile,           // TLS 指纹配置数据
    pub user_agent: String,               // 对应的用户代理字符串
    pub hello_client_id: String,          // 客户端问候识别符
    pub headers: HTTPHeaders,             // 标准 HTTP 请求头集合
}
```

`FingerprintResult` 是指纹生成函数的返回类型，包含了完整的浏览器标识信息。其中的 `profile` 字段包含了传输层安全握手的指纹配置，`user_agent` 是对应的用户代理字符串，`hello_client_id` 用于标识网络请求的唯一身份，`headers` 包含了与此指纹相匹配的 HTTP 请求头。

### HTTP 请求头

```rust
pub struct HTTPHeaders {
    pub accept: String,                   // Accept 请求头字段
    pub accept_language: String,          // Accept-Language 语言偏好设置
    pub accept_encoding: String,          // Accept-Encoding 压缩编码方式
    pub user_agent: String,               // 用户代理浏览器信息
    pub sec_fetch_site: String,           // Sec-Fetch-Site 请求来源站点
    pub sec_fetch_mode: String,           // Sec-Fetch-Mode 请求模式类型
    pub sec_fetch_user: String,           // Sec-Fetch-User 用户交互指示符
    pub sec_fetch_dest: String,           // Sec-Fetch-Dest 目标资源类型
    pub sec_ch_ua: String,                // Sec-CH-UA 客户端提示用户代理
    pub sec_ch_ua_mobile: String,         // Sec-CH-UA-Mobile 移动设备指示符
    pub sec_ch_ua_platform: String,       // Sec-CH-UA-Platform 平台信息
    pub upgrade_insecure_requests: String,// Upgrade-Insecure-Requests HTTPS升级
    pub custom: HashMap<String, String>,  // 用户自定义的请求头字段
}

impl HTTPHeaders {
    // 创建新的 HTTPHeaders 实例
    pub fn new() -> Self
    
    // 克隆 HTTPHeaders 对象
    pub fn clone(&self) -> Self
    
    // 为指定的键设置单个请求头值
    pub fn set(&mut self, key: &str, value: &str)
    
    // 批量设置多个请求头字段
    pub fn set_headers(&mut self, custom_headers: &[(&str, &str)])
    
    // 将所有请求头转换为 HashMap 数据结构
    pub fn to_map(&self) -> HashMap<String, String>
    
    // 将请求头转换为 HashMap 并包含额外的自定义字段
    pub fn to_map_with_custom(&self, custom_headers: &[(&str, &str)]) -> HashMap<String, String>
}
```

`HTTPHeaders` 结构体提供了对 HTTP 请求头的完整管理和操作。它包含了浏览器请求的标准字段，以及一个灵活的自定义字段映射表。通过提供的方法，可以方便地创建、修改和转换请求头数据。

### 浏览器类型

```rust
pub enum BrowserType {
    Chrome,    // 谷歌浏览器
    Firefox,   // 火狐浏览器
    Safari,    // 苹果浏览器
    Opera,     // 欧朋浏览器
    Edge,      // 微软 Edge 浏览器
}

impl BrowserType {
    // 从字符串转换为对应的 BrowserType 枚举值
    pub fn from_str(s: &str) -> Option<Self>
    
    // 将 BrowserType 转换为字符串表示形式
    pub fn as_str(&self) -> &'static str
}
```

`BrowserType` 枚举定义了系统支持的主要浏览器类型。提供的转换方法允许在字符串和枚举值之间灵活转换，便于处理用户输入和配置数据。

### 操作系统

```rust
pub enum OperatingSystem {
    Windows10,        // Windows 10 操作系统
    Windows11,        // Windows 11 操作系统
    MacOS13,          // macOS 13 操作系统
    MacOS14,          // macOS 14 操作系统
    MacOS15,          // macOS 15 操作系统
    Linux,            // Linux 通用发行版
    LinuxUbuntu,      // Ubuntu Linux 发行版
    LinuxDebian,      // Debian Linux 发行版
}

impl OperatingSystem {
    // 将操作系统枚举值转换为对应的字符串标识
    pub fn as_str(&self) -> &'static str
}
```

`OperatingSystem` 枚举列举了系统支持的各种操作系统和发行版本。这允许指纹生成时指定目标操作系统，生成更符合该操作系统的浏览器指纹和用户代理字符串。

## 使用示例

### 基础使用

```rust
use fingerprint::*;

// 获取一个随机指纹
let result = get_random_fingerprint()?;
println!("User-Agent: {}", result.user_agent);

// 获取 HTTP 请求头的 HashMap 形式，便于在网络请求中使用
let headers_map = result.headers.to_map();

// 为指纹添加自定义请求头，如设置会话标识符
result.headers.set("Cookie", "session_id=abc123");
```

这个基础示例展示了如何获取随机指纹并提取其信息。首先调用 `get_random_fingerprint()` 获取完整的指纹数据，然后可以访问其中的用户代理字符串，或将请求头转换为 HashMap 格式供网络库使用。

### 指定浏览器类型

```rust
// 获取 Chrome 浏览器的随机指纹
let result = get_random_fingerprint_by_browser("chrome")?;

// 指定浏览器类型和操作系统来获取特定的指纹
let result = get_random_fingerprint_by_browser_with_os(
    "firefox",
    Some(OperatingSystem::Windows10),
)?;
```

这个示例展示了如何生成特定浏览器和操作系统组合的指纹。当需要模拟特定浏览器或系统环境时，可以使用这些专门的函数，而不是依赖随机生成。

### 用户代理字符串生成

```rust
// 根据预定义的配置文件名称获取对应的用户代理字符串
let ua = get_user_agent_by_profile_name("chrome_120")?;

// 获取特定配置文件在指定操作系统下的用户代理字符串
let ua = get_user_agent_by_profile_name_with_os(
    "chrome_120",
    OperatingSystem::MacOS14,
)?;
```

这个示例说明了如何生成指定版本浏览器的用户代理字符串。配置文件名称（如 `chrome_120`）对应于系统中预加载的浏览器配置。通过指定操作系统，可以确保生成的用户代理字符串与该系统环境相匹配。

### HTTP 请求头管理

```rust
use fingerprint::headers::generate_headers;

// 根据浏览器类型和用户代理字符串生成完整的 HTTP 请求头
let headers = generate_headers(
    BrowserType::Chrome,
    user_agent,
    false, // 设置为 false 表示桌面终端，true 表示移动终端
);

// 为生成的请求头添加单个自定义字段
headers.set("Cookie", "session_id=abc123");

// 批量设置多个自定义请求头
headers.set_headers(&[
    ("Authorization", "Bearer token"),
    ("X-API-Key", "key"),
]);

// 将所有请求头转换为 HashMap 格式供网络库使用
let headers_map = headers.to_map();
```

这个示例展示了请求头的完整生成和定制流程。首先基于浏览器信息生成标准的 HTTP 请求头，然后可以添加或修改各种字段以满足具体需求。

### HTTP 客户端

```rust
use fingerprint::{HttpClient, HttpClientConfig, chrome_133};

// 创建 HTTP 客户端的配置对象
let config = HttpClientConfig {
    profile: Some(chrome_133()),
    max_redirects: 10,  // 设置最多允许 10 次 HTTP 重定向
    verify_tls: true,    // 启用传输层安全证书验证以确保安全连接
    prefer_http2: true, // 优先选择 HTTP/2 协议进行通信
    ..Default::default()
};

// 使用配置创建 HTTP 客户端实例
let client = HttpClient::new(config);

// 发送 GET 请求并自动处理重定向
let response = client.get("https://example.com")?;

// 发送 POST 请求并附带请求体数据
let response = client.post("https://example.com/api", b"data")?;

// 查看响应的状态码和响应体内容
println!("HTTP 状态码: {}", response.status_code);
println!("响应体: {}", response.body_as_string()?);
```

这个示例演示了如何创建并使用 HTTP 客户端。客户端自动应用已配置的浏览器指纹，包括用户代理字符串和传输层安全配置，使得网络请求看起来来自真实的浏览器。

### 连接池支持

```rust
use fingerprint::http_client::PoolManagerConfig;

// 创建连接池的配置对象
let pool_config = PoolManagerConfig {
    max_connections: 100,       // 连接池中最多维持 100 个连接
    min_idle: 10,               // 保持至少 10 个空闲连接待命
    enable_reuse: true,         // 启用连接复用以提高性能
    ..Default::default()
};

// 创建带有连接池支持的 HTTP 客户端
let client = HttpClient::with_pool(config, pool_config);

// 自动通过连接池发送请求，提高频繁请求的效率
let response = client.get("http://example.com/")?;

// 获取并查看连接池的运行统计信息
if let Some(stats) = client.pool_stats() {
    for stat in stats {
        stat.print();
    }
}
```

这个示例展示了如何配置和使用连接池来优化多次网络请求的性能。连接池通过重复使用 TCP 连接来减少建立新连接的开销，特别是在需要进行大量网络请求时效果显著。
