# TCP 指纹应用指南

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
