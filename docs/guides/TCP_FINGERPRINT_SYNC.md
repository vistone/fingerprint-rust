# TCP 指纹自动同步说明

**版本**: v1.0  
**最后更新**: 2026-02-13  
**文档类型**: 技术文档

---



## 概述

**是的，现在 TCP 指纹和浏览器指纹是完全同步的！**

每次选择浏览器指纹时，TCP 指纹都会根据 User-Agent 自动同步，确保两者完全一致。

## 自动同步机制

### 已修复的函数

以下函数现在都会自动同步 TCP Profile：

1. **`get_random_fingerprint()`**
   - 随机选择浏览器指纹
   - 自动根据生成的 User-Agent 同步 TCP Profile

2. **`get_random_fingerprint_by_browser(browser_type)`**
   - 按浏览器类型随机选择
   - 自动根据生成的 User-Agent 同步 TCP Profile

3. **`get_random_fingerprint_with_os(os)`**
   - 指定操作系统随机选择
   - 自动根据生成的 User-Agent 同步 TCP Profile
   - **注意**：当指定操作系统时，会自动过滤移动端 profile（如 Android、iOS），因为移动端 profile 的 User-Agent 是固定的，无法切换到其他操作系统

4. **`get_random_fingerprint_by_browser_with_os(browser_type, os)`**
   - 按浏览器类型和操作系统选择
   - 自动根据生成的 User-Agent 同步 TCP Profile

### 同步逻辑

```rust
// 在 random.rs 中，每次生成指纹时都会执行：
let mut profile = clients.get(&random_name)?.clone();
let ua = get_user_agent_by_profile_name(&random_name)?;

// 🔥 关键：根据 User-Agent 同步 TCP Profile
profile = profile.with_synced_tcp_profile(&ua);
```

### 同步规则

| User-Agent 包含 | 操作系统 | TCP TTL | TCP Window Size |
|----------------|---------|---------|----------------|
| `Windows NT 10.0` / `Windows NT 11.0` | Windows | 128 | 64240 |
| `Macintosh` / `Mac OS X` | macOS | 64 | 65535 |
| `Linux` / `X11` | Linux | 64 | 65535 |

## 使用示例

### 示例 1: 随机选择（自动同步）

```rust
use fingerprint::*;

// 随机选择浏览器指纹
let result = get_random_fingerprint()?;

// ✅ TCP Profile 已自动同步
// - 如果 User-Agent 是 Windows，TCP TTL = 128
// - 如果 User-Agent 是 Linux，TCP TTL = 64
// - 如果 User-Agent 是 macOS，TCP TTL = 64

let config = HttpClientConfig {
    user_agent: result.user_agent.clone(),
    profile: Some(result.profile), // TCP Profile 已同步
    ..Default::default()
};
```

### 示例 2: 按浏览器类型选择（自动同步）

```rust
// 随机选择 Chrome 指纹
let result = get_random_fingerprint_by_browser("chrome")?;

// ✅ TCP Profile 已自动同步
// 无论 User-Agent 是 Windows、Linux 还是 macOS，TCP Profile 都会匹配
```

### 示例 3: 指定操作系统（自动同步）

```rust
use fingerprint_core::types::OperatingSystem;

// 指定 Linux 操作系统
let result = get_random_fingerprint_with_os(Some(OperatingSystem::Linux))?;

// ✅ TCP Profile 已自动同步为 Linux
// TTL = 64, Window Size = 65535
```

## 验证同步

### 验证方法 1: 检查 TCP Profile

```rust
let result = get_random_fingerprint()?;
let tcp_profile = result.profile.tcp_profile.as_ref().unwrap();

// 从 User-Agent 推断操作系统
let os = if result.user_agent.contains("Windows") { "Windows" }
    else if result.user_agent.contains("Macintosh") { "macOS" }
    else { "Linux" };

// 验证 TTL 是否匹配
let expected_ttl = if os == "Windows" { 128 } else { 64 };
assert_eq!(tcp_profile.ttl, expected_ttl);
```

### 验证方法 2: 使用统一指纹生成函数

```rust
use fingerprint_profiles::profiles::generate_unified_fingerprint;

// 显式使用统一指纹生成函数（推荐）
let user_agent = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) ...";
let profile = generate_unified_fingerprint("chrome_135", user_agent)?;

// ✅ TCP Profile 已同步
assert_eq!(profile.tcp_profile.unwrap().ttl, 128);
```

## 重要说明

### ✅ 已自动同步的场景

- `get_random_fingerprint()` - ✅ 自动同步
- `get_random_fingerprint_by_browser()` - ✅ 自动同步
- `get_random_fingerprint_with_os()` - ✅ 自动同步
- `generate_unified_fingerprint()` - ✅ 自动同步

### ⚠️ 需要手动同步的场景

如果你直接使用以下函数，需要手动同步：

```rust
use fingerprint_profiles::profiles::{get_client_profile, ClientProfile};

// 直接获取 profile（不会自动同步）
let profile = get_client_profile("chrome_135")?;
let user_agent = get_user_agent_by_profile_name("chrome_135")?;

// 需要手动同步
let synced_profile = profile.with_synced_tcp_profile(&user_agent);
```

## 总结

**是的，TCP 指纹和浏览器指纹现在是完全同步的！**

- ✅ 每次选择浏览器指纹时，TCP 指纹都会根据 User-Agent 自动匹配
- ✅ 无需手动操作，系统会自动确保一致性
- ✅ 避免因指纹不匹配而被检测系统识别为异常

**使用建议**：
- 优先使用 `get_random_fingerprint()` 等自动同步的函数
- 如果直接使用 `get_client_profile()`，记得调用 `with_synced_tcp_profile()` 同步
