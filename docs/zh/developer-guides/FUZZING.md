# fingerprint-rust 模糊测试指南

**版本**: v1.0  
**最后更新**: 2026-02-13  
**文档类型**: 技术文档

---

本指南介绍如何对 fingerprint-rust 项目进行模糊测试，以发现潜在的安全漏洞。

## 概述

模糊测试（Fuzzing）是一种自动化软件测试技术，它向程序提供无效、异常或随机数据作为输入，并监控程序是否出现崩溃、断言失败或潜在内存泄漏等异常。

## 前置条件

安装 cargo-fuzz：

```bash
cargo install cargo-fuzz
```

## 模糊测试目标

### 1. 数据包解析

**目标**: IPv4/IPv6 和 TCP/UDP 数据包解析  
**位置**: `crates/fingerprint-defense/src/passive/packet.rs`  
**风险**: 缓冲区溢出、整数溢出、panic

```bash
# Create fuzz target
cargo fuzz init

# Add to fuzz/Cargo.toml
[[bin]]
name = "fuzz_packet_parsing"
path = "fuzz_targets/fuzz_packet_parsing.rs"

# Run fuzzing
cargo fuzz run fuzz_packet_parsing
```

示例目标（`fuzz/fuzz_targets/fuzz_packet_parsing.rs`）：

```rust
#![no_main]
use libfuzzer_sys::fuzz_target;
use fingerprint_defense::passive::packet::PacketParser;

fuzz_target!(|data: &[u8]| {
    // Attempt to parse as IPv4 packet
    let _ = PacketParser::parse_ipv4(data);
    
    // Attempt to parse as TCP packet
    if data.len() >= 20 {
        let _ = PacketParser::parse_tcp(data);
    }
});
```

### 2. TLS ClientHello 解析

**目标**: TLS ClientHello 解析与指纹生成  
**位置**: `crates/fingerprint-tls/src/`  
**风险**: 解析错误、扩展字段异常、无效密码套件

```bash
cargo fuzz run fuzz_tls_parsing
```

示例目标：

```rust
#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Parse TLS ClientHello
    // Implementation depends on your parsing code
});
```

### 3. HTTP 头部解析

**目标**: HTTP 头部解析  
**位置**: `crates/fingerprint-http/src/`  
**风险**: 头部注入、解析错误、缓冲区溢出

```bash
cargo fuzz run fuzz_http_headers
```

### 4. DNS 响应解析

**目标**: DNS 响应解析  
**位置**: `crates/fingerprint-dns/src/`  
**风险**: 伪造 DNS 响应、整数溢出

```bash
cargo fuzz run fuzz_dns_parsing
```

## 运行模糊测试

### 持续运行

长时间连续运行模糊测试：

```bash
# Run for 1 hour
cargo fuzz run fuzz_packet_parsing -- -max_total_time=3600

# Run with specific number of workers
cargo fuzz run fuzz_packet_parsing -- -workers=4

# Run with memory limit (2GB)
cargo fuzz run fuzz_packet_parsing -- -rss_limit_mb=2048
```

### 最小化崩溃用例

如果发现崩溃，先最小化输入：

```bash
cargo fuzz cmin fuzz_packet_parsing
cargo fuzz tmin fuzz_packet_parsing fuzz/artifacts/crash-file
```

### 代码覆盖率

生成覆盖率报告：

```bash
cargo fuzz coverage fuzz_packet_parsing
```

## 最佳实践

### 1. 字典文件

为提高效率创建字典文件：

```
# fuzz/dictionaries/packet.dict
"GET "
"POST "
"HTTP/1.1"
"\r\n"
"Content-Length: "
```

使用方式：

```bash
cargo fuzz run fuzz_http_headers -- -dict=fuzz/dictionaries/http.dict
```

### 2. 语料库管理

维护一份有意义的输入语料库：

```bash
# Add valid test cases to corpus
cp test_data/*.bin fuzz/corpus/fuzz_packet_parsing/
```

### 3. 结构化模糊测试

对复杂输入使用结构化模糊测试：

```rust
use arbitrary::Arbitrary;

#[derive(Arbitrary, Debug)]
struct FuzzInput {
    packet_type: u8,
    flags: u16,
    data: Vec<u8>,
}

fuzz_target!(|input: FuzzInput| {
    // Use structured input
});
```

## 与 CI/CD 集成

将模糊测试加入 CI 管道：

```yaml
# .github/workflows/fuzz.yml
name: Fuzzing

on:
  schedule:
    - cron: '0 2 * * *'  # Run nightly

jobs:
  fuzz:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: dtolnay/rust-toolchain@nightly
      - run: cargo install cargo-fuzz
      - run: |
          for target in $(cargo fuzz list); do
            cargo fuzz run $target -- -max_total_time=300
          done
```

## 崩溃处理

发现崩溃时：

1. **最小化输入**：
   ```bash
   cargo fuzz tmin fuzz_packet_parsing artifacts/crash-file
   ```

2. **本地复现**：
   ```bash
   cargo fuzz run fuzz_packet_parsing artifacts/minimized-crash
   ```

3. **创建回归测试**：
   ```rust
   #[test]
   fn test_fuzz_crash() {
       let data = include_bytes!("../fuzz/artifacts/minimized-crash");
       // Verify fix prevents crash
   }
   ```

4. **修复问题**

5. **重新运行模糊测试**，确保修复完成

## 性能建议

### 1. 使用 Release 模式

```bash
cargo fuzz run --release fuzz_packet_parsing
```

### 2. 启用 Sanitizer

```bash
# Address Sanitizer
RUSTFLAGS="-Zsanitizer=address" cargo fuzz run fuzz_packet_parsing

# Memory Sanitizer
RUSTFLAGS="-Zsanitizer=memory" cargo fuzz run fuzz_packet_parsing
```

### 3. 并行模糊测试

```bash
# Run multiple instances
for i in {1..4}; do
  cargo fuzz run fuzz_packet_parsing -- -workers=1 &
done
```

## 预期结果

高质量的模糊测试应当：
- 长时间运行（数小时/数天）无崩溃
- 达到较高的代码覆盖率（>80%）
- 捕捉解析逻辑的边界情况
- 覆盖错误处理路径

## 参考资料

- [cargo-fuzz documentation](https://rust-fuzz.github.io/book/cargo-fuzz.html)
- [libFuzzer documentation](https://llvm.org/docs/LibFuzzer.html)
- [Rust Fuzzing Authority](https://github.com/rust-fuzz)

## 安全披露

如果模糊测试发现安全漏洞：
1. 不要立即公开披露
2. 通过 GitHub Security Advisories 报告给维护者
3. 为修复留出合理时间
4. 协调公开披露

## 维护建议

- 每周复查模糊测试结果
- 用新的有效输入更新语料库
- 扩展覆盖到新增代码路径
- 重大变更后重新运行模糊测试

**最后更新**: 2026-01-06
