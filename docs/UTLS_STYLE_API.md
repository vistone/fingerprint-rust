# uTLS 风格 API 使用指南

## 概述

本项目提供了类似 [uTLS](https://github.com/refraction-networking/utls) 的 API 风格，通过 `ClientHelloCustomizer` 和 `TLSHandshakeBuilder` 提供对 ClientHello 的控制，用于模拟浏览器指纹。

**注意**：当前主要通过 `rustls` 的 `ClientHelloCustomizer` 来应用浏览器指纹，而不是完全自定义的 TLS 实现。`TLSHandshakeBuilder` 可以构建完整的 ClientHello 消息，但需要配合完整的 TLS 握手实现才能使用。

## 核心接口

### 1. ClientHelloID

对应 Go 版本的 `tls.ClientHelloID`，用于选择浏览器指纹：

```rust
use fingerprint::http_client::utls::ClientHelloID;

// 可用的 ClientHello ID
let hello_id = ClientHelloID::HelloChrome133;  // Chrome 133
let hello_id = ClientHelloID::HelloFirefox133; // Firefox 133
let hello_id = ClientHelloID::HelloSafari160;  // Safari 16.0
let hello_id = ClientHelloID::HelloCustom;     // 完全自定义
```

### 2. UConfig

对应 Go 版本的 `tls.Config`，用于配置 TLS 连接：

```rust
use fingerprint::http_client::utls::{UConfig, ClientHelloID};

let config = UConfig::new()
    .with_server_name("example.com".to_string())
    .with_client_hello_id(ClientHelloID::HelloChrome133);
```

### 3. UClient

对应 Go 版本的 `utls.UClient()`，创建自定义 TLS 连接：

```rust
use fingerprint::http_client::utls::{UClient, UConfig, ClientHelloID};
use std::net::TcpStream;

// 建立 TCP 连接
let tcp_stream = TcpStream::connect("example.com:443")?;

// 创建配置
let config = UConfig::new()
    .with_server_name("example.com".to_string())
    .with_client_hello_id(ClientHelloID::HelloChrome133);

// 创建 uTLS 连接
let mut uconn = UClient(tcp_stream, config, ClientHelloID::HelloChrome133)?;

// 现在可以使用 uconn 进行读写
uconn.write_all(b"GET / HTTP/1.1\r\nHost: example.com\r\n\r\n")?;
```

## 使用示例

### 示例 1: 基本使用（类似 Go 版本的迁移）

**Go 版本 (uTLS)**:
```go
dialConn, err := net.Dial("tcp", "172.217.11.46:443")
config := tls.Config{ServerName: "www.google.com"}
tlsConn := tls.UClient(dialConn, &config, tls.HelloChrome_133)
```

**Rust 版本**:
```rust
use fingerprint::http_client::utls::{UClient, UConfig, ClientHelloID};
use std::net::TcpStream;

let tcp_stream = TcpStream::connect("172.217.11.46:443")?;
let config = UConfig::new()
    .with_server_name("www.google.com".to_string());
let uconn = UClient(tcp_stream, config, ClientHelloID::HelloChrome133)?;
```

### 示例 2: 使用自定义 ClientHelloSpec

**Go 版本 (uTLS)**:
```go
uConn := UClient(&net.TCPConn{}, nil, HelloCustom)
generatedSpec, err := fingerprinter.FingerprintClientHello(rawCapturedClientHelloBytes)
uConn.ApplyPreset(generatedSpec)
```

**Rust 版本**:
```rust
use fingerprint::http_client::utls::{UConfig, ClientHelloID, apply_preset};
use fingerprint::{chrome_133, TLSHandshakeBuilder};

// 获取 ClientHelloSpec
let profile = chrome_133();
let spec = profile.get_client_hello_spec()?;

// 创建配置并应用预设
let mut config = UConfig::new()
    .with_server_name("example.com".to_string());
apply_preset(&mut config, spec);

// 使用配置创建连接
let tcp_stream = TcpStream::connect("example.com:443")?;
let uconn = UClient(tcp_stream, config, ClientHelloID::HelloCustom)?;
```

### 示例 3: 在 HTTP 客户端中使用

```rust
use fingerprint::{HttpClient, HttpClientConfig, chrome_133};

// 创建配置，包含浏览器指纹
let config = HttpClientConfig {
    profile: Some(chrome_133()),
    ..Default::default()
};

// 创建 HTTP 客户端
let client = HttpClient::new(config);

// 发送请求（将自动使用自定义 ClientHello）
let response = client.get("https://example.com")?;
```

### 示例 4: 完全自定义 ClientHello

```rust
use fingerprint::http_client::utls::{UConfig, ClientHelloID};
use fingerprint::tls_config::ClientHelloSpec;
use fingerprint::tls_extensions::*;

// 创建自定义 ClientHelloSpec
let mut spec = ClientHelloSpec::new();
spec.cipher_suites = vec![0xc02f, 0xc030, 0x1301];
spec.compression_methods = vec![0];
spec.extensions = vec![
    Box::new(SNIExtension::new("example.com")),
    Box::new(SupportedVersionsExtension::new(vec![0x0304])),
    // ... 添加更多扩展
];

// 使用自定义 spec
let config = UConfig::new()
    .with_server_name("example.com".to_string())
    .with_custom_spec(spec);

let tcp_stream = TcpStream::connect("example.com:443")?;
let uconn = UClient(tcp_stream, config, ClientHelloID::HelloCustom)?;
```

## API 对比

| Go (uTLS) | Rust (本项目) | 说明 |
|-----------|----------------|------|
| `tls.UClient()` | `UClient()` | 创建自定义 TLS 连接 |
| `tls.HelloChrome_133` | `ClientHelloID::HelloChrome133` | Chrome 133 指纹 |
| `tls.HelloFirefox_133` | `ClientHelloID::HelloFirefox133` | Firefox 133 指纹 |
| `tls.HelloCustom` | `ClientHelloID::HelloCustom` | 完全自定义模式 |
| `uconn.ApplyPreset()` | `apply_preset()` | 应用预设指纹 |
| `uconn.BuildHandshakeState()` | `uconn.build_handshake_state()` | 构建握手状态 |
| `tls.Config` | `UConfig` | TLS 配置 |

## 特性

### ✅ 已实现

1. **ClientHelloID 枚举** - 支持多种浏览器指纹
2. **UConfig 配置** - 灵活的配置选项
3. **UClient 函数** - 类似 Go 版本的接口
4. **ApplyPreset** - 应用预设指纹
5. **自动集成** - HTTP 客户端自动使用自定义 ClientHello

### 当前实现状态

1. ✅ **TLS 指纹应用** - 已通过 `ClientHelloCustomizer` 实现扩展顺序调整
2. ✅ **ClientHello 构建** - `TLSHandshakeBuilder` 可以构建完整的 ClientHello 消息
3. ⚠️ **完整 TLS 握手** - 当前使用 rustls 处理完整握手，`TLSHandshakeBuilder` 只构建 ClientHello 消息
4. ⚠️ **HelloRandomized** - 当前使用固定的浏览器指纹配置，不支持随机化
5. ⚠️ **Session Tickets** - 由 rustls 处理，当前不支持自定义假 Session Tickets

## 与标准 TLS 的对比

### 标准 TLS (rustls)

```rust
use rustls::ClientConfig;
let config = ClientConfig::builder()
    .with_safe_defaults()
    .with_root_certificates(root_store)
    .with_no_client_auth();
let conn = rustls::ClientConnection::new(Arc::new(config), server_name)?;
```

**限制**: 无法自定义 ClientHello 的细节（密码套件顺序、扩展顺序等）

### uTLS 风格 (本项目)

```rust
use fingerprint::http_client::utls::{UClient, UConfig, ClientHelloID};
let config = UConfig::new()
    .with_server_name("example.com".to_string())
    .with_client_hello_id(ClientHelloID::HelloChrome133);
let uconn = UClient(tcp_stream, config, ClientHelloID::HelloChrome133)?;
```

**优势**: 完全控制 ClientHello，可以精确模拟浏览器指纹

## 启用方式

在 `Cargo.toml` 中启用 `custom-tls` feature：

```toml
[dependencies]
fingerprint = { path = ".", features = ["custom-tls", "rustls-tls"] }
```

或通过命令行：

```bash
cargo build --features custom-tls
```

## 参考

- [uTLS GitHub](https://github.com/refraction-networking/utls) - Go 版本的 uTLS 实现
- [项目文档](./CUSTOM_TLS_IMPLEMENTATION.md) - 自定义 TLS 实现详情

