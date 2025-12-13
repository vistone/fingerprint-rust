# TLS 指纹配置文档

## 概述

本库提供了真实的 TLS Client Hello 配置和 HTTP/2 Settings，对应 Go 版本的 `utls.ClientHelloID` 和 `http2.Settings`。

## TLS Client Hello 配置

### ClientHelloSpec

`ClientHelloSpec` 结构包含了完整的 TLS Client Hello 配置：

```rust
pub struct ClientHelloSpec {
    pub tls_versions: Vec<u16>,                    // TLS 版本
    pub cipher_suites: Vec<CipherSuiteID>,        // 密码套件列表
    pub compression_methods: Vec<u8>,              // 压缩方法
    pub extensions: Vec<Extension>,                // 扩展列表
    pub elliptic_curves: Vec<u16>,                 // 椭圆曲线列表
    pub elliptic_curve_point_formats: Vec<u8>,     // 椭圆曲线点格式
    pub alpn_protocols: Vec<String>,               // ALPN 协议列表
    pub signature_algorithms: Vec<u16>,             // 签名算法
    pub signature_algorithms_cert: Vec<u16>,        // 签名算法证书
    pub supported_groups: Vec<u16>,                 // 支持的组（TLS 1.3）
    pub supported_versions: Vec<u16>,              // 支持的版本（TLS 1.3）
    pub psk_key_exchange_modes: Vec<u8>,          // PSK 密钥交换模式
    pub custom_extensions: HashMap<u16, Vec<u8>>,  // 自定义扩展
}
```

### 使用方法

```rust
use fingerprint::*;

// 获取指纹配置
let profile = mapped_tls_clients().get("chrome_133").unwrap();

// 获取 TLS Client Hello Spec
let client_hello_spec = profile.get_client_hello_spec()?;

// 使用配置
println!("密码套件: {:?}", client_hello_spec.cipher_suites);
println!("椭圆曲线: {:?}", client_hello_spec.elliptic_curves);
println!("ALPN: {:?}", client_hello_spec.alpn_protocols);
```

## HTTP/2 配置

### HTTP/2 Settings

HTTP/2 Settings 对应 Go 版本的 `map[http2.SettingID]uint32`：

```rust
use fingerprint::*;

let profile = mapped_tls_clients().get("chrome_133").unwrap();
let settings = profile.get_settings();

// 访问具体的 Setting
let header_table_size = settings.get(&1).unwrap(); // HeaderTableSize
let max_concurrent_streams = settings.get(&3).unwrap(); // MaxConcurrentStreams
```

### Pseudo Header Order

Pseudo Header Order 定义了 HTTP/2 伪头部的顺序：

```rust
let chrome_order = profile.get_pseudo_header_order();
// Chrome: [":method", ":authority", ":scheme", ":path"]
// Firefox: [":method", ":path", ":authority", ":scheme"]
// Safari: [":method", ":scheme", ":path", ":authority"]
```

### Header Priority

Header Priority 定义了 HTTP/2 请求的优先级：

```rust
if let Some(priority) = profile.get_header_priority() {
    println!("Weight: {}", priority.weight);
    println!("Stream Dependency: {}", priority.stream_dependency);
    println!("Exclusive: {}", priority.exclusive);
}
```

## 浏览器差异

不同浏览器的配置差异：

### Chrome
- **Pseudo Header Order**: `:method`, `:authority`, `:scheme`, `:path`
- **Initial Window Size**: 6291456
- **Header Priority**: Weight=256, Exclusive=false

### Firefox
- **Pseudo Header Order**: `:method`, `:path`, `:authority`, `:scheme`
- **Initial Window Size**: 131072
- **Header Priority**: None

### Safari
- **Pseudo Header Order**: `:method`, `:scheme`, `:path`, `:authority`
- **Initial Window Size**: 65535
- **Max Frame Size**: 16777215

## 集成到 HTTP 客户端

这些配置可以用于配置各种 Rust HTTP 客户端库：

### 使用 rustls

```rust
use fingerprint::*;
use rustls::ClientConfig;

let profile = mapped_tls_clients().get("chrome_133").unwrap();
let client_hello_spec = profile.get_client_hello_spec()?;

// 配置 rustls ClientConfig
// 注意：rustls 不直接支持自定义 Client Hello，但可以使用这些配置
// 来选择最接近的密码套件和扩展
```

### 使用 reqwest

```rust
use fingerprint::*;
use reqwest::Client;

let profile = mapped_tls_clients().get("chrome_133").unwrap();
let headers = profile.headers.to_map();

// 创建带有自定义 Headers 的客户端
let client = Client::builder()
    .default_headers(headers.into())
    .build()?;
```

## 注意事项

1. **TLS Client Hello**: Rust 的 `rustls` 库不直接支持完全自定义的 Client Hello。如果需要完全控制 Client Hello，可能需要使用其他库或 FFI 绑定。

2. **HTTP/2 Settings**: 这些 Settings 可以用于配置支持 HTTP/2 的客户端库（如 `h2` crate）。

3. **兼容性**: 配置基于真实浏览器的行为，但实际使用中可能需要根据具体的 TLS/HTTP 库进行调整。

## 示例

查看 `examples/tls_config.rs` 获取完整的使用示例。
