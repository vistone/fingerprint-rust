# HTTP/2 SETTINGS 分析增强

## 概述

为 PCAP 分析器添加 HTTP/2 SETTINGS 帧解析功能，通过分析 SETTINGS 值和顺序来识别浏览器指纹。

## HTTP/2 SETTINGS 背景

### SETTINGS 帧格式

```
+---------------+
|Pad Length? (8)|
+-+-------------+-----------------------------------------------+
|E|                 Stream Dependency? (31)                     |
+-+-------------+-----------------------------------------------+
|  Weight? (8)  |
+-+-------------+-----------------------------------------------+
|                   Header Block Fragment (*)                 ...
+---------------------------------------------------------------+
|                           Padding (*)                       ...
+---------------------------------------------------------------+

SETTINGS Frame:
+-------------------------------+
|       Identifier (16)         |
+-------------------------------+-------------------------------+
|                        Value (32)                             |
+---------------------------------------------------------------+
```

### 关键 SETTINGS 参数

| ID | 名称 | Chrome | Firefox | Safari | 说明 |
|----|------|--------|---------|--------|------|
| 1 | HEADER_TABLE_SIZE | 65536 | 65536 | 65536 | HPACK 表大小 |
| 2 | ENABLE_PUSH | 0 | 0 | 1 | 服务器推送 |
| 3 | MAX_CONCURRENT_STREAMS | 1000 | 1000 | 100 | 最大并发流 |
| 4 | INITIAL_WINDOW_SIZE | 6291456 | 131072 | 2097152 | 初始窗口大小 ⭐ |
| 5 | MAX_FRAME_SIZE | 16384 | 16384 | 16384 | 最大帧大小 |
| 6 | MAX_HEADER_LIST_SIZE | 262144 | 262144 | - | 最大头列表大小 |

**关键差异:**
- **INITIAL_WINDOW_SIZE** - Chrome (6291456) vs Firefox (131072) vs Safari (2097152)
- **ENABLE_PUSH** - Chrome/Firefox (0) vs Safari (1)
- **MAX_CONCURRENT_STREAMS** - Chrome/Firefox (1000) vs Safari (100)

## 实现设计

### Phase 1: HTTP/2 帧解析器 ✅ (部分实现)

**文件:** `crates/fingerprint-core/src/http2_frame_parser.rs` (NEW)

```rust
/// HTTP/2 帧类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Http2FrameType {
    Data = 0x0,
    Headers = 0x1,
    Priority = 0x2,
    RstStream = 0x3,
    Settings = 0x4,  // ← 我们关注的
    PushPromise = 0x5,
    Ping = 0x6,
    GoAway = 0x7,
    WindowUpdate = 0x8,
    Continuation = 0x9,
}

/// HTTP/2 帧头 (9 bytes)
pub struct Http2FrameHeader {
    pub length: u32,       // 24 bits
    pub frame_type: u8,    // 8 bits
    pub flags: u8,         // 8 bits
    pub stream_id: u32,    // 31 bits (1 bit reserved)
}

impl Http2FrameHeader {
    pub fn parse(data: &[u8]) -> Result<Self, Http2ParseError> {
        if data.len() < 9 {
            return Err(Http2ParseError::TooShort);
        }
        
        // Length (24 bits)
        let length = u32::from_be_bytes([0, data[0], data[1], data[2]]);
        
        // Type, Flags (8 bits each)
        let frame_type = data[3];
        let flags = data[4];
        
        // Stream ID (31 bits, highest bit reserved)
        let stream_id = u32::from_be_bytes([data[5], data[6], data[7], data[8]]) & 0x7FFFFFFF;
        
        Ok(Http2FrameHeader {
            length,
            frame_type,
            flags,
            stream_id,
        })
    }
    
    pub fn is_settings(&self) -> bool {
        self.frame_type == Http2FrameType::Settings as u8
    }
}

/// HTTP/2 SETTINGS 帧
pub struct Http2SettingsFrame {
    pub header: Http2FrameHeader,
    pub settings: Vec<(u16, u32)>,  // (identifier, value)
}

impl Http2SettingsFrame {
    pub fn parse(data: &[u8]) -> Result<Self, Http2ParseError> {
        // 解析帧头
        let header = Http2FrameHeader::parse(data)?;
        
        if !header.is_settings() {
            return Err(Http2ParseError::NotSettingsFrame);
        }
        
        // 解析 SETTINGS 参数 (每个 6 bytes)
        let payload = &data[9..9 + header.length as usize];
        let mut settings = Vec::new();
        
        for chunk in payload.chunks_exact(6) {
            let identifier = u16::from_be_bytes([chunk[0], chunk[1]]);
            let value = u32::from_be_bytes([chunk[2], chunk[3], chunk[4], chunk[5]]);
            settings.push((identifier, value));
        }
        
        Ok(Http2SettingsFrame {
            header,
            settings,
        })
    }
    
    /// 转换为 HashMap
    pub fn to_map(&self) -> HashMap<u16, u32> {
        self.settings.iter().cloned().collect()
    }
    
    /// 获取 SETTINGS 顺序
    pub fn get_order(&self) -> Vec<u16> {
        self.settings.iter().map(|(id, _)| *id).collect()
    }
}
```

### Phase 2: HTTP/2 连接检测 ✅

```rust
/// 检测 HTTP/2 连接 Preface (magic string)
pub fn is_http2_connection(data: &[u8]) -> bool {
    const HTTP2_PREFACE: &[u8] = b"PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n";
    
    data.len() >= HTTP2_PREFACE.len() && data.starts_with(HTTP2_PREFACE)
}

/// 从 TCP payload 中查找 SETTINGS 帧
pub fn find_settings_frame(data: &[u8]) -> Option<Http2SettingsFrame> {
    // 跳过 HTTP/2 Preface (24 bytes)
    let offset = if is_http2_connection(data) {
        24
    } else {
        0
    };
    
    let mut pos = offset;
    
    // 扫描帧查找 SETTINGS
    while pos + 9 <= data.len() {
        if let Ok(header) = Http2FrameHeader::parse(&data[pos..]) {
            if header.is_settings() && pos + 9 + header.length as usize <= data.len() {
                return Http2SettingsFrame::parse(&data[pos..]).ok();
            }
            pos += 9 + header.length as usize;
        } else {
            break;
        }
    }
    
    None
}
```

### Phase 3: 浏览器指纹匹配 ⏳

```rust
use fingerprint_headers::http2_config::*;

#[derive(Debug, Clone, PartialEq)]
pub enum BrowserType {
    Chrome,
    Firefox,
    Safari,
    Edge,
    Opera,
    Unknown,
}

/// HTTP/2 SETTINGS 指纹匹配器
pub struct Http2SettingsMatcher {
    chrome_settings: HashMap<u16, u32>,
    firefox_settings: HashMap<u16, u32>,
    safari_settings: HashMap<u16, u32>,
}

impl Http2SettingsMatcher {
    pub fn new() -> Self {
        let (chrome_settings, _) = chrome_http2_settings();
        let (firefox_settings, _) = firefox_http2_settings();
        let (safari_settings, _) = safari_http2_settings();
        
        Self {
            chrome_settings,
            firefox_settings,
            safari_settings,
        }
    }
    
    /// 匹配浏览器类型
    pub fn match_browser(&self, settings: &HashMap<u16, u32>) -> (BrowserType, f64) {
        let chrome_score = self.calculate_similarity(settings, &self.chrome_settings);
        let firefox_score = self.calculate_similarity(settings, &self.firefox_settings);
        let safari_score = self.calculate_similarity(settings, &self.safari_settings);
        
        let max_score = chrome_score.max(firefox_score).max(safari_score);
        
        if max_score < 0.70 {
            return (BrowserType::Unknown, max_score);
        }
        
        if chrome_score == max_score {
            (BrowserType::Chrome, chrome_score)
        } else if firefox_score == max_score {
            (BrowserType::Firefox, firefox_score)
        } else {
            (BrowserType::Safari, safari_score)
        }
    }
    
    /// 计算相似度 (0.0 - 1.0)
    fn calculate_similarity(&self, a: &HashMap<u16, u32>, b: &HashMap<u16, u32>) -> f64 {
        if a.is_empty() || b.is_empty() {
            return 0.0;
        }
        
        let mut matched = 0;
        let mut total = 0;
        
        // 对于 b 中的每个设置
        for (key, expected_value) in b {
            total += 1;
            if let Some(actual_value) = a.get(key) {
                if actual_value == expected_value {
                    matched += 1;
                } else {
                    // 部分匹配 (INITIAL_WINDOW_SIZE 可能有多个变体)
                    if *key == 4 && is_valid_window_size(*actual_value) {
                        matched += 1;  // 宽松匹配
                    }
                }
            }
        }
        
        matched as f64 / total as f64
    }
}

/// 检查窗口大小是否合理
fn is_valid_window_size(size: u32) -> bool {
    // 常见窗口大小: 65535 (default), 131072 (128KB), 6291456 (6MB), 2097152 (2MB)
    matches!(size, 65535 | 131072 | 262144 | 524288 | 1048576 | 2097152 | 4194304 | 6291456)
}
```

### Phase 4: 集成到 PCAP 分析器 ⏳

**文件:** `crates/fingerprint/src/bin/fingerprint_analyze.rs` (增强)

```rust
use fingerprint_core::{Http2SettingsFrame, Http2SettingsMatcher, BrowserType};

struct BrowserFingerprint {
    // 现有字段
    window_size: Option<u16>,
    ttl: Option<u8>,
    packet_count: usize,
    confidence: f64,
    
    // 新增 HTTP/2 字段
    http2_settings: Option<HashMap<u16, u32>>,
    http2_browser: Option<BrowserType>,
    http2_confidence: Option<f64>,
}

fn analyze_pcap(path: &Path) -> Result<BrowserFingerprint, Box<dyn Error>> {
    // ... 现有 TCP 解析 ...
    
    // 新增 HTTP/2 SETTINGS 解析
    let matcher = Http2SettingsMatcher::new();
    
    for packet in &packets {
        if let Some(tcp_payload) = packet.tcp_payload() {
            // 查找 SETTINGS 帧
            if let Some(settings_frame) = find_settings_frame(tcp_payload) {
                let settings = settings_frame.to_map();
                let (browser, conf) = matcher.match_browser(&settings);
                
                fp.http2_settings = Some(settings);
                fp.http2_browser = Some(browser);
                fp.http2_confidence = Some(conf);
                
                // 根据 HTTP/2 指纹调整总置信度
                if conf >= 0.90 {
                    fp.confidence += 0.15;  // 高置信度 HTTP/2 匹配
                } else if conf >= 0.75 {
                    fp.confidence += 0.10;
                }
                
                break;  // 只需要第一个 SETTINGS 帧
            }
        }
    }
    
    fp.confidence = fp.confidence.min(1.0);
    Ok(fp)
}

fn print_fingerprint_report(filename: &str, fp: &BrowserFingerprint) {
    // ... 现有 TCP 输出 ...
    
    // 新增 HTTP/2 输出
    if let Some(settings) = &fp.http2_settings {
        println!("\n  HTTP/2 SETTINGS:");
        
        // 显示关键设置
        if let Some(window_size) = settings.get(&4) {
            println!("    Initial Window Size: {} bytes ({} KB)", 
                window_size, window_size / 1024);
        }
        if let Some(max_conc) = settings.get(&3) {
            println!("    Max Concurrent Streams: {}", max_conc);
        }
        if let Some(enable_push) = settings.get(&2) {
            println!("    Server Push: {}", if *enable_push == 1 { "Enabled" } else { "Disabled" });
        }
        
        // 显示匹配结果
        if let (Some(browser), Some(conf)) = (&fp.http2_browser, fp.http2_confidence) {
            println!("    Detected Browser: {:?} ({:.1}% confidence)", browser, conf * 100.0);
        }
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
    fn test_parse_settings_frame() {
        // Chrome SETTINGS 帧
        let data = vec![
            // Frame Header (9 bytes)
            0x00, 0x00, 0x24,       // Length: 36 (6 settings × 6 bytes)
            0x04,                   // Type: SETTINGS
            0x00,                   // Flags: none
            0x00, 0x00, 0x00, 0x00, // Stream ID: 0
            // SETTINGS Payload (36 bytes)
            0x00, 0x01, 0x00, 0x01, 0x00, 0x00,  // HEADER_TABLE_SIZE: 65536
            0x00, 0x02, 0x00, 0x00, 0x00, 0x00,  // ENABLE_PUSH: 0
            // ... 其他 settings
        ];
        
        let frame = Http2SettingsFrame::parse(&data).unwrap();
        assert_eq!(frame.settings.len(), 6);
        assert_eq!(frame.settings[0].0, 1);  // HEADER_TABLE_SIZE
        assert_eq!(frame.settings[1].0, 2);  // ENABLE_PUSH
    }
    
    #[test]
    fn test_match_chrome() {
        let matcher = Http2SettingsMatcher::new();
        let (chrome_settings, _) = chrome_http2_settings();
        
        let (browser, confidence) = matcher.match_browser(&chrome_settings);
        assert_eq!(browser, BrowserType::Chrome);
        assert!(confidence >= 0.95);
    }
    
    #[test]
    fn test_match_firefox() {
        let matcher = Http2SettingsMatcher::new();
        let (firefox_settings, _) = firefox_http2_settings();
        
        let (browser, confidence) = matcher.match_browser(&firefox_settings);
        assert_eq!(browser, BrowserType::Firefox);
        assert!(confidence >= 0.95);
    }
}
```

### 集成测试

```rust
#[test]
fn test_pcap_with_http2() {
    let pcap_path = Path::new("test_data/pcap/chrome_136.pcap");
    let fp = analyze_pcap(pcap_path).unwrap();
    
    // 验证 HTTP/2 SETTINGS 被提取
    assert!(fp.http2_settings.is_some());
    assert_eq!(fp.http2_browser, Some(BrowserType::Chrome));
    assert!(fp.http2_confidence.unwrap() > 0.85);
}
```

## 关键特征差异表

### Chrome vs Firefox vs Safari

| 特征 | Chrome 136 | Firefox 145 | Safari 18 | 识别能力 |
|------|-----------|-------------|-----------|----------|
| **INITIAL_WINDOW_SIZE** | 6291456 (6MB) | 131072 (128KB) | 2097152 (2MB) | ⭐⭐⭐⭐⭐ |
| **ENABLE_PUSH** | 0 (disabled) | 0 (disabled) | 1 (enabled) | ⭐⭐⭐ |
| **MAX_CONCURRENT_STREAMS** | 1000 | 1000 | 100 | ⭐⭐⭐ |
| **SETTINGS 顺序** | 固定 | 固定 | 固定 | ⭐⭐ |

**最具区分度:** INITIAL_WINDOW_SIZE (每个浏览器都不同)

## 性能考虑

### 优化策略

1. **早期过滤**
   ```rust
   // 只检查 HTTPS 端口 (443)
   if tcp.dst_port() != 443 && tcp.src_port() != 443 {
       continue;
   }
   ```

2. **缓存匹配器**
   ```rust
   lazy_static! {
       static ref MATCHER: Http2SettingsMatcher = Http2SettingsMatcher::new();
   }
   ```

3. **单次解析**
   ```rust
   // 找到第一个 SETTINGS 帧后立即停止
   if found_settings {
       break;
   }
   ```

### 性能目标

| 指标 | 目标 | 预期影响 |
|------|------|----------|
| HTTP/2 检测延迟 | <5μs/包 | 0.5% |
| SETTINGS 解析 | <1μs | 0.1% |
| 匹配计算 | <10μs | 1% |
| 总开销 | <2% | 可接受 |

## 错误处理

```rust
#[derive(Debug)]
pub enum Http2ParseError {
    TooShort,
    InvalidFrameType,
    NotSettingsFrame,
    IncompletePayload,
    InvalidSettingID,
}

impl std::fmt::Display for Http2ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Http2ParseError::TooShort => write!(f, "Data too short for HTTP/2 frame"),
            Http2ParseError::NotSettingsFrame => write!(f, "Not a SETTINGS frame"),
            // ...
        }
    }
}

impl std::error::Error for Http2ParseError {}
```

## 真实数据示例

### Chrome 136 SETTINGS

```
HEADER_TABLE_SIZE: 65536        (64 KB)
ENABLE_PUSH: 0                  (disabled)
MAX_CONCURRENT_STREAMS: 1000
INITIAL_WINDOW_SIZE: 6291456    (6 MB) ⭐
MAX_FRAME_SIZE: 16384           (16 KB)
MAX_HEADER_LIST_SIZE: 262144    (256 KB)
```

### Firefox 145 SETTINGS

```
HEADER_TABLE_SIZE: 65536        (64 KB)
ENABLE_PUSH: 0                  (disabled)
MAX_CONCURRENT_STREAMS: 1000
INITIAL_WINDOW_SIZE: 131072     (128 KB) ⭐
MAX_FRAME_SIZE: 16384           (16 KB)
MAX_HEADER_LIST_SIZE: 262144    (256 KB)
```

### Safari 18 SETTINGS

```
HEADER_TABLE_SIZE: 65536        (64 KB)
ENABLE_PUSH: 1                  (enabled) ⭐
MAX_CONCURRENT_STREAMS: 100     ⭐
INITIAL_WINDOW_SIZE: 2097152    (2 MB) ⭐
MAX_FRAME_SIZE: 16384           (16 KB)
```

## 路线图

### v1.0 - 基础 HTTP/2 检测 (当前)
- ✅ HTTP/2 SETTINGS 定义 (已有)
- ⏳ SETTINGS 帧解析器
- ⏳ 浏览器匹配算法

### v1.1 - 完整集成 (1 周)
- ⏳ 集成到 PCAP 分析器
- ⏳ HTTP/2 置信度计算
- ⏳ 测试覆盖

### v1.2 - 高级特性 (2 周)
- ⏳ WINDOW_UPDATE 帧分析
- ⏳ PRIORITY 帧分析
- ⏳ HPACK 头压缩分析

### v2.0 - HTTP/3 QUIC 支持 (1 个月)
- ⏳ QUIC SETTINGS 解析
- ⏳ QPACK 分析
- ⏳ 0-RTT 检测

---

**状态:** 📋 设计文档完成  
**下一步:** 实现 HTTP/2 帧解析器  
**预计时间:** 1-2 天  
**优先级:** P2 (中等 - 实用价值高)
