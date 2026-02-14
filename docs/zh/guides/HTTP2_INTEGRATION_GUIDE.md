# HTTP/2 SETTINGS 解析器集成指南

## 概述

本文档说明如何在 `fingerprint_analyze` 中使用新的 HTTP/2 SETTINGS 解析器。

## 实现功能

✅ **HTTP/2 帧解析器** (`http2_frame_parser.rs`)
- HTTP/2 帧头解析
- SETTINGS 帧提取
- 连接 preface 检测
- 自动扫描 TCP payload

✅ **浏览器指纹匹配器** (`Http2SettingsMatcher`)
- 基于 INITIAL_WINDOW_SIZE 区分浏览器
  - Chrome: 6291456 (6MB) - 95% 置信度
  - Firefox: 131072 (128KB) - 95% 置信度
  - Safari: 2097152 (2MB) - 95% 置信度
- 完整 SETTINGS 相似度计算

✅ **测试覆盖**
- 8 个单元测试全部通过
- 帧解析测试
- 浏览器匹配测试
- HTTP/2 preface 检测测试

## 使用示例 (Usage Examples)

### 基本用法

```rust
use fingerprint_core::{
    find_settings_frame, Http2SettingsMatcher,
};

// 从 TCP payload 中查找 SETTINGS 帧
let tcp_payload: &[u8] = /* ... */;

if let Some(settings_frame) = find_settings_frame(tcp_payload) {
    // 转换为 HashMap
    let settings = settings_frame.to_map();
    
    // 匹配浏览器
    let matcher = Http2SettingsMatcher::new();
    let (browser, confidence) = matcher.match_browser(&settings);
    
    println!("检测到浏览器: {} (置信度: {:.1}%)", 
             browser, confidence * 100.0);
    
    // 查看关键设置
    if let Some(&window_size) = settings.get(&4) {
        println!("INITIAL_WINDOW_SIZE: {} bytes", window_size);
    }
}
```

### 完整集成示例（fingerprint_analyze）

```rust
// 在 fingerprint_analyze.rs 中添加

use fingerprint_core::{
    find_settings_frame, Http2SettingsMatcher,
    http2_frame_parser::BrowserType,
};

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

fn analyze_pcap_with_http2(path: &Path) -> Result<BrowserFingerprint> {
    let mut fp = BrowserFingerprint::default();
    let matcher = Http2SettingsMatcher::new();
    
    // 读取 PCAP
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    
    // 跳过全局头
    let mut header_buf = [0u8; 24];
    reader.read_exact(&mut header_buf)?;
    
    // 解析每个包
    loop {
        // 读取包头（16 bytes: ts_sec, ts_usec, incl_len, orig_len）
        let mut pkt_header = [0u8; 16];
        if reader.read_exact(&mut pkt_header).is_err() {
            break;
        }
        
        let incl_len = u32::from_le_bytes([
            pkt_header[8], pkt_header[9], pkt_header[10], pkt_header[11]
        ]) as usize;
        
        // 读取包数据
        let mut pkt_data = vec![0u8; incl_len];
        reader.read_exact(&mut pkt_data)?;
        
        fp.packet_count += 1;
        
        // 解析 Ethernet + IP + TCP
        if incl_len < 54 { continue; }  // 最小 TCP 包大小
        
        // 跳过 Ethernet (14) + IP header (variable) + TCP header (variable)
        // 简化版本：假设 IP header = 20, TCP header = 20
        let tcp_payload_offset = 14 + 20 + 20;
        if incl_len <= tcp_payload_offset { continue; }
        
        let tcp_payload = &pkt_data[tcp_payload_offset..];
        
        // 尝试查找 HTTP/2 SETTINGS
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
                fp.confidence += 0.10;  // 中等置信度
            }
            
            break;  // 只需要第一个 SETTINGS 帧
        }
    }
    
    fp.confidence = fp.confidence.min(1.0);
    Ok(fp)
}

fn print_http2_info(fp: &BrowserFingerprint) {
    if let Some(settings) = &fp.http2_settings {
        println!("\n  HTTP/2 SETTINGS:");
        
        // 显示关键设置
        if let Some(&window_size) = settings.get(&4) {
            println!("    Initial Window Size: {} bytes ({} KB)", 
                     window_size, window_size / 1024);
        }
        if let Some(&max_conc) = settings.get(&3) {
            println!("    Max Concurrent Streams: {}", max_conc);
        }
        if let Some(&enable_push) = settings.get(&2) {
            println!("    Server Push: {}", 
                     if enable_push == 1 { "Enabled" } else { "Disabled" });
        }
        
        // 显示匹配结果
        if let (Some(browser), Some(conf)) = (&fp.http2_browser, fp.http2_confidence) {
            println!("    HTTP/2 Browser: {:?} ({:.1}% confidence)", 
                     browser, conf * 100.0);
        }
    }
}
```

## 预期效果

### Chrome 136 分析

```
📁 Analyzing: chrome_136.pcap
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  Browser: Chrome
  Packets: 432560
  Window Size: 16433
  TTL: 6
  
  HTTP/2 SETTINGS:
    Initial Window Size: 6291456 bytes (6144 KB)
    Max Concurrent Streams: 1000
    Server Push: Disabled
    HTTP/2 Browser: Chrome (95.0% confidence)
  
  Confidence: 85.0%  ← 70% (TCP) + 15% (HTTP/2)
  Status: ✓ GOOD
```

### Firefox 145 分析（预期）

```
📁 Analyzing: firefox_145.pcap
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  Browser: Firefox
  Packets: 180234
  Window Size: 65535
  TTL: 64
  
  HTTP/2 SETTINGS:
    Initial Window Size: 131072 bytes (128 KB)
    Max Concurrent Streams: 1000
    Server Push: Disabled
    HTTP/2 Browser: Firefox (95.0% confidence)
  
  Confidence: 90.0%  ← 75% (TCP) + 15% (HTTP/2)
  Status: ✓ EXCELLENT
```

## HTTP/2 SETTINGS 参数详解

| ID | 参数名                | Chrome   | Firefox | Safari  | 说明            |
|----|----------------------|----------|---------|---------|-----------------|
| 1  | HEADER_TABLE_SIZE    | 65536    | 65536   | 65536   | HPACK 表大小    |
| 2  | ENABLE_PUSH          | 0        | 0       | 1       | 服务器推送      |
| 3  | MAX_CONCURRENT_STREAMS| 1000    | 1000    | 100     | 最大并发流      |
| 4  | INITIAL_WINDOW_SIZE  | 6291456  | 131072  | 2097152 | 初始窗口大小 ⭐ |
| 5  | MAX_FRAME_SIZE       | 16384    | 16384   | 16384   | 最大帧大小      |
| 6  | MAX_HEADER_LIST_SIZE | 262144   | 262144  | -       | 最大头列表大小  |

**关键发现:**
- **INITIAL_WINDOW_SIZE** 是最强的浏览器区分特征
- Chrome 使用 6MB（激进策略，提高性能）
- Firefox 使用 128KB（保守策略，节省内存）
- Safari 使用 2MB（中间策略）

## 置信度计算

### 基础置信度（TCP 层）

```rust
let mut confidence = 0.0;

// Window Size (0.25)
if window_size_matches { confidence += 0.25; }

// TTL (0.10 - 0.25)
confidence += ttl_score();  // 根据 TTL 范围

// OS Fingerprint (0.20)
if os_matches { confidence += 0.20; }

// Total: ~0.60 - 0.70
```

### HTTP/2 增强（+0.10 - 0.15）

```rust
// HTTP/2 SETTINGS 匹配
if let Some(http2_conf) = http2_confidence {
    if http2_conf >= 0.90 {
        confidence += 0.15;  // 高置信度
    } else if http2_conf >= 0.75 {
        confidence += 0.10;  // 中等置信度
    }
}

// Final: 0.70 - 0.85 (更高的准确性)
```

## 置信度等级

| 置信度范围 | 等级      | 描述                           |
|-----------|-----------|--------------------------------|
| ≥ 90%     | EXCELLENT | 非常高的置信度，几乎确定       |
| 80-89%    | GOOD      | 高置信度，可靠的识别           |
| 70-79%    | FAIR      | 中等置信度，可能正确           |
| 60-69%    | LOW       | 低置信度，不太可靠             |
| < 60%     | POOR      | 很低的置信度，可能不准确       |

## 测试结果

```bash
$ cargo test --package fingerprint-core --lib http2_frame_parser

running 8 tests
test http2_frame_parser::tests::test_find_settings_frame ... ok
test http2_frame_parser::tests::test_http2_preface ... ok
test http2_frame_parser::tests::test_match_chrome ... ok
test http2_frame_parser::tests::test_match_firefox ... ok
test http2_frame_parser::tests::test_match_safari ... ok
test http2_frame_parser::tests::test_parse_frame_header ... ok
test http2_frame_parser::tests::test_match_unknown ... ok
test http2_frame_parser::tests::test_parse_settings_frame ... ok

test result: ok. 8 passed; 0 failed; 0 ignored
```

## 性能影响

- **解析开销**: 极小（仅扫描 TCP payload 前几个帧）
- **内存占用**: 最小（只保存一个 HashMap<u16, u32>）
- **匹配速度**: 纳秒级（简单整数比较）

## 下一步

1. ✅ HTTP/2 Frame Parser 实现（完成）
2. ✅ 浏览器指纹匹配器（完成）
3. ✅ 单元测试（8 个测试通过）
4. ⏳ 集成到 `fingerprint_analyze.rs`（待实现）
5. ⏳ 真实流量验证（Chrome + Firefox）
6. ⏳ 文档和示例更新

## 相关文件

- 实现: [`crates/fingerprint-core/src/http2_frame_parser.rs`](../../crates/fingerprint-core/src/http2_frame_parser.rs)
- 设计: [`docs/HTTP2_SETTINGS_ANALYSIS_DESIGN.md`](HTTP2_SETTINGS_ANALYSIS_DESIGN.md)
- 集成示例: [`examples/http2_analysis.rs`](../../examples/http2_analysis.rs) (待创建)

## 参考资料

- [RFC 7540 - HTTP/2](https://datatracker.ietf.org/doc/html/rfc7540)
- [RFC 7541 - HPACK](https://datatracker.ietf.org/doc/html/rfc7541)
- [HTTP/2 Frame Format](https://httpwg.org/specs/rfc7540.html#FrameHeader)
- [HTTP/2 SETTINGS](https://httpwg.org/specs/rfc7540.html#SETTINGS)
