# TLS Handshake 模块文档

**版本**: v1.0  
**最后更新**: 2026-02-13  
**文档类型**: 技术文档

---



## 概述

`tls_handshake` 模块提供完全自主的 TLS ClientHello 消息构建功能，不依赖外部 TLS 库。

## 模块位置

`src/tls_handshake/`

## 核心类型

### TLSHandshakeBuilder

TLS 握手构建器，根据 ClientHelloSpec 构建完整的 TLS ClientHello 消息。

```rust
pub struct TLSHandshakeBuilder;

impl TLSHandshakeBuilder {
    pub fn build_client_hello(
        spec: &ClientHelloSpec,
        server_name: &str,
    ) -> Result<Vec<u8>, String>;
}
```

### TLSRecord

TLS 记录层封装。

```rust
pub struct TLSRecord {
    pub content_type: TLSRecordType,
    pub version: u16,
    pub fragment: Vec<u8>,
}
```

### TLSHandshake

TLS 握手消息封装。

```rust
pub struct TLSHandshake {
    pub msg_type: TLSHandshakeType,
    pub body: Vec<u8>,
}
```

### ClientHelloMessage

ClientHello 消息结构。

```rust
pub struct ClientHelloMessage {
    pub client_version: u16,
    pub random: Vec<u8>,
    pub session_id: Vec<u8>,
    pub cipher_suites: Vec<u16>,
    pub compression_methods: Vec<u8>,
    pub extensions: Vec<u8>,
}
```

## 主要功能

### 1. ClientHello 构建

根据 `ClientHelloSpec` 构建完整的 TLS ClientHello 消息：

```rust
use fingerprint::{TLSHandshakeBuilder, chrome_133};

let profile = chrome_133();
let spec = profile.get_client_hello_spec()?;

// 构建 ClientHello
let client_hello = TLSHandshakeBuilder::build_client_hello(
    &spec,
    "www.example.com"
)?;

// 可以直接发送到服务器
stream.write_all(&client_hello)?;
```

### 2. 真实密钥生成

使用 `ring` 库生成真实的密钥对（需要 `crypto` feature）：

- X25519 密钥对
- P-256 密钥对
- P-384 密钥对

### 3. TLS 1.3 兼容

- ✅ Non-empty Session ID (32 bytes)
- ✅ ChangeCipherSpec after ClientHello
- ✅ BoringSSL Padding Style
- ✅ 真实的 KeyShare 公钥

## 模块结构

```
tls_handshake/
├── mod.rs          # 模块入口
├── builder.rs      # TLSHandshakeBuilder
├── handshake.rs    # TLSHandshake 消息
├── messages.rs     # ClientHelloMessage
└── record.rs       # TLSRecord 记录层
```

## 使用示例

```rust
use fingerprint::{TLSHandshakeBuilder, chrome_133};
use std::net::TcpStream;
use std::io::Write;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. 获取浏览器指纹
    let profile = chrome_133();
    let spec = profile.get_client_hello_spec()?;
    
    // 2. 构建真实的 TLS ClientHello
    let client_hello = TLSHandshakeBuilder::build_client_hello(
        &spec,
        "www.google.com"
    )?;
    
    // 3. 发送到服务器
    let mut stream = TcpStream::connect("www.google.com:443")?;
    stream.write_all(&client_hello)?;
    
    // 4. 发送 ChangeCipherSpec (TLS 1.3 兼容)
    let ccs = [0x14, 0x03, 0x01, 0x00, 0x01, 0x01];
    stream.write_all(&ccs)?;
    
    Ok(())
}
```

## 相关文档

- [TLS 配置文档](tls_config.md)
- [自定义 TLS 指纹文档](../archive/history/CUSTOM_TLS_FINGERPRINT.md)
