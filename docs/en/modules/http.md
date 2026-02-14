# HTTP模块文档

**版本**: v1.0  
**最后更新**: 2026-02-13  
**模块**: fingerprint-http

---

## 🎯 模块概述

fingerprint-http 是HTTP客户端实现模块，支持HTTP/1.1、HTTP/2和HTTP/3协议，同时包含QUIC初始数据包指纹识别功能，提供完整的现代HTTP协议栈实现。

## 📦 主要组件

### 1. HTTP客户端 (http_client)
完整的HTTP客户端实现，支持：
- HTTP/1.1 基础请求（分块传输、压缩、重定向）
- HTTP/2 多路复用（HPACK压缩、服务器推送）
- HTTP/3 QUIC传输（0-RTT、连接迁移）
- 连接池管理
- 请求重试机制
- 协议自动协商

### 2. QUIC指纹 (quic_fingerprint)
QUIC协议指纹识别，包括：
- 初始数据包分析
- 版本协商检测
- 数据包类型识别
- 指纹特征提取
- JA4Q指纹生成

## 🔧 核心功能

### 多协议HTTP客户端
```rust
use fingerprint_http::{HttpClient, HttpRequest, HttpVersion};

let client = HttpClient::builder()
    .enable_http2(true)
    .enable_http3(true)
    .connection_pool_size(100)
    .idle_timeout(Duration::from_secs(300))
    .build()?;

let request = HttpRequest::get("https://example.com")
    .header("User-Agent", "Custom Browser/1.0")
    .header("Accept", "*/*")
    .build();

let response = client.send(request).await?;
println!("Status: {}", response.status());
println!("Protocol: {:?}", response.version());
```

### QUIC指纹识别
```rust
use fingerprint_http::{QuicInitialPacket, QuicVersion, QuicPacketType};

// 解析QUIC初始包
let raw_packet = vec![0xc0, 0x00, 0x00, 0x00, /* ... */];
let quic_packet = QuicInitialPacket::parse(&raw_packet)?;

// 获取版本信息
let version = quic_packet.version();
println!("QUIC Version: {:?}", version);

// 生成指纹
let fingerprint = quic_packet.generate_fingerprint();
println!("JA4Q Fingerprint: {}", fingerprint);
```

### 协议强制使用
```rust
use fingerprint_http::HttpVersion;

// 强制使用HTTP/3
let response = client.get("https://example.com")
    .force_protocol(HttpVersion::Http3)
    .await?;

// 强制使用HTTP/2
let response = client.get("https://example.com")
    .force_protocol(HttpVersion::Http2)
    .await?;
```

## 📊 协议支持详情

### HTTP/1.1
**特性支持**:
- ✅ 标准HTTP请求/响应
- ✅ Keep-Alive连接复用
- ✅ Chunked传输编码
- ✅ Gzip/Deflate/Brotli压缩
- ✅ 自动重定向处理
- ✅ 基本身份验证

### HTTP/2
**特性支持**:
- ✅ 二进制协议帧
- ✅ 多路复用流
- ✅ 服务器推送
- ✅ 头部压缩(HPACK)
- ✅ 流量控制
- ✅ 优先级管理

### HTTP/3
**特性支持**:
- ✅ 基于QUIC传输
- ✅ 0-RTT连接建立
- ✅ 连接迁移
- ✅ 流量控制
- ✅ 多路复用
- ✅ 前向纠错

## ⚡ 性能优化

### 连接管理
```rust
let client = HttpClient::builder()
    .connection_pool_size(100)           // 连接池大小
    .idle_timeout(Duration::from_secs(300))  // 空闲超时
    .connection_timeout(Duration::from_secs(10)) // 连接超时
    .max_redirects(5)                    // Maximum redirect hops
    .build()?;
```

### 协议协商策略
```rust
// 默认自动协商（HTTP/3 → HTTP/2 → HTTP/1.1）
let response = client.get("https://example.com").await?;

// 自定义协商顺序
let client = HttpClient::builder()
    .preferred_versions(vec![HttpVersion::Http2, HttpVersion::Http11])
    .build()?;
```

## 🔒 安全特性

### TLS集成
- **TLS 1.3支持**: 现代加密标准
- **证书验证**: 严格的证书链验证
- **ALPN协商**: 协议自动协商
- **前向保密**: 完美的前向保密支持

### 安全头部
```rust
let request = HttpRequest::get("https://example.com")
    .header("Sec-Fetch-Site", "none")
    .header("Sec-Fetch-Mode", "navigate")
    .header("Sec-Fetch-User", "?1")
    .header("Sec-Fetch-Dest", "document")
    .build();
```

## 🧪 使用示例

### 基础GET请求
```rust
use fingerprint_http::HttpClient;

let client = HttpClient::new()?;
let response = client.get("https://httpbin.org/get").await?;
println!("Response: {}", response.text().await?);
```

### POST请求带数据
```rust
let response = client
    .post("https://httpbin.org/post")
    .json(&serde_json::json!({"key": "value"}))
    .await?;
```

### 文件上传
```rust
let form = Form::new()
    .text("key", "value")
    .file("file", "/path/to/file.txt")?;

let response = client
    .post("https://httpbin.org/post")
    .multipart(form)
    .await?;
```

### WebSocket支持
```rust
let (ws_stream, response) = client
    .websocket("wss://echo.websocket.org")
    .await?;
```

## 🔗 相关模块

- [fingerprint-tls](tls.md) - TLS协议支持
- [fingerprint-core](core.md) - 核心抽象层
- [fingerprint-gateway](gateway.md) - API网关集成

---
*最后更新: 2026-02-13*