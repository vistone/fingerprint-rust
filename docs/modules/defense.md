# 🛡️ fingerprint-defense 模块

**Crate**: `fingerprint-defense`  
**版本**: 2.1.0  
**用途**: 被动网络分析和指纹识别

---

## 📋 概述

`fingerprint-defense` 模块提供被动网络分析能力，可以分析网络流量、数据包和协议，识别客户端的网络指纹特征。

### 核心功能

- 🔍 **被动分析** - 无需修改网络流量即可分析
- 📊 **多层识别** - 支持 HTTP、TLS、TCP 层分析
- 🎯 **指纹识别** - 识别客户端的特征和身份

---

## 🏗️ 模块结构

### 主要类型

#### 1. `PacketParser`
解析网络数据包的工具。

```rust
pub struct PacketParser;

impl PacketParser {
    pub fn parse_http(data: &[u8]) -> Result<HttpFingerprint, PassiveError>;
    pub fn parse_tls(data: &[u8]) -> Result<TlsFingerprint, PassiveError>;
    pub fn parse_tcp(data: &[u8]) -> Result<TcpFingerprint, PassiveError>;
}
```

#### 2. `PassiveAnalyzer`
执行被动分析的核心模块。

```rust
pub struct PassiveAnalyzer {
    // 分析器配置
}

impl PassiveAnalyzer {
    pub fn analyze(&self, packet: &Packet) -> Result<PassiveAnalysisResult, PassiveError>;
    pub fn analyze_http(&self, data: &[u8]) -> Result<HttpFingerprint, PassiveError>;
    pub fn analyze_tls(&self, data: &[u8]) -> Result<TlsFingerprint, PassiveError>;
}
```

#### 3. `Packet`
表示网络数据包的结构。

```rust
pub struct Packet {
    pub data: Vec<u8>,
    pub timestamp: u64,
    pub direction: PacketDirection,
}
```

#### 4. 指纹类型

**HttpFingerprint** - HTTP 指纹
```rust
pub struct HttpFingerprint {
    pub method: String,
    pub path: String,
    pub headers: HashMap<String, String>,
    pub user_agent: Option<String>,
}
```

**TlsFingerprint** - TLS 指纹
```rust
pub struct TlsFingerprint {
    pub version: u16,
    pub cipher_suites: Vec<u16>,
    pub extensions: Vec<u16>,
    pub signature_algs: Vec<u16>,
}
```

**TcpFingerprint** - TCP 指纹
```rust
pub struct TcpFingerprint {
    pub ttl: u8,
    pub window_size: u16,
    pub mss: Option<u16>,
}
```

---

## 🔍 使用场景

### 场景 1: 分析 HTTP 请求
```rust
use fingerprint_defense::{PassiveAnalyzer, PacketParser};

let analyzer = PassiveAnalyzer::new();
let http_data = b"GET /path HTTP/1.1\r\nHost: example.com\r\n\r\n";
let fingerprint = analyzer.analyze_http(http_data)?;

println!("User-Agent: {:?}", fingerprint.user_agent);
println!("Headers: {:?}", fingerprint.headers);
```

### 场景 2: 分析 TLS 握手
```rust
use fingerprint_defense::{PassiveAnalyzer};

let analyzer = PassiveAnalyzer::new();
let tls_data = /* TLS Client Hello 数据 */;
let fingerprint = analyzer.analyze_tls(tls_data)?;

println!("Cipher Suites: {:?}", fingerprint.cipher_suites);
println!("Extensions: {:?}", fingerprint.extensions);
```

### 场景 3: 分析 TCP 特征
```rust
use fingerprint_defense::{PassiveAnalyzer};

let analyzer = PassiveAnalyzer::new();
let tcp_data = /* TCP 数据包 */;
let fingerprint = analyzer.analyze_tcp(tcp_data)?;

println!("TTL: {}", fingerprint.ttl);
println!("Window Size: {}", fingerprint.window_size);
```

---

## 📊 输出结果

### `PassiveAnalysisResult`
被动分析的完整结果。

```rust
pub struct PassiveAnalysisResult {
    pub http: Option<HttpFingerprint>,
    pub tls: Option<TlsFingerprint>,
    pub tcp: Option<TcpFingerprint>,
    pub confidence: f32,
}
```

---

## 🚀 高级用法

### 组合分析
```rust
use fingerprint_defense::{PassiveAnalyzer, Packet};

let analyzer = PassiveAnalyzer::new();
let packet = Packet {
    data: vec![/* 完整数据包 */],
    timestamp: std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs(),
    direction: PacketDirection::ClientToServer,
};

let result = analyzer.analyze(&packet)?;
println!("Confidence: {}%", result.confidence * 100.0);
```

---

## 🛠️ 错误处理

```rust
pub enum PassiveError {
    InvalidData,
    ParseError(String),
    UnsupportedProtocol,
    Other(String),
}
```

使用示例：
```rust
match analyzer.analyze_http(data) {
    Ok(fingerprint) => println!("分析成功: {:?}", fingerprint),
    Err(PassiveError::InvalidData) => println!("无效的数据"),
    Err(PassiveError::ParseError(e)) => println!("解析错误: {}", e),
    Err(e) => println!("其他错误: {:?}", e),
}
```

---

## 📝 特性

该模块支持以下 Cargo 特性：

```toml
[features]
default = []
# 启用被动分析
passive-analysis = []
# 启用 HTTP 分析
http-analysis = []
# 启用 TLS 分析
tls-analysis = []
```

---

## 🔗 相关模块

- **fingerprint-tls** - TLS 指纹生成和分析
- **fingerprint-http** - HTTP 客户端和协议处理
- **fingerprint-core** - 核心类型定义

---

## 📚 参考资源

- [PassiveAnalyzer API 文档](../API.md)
- [网络分析指南](../guides/PASSIVE_ANALYSIS_GUIDE.md)
- [TLS 深度分析](../CLIENTHELLO_ANALYSIS.md)

---

**最后更新**: 2026-02-11  
**作者**: fingerprint-rust 项目


