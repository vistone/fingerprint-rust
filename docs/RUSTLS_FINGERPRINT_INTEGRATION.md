# rustls 浏览器指纹集成

## 概述

本项目使用**官方 rustls** 作为底层 TLS 实现，通过 `ClientHelloCustomizer` 来模拟市场成熟浏览器的 TLS 指纹。

## 架构设计

### 核心原则

1. **使用官方 rustls** - 不自己实现完整的 TLS 握手
2. **通过 ClientHelloCustomizer 应用指纹** - 利用 rustls 的扩展机制
3. **模拟真实浏览器** - 使用已有的浏览器配置（Chrome、Firefox、Safari 等）
4. **不自定义指纹** - 只使用项目中已有的浏览器指纹配置

### 实现方式

```
┌─────────────────────────────────────────┐
│  HTTP Client (HTTP/1.1, HTTP/2, HTTP/3) │
└─────────────────┬───────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────┐
│  rustls (官方 TLS 库)                    │
│  - ClientConnection                      │
│  - ClientConfig                          │
│  - ClientHelloCustomizer (应用指纹)      │
└─────────────────┬───────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────┐
│  ClientHelloCustomizer                   │
│  - 根据 ClientProfile 调整扩展顺序       │
│  - 应用浏览器指纹配置                    │
└─────────────────┬───────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────┐
│  ClientProfile (浏览器指纹配置)          │
│  - Chrome 133, Firefox 133, Safari 等    │
│  - ClientHelloSpec                       │
└─────────────────────────────────────────┘
```

## 使用方法

### 基本使用

```rust
use fingerprint::{HttpClient, HttpClientConfig, chrome_133};

// 创建配置，包含浏览器指纹
let config = HttpClientConfig {
    profile: Some(chrome_133()),  // 使用 Chrome 133 指纹
    ..Default::default()
};

// 创建 HTTP 客户端
let client = HttpClient::new(config);

// 发送请求（自动使用 Chrome 133 的 TLS 指纹）
let response = client.get("https://example.com")?;
```

### 支持的浏览器指纹

项目支持 66+ 个真实浏览器指纹：

- **Chrome 系列**: chrome_103, chrome_133, chrome_120 等
- **Firefox 系列**: firefox_102, firefox_133, firefox_135 等
- **Safari 系列**: safari_16_0, safari_ios_18_0 等
- **Opera 系列**: opera_89, opera_91 等

```rust
use fingerprint::{chrome_133, firefox_133, safari_16_0};

// 使用 Chrome 133
let config = HttpClientConfig {
    profile: Some(chrome_133()),
    ..Default::default()
};

// 使用 Firefox 133
let config = HttpClientConfig {
    profile: Some(firefox_133()),
    ..Default::default()
};

// 使用 Safari 16.0
let config = HttpClientConfig {
    profile: Some(safari_16_0()),
    ..Default::default()
};
```

## 技术实现

### ClientHelloCustomizer

rustls 的 `ClientHelloCustomizer` 允许我们在发送 ClientHello 之前进行定制：

```rust
impl ClientHelloCustomizer for ProfileClientHelloCustomizer {
    fn customize_client_hello(
        &self,
        _ctx: ClientHelloContext<'_>,
        hello: &mut ClientHello<'_>,
    ) -> Result<(), rustls::Error> {
        // 调整扩展顺序以匹配浏览器指纹
        let used = hello.extension_encoding_order();
        let order = reorder_used_extensions(used, &self.desired_extension_ids);
        hello.set_extension_encoding_order(order)?;
        Ok(())
    }
}
```

### 当前支持的功能

✅ **扩展顺序调整** - 根据浏览器指纹调整 TLS 扩展的顺序
✅ **自动应用指纹** - 配置 profile 后自动应用
✅ **HTTP/1.1, HTTP/2, HTTP/3** - 所有协议都支持

### rustls 的限制

rustls 的 `ClientHelloCustomizer` 功能有限，只能：

- ✅ 调整扩展顺序
- ✅ 修改部分扩展内容（如果 rustls 支持）
- ❌ 无法完全控制密码套件顺序
- ❌ 无法完全控制所有扩展的完整内容
- ❌ 无法控制 Random 值等

### 启用方式

在 `Cargo.toml` 中启用 `rustls-client-hello-customizer` feature：

```toml
[dependencies]
fingerprint = { path = ".", features = ["rustls-tls", "rustls-client-hello-customizer"] }
```

**注意**: `rustls-client-hello-customizer` 需要 rustls 的 fork 版本支持，或者等待官方 rustls 添加此功能。

## 代码结构

```
src/http_client/
├── rustls_utils.rs              # rustls 配置工具
│   └── build_client_config()    # 构建 rustls 配置，应用指纹
├── rustls_client_hello_customizer.rs  # ClientHello 定制器
│   └── ProfileClientHelloCustomizer   # 应用浏览器指纹
├── tls.rs                        # HTTP/1.1 TLS 实现
├── http2.rs                      # HTTP/2 TLS 实现
└── http3.rs                      # HTTP/3 TLS 实现
```

## 与 uTLS (Go) 的对比

| 特性 | uTLS (Go) | 本项目 (Rust) |
|------|-----------|---------------|
| 底层 TLS | 自己实现 | 官方 rustls |
| ClientHello 控制 | 完全控制 | 部分控制（扩展顺序） |
| 浏览器指纹 | 支持 | 支持 |
| 实现复杂度 | 高 | 低（依赖 rustls） |
| 维护成本 | 高 | 低 |

## 优势

1. **使用官方库** - rustls 是 Rust 生态系统中成熟的 TLS 库
2. **安全性** - 由专业团队维护，安全性有保障
3. **兼容性** - 与标准 TLS 实现兼容
4. **维护成本低** - 不需要自己实现完整的 TLS 握手

## 限制

1. **功能有限** - rustls 的 ClientHelloCustomizer 只能调整扩展顺序
2. **需要 fork** - 可能需要 rustls 的 fork 版本支持 ClientHelloCustomizer
3. **无法完全模拟** - 无法完全控制所有 ClientHello 细节

## 未来改进

1. **等待 rustls 官方支持** - 如果 rustls 官方添加更多 ClientHello 定制功能
2. **使用 rustls fork** - 使用支持更多定制功能的 rustls fork
3. **混合方案** - 部分使用 rustls，部分自己实现（如果需要完全控制）

## 参考

- [rustls GitHub](https://github.com/rustls/rustls) - 官方 rustls 库
- [uTLS GitHub](https://github.com/refraction-networking/utls) - Go 版本的实现参考

