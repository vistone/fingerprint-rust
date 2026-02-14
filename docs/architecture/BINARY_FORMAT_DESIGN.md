# 二进制格式设计文档

## 概述

本文档说明 `fingerprint-rust` 如何处理二进制数据格式，特别是 TLS ClientHello 消息的序列化和反序列化。

## 核心设计原则

### 1. 数据驱动设计 ✅

**实际实现**：
- ✅ **不使用 `byteorder` 库**：我们使用 Rust 标准库的内置方法
- ✅ **使用标准库方法**：`to_be_bytes()` 和 `from_be_bytes()` 处理字节序
- ✅ **手动二进制解析**：直接操作字节数组，完全控制格式

**代码示例**：
```rust
// src/tls_handshake/record.rs
impl TLSRecord {
    /// 序列化为字节流
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        
        // Content Type (1 byte)
        bytes.push(self.content_type.as_u8());
        
        // Version (2 bytes, big-endian)
        bytes.extend_from_slice(&self.version.to_be_bytes());
        
        // Length (2 bytes, big-endian)
        let length = self.fragment.len() as u16;
        bytes.extend_from_slice(&length.to_be_bytes());
        
        // Fragment
        bytes.extend_from_slice(&self.fragment);
        
        bytes
    }
    
    /// 从字节流解析
    pub fn from_bytes(data: &[u8]) -> Result<(Self, usize), String> {
        // 使用 from_be_bytes 解析大端序数据
        let version = u16::from_be_bytes([data[1], data[2]]);
        let length = u16::from_be_bytes([data[3], data[4]]) as usize;
        // ...
    }
}
```

**为什么不用 `byteorder`**：
- Rust 标准库已经提供了 `to_be_bytes()` 和 `from_be_bytes()` 方法
- 无需额外依赖，减少编译时间和二进制大小
- 代码更简洁，类型安全

### 2. 鲁棒性和互操作性 ✅

**实际实现**：
- ✅ **serde 序列化**：用于配置导出（JSON 格式）
- ✅ **serde_json**：将 `ClientHelloSpec` 导出为 JSON，供其他语言使用
- ✅ **多格式支持**：DNS 模块支持 JSON、YAML、TOML 三种格式

**代码示例**：
```rust
// src/export.rs
#[derive(Serialize, Deserialize, Debug)]
pub struct ExportConfig {
    pub cipher_suites: Vec<u16>,
    pub compression_methods: Vec<u8>,
    pub extensions: Vec<ExportExtension>,
    pub tls_vers_min: u16,
    pub tls_vers_max: u16,
}

/// 将 ClientHelloSpec 转换为 JSON 字符串
pub fn export_config_json(spec: &ClientHelloSpec) -> Result<String, serde_json::Error> {
    let export = ExportConfig::from(spec);
    serde_json::to_string_pretty(&export)
}
```

**使用场景**：
- 配置导出：将 TLS 指纹配置导出为 JSON，供 Go uTLS 等使用
- DNS 数据存储：支持多种格式，便于集成和调试
- API 集成：通过 JSON 格式与其他系统交换数据

### 3. 安全性 ✅

**实际实现**：
- ✅ **sha2 库**：用于 JA4 指纹生成（SHA256 哈希）
- ✅ **安全存储**：指纹哈希而非原始数据
- ✅ **验证机制**：通过哈希验证指纹完整性

**代码示例**：
```rust
// src/tls_config/ja4.rs
use sha2::{Digest, Sha256};

/// 生成 12 字符哈希（SHA256 的前 12 个字符）
pub fn hash12(input: &str) -> String {
    let hash = Sha256::digest(input.as_bytes());
    let hash_hex = format!("{:x}", hash);
    hash_hex.get(..12).unwrap_or(&hash_hex).to_string()
}
```

**安全考虑**：
- 使用 SHA256 生成指纹哈希，确保唯一性和不可逆性
- JA4 指纹只使用哈希的前 12 个字符，平衡唯一性和可读性
- 不存储敏感信息，只存储配置和哈希值

### 4. 错误处理 ✅

**实际实现**：
- ✅ **thiserror 库**：提供结构化的错误类型
- ✅ **Result 类型**：所有可能失败的操作都返回 `Result`
- ✅ **详细错误信息**：提供清晰的错误上下文

**代码示例**：
```rust
// 使用 thiserror 定义错误类型
#[derive(Debug, thiserror::Error)]
pub enum HttpClientError {
    #[error("IO 错误: {0}")]
    Io(#[from] std::io::Error),
    #[error("无效的 URL: {0}")]
    InvalidUrl(String),
    #[error("无效的响应: {0}")]
    InvalidResponse(String),
    // ...
}
```

## 二进制格式处理

### TLS 记录层格式

```
TLS Record:
┌─────────────┬──────────┬──────────┬─────────────┐
│ Content Type│  Version │  Length  │   Fragment  │
│   (1 byte)  │ (2 bytes)│ (2 bytes)│ (variable)  │
└─────────────┴──────────┴──────────┴─────────────┘
```

**实现**：`src/tls_handshake/record.rs`

### TLS 握手层格式

```
TLS Handshake:
┌──────────────┬──────────────┬─────────────┐
│ Message Type │    Length    │    Body     │
│   (1 byte)   │  (3 bytes)   │ (variable)  │
└──────────────┴──────────────┴─────────────┘
```

**实现**：`src/tls_handshake/handshake.rs`

### ClientHello 消息格式

```
ClientHello:
┌──────────────┬──────────┬─────────────┬──────────────┬──────────────┬─────────────┐
│   Version    │  Random  │ Session ID  │ Cipher Suites│ Compression  │ Extensions  │
│  (2 bytes)   │(32 bytes)│ (variable)  │  (variable)  │  (variable)  │ (variable)  │
└──────────────┴──────────┴─────────────┴──────────────┴──────────────┴─────────────┘
```

**实现**：`src/tls_handshake/messages.rs`

## 字节序处理

### 大端序（Big-Endian）

TLS 协议使用**网络字节序**（大端序），所有多字节字段都使用大端序：

```rust
// 序列化（写入）
bytes.extend_from_slice(&self.version.to_be_bytes());

// 反序列化（读取）
let version = u16::from_be_bytes([data[1], data[2]]);
```

### 使用的标准库方法

- `u16::to_be_bytes()` - 将 u16 转换为大端序字节数组
- `u16::from_be_bytes()` - 从大端序字节数组解析 u16
- `u32::to_be_bytes()` - 将 u32 转换为大端序字节数组
- `u32::from_be_bytes()` - 从大端序字节数组解析 u32

## 序列化策略

### 1. 二进制序列化（TLS 协议）

**用途**：生成可发送的 TLS ClientHello 消息

**实现**：
- 手动构建字节数组
- 使用 `to_be_bytes()` 处理多字节字段
- 完全控制格式，符合 TLS 规范

**位置**：
- `src/tls_handshake/record.rs::to_bytes()`
- `src/tls_handshake/handshake.rs::to_bytes()`
- `src/tls_handshake/messages.rs::to_bytes()`

### 2. JSON 序列化（配置导出）

**用途**：导出配置供其他语言使用

**实现**：
- 使用 `serde` 和 `serde_json`
- 定义 `ExportConfig` 结构体
- 支持 `Serialize` 和 `Deserialize` trait

**位置**：
- `src/export.rs::export_config_json()`

### 3. 多格式序列化（DNS 模块）

**用途**：DNS 数据存储和交换

**实现**：
- 支持 JSON、YAML、TOML 三种格式
- 使用 `serde` 统一序列化接口
- 原子性写入，确保数据安全

**位置**：
- `src/dns/storage.rs::save_domain_ips()`

## 反序列化策略

### 1. 二进制解析（TLS 协议）

**实现**：
- 手动解析字节数组
- 使用 `from_be_bytes()` 解析多字节字段
- 验证数据完整性和格式

**位置**：
- `src/tls_handshake/record.rs::from_bytes()`
- `src/tls_handshake/handshake.rs::from_bytes()`

### 2. JSON 解析（配置导入）

**实现**：
- 使用 `serde_json::from_str()`
- 自动验证格式和类型
- 提供详细的错误信息

## 错误处理

### 使用 thiserror

```rust
#[derive(Debug, thiserror::Error)]
pub enum DNSError {
    #[error("IO 错误: {0}")]
    IO(#[from] std::io::Error),
    #[error("配置错误: {0}")]
    Config(String),
    #[error("YAML 解析错误: {0}")]
    Yaml(String),
    // ...
}
```

### 错误传播

- 使用 `?` 操作符自动传播错误
- 使用 `Result<T, E>` 类型明确错误可能性
- 提供上下文信息，便于调试

## 总结

### 实际依赖

| 依赖 | 用途 | 状态 |
|------|------|------|
| **sha2** | JA4 指纹哈希生成 | ✅ 使用中 |
| **serde** | 配置序列化/反序列化 | ✅ 使用中（可选） |
| **serde_json** | JSON 格式支持 | ✅ 使用中（可选） |
| **thiserror** | 结构化错误处理 | ✅ 使用中 |
| **byteorder** | ❌ **不存在** | 使用标准库方法替代 |

### 设计特点

1. **零额外依赖**：二进制处理使用标准库，无需 `byteorder`
2. **类型安全**：充分利用 Rust 类型系统
3. **完全控制**：手动构建和解析，确保格式正确
4. **互操作性**：通过 JSON 导出支持跨语言集成
5. **安全性**：使用 SHA256 哈希，不存储敏感数据

### 与 ISO 19794-2 的区别

| 特性 | ISO 19794-2 | fingerprint-rust |
|------|-------------|------------------|
| **用途** | 生物识别指纹（手指） | TLS 网络协议指纹 |
| **格式** | 指纹图像/特征点 | TLS ClientHello 消息 |
| **标准** | ISO/IEC 19794-2 | TLS 1.3 (RFC 8446) |
| **字节序** | 可能使用小端序 | 网络字节序（大端） |
| **序列化** | 专用二进制格式 | TLS 协议格式 |

---

**结论**：`fingerprint-rust` 使用 Rust 标准库处理二进制数据，不依赖 `byteorder`。通过 `serde` 提供配置导出功能，通过 `sha2` 提供安全哈希，通过 `thiserror` 提供可靠的错误处理。
