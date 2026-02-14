# fingerprint-http

HTTP 客户端实现和协议特征提取模块，用于构建指纹识别的 HTTP 客户端。

## 功能特性

- ✅ HTTP/1.1 和 HTTP/2 支持
- ✅ 自定义头部顺序
- ✅ 连接池管理
- ✅ 请求-响应拦截
- ✅ GZIP 压缩特性分析
- 🔧 可选的 HTTP/3 (QUIC) 支持

## 快速开始

```rust
use fingerprint_http::HttpClient;

let client = HttpClient::new();
let response = client.get("https://example.com").await?;
println!("HTTP/2 enabled: {}", response.http_version);
```

## API 概览

| 类型 | 说明 |
|-----|------|
| `HttpClient` | HTTP 客户端 |
| `HttpFingerprint` | HTTP 指纹 |
| `RequestBuilder` | 请求构建器 |
| `ConnectionPool` | 连接池 |

## 项目结构

```
src/
├── lib.rs          # 模块入口
├── client.rs       # HTTP 客户端
├── pool.rs         # 连接池
├── fingerprint.rs  # 指纹提取
└── features.rs     # 特征分析
```

## 依赖关系

| 依赖 | 用途 |
|-----|------|
| `reqwest` | HTTP 客户端库 |
| `hyper` | HTTP 框架 |
| `tokio` | 异步运行时 |

## 许可证

MIT 许可证。详见：[LICENSE](../../LICENSE)

---

**最后更新：** 2026年2月14日
