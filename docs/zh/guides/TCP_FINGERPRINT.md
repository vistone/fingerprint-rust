# TCP 指纹应用指南

**版本**: v1.0  
**最后更新**: 2026-02-13  
**文档类型**: 技术文档

---



## 概述

fingerprint-rust 现在支持在创建 TCP 连接时应用 TCP Profile，确保 TCP 指纹（TTL、Window Size、MSS、Window Scale）与浏览器指纹一致，避免被检测系统识别为异常。

## 功能说明

### 已实现的功能

1. **TCP Profile 应用模块** (`tcp_fingerprint.rs`)
   - `apply_tcp_profile()` - 应用 TCP Profile 到 socket
   - `create_tcp_socket_with_profile()` - 创建带有 TCP Profile 的 socket
   - `connect_tcp_with_profile()` - 创建带有 TCP Profile 的异步 TcpStream
   - `connect_tcp_with_profile_sync()` - 创建带有 TCP Profile 的同步 TcpStream

2. **HTTP/2 连接自动应用**
   - 在 `http2.rs` 中，如果 `config.profile.tcp_profile` 存在，会自动应用 TCP Profile

### 应用的限制

**重要说明**：由于 TCP 协议的特性，某些参数无法在连接建立后修改：

1. **TTL (Time To Live)**
   - ✅ **可以设置**：通过 `socket.set_ttl()` 设置
   - ✅ **会被发送**：TTL 值会在 IP 包头中发送

2. **Window Size (接收窗口大小)**
   - ⚠️ **部分设置**：可以通过 `socket.set_recv_buffer_size()` 设置接收缓冲区
   - ⚠️ **实际值由系统协商**：实际的 TCP Window Size 是在握手时由操作系统和网络栈协商的
   - ⚠️ **可能不完全匹配**：设置缓冲区大小会影响 Window Size，但不保证完全一致

3. **MSS (Maximum Segment Size)**
   - ❌ **无法直接设置**：MSS 是在 TCP 握手时通过 TCP 选项协商的
   - ⚠️ **系统默认**：通常由 MTU 自动计算（MTU - 40 bytes）

4. **Window Scale**
   - ❌ **无法直接设置**：Window Scale 是在 TCP 握手时通过 TCP 选项协商的
   - ⚠️ **系统默认**：由操作系统和网络栈决定

### 实际效果

虽然无法完全控制所有 TCP 参数，但通过设置 TTL 和缓冲区大小，我们可以：

1. **TTL 完全匹配**：TTL 值会准确发送，这是 p0f 识别操作系统的重要指标
2. **Window Size 近似匹配**：通过设置缓冲区大小，Window Size 会接近目标值
3. **降低检测风险**：即使 MSS 和 Window Scale 不完全匹配，TTL 和 Window Size 的匹配已经大大降低了被检测的风险

## 使用方法

### 方法 1: 自动应用（推荐）

使用 `generate_unified_fingerprint()` 生成统一的指纹，TCP Profile 会自动应用到 HTTP/2 连接：

```rust
use fingerprint_profiles::profiles::generate_unified_fingerprint;

let user_agent = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/135.0.0.0 Safari/537.36";
let profile = generate_unified_fingerprint("chrome_135", user_agent)?;

// 在 HttpClientConfig 中使用 profile
let config = HttpClientConfig {
    profile: Some(profile),
    // ... 其他配置
};

// HTTP/2 连接会自动应用 TCP Profile
```

### 方法 2: 手动应用

```rust
use fingerprint_http::http_client::tcp_fingerprint;
use fingerprint_core::tcp::TcpProfile;
use fingerprint_core::types::OperatingSystem;

let tcp_profile = TcpProfile::for_os(OperatingSystem::Windows10);
let addr: SocketAddr = "example.com:443".parse()?;

// 异步连接
let stream = tcp_fingerprint::connect_tcp_with_profile(addr, Some(&tcp_profile)).await?;

// 同步连接
let stream = tcp_fingerprint::connect_tcp_with_profile_sync(addr, Some(&tcp_profile))?;
```

## 验证 TCP 指纹

### 使用 fingerprint-defense 验证

```rust
use fingerprint_defense::PassiveAnalyzer;

let analyzer = PassiveAnalyzer::new()?;

// 捕获网络数据包（需要 root 权限）
// 分析 TCP 连接，验证 TTL、Window Size 等参数是否匹配
```

### 使用 tcpdump/wireshark 验证

```bash
# 捕获 TCP 连接
sudo tcpdump -i any -w capture.pcap 'tcp and host example.com'

# 使用 wireshark 分析
wireshark capture.pcap

# 检查：
# 1. IP 包头的 TTL 值
# 2. TCP 握手中的 Window Size
# 3. TCP 选项中的 MSS 和 Window Scale
```

## 注意事项

1. **权限要求**
   - 设置 TTL 通常需要 root 权限（在某些系统上）
   - 如果无法设置 TTL，连接仍会建立，但 TTL 将使用系统默认值

2. **操作系统限制**
   - 不同操作系统对 TCP 参数的控制能力不同
   - Linux 通常提供更多的控制选项
   - Windows/macOS 可能有一些限制

3. **网络环境**
   - 某些网络环境可能会修改 TCP 参数（如 NAT、防火墙）
   - 实际发送的参数可能与设置的值不完全一致

4. **连接池**
   - 连接池中的连接在创建时应用 TCP Profile
   - 复用的连接会保持原有的 TCP 参数
   - 建议在创建连接池之前就同步 TCP Profile

## 最佳实践

1. **始终使用统一指纹生成**
   ```rust
   let profile = generate_unified_fingerprint(profile_name, user_agent)?;
   ```

2. **验证 TCP 指纹**
   - 使用 `fingerprint-defense` 的 `PassiveAnalyzer` 验证
   - 或使用 tcpdump/wireshark 抓包分析

3. **处理权限问题**
   - 如果无法设置 TTL，记录警告但继续执行
   - 考虑在容器或虚拟环境中运行以获得更多控制

4. **监控和调试**
   - 记录应用的 TCP Profile 参数
   - 对比实际发送的参数，识别差异

## 技术细节

### TTL 设置

```rust
socket.set_ttl(tcp_profile.ttl as u32)?;
```

- TTL 在 IP 包头中发送
- 每个路由器会递减 TTL
- p0f 通过观察到的 TTL 推断初始 TTL

### Window Size 设置

```rust
socket.set_recv_buffer_size(tcp_profile.window_size as usize)?;
socket.set_send_buffer_size(tcp_profile.window_size as usize)?;
```

- 缓冲区大小会影响 TCP Window Size
- 实际 Window Size 由系统协商决定
- 通常接近但不完全等于设置的值

### MSS 和 Window Scale

- 这些参数在 TCP 握手时通过 TCP 选项协商
- 无法在连接建立后修改
- 由操作系统和网络栈自动处理

## 总结

虽然无法完全控制所有 TCP 参数，但通过设置 TTL 和缓冲区大小，我们已经能够：

- ✅ **TTL 完全匹配**：准确发送目标操作系统的 TTL 值
- ✅ **Window Size 近似匹配**：通过缓冲区设置影响 Window Size
- ✅ **降低检测风险**：TTL 和 Window Size 的匹配已经大大降低了被检测的风险

对于 MSS 和 Window Scale，虽然无法直接控制，但它们通常不会成为主要的检测指标。TTL 和 Window Size 的匹配已经足够让我们的指纹看起来像真实的浏览器。


## 同步指南

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
