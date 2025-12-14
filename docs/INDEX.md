# 📚 fingerprint-rust 文档索引

**最后更新**: 2025-12-15  
**版本**: v1.0.0

---

## 🎯 快速导航

### 新用户入门
1. [README.md](../README.md) ⭐ - 项目介绍和快速开始
2. [API.md](API.md) - API 文档和使用说明
3. [examples/](../examples/) - 代码示例

### 开发者文档
4. [ARCHITECTURE.md](ARCHITECTURE.md) - 系统架构设计
5. [模块文档](modules/) - 各模块详细文档

---

## 📖 核心文档

### 必读文档
- **[README.md](../README.md)** - 项目主页，快速开始指南
- **[API.md](API.md)** - 完整 API 参考文档
- **[ARCHITECTURE.md](ARCHITECTURE.md)** - 系统架构设计文档

---

## 📦 模块文档

按代码模块组织的详细文档：

### 核心模块
- **[profiles.md](modules/profiles.md)** - 指纹配置管理（66+ 浏览器）
- **[useragent.md](modules/useragent.md)** - User-Agent 生成
- **[headers.md](modules/headers.md)** - HTTP Headers 生成
- **[random.md](modules/random.md)** - 随机指纹选择

### TLS 相关模块
- **[tls_config.md](modules/tls_config.md)** - TLS 配置和 ClientHello Spec
- **[tls_handshake.md](modules/tls_handshake.md)** - TLS 握手消息构建
- **[tls_limitations.md](modules/tls_limitations.md)** - TLS 指纹限制说明
- **[dicttls.md](modules/dicttls.md)** - TLS 字典实现（常量定义）
- **[utls.md](modules/utls.md)** - uTLS 兼容性说明

### HTTP 客户端模块
- **[http_client.md](modules/http_client.md)** - HTTP 客户端实现（HTTP/1.1/2/3）
- **[http_pool.md](modules/http_pool.md)** - HTTP 连接池支持
- **[http3_optimization.md](modules/http3_optimization.md)** - HTTP/3 优化说明
- **[netconnpool.md](modules/netconnpool.md)** - 连接池集成

### 其他模块
- **[validation_limitations.md](modules/validation_limitations.md)** - 验证限制说明

---

## 📊 代码结构对应关系

文档结构与代码结构完全对齐：

```
src/                          docs/
├── lib.rs                    ├── INDEX.md (本文档)
├── profiles.rs              ├── modules/profiles.md
├── useragent.rs             ├── modules/useragent.md
├── headers.rs               ├── modules/headers.md
├── random.rs                ├── modules/random.md
├── tls_config/              ├── modules/tls_config.md
│   ├── mod.rs               │
│   ├── builder.rs           │
│   ├── ja4.rs               │
│   └── ...                  │
├── tls_handshake/           ├── modules/tls_handshake.md
│   ├── builder.rs           │
│   └── ...                  │
├── dicttls/                 ├── modules/dicttls.md
│   └── ...                  │
└── http_client/             ├── modules/http_client.md
    ├── http1.rs            ├── modules/http_pool.md
    ├── http2.rs            ├── modules/http3_optimization.md
    ├── http3.rs            └── modules/netconnpool.md
    └── ...
```

---

## 🔍 按用途查找

### 想了解项目？
👉 从 [README.md](../README.md) 开始

### 想使用 API？
👉 查看 [API.md](API.md) 和 [examples/](../examples/)

### 想了解架构？
👉 阅读 [ARCHITECTURE.md](ARCHITECTURE.md)

### 想了解某个模块？
👉 查看 [modules/](modules/) 目录下的对应文档

### 想了解 TLS 指纹？
👉 查看 [tls_config.md](modules/tls_config.md) 和 [tls_handshake.md](modules/tls_handshake.md)

### 想使用 HTTP 客户端？
👉 查看 [http_client.md](modules/http_client.md)

### 想了解限制？
👉 查看 [tls_limitations.md](modules/tls_limitations.md) 和 [validation_limitations.md](modules/validation_limitations.md)

---

## 📁 文档组织说明

### 核心文档 (`docs/`)
- `README.md` - 项目说明（在根目录）
- `INDEX.md` - 文档索引（本文档）
- `API.md` - API 参考文档
- `ARCHITECTURE.md` - 架构设计文档

### 模块文档 (`docs/modules/`)
按代码模块组织的详细文档，与 `src/` 目录结构对应。

### 归档文档 (`docs/archive/`)
历史文档和临时文档，按类型分类：
- `archive/reports/` - 测试报告、审核报告等
- `archive/status/` - 项目状态文档
- `archive/history/` - 实现历史文档
- `archive/` - 其他归档文档

---

## 🎓 推荐阅读路径

### 路径 1: 快速入门
1. [README.md](../README.md)
2. [API.md](API.md)
3. [examples/basic.rs](../examples/basic.rs)
4. [http_client.md](modules/http_client.md)

### 路径 2: 深入理解
1. [README.md](../README.md)
2. [ARCHITECTURE.md](ARCHITECTURE.md)
3. [tls_config.md](modules/tls_config.md)
4. [tls_handshake.md](modules/tls_handshake.md)
5. [http_client.md](modules/http_client.md)

### 路径 3: 开发贡献
1. [ARCHITECTURE.md](ARCHITECTURE.md)
2. [API.md](API.md)
3. [modules/](modules/) - 各模块文档
4. [tls_limitations.md](modules/tls_limitations.md)

---

## 📊 文档统计

- **核心文档**: 3 个（README, API, ARCHITECTURE）
- **模块文档**: 9 个（按代码模块组织）
- **归档文档**: 58 个（历史文档）

---

## 🔗 外部资源

### GitHub
- **仓库**: https://github.com/vistone/fingerprint-rust
- **Issues**: https://github.com/vistone/fingerprint-rust/issues

### 相关项目
- **netconnpool-rust**: https://github.com/vistone/netconnpool-rust
- **Go uTLS**: https://github.com/refraction-networking/utls

---

**维护者**: fingerprint-rust team  
**最后更新**: 2025-12-15
