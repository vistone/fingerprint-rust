# 自定义 ClientHello 集成实现

## 概述

已实现基础的自定义 TLS ClientHello 支持，可以在所有 HTTP 请求中使用项目自己的 ClientHello 构建功能。

## 实现状态

### ✅ 已完成

1. **自定义 TLS 模块** (`src/http_client/custom_tls.rs`)
   - 创建了 `CustomTlsConnector` 用于执行自定义 TLS 握手
   - 使用 `TLSHandshakeBuilder::build_client_hello()` 发送自定义 ClientHello
   - 接收并验证 ServerHello 响应

2. **HTTP/1.1 集成**
   - 修改了 `send_https_request()` 函数
   - 当配置了 `ClientProfile` 时，自动使用自定义 ClientHello
   - 通过 `custom-tls` feature 控制

3. **特性标志**
   - 添加了 `custom-tls` feature 到 `Cargo.toml`
   - 可以通过 `--features custom-tls` 启用

### ⚠️ 当前限制

当前的实现是一个**简化版本**，只完成了：
- ✅ 发送自定义 ClientHello
- ✅ 接收 ServerHello
- ⚠️ **未完成完整的 TLS 握手**

完整的 TLS 握手还需要：
- 密钥交换（ECDHE, DHE等）
- 证书验证
- ChangeCipherSpec 处理
- Finished 消息
- 对称加密连接的建立

## 使用方法

### 1. 启用自定义 TLS

在 `Cargo.toml` 中启用 `custom-tls` feature：

```toml
[dependencies]
fingerprint = { path = ".", features = ["custom-tls", "rustls-tls"] }
```

或者通过命令行：

```bash
cargo build --features custom-tls
```

### 2. 配置 ClientProfile

```rust
use fingerprint::{chrome_133, HttpClient, HttpClientConfig};

// 创建配置，包含浏览器指纹
let config = HttpClientConfig {
    profile: Some(chrome_133()),
    ..Default::default()
};

// 创建 HTTP 客户端
let client = HttpClient::new(config);

// 发送请求（将自动使用自定义 ClientHello）
let response = client.get("https://example.com")?;
```

### 3. 直接使用自定义 TLS

```rust
use fingerprint::http_client::custom_tls::send_https_request_with_custom_tls;
use fingerprint::{chrome_133, HttpClientConfig, HttpRequest, HttpMethod};

let config = HttpClientConfig {
    profile: Some(chrome_133()),
    ..Default::default()
};

let request = HttpRequest::new(HttpMethod::Get, "https://example.com");
let response = send_https_request_with_custom_tls(
    "example.com",
    443,
    "/",
    &request,
    &config,
)?;
```

## 实现细节

### 自定义 TLS 握手流程

1. **获取 ClientHelloSpec**
   ```rust
   let spec = profile.get_client_hello_spec()?;
   ```

2. **构建自定义 ClientHello**
   ```rust
   let client_hello_bytes = TLSHandshakeBuilder::build_client_hello(&spec, host)?;
   ```

3. **发送 ClientHello**
   ```rust
   stream.write_all(&client_hello_bytes)?;
   ```

4. **发送 ChangeCipherSpec** (TLS 1.3 兼容)
   ```rust
   let ccs = [0x14, 0x03, 0x01, 0x00, 0x01, 0x01];
   stream.write_all(&ccs)?;
   ```

5. **接收 ServerHello**
   ```rust
   let mut response_header = vec![0u8; 5];
   stream.read_exact(&mut response_header)?;
   ```

## 后续改进方向

### 方案 1: 完整实现 TLS 握手

实现完整的 TLS 握手协议，包括：
- 解析 ServerHello
- 密钥交换（ECDHE/DHE）
- 证书验证
- 对称加密建立

**优点**: 完全控制 TLS 指纹
**缺点**: 实现复杂，需要大量工作

### 方案 2: 混合方案

使用自定义 ClientHello 发送初始握手，然后使用 rustls 处理后续：
- 发送自定义 ClientHello
- 接收 ServerHello
- 切换到 rustls 处理后续握手

**优点**: 平衡了控制和复杂度
**缺点**: 需要修改 rustls 或使用 fork 版本

### 方案 3: 使用支持自定义 ClientHello 的库

使用专门支持自定义 ClientHello 的 TLS 库，如：
- Go 的 uTLS（通过 FFI）
- 其他支持自定义 ClientHello 的 Rust 库

**优点**: 功能完整，经过测试
**缺点**: 需要外部依赖

## 当前代码结构

```
src/http_client/
├── custom_tls.rs          # 自定义 TLS 实现
├── tls.rs                 # 标准 TLS 实现（已集成自定义 TLS）
├── http2.rs               # HTTP/2 实现（待集成）
└── http3.rs               # HTTP/3 实现（待集成）
```

## 测试

当前实现可以通过以下方式测试：

```rust
#[test]
#[ignore] // 需要网络连接
fn test_custom_tls() {
    use fingerprint::{chrome_133, HttpClientConfig, HttpRequest, HttpMethod};
    use fingerprint::http_client::custom_tls::send_https_request_with_custom_tls;
    
    let config = HttpClientConfig {
        profile: Some(chrome_133()),
        ..Default::default()
    };
    
    let request = HttpRequest::new(HttpMethod::Get, "https://httpbin.org/get");
    let response = send_https_request_with_custom_tls(
        "httpbin.org",
        443,
        "/get",
        &request,
        &config,
    );
    
    // 注意：当前实现可能无法完成完整握手
    // 这取决于服务器的响应和 TLS 版本
}
```

## 注意事项

1. **当前实现是简化版本**，可能无法与所有服务器完成完整握手
2. **需要启用 `custom-tls` feature** 才能使用
3. **必须配置 `ClientProfile`** 才能使用自定义 ClientHello
4. **HTTP/2 和 HTTP/3 尚未集成**，需要后续实现

## 下一步

1. ✅ 基础自定义 TLS 模块
2. ✅ HTTP/1.1 集成
3. ⏳ HTTP/2 集成
4. ⏳ HTTP/3 集成
5. ⏳ 完整 TLS 握手实现（可选）

