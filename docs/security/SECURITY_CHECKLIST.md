# 安全漏洞快速参考清单

快速查找和修复指南 - 按文件组织

---

## 🔴 高危漏洞 (立即修复)

### `crates/fingerprint-http/src/http_client/io.rs`

**问题 #1**: 缓冲区溢出 (CVSS 9.1)
```rust
// ❌ 问题代码 (Line 87)
target_len = Some(end.saturating_add(cl));

// ✅ 修复方案
const MAX_CONTENT_LENGTH: usize = 100 * 1024 * 1024;
if cl > MAX_CONTENT_LENGTH {
    return Err(io::Error::other("Content-Length 过大"));
}
```

---

### `crates/fingerprint-http/src/http_client/response.rs`

**问题 #2**: Chunked 解析漏洞 (CVSS 8.6)
```rust
// ❌ 问题代码 (Line 164)
let size = usize::from_str_radix(size_str, 16)?;

// ✅ 修复方案
const MAX_CHUNK_SIZE: usize = 10 * 1024 * 1024;
if size > MAX_CHUNK_SIZE {
    return Err(format!("Chunk 过大: {}", size));
}
```

---

### `crates/fingerprint-tls/src/tls_handshake/messages.rs`

**问题 #4**: 弱随机数 (CVSS 7.8)
```rust
// ❌ 问题代码 (Line 66-74)
#[cfg(not(feature = "crypto"))]
{
    // 使用 LCG - 不安全！
    hash = hash.wrapping_mul(1103515245).wrapping_add(12345);
}

// ✅ 修复方案
#[cfg(not(feature = "crypto"))]
{
    return Err("需要 crypto feature".to_string());
}
```

---

### `crates/fingerprint-dns/src/dns/ipinfo.rs`

**问题 #5**: Token 泄露 (CVSS 7.2)
```rust
// ❌ 问题代码 (Line 23)
let url = format!("https://ipinfo.io/{}?token={}", ip, self.token);

// ✅ 修复方案
let url = format!("https://ipinfo.io/{}", ip);
// 在 HTTP Header 中传递 token
request.headers.insert("Authorization", format!("Bearer {}", self.token));
```

---

## 🟡 中高危漏洞 (本周修复)

### `crates/fingerprint-dns/src/dns/serverpool.rs`

**问题 #3**: 锁中毒 (CVSS 7.5)
```rust
// ❌ 问题代码 (Line 107, 116, 130)
let mut stats = self.stats.write().unwrap();

// ✅ 修复方案
let mut stats = self.stats.write()
    .map_err(|e| format!("Lock poisoned: {}", e))?;
```

**问题 #8**: 资源耗尽 (CVSS 6.5)
```rust
// ❌ 问题代码 (Line 344-395)
let test_tasks = stream::iter(servers_to_test)  // 可能数万个
    .buffer_unordered(max_concurrency);

// ✅ 修复方案
const BATCH_SIZE: usize = 1000;
for chunk in servers_to_test.chunks(BATCH_SIZE) {
    // 分批处理
}
```

---

### `crates/fingerprint-http/src/http_client/mod.rs`

**问题 #6**: 重定向循环 (CVSS 6.8)
```rust
// ❌ 问题代码 (Line 258)
let mut redirect_request = request.clone();
redirect_request.url = location.clone();

// ✅ 修复方案
let mut visited_urls = HashSet::new();
if visited_urls.contains(&location) {
    return Err("重定向循环");
}
visited_urls.insert(location.clone());
```

---

## 🟢 中危漏洞 (本月修复)

### `crates/fingerprint-tls/src/tls_handshake/messages.rs`

**问题 #7**: 时间戳溢出 (CVSS 5.3)
```rust
// ❌ 问题代码 (Line 48-51)
.map(|d| d.as_secs() as u32)  // 2038 年溢出

// ✅ 修复方案
.map(|d| (d.as_secs() & 0xFFFFFFFF) as u32)  // 明确截断
```

---

### `crates/fingerprint-dns/src/dns/serverpool.rs`

**问题 #9**: 文件竞态 (CVSS 5.5)
```rust
// ❌ 问题代码 (Line 211-215)
let temp_path = path.with_extension("tmp");
fs::write(&temp_path, json_content)?;
fs::rename(&temp_path, path)?;

// ✅ 修复方案
let temp_path = path.with_extension(&format!("tmp.{}", std::process::id()));
// ... 添加错误处理和清理
```

---

### `crates/fingerprint-tls/src/tls_handshake/messages.rs`

**问题 #10**: Session ID 为空 (CVSS 4.8)
```rust
// ❌ 问题代码 (Line 78)
let session_id = Vec::new();  // 无法使用会话恢复

// ✅ 修复方案
// 实现 Session ID 缓存机制
```

---

## 📋 其他问题清单

### Cookie 管理 (`cookie.rs`)
- [ ] **#11**: 无大小限制 - 添加 `MAX_COOKIES` 常量
- [ ] **#28**: 无数量限制 - 添加 `MAX_COOKIES_PER_DOMAIN` 常量

### HTTP 解析 (`response.rs`, `io.rs`)
- [ ] **#12**: Header 无大小限制 - 添加 `MAX_HEADER_SIZE`
- [ ] **#18**: 压缩炸弹 - 添加解压后大小检查
- [ ] **#27**: Header 数量限制 - 添加 `MAX_HEADERS_COUNT`

### 扩展处理 (`messages.rs`)
- [ ] **#13**: 扩展读取错误 - 改进错误处理
- [ ] **#34**: 扩展长度验证 - 添加 `MAX_EXTENSION_LENGTH`

### IP 验证 (`serverpool.rs`)
- [ ] **#14**: IP 验证不完整 - 使用 `std::net::IpAddr`

### 错误处理 (多个文件)
- [ ] **#15**: JSON 错误泄露 - 清理错误消息
- [ ] **#23**: 调试输出泄露 - 使用条件编译

### 超时和限制 (多个文件)
- [ ] **#16**: DNS 查询超时 - 添加全局超时
- [ ] **#24**: 默认超时过长 - 调整默认值
- [ ] **#25**: 缺少速率限制 - 实施 token bucket
- [ ] **#26**: 请求大小限制 - 添加 `MAX_REQUEST_SIZE`

### 资源清理 (`http3.rs`, `http2.rs`)
- [ ] **#17**: HTTP/3 连接泄露 - 实现 Drop trait

### 指纹和隐私 (`useragent.rs`, `messages.rs`)
- [ ] **#19**: UA 可预测 - 增加随机性
- [ ] **#20**: 扩展顺序泄露 - 随机化顺序

### 输入验证 (多个文件)
- [ ] **#29**: 域名长度 - 添加 `MAX_DOMAIN_LENGTH`
- [ ] **#30**: 端口范围 - 验证 1-65535
- [ ] **#31**: 协议版本 - 验证有效版本
- [ ] **#32**: 密码套件 - 验证支持的套件
- [ ] **#33**: 压缩方法 - 验证支持的方法
- [ ] **#35**: GREASE 值 - 验证范围

---

## 🔧 快速修复命令

### 1. 查找所有 unwrap()
```bash
grep -rn "\.unwrap()" crates/ --include="*.rs" | grep -v "test"
```

### 2. 查找所有 expect()
```bash
grep -rn "\.expect(" crates/ --include="*.rs" | grep -v "test"
```

### 3. 查找所有 panic!
```bash
grep -rn "panic!" crates/ --include="*.rs" | grep -v "test"
```

### 4. 查找所有 unsafe
```bash
grep -rn "unsafe" crates/ --include="*.rs"
```

### 5. 运行安全检查
```bash
cargo audit
cargo clippy -- -W clippy::all -W clippy::pedantic
cargo deny check
```

---

## 📊 修复进度跟踪

### P0 (立即) - 4 个
- [ ] #1 - 缓冲区溢出 (io.rs)
- [ ] #2 - Chunked 解析 (response.rs)
- [ ] #4 - 弱随机数 (messages.rs)
- [ ] #5 - Token 泄露 (ipinfo.rs)

### P1 (本周) - 3 个
- [ ] #3 - 锁中毒 (serverpool.rs)
- [ ] #6 - 重定向循环 (mod.rs)
- [ ] #8 - 资源耗尽 (serverpool.rs)

### P2 (本月) - 10 个
- [ ] #7 - 时间戳溢出
- [ ] #9 - 文件竞态
- [ ] #10 - Session ID
- [ ] #11 - Cookie 大小
- [ ] #12 - Header 大小
- [ ] #13 - 扩展错误
- [ ] #14 - IP 验证
- [ ] #15 - 错误泄露
- [ ] #16 - DNS 超时
- [ ] #18 - 压缩炸弹

### P3 (长期) - 18 个
- [ ] #17, #19-35

---

## 🎯 每日检查清单

### 开发前
- [ ] 拉取最新代码
- [ ] 检查安全公告
- [ ] 更新依赖

### 开发中
- [ ] 避免使用 `unwrap()`
- [ ] 验证所有输入
- [ ] 设置资源限制
- [ ] 使用安全的随机数

### 提交前
- [ ] 运行 `cargo test`
- [ ] 运行 `cargo clippy`
- [ ] 运行 `cargo audit`
- [ ] 代码审查

---

**最后更新**: 2025-12-29  
**下次审查**: 每周一
