# 实现总结

## 核心设计

本项目使用**官方 rustls** 作为底层 TLS 实现，通过 `ClientHelloCustomizer` 来模拟市场成熟浏览器的 TLS 指纹。

### 关键原则

1. ✅ **使用官方 rustls** - 不自己实现完整的 TLS 握手
2. ✅ **通过 ClientHelloCustomizer 应用指纹** - 利用 rustls 的扩展机制
3. ✅ **模拟真实浏览器** - 使用已有的浏览器配置（Chrome、Firefox、Safari 等）
4. ✅ **不自定义指纹** - 只使用项目中已有的浏览器指纹配置

## 架构

```
HTTP Client
    ↓
rustls (官方 TLS 库)
    ↓
ClientHelloCustomizer (应用浏览器指纹)
    ↓
ClientProfile (浏览器指纹配置)
```

## 使用方法

```rust
use fingerprint::{HttpClient, HttpClientConfig, chrome_133};

// 配置浏览器指纹
let config = HttpClientConfig {
    profile: Some(chrome_133()),  // 使用 Chrome 133 指纹
    ..Default::default()
};

// 创建客户端并发送请求
let client = HttpClient::new(config);
let response = client.get("https://example.com")?;
// 自动使用 Chrome 133 的 TLS 指纹
```

## 支持的浏览器指纹

- Chrome 系列: chrome_103, chrome_133, chrome_120 等
- Firefox 系列: firefox_102, firefox_133, firefox_135 等
- Safari 系列: safari_16_0, safari_ios_18_0 等
- Opera 系列: opera_89, opera_91 等

共 66+ 个真实浏览器指纹配置。

## 技术实现

### ClientHelloCustomizer

通过 rustls 的 `ClientHelloCustomizer` 接口，我们可以：

- ✅ 调整 TLS 扩展的顺序（匹配浏览器指纹）
- ✅ 自动应用浏览器指纹配置

### 限制

rustls 的 `ClientHelloCustomizer` 功能有限：

- ✅ 可以调整扩展顺序
- ❌ 无法完全控制密码套件顺序
- ❌ 无法完全控制所有扩展的完整内容

## 文件结构

```
src/http_client/
├── rustls_utils.rs                    # rustls 配置工具
│   └── build_client_config()         # 构建配置，应用指纹
├── rustls_client_hello_customizer.rs  # ClientHello 定制器
│   └── ProfileClientHelloCustomizer   # 应用浏览器指纹
├── tls.rs                             # HTTP/1.1 TLS 实现
├── http2.rs                           # HTTP/2 TLS 实现
└── http3.rs                           # HTTP/3 TLS 实现
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

1. **使用官方库** - rustls 是成熟的 TLS 库
2. **安全性** - 由专业团队维护
3. **兼容性** - 与标准 TLS 实现兼容
4. **维护成本低** - 不需要自己实现完整的 TLS 握手

## 参考文档

- [rustls 浏览器指纹集成](./RUSTLS_FINGERPRINT_INTEGRATION.md) - 详细技术文档
- [uTLS 风格 API](./UTLS_STYLE_API.md) - API 使用指南

