# 统一指纹生成指南

## 概述

fingerprint-rust 现在支持**统一指纹生成**，确保浏览器指纹（User-Agent、TLS 指纹）和 TCP 指纹（p0f）完全同步，避免因指纹不一致而被检测。

## 问题背景

在之前的实现中，浏览器指纹和 TCP 指纹是独立生成的：

- **浏览器指纹**：包含 User-Agent、TLS ClientHello、HTTP/2 Settings 等
- **TCP 指纹**：包含 TTL、Window Size、MSS、Window Scale 等

如果浏览器指纹显示是 "Chrome on Windows"，但 TCP 指纹显示是 "Linux"，就会被检测系统识别为异常。

## 解决方案

通过统一指纹生成系统，确保：

1. **从 User-Agent 推断操作系统**
2. **根据操作系统生成匹配的 TCP Profile**
3. **确保浏览器指纹和 TCP 指纹完全同步**

## 使用方法

### 方法 1: 使用统一指纹生成函数（推荐）

```rust
use fingerprint_profiles::profiles::generate_unified_fingerprint;

// 生成 User-Agent
let user_agent = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/135.0.0.0 Safari/537.36";

// 生成统一的指纹（浏览器指纹 + TCP 指纹）
let profile = generate_unified_fingerprint("chrome_135", user_agent)?;

// profile.tcp_profile 现在包含与 User-Agent 匹配的 TCP 指纹
// Windows -> TTL=128, Window Size=64240, MSS=1460, Window Scale=8
```

### 方法 2: 手动同步 TCP Profile

```rust
use fingerprint_profiles::profiles::{get_client_profile, ClientProfile};
use fingerprint_core::tcp::TcpProfile;

// 获取基础 profile
let mut profile = get_client_profile("chrome_135")?;

// 根据 User-Agent 同步 TCP Profile
let user_agent = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/135.0.0.0 Safari/537.36";
let profile = profile.with_synced_tcp_profile(user_agent);

// 或者根据操作系统类型同步
use fingerprint_core::types::OperatingSystem;
let profile = profile.with_tcp_profile_for_os(OperatingSystem::Linux);
```

### 方法 3: 直接使用 TcpProfile

```rust
use fingerprint_core::tcp::TcpProfile;
use fingerprint_core::types::OperatingSystem;

// 根据操作系统生成 TCP Profile
let tcp_profile = TcpProfile::for_os(OperatingSystem::Windows10);
// TTL=128, Window Size=64240, MSS=1460, Window Scale=8

// 从 User-Agent 推断并生成
let user_agent = "Mozilla/5.0 (Macintosh; Intel Mac OS X 14_0_0) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/135.0.0.0 Safari/537.36";
let tcp_profile = TcpProfile::from_user_agent(user_agent);
// TTL=64, Window Size=65535, MSS=1460, Window Scale=6
```

## TCP Profile 映射表

| 操作系统 | TTL | Window Size | MSS | Window Scale |
|---------|-----|-------------|-----|--------------|
| Windows 10/11 | 128 | 64240 | 1460 | 8 |
| macOS 13/14/15 | 64 | 65535 | 1460 | 6 |
| Linux/Ubuntu/Debian | 64 | 65535 | 1460 | 7 |

## 完整示例

```rust
use fingerprint_profiles::profiles::generate_unified_fingerprint;
use fingerprint_headers::useragent::get_user_agent_by_profile_name;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. 生成 User-Agent
    let user_agent = get_user_agent_by_profile_name("chrome_135")?;
    println!("User-Agent: {}", user_agent);
    
    // 2. 生成统一的指纹（浏览器指纹 + TCP 指纹）
    let profile = generate_unified_fingerprint("chrome_135", &user_agent)?;
    
    // 3. 验证 TCP Profile 已同步
    if let Some(tcp_profile) = profile.tcp_profile {
        println!("TCP Profile:");
        println!("  TTL: {}", tcp_profile.ttl);
        println!("  Window Size: {}", tcp_profile.window_size);
        println!("  MSS: {:?}", tcp_profile.mss);
        println!("  Window Scale: {:?}", tcp_profile.window_scale);
    }
    
    // 4. 使用 profile 进行 HTTP 请求
    // ... 你的 HTTP 客户端代码 ...
    
    Ok(())
}
```

## 最佳实践

1. **始终使用统一指纹生成函数**
   - 使用 `generate_unified_fingerprint()` 确保浏览器指纹和 TCP 指纹同步

2. **在生成 User-Agent 后立即同步**
   - 不要单独生成浏览器指纹和 TCP 指纹
   - 确保它们来自同一个 User-Agent

3. **验证指纹一致性**
   - 在发送请求前，验证 `profile.tcp_profile` 是否与 User-Agent 匹配

4. **使用防御侧验证**
   - 使用 `fingerprint-defense` 的 `PassiveAnalyzer` 验证生成的指纹是否逼真

## 注意事项

- **默认 TCP Profile**: 所有 profile 函数（如 `chrome_135()`, `firefox_133()`）默认使用 Windows 的 TCP Profile
- **需要手动同步**: 如果使用默认 profile，需要通过 `with_synced_tcp_profile()` 或 `with_tcp_profile_for_os()` 同步
- **操作系统推断**: `TcpProfile::from_user_agent()` 会从 User-Agent 推断操作系统，如果无法推断，默认使用 Windows

## 相关 API

- `generate_unified_fingerprint(profile_name, user_agent)` - 统一指纹生成函数
- `ClientProfile::with_synced_tcp_profile(user_agent)` - 根据 User-Agent 同步 TCP Profile
- `ClientProfile::with_tcp_profile_for_os(os)` - 根据操作系统类型同步 TCP Profile
- `TcpProfile::for_os(os)` - 根据操作系统生成 TCP Profile
- `TcpProfile::from_user_agent(user_agent)` - 从 User-Agent 推断并生成 TCP Profile
