# Google Earth API 测试状态

## 测试目标
验证 `https://kh.google.com/rt/earth/PlanetoidMetadata` 在所有 HTTP 协议版本下都能正常工作：
- HTTP/1.1
- HTTP/2
- HTTP/3

## 当前状态

### ✅ 成功的测试

1. **reqwest 测试** - 使用 `reqwest` 客户端可以成功访问
   - 状态码: 200 OK
   - HTTP 版本: HTTP/2.0
   - Content-Type: application/x-protobuffer
   - Body 大小: 21 bytes

2. **基础 HTTPS 测试** - 我们的客户端可以成功访问其他 HTTPS 端点
   - `https://example.com/` - 成功
   - 状态码: 200
   - Body 大小: 513 bytes

### ❌ 失败的测试

所有针对 `https://kh.google.com/rt/earth/PlanetoidMetadata` 的测试都失败：

1. **HTTP/1.1 测试**
   ```
   错误: IO 错误: unexpected end of file
   ```
   - 问题：TLS 连接建立后，发送 HTTP/1.1 请求，但读取响应时连接被关闭

2. **HTTP/2 测试**
   ```
   错误: InvalidResponse("接收响应失败: stream error received: unspecific protocol error detected")
   ```
   - 问题：HTTP/2 握手成功，但接收响应时出现协议错误

3. **HTTP/3 测试**
   ```
   错误: 连接失败: 结束请求失败: quic transport error
   ```
   - 问题：QUIC 连接或 HTTP/3 请求处理失败

## 分析

### 服务器特性

通过 `openssl` 检查：
```bash
openssl s_client -connect kh.google.com:443 -alpn h2,http/1.1
# 结果: ALPN protocol: h2
```

服务器**强制使用 HTTP/2**，即使客户端声明支持 HTTP/1.1。

### 问题根源

1. **ALPN 协商**
   - Google Earth API 服务器优先选择 HTTP/2
   - 即使客户端只声明支持 `http/1.1`，服务器可能仍返回 HTTP/2

2. **我们的实现限制**
   - HTTP/1.1 客户端（`tls.rs`）没有检查 ALPN 协商结果
   - 当服务器选择 HTTP/2 时，我们仍按 HTTP/1.1 发送请求，导致连接被关闭
   
3. **HTTP/2 实现问题**
   - 设置了正确的 ALPN (`h2`, `http/1.1`)
   - HTTP/2 握手成功
   - 但接收响应时出现 "unspecific protocol error" - 可能是请求格式或设置不正确

## 已尝试的修复

### 1. 添加标准 HTTP Headers ❌
```rust
headers.accept = "*/*".to_string();
headers.accept_language = "en-US,en;q=0.9".to_string();
```
结果：仍然失败

### 2. 修改 ALPN 配置 ❌
```rust
// 只声明支持 HTTP/1.1
tls_config.alpn_protocols = vec![b"http/1.1".to_vec()];
```
结果：服务器仍然可能选择 HTTP/2

### 3. 检测 ALPN 协商结果 ⚠️
```rust
let negotiated_protocol = tls_stream.conn.alpn_protocol();
if proto == b"h2" && !config.prefer_http2 {
    return Err(...);
}
```
结果：添加了检测，但仍需要正确处理 HTTP/2 响应

## 下一步行动

### 优先级 1: 修复 HTTP/2 实现
- [ ] 添加详细的 HTTP/2 调试日志
- [ ] 检查 HTTP/2 请求构建（伪 headers）
- [ ] 验证 HTTP/2 Settings
- [ ] 测试简单的 Google URL（如 `www.google.com`）

### 优先级 2: 改进 ALPN 处理
- [ ] 在 HTTP/1.1 客户端中检查 ALPN 结果
- [ ] 当服务器选择 HTTP/2 时，自动切换到 HTTP/2 客户端
- [ ] 添加 ALPN 协商失败的明确错误信息

### 优先级 3: HTTP/3 支持
- [ ] 修复 QUIC 连接问题
- [ ] 验证 HTTP/3 请求格式
- [ ] 测试 HTTP/3 连接池

## 参考

### 成功的 reqwest 配置
```rust
let client = reqwest::Client::builder()
    .use_rustls_tls()
    .build()
    .unwrap();

client.get("https://kh.google.com/rt/earth/PlanetoidMetadata")
    .send()
    .await
```

### 服务器响应
- 状态码: 200 OK (reqwest) / 404 (curl 默认)
- Content-Type: application/x-protobuffer
- ALPN: h2
- Alt-Svc: h3=":443"; ma=2592000,h3-29=":443"; ma=2592000

## 测试文件

- ✅ `tests/simple_https_test.rs` - 基础 HTTPS 测试（通过）
- ❌ `tests/google_earth_full_test.rs` - 全协议测试（失败）
- ❌ `tests/debug_google_earth_test.rs` - 调试测试（失败）
- ❌ `tests/test_with_http2.rs` - HTTP/2 专项测试（失败）

## 结论

当前的 HTTP 客户端实现可以处理标准的 HTTPS 请求（HTTP/1.1），但在处理强制使用 HTTP/2 的服务器时存在问题。主要问题在于：

1. **ALPN 协商处理不完整** - 没有根据协商结果选择正确的协议处理器
2. **HTTP/2 实现存在问题** - 请求格式或配置导致协议错误
3. **HTTP/3 实现需要完善** - QUIC 连接建立失败

建议的解决方案：
1. 优先修复 HTTP/2 实现，使其能够正确处理标准的 HTTP/2 请求
2. 改进协议协商逻辑，根据 ALPN 结果自动选择正确的处理器
3. 完善 HTTP/3 支持，确保 QUIC 连接可以正常建立

---

*最后更新: 2025-12-14*
