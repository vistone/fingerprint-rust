# TLS ClientHello 解析设计

## 概述

为 PCAP 分析器添加 TLS ClientHello 解析功能，以提取更精确的浏览器指纹特征。

## 当前状态

### ✅ 已实现
- `ClientHelloSignature` 结构 (signature.rs)
- JA3/JA4 指纹生成器
- TLS ClientHello 生成器 (用于客户端)
- TLS 配置和规格 (TlsConfig, ClientHelloSpec)

### ⏳ 待实现
- 从 PCAP 数据包中解析 TLS ClientHello
- 提取 ClientHello 字段到 `ClientHelloSignature`
- 在 PCAP 分析器中集成 TLS 指纹
- TLS 指纹比对和匹配

## 架构设计

### 数据流

```
PCAP 文件
    ↓
解析以太网帧
    ↓
解析 IP 包
    ↓
解析 TCP 段
    ↓
提取 TCP Payload
    ↓
[NEW] 检测 TLS 记录
    ↓
[NEW] 解析 Handshake 消息
    ↓
[NEW] 提取 ClientHello 字段
    ↓
生成 ClientHelloSignature
    ↓
计算 JA3/JA4 指纹
    ↓
匹配已知浏览器 Profile
```

### 模块划分

```rust
crates/fingerprint-core/src/
  ├── tls_parser.rs        [NEW] - TLS 记录层解析
  ├── client_hello_parser.rs [NEW] - ClientHello 解析
  ├── signature.rs         [EXIST] - ClientHelloSignature
  ├── ja3.rs              [EXIST] - JA3 指纹
  └── ja4.rs              [EXIST] - JA4 指纹

crates/fingerprint/src/bin/
  └── fingerprint_analyze.rs [ENHANCE] - 集成 TLS 解析
```

## TLS 记录层格式

### TLS Record Structure

```
+----------------+----------------+
|  Content Type  |  Version (2)   |
| (1 byte)       |                |
+----------------+----------------+
|  Length (2 bytes)              |
+--------------------------------+
|  Fragment (variable length)    |
|  ...                           |
+--------------------------------+
```

**Content Types:**
- 0x16: Handshake
- 0x14: ChangeCipherSpec
- 0x15: Alert
- 0x17: Application Data

### Handshake Message Structure  

```
+----------------+
|  Message Type  |  (1 byte)
+----------------+----------------+
|  Length (3 bytes)              |
+--------------------------------+
|  Handshake Body (variable)     |
|  ...                           |
+--------------------------------+
```

**Message Types:**
- 0x01: ClientHello ← **我们关注的**
- 0x02: ServerHello
- 0x0B: Certificate
- 0x10: ClientKeyExchange

## ClientHello 结构

### 完整格式

```rust
struct ClientHello {
    // TLS 版本 (Version) (2 bytes)
    client_version: u16,  // 0x0303 = TLS 1.2
    
    // 随机数 (32 bytes)
    random: [u8; 32],
    
    // Session ID (变长)
    session_id_length: u8,
    session_id: Vec<u8>,
    
    // 密码套件 (变长)
    cipher_suites_length: u16,
    cipher_suites: Vec<u16>,
    
    // 压缩方法 (变长)
    compression_methods_length: u8,
    compression_methods: Vec<u8>,
    
    // 扩展 (变长)
    extensions_length: u16,
    extensions: Vec<Extension>,
}
```

### Extension 格式

```rust
struct Extension {
    extension_type: u16,  // e.g., 0x0000 = SNI
    extension_data_length: u16,
    extension_data: Vec<u8>,
}
```

**关键扩展:**
- 0x0000: server_name (SNI)
- 0x0010: application_layer_protocol_negotiation (ALPN)
- 0x000a: supported_groups (椭圆曲线)
- 0x000b: ec_point_formats
- 0x000d: signature_algorithms
- 0x002b: supported_versions (TLS 1.3+)

## 实现计划

### Phase 1: TLS 记录解析器 ⏳

**文件:** `crates/fingerprint-core/src/tls_parser.rs`

```rust
/// TLS 记录头
pub struct TlsRecord {
    pub content_type: u8,
    pub version: u16,
    pub length: u16,
    pub fragment: Vec<u8>,
}

impl TlsRecord {
    /// 从字节流解析 TLS 记录
    pub fn parse(data: &[u8]) -> Result<Self, TlsParseError> {
        if data.len() < 5 {
            return Err(TlsParseError::TooShort);
        }
        
        let content_type = data[0];
        let version = u16::from_be_bytes([data[1], data[2]]);
        let length = u16::from_be_bytes([data[3], data[4]]);
        
        if data.len() < 5 + length as usize {
            return Err(TlsParseError::IncompleteRecord);
        }
        
        let fragment = data[5..5 + length as usize].to_vec();
        
        Ok(TlsRecord {
            content_type,
            version,
            length,
            fragment,
        })
    }
    
    /// 检查是否为 Handshake 记录
    pub fn is_handshake(&self) -> bool {
        self.content_type == 0x16
    }
}
```

### Phase 2: Handshake 消息解析器 ⏳

```rust
/// Handshake 消息类型
pub enum HandshakeType {
    ClientHello = 0x01,
    ServerHello = 0x02,
    Certificate = 0x0B,
    // ...
}

/// Handshake 消息
pub struct HandshakeMessage {
    pub msg_type: u8,
    pub length: u32,  // 3 bytes实际
    pub body: Vec<u8>,
}

impl HandshakeMessage {
    pub fn parse(data: &[u8]) -> Result<Self, TlsParseError> {
        if data.len() < 4 {
            return Err(TlsParseError::TooShort);
        }
        
        let msg_type = data[0];
        let length = u32::from_be_bytes([0, data[1], data[2], data[3]]);
        
        if data.len() < 4 + length as usize {
            return Err(TlsParseError::IncompleteMessage);
        }
        
        let body = data[4..4 + length as usize].to_vec();
        
        Ok(HandshakeMessage {
            msg_type,
            length,
            body,
        })
    }
    
    pub fn is_client_hello(&self) -> bool {
        self.msg_type == 0x01
    }
}
```

### Phase 3: ClientHello 字段提取器 ⏳

**文件:** `crates/fingerprint-core/src/client_hello_parser.rs`

```rust
use crate::signature::ClientHelloSignature;

pub struct ClientHelloParser;

impl ClientHelloParser {
    /// 从 Handshake body 解析 ClientHello
    pub fn parse(body: &[u8]) -> Result<ClientHelloSignature, TlsParseError> {
        let mut offset = 0;
        
        // 1. TLS Version (2 bytes)
        let client_version = u16::from_be_bytes([body[offset], body[offset + 1]]);
        offset += 2;
        
        // 2. Random (32 bytes)
        offset += 32;  // 跳过 random
        
        // 3. Session ID
        let session_id_len = body[offset] as usize;
        offset += 1 + session_id_len;
        
        // 4. Cipher Suites
        let cipher_suites_len = u16::from_be_bytes([body[offset], body[offset + 1]]) as usize;
        offset += 2;
        let mut cipher_suites = Vec::new();
        for _ in 0..(cipher_suites_len / 2) {
            let suite = u16::from_be_bytes([body[offset], body[offset + 1]]);
            cipher_suites.push(suite);
            offset += 2;
        }
        
        // 5. Compression Methods
        let comp_methods_len = body[offset] as usize;
        offset += 1 + comp_methods_len;
        
        // 6. Extensions
        if offset < body.len() {
            let ext_len = u16::from_be_bytes([body[offset], body[offset + 1]]) as usize;
            offset += 2;
            
            // TODO: 解析每个扩展
        }
        
        // 构造 ClientHelloSignature
        let mut signature = ClientHelloSignature::new();
        signature.version = TlsVersion::from_u16(client_version);
        signature.cipher_suites = cipher_suites;
        // ... 设置other字段
        
        Ok(signature)
    }
    
    /// 解析扩展
    fn parse_extensions(data: &[u8]) -> Vec<(u16, Vec<u8>)> {
        let mut extensions = Vec::new();
        let mut offset = 0;
        
        while offset + 4 <= data.len() {
            let ext_type = u16::from_be_bytes([data[offset], data[offset + 1]]);
            let ext_len = u16::from_be_bytes([data[offset + 2], data[offset + 3]]) as usize;
            offset += 4;
            
            if offset + ext_len <= data.len() {
                let ext_data = data[offset..offset + ext_len].to_vec();
                extensions.push((ext_type, ext_data));
                offset += ext_len;
            } else {
                break;
            }
        }
        
        extensions
    }
}
```

### Phase 4: 集成到 PCAP 分析器 ⏳

**文件:** `crates/fingerprint/src/bin/fingerprint_analyze.rs` (增强)

```rust
use fingerprint_core::{TlsRecord, HandshakeMessage, ClientHelloParser};

struct BrowserFingerprint {
    // 现有字段
    window_size: Option<u16>,
    ttl: Option<u8>,
    packet_count: usize,
    confidence: f64,
    
    // 新增 TLS 字段
    tls_signature: Option<ClientHelloSignature>,
    ja3_hash: Option<String>,
    ja4_hash: Option<String>,
}

fn analyze_pcap(path: &Path) -> Result<BrowserFingerprint, Box<dyn Error>> {
    // ... 现有 TCP 解析 ...
    
    // 新增 TLS 解析
    for packet in &packets {
        if let Some(tcp_payload) = packet.tcp_payload() {
            // 尝试解析 TLS 记录
            if let Ok(tls_record) = TlsRecord::parse(tcp_payload) {
                if tls_record.is_handshake() {
                    if let Ok(handshake) = HandshakeMessage::parse(&tls_record.fragment) {
                        if handshake.is_client_hello() {
                            // 解析 ClientHello
                            if let Ok(signature) = ClientHelloParser::parse(&handshake.body) {
                                // 计算 JA3/JA4
                                let ja3 = Ja3::from_client_hello(&signature);
                                let ja4 = Ja4::from_client_hello(&signature);
                                
                                // 保存指纹
                                fp.tls_signature = Some(signature);
                                fp.ja3_hash = Some(ja3.hash());
                                fp.ja4_hash = Some(ja4.hash());
                            }
                        }
                    }
                }
            }
        }
    }
    
    Ok(fp)
}

fn print_fingerprint_report(filename: &str, fp: &BrowserFingerprint) {
    // ... 现有 TCP 输出 ...
    
    // 新增 TLS 输出
    if let Some(tls) = &fp.tls_signature {
        println!("\n  TLS ClientHello:");
        println!("    Version: {:?}", tls.version);
        println!("    Cipher Suites: {} (top 3: {:?})", 
            tls.cipher_suites.len(),
            &tls.cipher_suites[..3.min(tls.cipher_suites.len())]
        );
        println!("    Extensions: {}", tls.extensions.len());
        if let Some(sni) = &tls.sni {
            println!("    SNI: {}", sni);
        }
        if let Some(alpn) = &tls.alpn {
            println!("    ALPN: {}", alpn);
        }
    }
    
    if let Some(ja3) = &fp.ja3_hash {
        println!("  JA3: {}", ja3);
    }
    if let Some(ja4) = &fp.ja4_hash {
        println!("  JA4: {}", ja4);
    }
}
```

## 测试策略

### 单元测试

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_tls_record() {
        // TLS 1.2 ClientHello 记录
        let data = vec![
            0x16,       // Content Type: Handshake
            0x03, 0x03, // Version: TLS 1.2
            0x00, 0x05, // Length: 5 bytes
            // Fragment (5 bytes)
            0x01, 0x02, 0x03, 0x04, 0x05,
        ];
        
        let record = TlsRecord::parse(&data).unwrap();
        assert_eq!(record.content_type, 0x16);
        assert_eq!(record.version, 0x0303);
        assert_eq!(record.length, 5);
        assert!(record.is_handshake());
    }
    
    #[test]
    fn test_parse_client_hello() {
        // 真实 Chrome ClientHello 数据
        let data = include_bytes!("../../../test_data/client_hello_chrome.bin");
        
        let signature = ClientHelloParser::parse(data).unwrap();
        assert!(!signature.cipher_suites.is_empty());
        assert!(!signature.extensions.is_empty());
        assert_eq!(signature.version, TlsVersion::V1_2);
    }
}
```

### 集成测试

```rust
#[test]
fn test_pcap_with_tls() {
    let pcap_path = Path::new("test_data/pcap/chrome_136.pcap");
    let fp = analyze_pcap(pcap_path).unwrap();
    
    // 验证 TLS 指纹被提取
    assert!(fp.tls_signature.is_some());
    assert!(fp.ja3_hash.is_some());
    
    // 验证 JA3 格式
    let ja3 = fp.ja3_hash.unwrap();
    assert_eq!(ja3.len(), 32);  // MD5 hash
}
```

## 性能考虑

### 优化策略

1. **早期退出**
   - 检查端口 443 (HTTPS)
   - 检查 TCP payload 长度 (至少 5 字节)
   - 检查 TLS 内容类型

2. **零拷贝解析**
   - 使用 `&[u8]` 切片而不是 `Vec<u8>`
   - 避免不必要的内存分配

3. **缓存结果**
   - 每个 TCP 连接只解析一次 ClientHello
   - 使用 HashMap<connection_id, ClientHelloSignature>

### 性能目标

| 指标 | 目标 | 基准 |
|------|------|------|
| PCAP 解析速度 | >500 MB/s | 589 MB/s (当前 TCP only) |
| TLS 解析延迟 | <10μs/包 | -  |
| 内存占用 | <100 MB | 30 MB (当前) |

## 安全考虑

### 输入验证

```rust
// 检查长度边界
if offset + field_len > data.len() {
    return Err(TlsParseError::BufferOverflow);
}

// 检查合理性
if cipher_suites_len > 1000 {  // 异常大
    return Err(TlsParseError::InvalidData);
}

// 防止无限循环
let mut max_iterations = 100;
while offset < data.len() && max_iterations > 0 {
    // ...
    max_iterations -= 1;
}
```

### 错误处理

```rust
#[derive(Debug)]
pub enum TlsParseError {
    TooShort,
    IncompleteRecord,
    IncompleteMessage,
    InvalidVersion,
    BufferOverflow,
    InvalidData,
}

impl std::fmt::Display for TlsParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            TlsParseError::TooShort => write!(f, "Data too short"),
            TlsParseError::IncompleteRecord => write!(f, "Incomplete TLS record"),
            // ...
        }
    }
}

impl std::error::Error for TlsParseError {}
```

## 参考资料

### RFC 标准

- **RFC 5246** - TLS 1.2 Protocol
  - Section 7.4.1: ClientHello 格式
  
- **RFC 8446** - TLS 1.3 Protocol
  - Section 4.1.2: ClientHello 变更

- **RFC 6066** - TLS Extensions
  - SNI, ALPN 等扩展定义

### 开源项目

- **JA3** - Salesforce
  - https://github.com/salesforce/ja3
  
- **JA4+** - FoxIO
  - https://github.com/FoxIO-LLC/ja4

- **rustls** - Rust TLS 实现
  - 可参考其 TLS 解析代码

## 路线图

### v1.0 - 基础 TLS 检测 (1 周)
- ✅ TLS 记录层解析
- ✅ Handshake 消息识别
- ✅ ClientHello 检测
- ⏳ 基本字段提取 (version, cipher_suites)

### v1.1 - 完整 TLS 指纹 (2 周)
- ⏳ 扩展解析
- ⏳ JA3 计算
- ⏳ JA4 计算
- ⏳ 指纹匹配

### v1.2 - TLS 1.3 支持 (1 周)
- ⏳ TLS 1.3 ClientHello 变更
- ⏳ encrypted_extensions 处理
- ⏳ PSK 模式支持

### v2.0 - 高级特性 (1 个月)
- ⏳ TLS 会话恢复检测
- ⏳ 0-RTT 数据检测
- ⏳ QUIC/TLS 支持
- ⏳ 机器学习匹配

---

**状态:** 📋 设计文档完成  
**下一步:** 实现 Phase 1 - TLS 记录解析器  
**预计时间:** 2-3 天  
**优先级:** P2 (中等)
