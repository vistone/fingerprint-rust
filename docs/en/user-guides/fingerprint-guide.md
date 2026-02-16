# 浏览器指纹使用指南

**版本**: v2.0  
**最后更新**: 2026-02-16  
**适用版本**: fingerprint-rust 2.1.0+

---

## 🎯 概述

本指南详细介绍如何使用 fingerprint-rust 项目中的浏览器指纹功能，包括配置、使用和最佳实践。

## 📦 支持的浏览器指纹

### 当前支持版本
项目目前支持 **90+** 预配置的浏览器指纹：

#### Chrome系列
- Chrome 103-138 (Windows)
- Chrome Mobile 120-137 (Android)
- Chrome PSK/0-RTT 变体

#### Firefox系列
- Firefox 102-138 (Windows)
- Firefox Mobile 120-135 (Android)

#### Safari系列
- Safari 15.0-18.3 (macOS)
- Safari iOS 15.5-18.5 (iOS)
- Safari iPad (iPadOS)

#### Edge系列
- Edge 120-137 (Windows)

#### Opera系列
- Opera 89-94 (Windows)

### 指纹配置文件结构
每个指纹配置文件包含以下关键信息：

```json
{
  "browser": "Chrome",
  "version": "133",
  "os": "Windows",
  "user_agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36...",
  "tls_fingerprint": {
    "cipher_suites": [...],
    "extensions": [...]
  },
  "http_headers": {
    "accept": "*/*",
    "accept_encoding": "gzip, deflate, br",
    "accept_language": "en-US,en;q=0.9"
  }
}
```

## 🚀 快速开始

### 1. 基本使用

```rust
use fingerprint::{get_random_fingerprint, get_random_fingerprint_by_browser};

// 获取随机浏览器指纹
let result = get_random_fingerprint()?;
println!("Profile ID: {}", result.profile_id);
println!("User-Agent: {}", result.user_agent);
println!("Browser Type: {:?}", result.browser_type);

// 获取特定浏览器的随机指纹
let chrome = get_random_fingerprint_by_browser("chrome")?;
println!("Chrome Profile: {}", chrome.profile_id);
```

### 2. 使用浏览器配置

```rust
use fingerprint::mapped_tls_clients;

// 获取所有可用的浏览器配置
let profiles = mapped_tls_clients();

// 获取特定浏览器配置
if let Some(chrome_133) = profiles.get("chrome_133") {
    // 获取 TLS Client Hello 配置
    let spec = chrome_133.get_client_hello_spec()?;
    println!("Cipher suites: {}", spec.cipher_suites.len());
    println!("Extensions: {}", spec.extensions.len());
    
    // 获取 HTTP/2 设置
    let settings = chrome_133.get_settings();
    println!("HTTP/2 settings: {}", settings.len());
}
```

## 🛠️ 高级功能

### 指定操作系统

```rust
use fingerprint::{get_random_fingerprint_with_os, OperatingSystem};

// 获取 Windows 系统的指纹
let windows_fp = get_random_fingerprint_with_os(Some(OperatingSystem::Windows10))?;

// 获取 macOS 系统的指纹
let macos_fp = get_random_fingerprint_with_os(Some(OperatingSystem::MacOS14))?;

// 获取 Linux 系统的指纹
let linux_fp = get_random_fingerprint_with_os(Some(OperatingSystem::Linux))?;
```

### TCP 指纹配置

```rust
use fingerprint_core::tcp::TcpProfile;

// 根据操作系统生成 TCP 配置
let tcp_profile = TcpProfile::for_os(OperatingSystem::Windows10);
println!("TTL: {}", tcp_profile.ttl);
println!("Window Size: {}", tcp_profile.window_size);

// 从 User-Agent 推断 TCP 配置
let tcp_from_ua = TcpProfile::from_user_agent("Mozilla/5.0 (Windows NT 10.0...");
```

### HTTP Headers 操作

```rust
use fingerprint::get_random_fingerprint;

let mut result = get_random_fingerprint()?;

// 添加自定义 Headers
result.headers.set("Cookie", "session_id=abc123");
result.headers.set("Authorization", "Bearer token123");

// 获取所有 Headers
let headers_map = result.headers.to_map();
for (key, value) in headers_map.iter() {
    println!("{}: {}", key, value);
}
```

## 🔧 配置选项

### TLS 指纹配置

浏览器配置包含完整的 TLS Client Hello 规范：

- **cipher_suites**: 密码套件列表
- **extensions**: TLS 扩展列表
- **tls_vers_min/max**: 支持的 TLS 版本范围
- **compression_methods**: 压缩方法

### HTTP Headers 配置

使用 `HTTPHeaders` 结构体管理 HTTP 请求头：

```rust
use fingerprint::get_random_fingerprint;

let result = get_random_fingerprint()?;

// 访问标准 headers
println!("Accept: {}", result.headers.accept);
println!("Accept-Language: {}", result.headers.accept_language);
println!("Accept-Encoding: {}", result.headers.accept_encoding);
println!("Sec-CH-UA: {}", result.headers.sec_ch_ua);
```

## 📊 性能优化

### HTTP/2 设置

浏览器配置还包含 HTTP/2 设置，可以通过以下方式获取：

```rust
use fingerprint::mapped_tls_clients;

let profiles = mapped_tls_clients();
if let Some(chrome) = profiles.get("chrome_133") {
    // HTTP/2 settings
    let settings = chrome.get_settings();
    for (id, value) in settings.iter() {
        println!("Setting {}: {}", id, value);
    }
    
    // Pseudo header order
    let order = chrome.get_pseudo_header_order();
    println!("Header order: {:?}", order);
    
    // Header priority
    if let Some(priority) = chrome.get_header_priority() {
        println!("Weight: {}", priority.weight);
        println!("Stream dependency: {}", priority.stream_dependency);
    }
}
```

## 🔒 安全考虑

### 指纹多样性

为了避免被检测，建议使用多种不同的指纹：

```rust
use fingerprint::get_random_fingerprint;

// 每次请求使用不同的随机指纹
for i in 0..10 {
    let fp = get_random_fingerprint()?;
    println!("Request {} using profile: {}", i, fp.profile_id);
    // ... 发送请求
}
```

### 操作系统一致性

确保 User-Agent 和 TCP 指纹的操作系统一致：

```rust
use fingerprint::{get_random_fingerprint_with_os, OperatingSystem};
use fingerprint_core::tcp::TcpProfile;

// 获取 Windows 指纹
let fp = get_random_fingerprint_with_os(Some(OperatingSystem::Windows10))?;

// 验证 User-Agent 包含 Windows
assert!(fp.user_agent.contains("Windows"));

// 生成匹配的 TCP 配置
let tcp = TcpProfile::from_user_agent(&fp.user_agent);
println!("TTL: {} (Windows should be 128)", tcp.ttl);
```

## 📈 监控和调试

### 启用详细日志

```bash
# 启用调试日志
RUST_LOG=fingerprint=debug cargo run

# 启用特定模块日志  
RUST_LOG=fingerprint_tls=trace cargo run
```

## 🆘 故障排除

### 常见问题

**Q: 指纹被识别为机器人？**
A: 确保 User-Agent、TLS 指纹和 TCP 指纹操作系统一致

**Q: TLS握手失败？**
A: 确保使用的 TLS 配置与目标服务器兼容

**Q: HTTP Headers 顺序不正确？**
A: 使用 `headers.to_map()` 获取正确排序的 headers

## 📚 相关资源

- [API参考文档](../reference/)
- [架构设计文档](../ARCHITECTURE.md)
- [开发指南](../developer-guides/)

---
*最后更新: 2026-02-16*  
*版本: v2.0*