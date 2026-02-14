# 使用指南

## 浏览器指纹使用方式

项目支持两种方式使用浏览器指纹：**随机选择**和**指定版本**。

### 方式 1: 随机选择浏览器指纹

#### 1.1 完全随机（所有浏览器）

```rust
use fingerprint::{get_random_fingerprint, HttpClient};

// 随机选择一个浏览器指纹（从所有 69 个指纹中）
let fp_result = get_random_fingerprint()?;

// 使用随机指纹创建 HTTP 客户端
let client = HttpClient::with_profile(
    fp_result.profile.clone(),
    fp_result.headers.clone(),
    fp_result.user_agent.clone(),
);

// 发送请求（自动使用随机选择的浏览器指纹）
let response = client.get("https://example.com")?;
```

#### 1.2 随机选择指定浏览器类型

```rust
use fingerprint::{get_random_fingerprint_by_browser, HttpClient};

// 随机选择一个 Chrome 版本的指纹
let fp_result = get_random_fingerprint_by_browser("chrome")?;

// 随机选择一个 Firefox 版本的指纹
let fp_result = get_random_fingerprint_by_browser("firefox")?;

// 随机选择一个 Safari 版本的指纹
let fp_result = get_random_fingerprint_by_browser("safari")?;

// 使用随机指纹创建客户端
let client = HttpClient::with_profile(
    fp_result.profile.clone(),
    fp_result.headers.clone(),
    fp_result.user_agent.clone(),
);
```

#### 1.3 随机选择并指定操作系统

```rust
use fingerprint::{get_random_fingerprint_with_os, OperatingSystem, HttpClient};

// 随机选择指纹，但指定操作系统为 Windows
let fp_result = get_random_fingerprint_with_os(Some(OperatingSystem::Windows))?;

// 随机选择 Chrome Fingerprint，指定操作系统为 macOS
let fp_result = get_random_fingerprint_by_browser_with_os(
    "chrome",
    Some(OperatingSystem::MacOS),
)?;

let client = HttpClient::with_profile(
    fp_result.profile.clone(),
    fp_result.headers.clone(),
    fp_result.user_agent.clone(),
);
```

### 方式 2: 指定特定浏览器版本

#### 2.1 使用预定义的函数

```rust
use fingerprint::{chrome_133, firefox_133, safari_16_0, HttpClient, HttpClientConfig};

// 指定使用 Chrome 133
let config = HttpClientConfig {
    profile: Some(chrome_133()),
    ..Default::default()
};
let client = HttpClient::new(config);

// 指定使用 Firefox 133
let config = HttpClientConfig {
    profile: Some(firefox_133()),
    ..Default::default()
};
let client = HttpClient::new(config);

// 指定使用 Safari 16.0
let config = HttpClientConfig {
    profile: Some(safari_16_0()),
    ..Default::default()
};
let client = HttpClient::new(config);
```

#### 2.2 从映射表中获取

```rust
use fingerprint::{mapped_tls_clients, HttpClient, HttpClientConfig};

// 获取所有指纹的映射表
let clients = mapped_tls_clients();

// 按名称获取特定指纹
if let Some(profile) = clients.get("chrome_133") {
    let config = HttpClientConfig {
        profile: Some(profile.clone()),
        ..Default::default()
    };
    let client = HttpClient::new(config);
}

// 可用的指纹名称示例：
// - chrome_103, chrome_104, ..., chrome_133
// - firefox_102, firefox_104, ..., firefox_135
// - safari_15_6_1, safari_16_0, safari_ios_18_0
// - opera_89, opera_90, opera_91
// 等等...共 69 个
```

## 完整示例

### 示例 1: 随机选择指纹发送请求

```rust
use fingerprint::{get_random_fingerprint, HttpClient};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 随机选择一个浏览器指纹
    let fp_result = get_random_fingerprint()?;
    
    println!("随机选择的指纹: {}", fp_result.hello_client_id);
    println!("User-Agent: {}", fp_result.user_agent);
    
    // 创建 HTTP 客户端
    let client = HttpClient::with_profile(
        fp_result.profile.clone(),
        fp_result.headers.clone(),
        fp_result.user_agent.clone(),
    );
    
    // 发送请求
    let response = client.get("https://httpbin.org/get")?;
    println!("Status Code: {}", response.status_code);
    
    Ok(())
}
```

### 示例 2: 随机选择 Chrome 版本

```rust
use fingerprint::{get_random_fingerprint_by_browser, HttpClient};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 随机选择一个 Chrome 版本的指纹
    let fp_result = get_random_fingerprint_by_browser("chrome")?;
    
    println!("随机选择的 Chrome 版本: {}", fp_result.hello_client_id);
    
    let client = HttpClient::with_profile(
        fp_result.profile.clone(),
        fp_result.headers.clone(),
        fp_result.user_agent.clone(),
    );
    
    let response = client.get("https://example.com")?;
    Ok(())
}
```

### 示例 3: 指定 Chrome 133

```rust
use fingerprint::{chrome_133, HttpClient, HttpClientConfig};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 指定使用 Chrome 133
    let config = HttpClientConfig {
        profile: Some(chrome_133()),
        ..Default::default()
    };
    
    let client = HttpClient::new(config);
    let response = client.get("https://example.com")?;
    
    Ok(())
}
```

### 示例 4: 批量请求使用不同指纹

```rust
use fingerprint::{get_random_fingerprint, HttpClient};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let urls = vec![
        "https://example.com/page1",
        "https://example.com/page2",
        "https://example.com/page3",
    ];
    
    for url in urls {
        // 每次请求使用不同的随机指纹
        let fp_result = get_random_fingerprint()?;
        let client = HttpClient::with_profile(
            fp_result.profile.clone(),
            fp_result.headers.clone(),
            fp_result.user_agent.clone(),
        );
        
        let response = client.get(url)?;
        println!("{}: {}", url, response.status_code);
    }
    
    Ok(())
}
```

## FingerprintResult 结构

`FingerprintResult` 包含完整的指纹信息：

```rust
pub struct FingerprintResult {
    pub profile: ClientProfile,        // TLS fingerprint configuration
    pub user_agent: String,            // 对应的 User-Agent
    pub hello_client_id: String,      // Client Hello ID（如 "Chrome-133"）
    pub headers: HTTPHeaders,          // 标准 HTTP 请求头
}
```

## 支持的浏览器类型

随机选择时可以使用以下浏览器类型：

- `"chrome"` - Chrome 系列（19 个版本）
- `"firefox"` - Firefox 系列（13 个版本）
- `"safari"` - Safari 系列（14 个版本）
- `"opera"` - Opera 系列（3 个版本）
- `"edge"` - Edge 系列（3 个版本）

## 所有可用的指纹

通过 `mapped_tls_clients()` 可以获取所有 69 个指纹的完整列表，包括：

- Chrome: chrome_103, chrome_104, ..., chrome_133, chrome_133_PSK 等
- Firefox: firefox_102, firefox_104, ..., firefox_135 等
- Safari: safari_15_6_1, safari_16_0, safari_ios_18_0 等
- Opera: opera_89, opera_90, opera_91
- Edge: edge_120, edge_124, edge_133
- 移动端: zalando_android_mobile, nike_ios_mobile 等

## 注意事项

1. **随机选择**适合需要模拟不同浏览器的场景
2. **指定版本**适合需要精确控制指纹的场景
3. 所有指纹都会自动应用到 TLS 握手（通过 rustls 的 ClientHelloCustomizer）
4. `FingerprintResult` 包含了完整的配置，包括 User-Agent 和 HTTP Headers

