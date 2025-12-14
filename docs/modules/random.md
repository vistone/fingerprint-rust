# Random 模块文档

## 概述

`random` 模块提供随机指纹选择功能，支持按浏览器类型和操作系统筛选。

## 模块位置

`src/random.rs`

## 核心类型

### FingerprintResult

随机指纹结果。

```rust
pub struct FingerprintResult {
    pub profile: ClientProfile,
    pub user_agent: String,
    pub hello_client_id: String,
    pub headers: HTTPHeaders,
}
```

## 主要函数

### 随机指纹获取

- `get_random_fingerprint() -> Result<FingerprintResult, String>`
  - 随机获取一个指纹（所有浏览器）

- `get_random_fingerprint_with_os(os: Option<OperatingSystem>) -> Result<FingerprintResult, String>`
  - 随机获取一个指纹，可指定操作系统

- `get_random_fingerprint_by_browser(browser_type: &str) -> Result<FingerprintResult, Box<dyn Error>>`
  - 根据浏览器类型随机获取指纹

- `get_random_fingerprint_by_browser_with_os(browser_type: &str, os: Option<OperatingSystem>) -> Result<FingerprintResult, Box<dyn Error>>`
  - 根据浏览器类型和操作系统随机获取指纹

## 使用示例

```rust
use fingerprint::{get_random_fingerprint, get_random_fingerprint_by_browser, OperatingSystem};

// 随机获取一个指纹
let result = get_random_fingerprint()?;
println!("浏览器: {}", result.hello_client_id);
println!("User-Agent: {}", result.user_agent);

// 指定操作系统
let result = get_random_fingerprint_with_os(Some(OperatingSystem::Windows))?;

// 指定浏览器类型
let result = get_random_fingerprint_by_browser("Chrome")?;

// 指定浏览器类型和操作系统
let result = get_random_fingerprint_by_browser_with_os(
    "Firefox",
    Some(OperatingSystem::Linux)
)?;
```

## 支持的浏览器类型

- `"Chrome"` - Chrome 系列
- `"Firefox"` - Firefox 系列
- `"Safari"` - Safari 系列
- `"Opera"` - Opera 系列

## 相关文档

- [Profiles 模块文档](profiles.md)
- [User-Agent 模块文档](useragent.md)
