# Profiles 模块文档

## 概述

`profiles` 模块管理所有浏览器指纹配置，提供 69 个真实浏览器的 TLS 和 HTTP/2 配置。

## 模块位置

**Crate**: `fingerprint-profiles`  
**代码路径**: `crates/fingerprint-profiles/src/profiles.rs`

## 核心类型

### ClientProfile

完整的浏览器指纹配置，包含 TLS 和 HTTP/2 设置。

```rust
pub struct ClientProfile {
    pub client_hello_id: ClientHelloID,
    pub settings: HTTP2Settings,
    pub settings_order: Vec<u16>,
    pub pseudo_header_order: Vec<String>,
    pub connection_flow: u32,
    pub priorities: Vec<String>,
    pub header_priority: Option<HTTP2PriorityParam>,
}
```

### ClientHelloID

浏览器指纹标识符。

```rust
pub struct ClientHelloID {
    pub client: String,      // 浏览器名称（如 "Chrome", "Firefox"）
    pub version: String,      // 版本号（如 "133", "120"）
    pub spec_factory: ClientHelloSpecFactory,
}
```

## 主要函数

### 核心浏览器配置

- `chrome_103()` - Chrome 103 指纹配置
- `chrome_133()` - Chrome 133 指纹配置
- `firefox_133()` - Firefox 133 指纹配置
- `safari_16_0()` - Safari 16.0 指纹配置
- `opera_91()` - Opera 91 指纹配置

### 全局映射

- `mapped_tls_clients()` - 返回所有浏览器指纹的映射表
- `default_client_profile()` - 返回默认指纹配置

## 支持的浏览器

### Chrome 系列（19 个版本）
- chrome_103, chrome_104, chrome_105, chrome_106, chrome_107
- chrome_108, chrome_109, chrome_110, chrome_111, chrome_112
- chrome_116_PSK, chrome_116_PSK_PQ, chrome_117, chrome_120
- chrome_124, chrome_130_PSK, chrome_131, chrome_131_PSK
- chrome_133, chrome_133_PSK

### Firefox 系列（13 个版本）
- firefox_102, firefox_104, firefox_105, firefox_106, firefox_108
- firefox_110, firefox_117, firefox_120, firefox_123
- firefox_132, firefox_133, firefox_135

### Safari 系列（14 个版本）
- safari_15_6_1, safari_16_0
- safari_ios_15_5, safari_ios_15_6, safari_ios_16_0
- safari_ios_17_0, safari_ios_18_0, safari_ios_18_5
- safari_ipad_15_6

### Opera 系列（3 个版本）
- opera_89, opera_90, opera_91

### Edge 系列（3 个版本）
- edge_120, edge_124, edge_133

### 移动客户端（17+ 个）
- OkHttp4 (Android 7-13)
- Mesh (Android/iOS)
- Nike, Zalando, MMS, Confirmed 等

## 使用示例

```rust
use fingerprint::{chrome_133, ClientProfile};

// 获取 Chrome 133 指纹配置
let profile = chrome_133();

// 获取 ClientHello Spec
let spec = profile.get_client_hello_spec()?;

// 获取 HTTP/2 Settings
let settings = profile.settings;

// 获取 User-Agent 字符串
let client_id = profile.get_client_hello_str();
// 输出: "Chrome-133"
```

## 相关文档

- [TLS 配置文档](tls_config.md)
- [HTTP/2 配置文档](http_client.md#http2-配置)
