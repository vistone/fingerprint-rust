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
- 🎯 **Fingerprinting** - 识别客户端的特征和身份

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

**HttpFingerprint** - HTTP Fingerprint
```rust
pub struct HttpFingerprint {
    pub method: String,
    pub path: String,
    pub headers: HashMap<String, String>,
    pub user_agent: Option<String>,
}
```

**TlsFingerprint** - TLS Fingerprint
```rust
pub struct TlsFingerprint {
    pub version: u16,
    pub cipher_suites: Vec<u16>,
    pub extensions: Vec<u16>,
    pub signature_algs: Vec<u16>,
}
```

**TcpFingerprint** - TCP Fingerprint
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

## 🔍 高级使用场景

### 场景 A: 从网络流量捕获实时指纹

```rust
use fingerprint_defense::PassiveAnalyzer;
use pnet::datalink;
use pnet::packet::ethernet::EtherTypes;

async fn capture_and_analyze() {
    let analyzer = PassiveAnalyzer::new();
    
    // 获取网络接口
    let interfaces = datalink::interfaces();
    let interface = interfaces.iter()
        .find(|i| !i.is_loopback())
        .expect("找不到非本地接口");
    
    println!("在接口 {} 上捕获数据包", interface.name);
    
    let (_, mut rx) = match datalink::channel(interface, Default::default()) {
        Ok(Channel::Ethernet(tx, rx)) => (tx, rx),
        Ok(_) => panic!("不支持的接口类型"),
        Err(e) => panic!("创建通道失败: {}", e),
    };
    
    // 捕获和分析数据包
    loop {
        match rx.next() {
            Ok(packet) => {
                let ethernet = EthernetPacket::new(packet);
                
                match ethernet.map(|eth| eth.get_ethertype()) {
                    Some(EtherTypes::Ipv4) => {
                        // 分析 IPv4 数据包
                        if let Ok(fingerprint) = analyzer.analyze_tcp(packet) {
                            println!("发现 TCP Fingerprint: TTL={}, Window={}", 
                                fingerprint.ttl, fingerprint.window_size);
                        }
                    }
                    Some(EtherTypes::Ipv6) => {
                        // 分析 IPv6 数据包
                    }
                    _ => {}
                }
            }
            Err(e) => {
                eprintln!("接收错误: {}", e);
                break;
            }
        }
    }
}
```

### 场景 B: 构建指纹数据库

```rust
use fingerprint_defense::PassiveAnalyzer;
use std::collections::HashMap;

struct FingerprintDatabase {
    http_fingerprints: HashMap<String, usize>,
    tls_fingerprints: HashMap<String, usize>,
    tcp_fingerprints: HashMap<String, usize>,
}

impl FingerprintDatabase {
    fn new() -> Self {
        FingerprintDatabase {
            http_fingerprints: HashMap::new(),
            tls_fingerprints: HashMap::new(),
            tcp_fingerprints: HashMap::new(),
        }
    }
    
    fn record_http(&mut self, analyzer: &PassiveAnalyzer, data: &[u8]) {
        if let Ok(fp) = analyzer.analyze_http(data) {
            let key = format!("{} {} {:?}", fp.method, fp.path, fp.user_agent);
            *self.http_fingerprints.entry(key).or_insert(0) += 1;
        }
    }
    
    fn record_tls(&mut self, analyzer: &PassiveAnalyzer, data: &[u8]) {
        if let Ok(fp) = analyzer.analyze_tls(data) {
            let key = format!("TLS 0x{:04x} (ciphers: {})", fp.version, fp.cipher_suites.len());
            *self.tls_fingerprints.entry(key).or_insert(0) += 1;
        }
    }
    
    fn get_statistics(&self) {
        println!("=== 指纹数据库统计 ===");
        println!("HTTP 指纹类型: {}", self.http_fingerprints.len());
        println!("TLS 指纹类型: {}", self.tls_fingerprints.len());
        println!("TCP 指纹类型: {}", self.tcp_fingerprints.len());
        
        // 输出最常见的指纹
        for (fp, count) in self.http_fingerprints.iter().take(5) {
            println!("  HTTP: {} (出现 {} 次)", fp, count);
        }
    }
}
```

### 场景 C: 实时异常检测

```rust
use fingerprint_defense::PassiveAnalyzer;

struct AnomalyDetector {
    normal_http_ua: Vec<String>,
    normal_tls_versions: Vec<u16>,
    normal_ttl_range: (u8, u8),
}

impl AnomalyDetector {
    fn new() -> Self {
        AnomalyDetector {
            normal_http_ua: vec![
                "Mozilla/5.0".to_string(),
                "Chrome/".to_string(),
            ],
            normal_tls_versions: vec![0x0303, 0x0304], // TLS 1.2, 1.3
            normal_ttl_range: (64, 255),
        }
    }
    
    fn check_anomaly(&self, analyzer: &PassiveAnalyzer, data: &[u8]) -> Vec<String> {
        let mut anomalies = Vec::new();
        
        // 检查 HTTP 异常
        if let Ok(http) = analyzer.analyze_http(data) {
            if let Some(ua) = http.user_agent {
                if !self.normal_http_ua.iter().any(|n| ua.contains(n)) {
                    anomalies.push(format!("异常 User-Agent: {}", ua));
                }
            }
        }
        
        // 检查 TLS 异常
        if let Ok(tls) = analyzer.analyze_tls(data) {
            if !self.normal_tls_versions.contains(&tls.version) {
                anomalies.push(format!("异常 TLS 版本: 0x{:04x}", tls.version));
            }
        }
        
        // 检查 TCP 异常
        if let Ok(tcp) = analyzer.analyze_tcp(data) {
            if tcp.ttl < self.normal_ttl_range.0 || tcp.ttl > self.normal_ttl_range.1 {
                anomalies.push(format!("异常 TTL: {}", tcp.ttl));
            }
        }
        
        anomalies
    }
}
```

---

## 🔍 高级使用场景

### 场景 A: 从网络流量捕获实时指纹

```rust
use fingerprint_defense::PassiveAnalyzer;
use pnet::datalink;
use pnet::packet::ethernet::EtherTypes;

async fn capture_and_analyze() {
    let analyzer = PassiveAnalyzer::new();
    
    // 获取网络接口
    let interfaces = datalink::interfaces();
    let interface = interfaces.iter()
        .find(|i| !i.is_loopback())
        .expect("找不到非本地接口");
    
    println!("在接口 {} 上捕获数据包", interface.name);
    
    let (_, mut rx) = match datalink::channel(interface, Default::default()) {
        Ok(Channel::Ethernet(tx, rx)) => (tx, rx),
        Ok(_) => panic!("不支持的接口类型"),
        Err(e) => panic!("创建通道失败: {}", e),
    };
    
    // 捕获和分析数据包
    loop {
        match rx.next() {
            Ok(packet) => {
                let ethernet = EthernetPacket::new(packet);
                
                match ethernet.map(|eth| eth.get_ethertype()) {
                    Some(EtherTypes::Ipv4) => {
                        // 分析 IPv4 数据包
                        if let Ok(fingerprint) = analyzer.analyze_tcp(packet) {
                            println!("发现 TCP Fingerprint: TTL={}, Window={}", 
                                fingerprint.ttl, fingerprint.window_size);
                        }
                    }
                    Some(EtherTypes::Ipv6) => {
                        // 分析 IPv6 数据包
                    }
                    _ => {}
                }
            }
            Err(e) => {
                eprintln!("接收错误: {}", e);
                break;
            }
        }
    }
}
```

### 场景 B: 构建指纹数据库

```rust
use fingerprint_defense::PassiveAnalyzer;
use std::collections::HashMap;

struct FingerprintDatabase {
    http_fingerprints: HashMap<String, usize>,
    tls_fingerprints: HashMap<String, usize>,
    tcp_fingerprints: HashMap<String, usize>,
}

impl FingerprintDatabase {
    fn new() -> Self {
        FingerprintDatabase {
            http_fingerprints: HashMap::new(),
            tls_fingerprints: HashMap::new(),
            tcp_fingerprints: HashMap::new(),
        }
    }
    
    fn record_http(&mut self, analyzer: &PassiveAnalyzer, data: &[u8]) {
        if let Ok(fp) = analyzer.analyze_http(data) {
            let key = format!("{} {} {:?}", fp.method, fp.path, fp.user_agent);
            *self.http_fingerprints.entry(key).or_insert(0) += 1;
        }
    }
    
    fn record_tls(&mut self, analyzer: &PassiveAnalyzer, data: &[u8]) {
        if let Ok(fp) = analyzer.analyze_tls(data) {
            let key = format!("TLS 0x{:04x} (ciphers: {})", fp.version, fp.cipher_suites.len());
            *self.tls_fingerprints.entry(key).or_insert(0) += 1;
        }
    }
    
    fn get_statistics(&self) {
        println!("=== 指纹数据库统计 ===");
        println!("HTTP 指纹类型: {}", self.http_fingerprints.len());
        println!("TLS 指纹类型: {}", self.tls_fingerprints.len());
        println!("TCP 指纹类型: {}", self.tcp_fingerprints.len());
        
        // 输出最常见的指纹
        for (fp, count) in self.http_fingerprints.iter().take(5) {
            println!("  HTTP: {} (出现 {} 次)", fp, count);
        }
    }
}
```

### 场景 C: 实时异常检测

```rust
use fingerprint_defense::PassiveAnalyzer;

struct AnomalyDetector {
    normal_http_ua: Vec<String>,
    normal_tls_versions: Vec<u16>,
    normal_ttl_range: (u8, u8),
}

impl AnomalyDetector {
    fn new() -> Self {
        AnomalyDetector {
            normal_http_ua: vec![
                "Mozilla/5.0".to_string(),
                "Chrome/".to_string(),
            ],
            normal_tls_versions: vec![0x0303, 0x0304], // TLS 1.2, 1.3
            normal_ttl_range: (64, 255),
        }
    }
    
    fn check_anomaly(&self, analyzer: &PassiveAnalyzer, data: &[u8]) -> Vec<String> {
        let mut anomalies = Vec::new();
        
        // 检查 HTTP 异常
        if let Ok(http) = analyzer.analyze_http(data) {
            if let Some(ua) = http.user_agent {
                if !self.normal_http_ua.iter().any(|n| ua.contains(n)) {
                    anomalies.push(format!("异常 User-Agent: {}", ua));
                }
            }
        }
        
        // 检查 TLS 异常
        if let Ok(tls) = analyzer.analyze_tls(data) {
            if !self.normal_tls_versions.contains(&tls.version) {
                anomalies.push(format!("异常 TLS 版本: 0x{:04x}", tls.version));
            }
        }
        
        // 检查 TCP 异常
        if let Ok(tcp) = analyzer.analyze_tcp(data) {
            if tcp.ttl < self.normal_ttl_range.0 || tcp.ttl > self.normal_ttl_range.1 {
                anomalies.push(format!("异常 TTL: {}", tcp.ttl));
            }
        }
        
        anomalies
    }
}
```

---

## 🔗 相关模块

- [fingerprint-core](core.md) - 系统级核心抽象
- [fingerprint-tls](tls.md) - TLS协议分析
- [fingerprint-http](http.md) - HTTP协议分析
- [fingerprint-anomaly](anomaly.md) - 异常检测模块

---
*最后更新: 2026-02-13*