# User-Agent 模块文档

## 概述

`useragent` 模块提供 User-Agent 生成功能，根据浏览器指纹自动生成匹配的 User-Agent 字符串。

## 模块位置

`src/useragent.rs`

## 核心类型

### UserAgentGenerator

User-Agent 生成器，支持多种浏览器和操作系统。

```rust
pub struct UserAgentGenerator {
    // 内部实现
}
```

### OperatingSystem

操作系统类型枚举。

```rust
pub enum OperatingSystem {
    Windows,
    MacOS,
    Linux,
    Android,
    iOS,
    // ...
}
```

## 主要函数

### User-Agent 生成

- `get_user_agent_by_profile_name(profile_name: &str) -> Result<String, String>`
  - 根据指纹名称生成 User-Agent

- `get_user_agent_by_profile_name_with_os(profile_name: &str, os: OperatingSystem) -> Result<String, String>`
  - 根据指纹名称和操作系统生成 User-Agent

- `random_os() -> OperatingSystem`
  - 随机选择操作系统

## 使用示例

```rust
use fingerprint::{get_user_agent_by_profile_name, random_os, OperatingSystem};

// 根据指纹名称生成 User-Agent
let ua = get_user_agent_by_profile_name("chrome_133")?;
println!("User-Agent: {}", ua);

// 指定操作系统生成 User-Agent
let ua = get_user_agent_by_profile_name_with_os(
    "chrome_133",
    OperatingSystem::Windows
)?;

// 随机操作系统
let os = random_os();
let ua = get_user_agent_by_profile_name_with_os("firefox_133", os)?;
```

## 支持的浏览器和操作系统

- **Windows**: Windows 10/11
- **macOS**: macOS 10.15+
- **Linux**: Ubuntu, Debian 等
- **Android**: Android 7-13
- **iOS**: iOS 15-18

## 相关文档

- [Headers 模块文档](headers.md)
- [Profiles 模块文档](profiles.md)
