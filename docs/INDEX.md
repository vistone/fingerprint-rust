# 文档索引

**最后更新**: 2025-12-29  
**项目版本**: v2.0.0 (Workspace)

---

## 📚 文档结构

### 核心文档

- **[README.md](../README.md)** - 项目主文档（在根目录）
- **[CHANGELOG.md](CHANGELOG.md)** - 更新日志

### 架构文档

- **[ARCHITECTURE.md](ARCHITECTURE.md)** - 系统架构设计（包含 Workspace 架构）

### 使用指南 (`guides/`)

- **[USAGE_GUIDE.md](guides/USAGE_GUIDE.md)** - 使用指南：如何随机选择和指定浏览器指纹
- **[CAPTURE_BROWSER_FINGERPRINTS.md](guides/CAPTURE_BROWSER_FINGERPRINTS.md)** - 如何抓取真实浏览器的 TLS 指纹
- **[GOOGLE_EARTH_TEST.md](guides/GOOGLE_EARTH_TEST.md)** - Google Earth API 测试说明
- **[TEST_GOOGLE_EARTH_EXECUTABLE.md](guides/TEST_GOOGLE_EARTH_EXECUTABLE.md)** - Google Earth API 测试可执行程序使用指南

### 技术文档

- **[API.md](API.md)** - API 参考文档
- **[RUSTLS_FINGERPRINT_INTEGRATION.md](RUSTLS_FINGERPRINT_INTEGRATION.md)** - rustls 指纹集成说明

### 测试报告

- **[TEST_REPORT.md](TEST_REPORT.md)** - 完整测试报告（包含所有测试结果）

### 模块文档 (`modules/`)

按 Crate 组织的模块文档：


#### fingerprint-tls
- **[tls_config.md](modules/tls_config.md)** - TLS 配置模块
- **[tls_handshake.md](modules/tls_handshake.md)** - TLS 握手模块

#### fingerprint-profiles
- **[profiles.md](modules/profiles.md)** - 浏览器指纹配置模块（69 个浏览器）

#### fingerprint-headers
- **[headers.md](modules/headers.md)** - HTTP Headers 生成模块
- **[useragent.md](modules/useragent.md)** - User-Agent 生成模块

#### fingerprint-http
- **[http_client.md](modules/http_client.md)** - HTTP 客户端模块（HTTP/1.1、HTTP/2、HTTP/3）

#### fingerprint-dns
- **[dns.md](modules/dns.md)** - DNS 预解析模块（需要 `dns` feature）

---

## 🚀 快速导航

### 新手入门

1. 阅读 [README.md](../README.md) 了解项目
2. 查看 [USAGE_GUIDE.md](guides/USAGE_GUIDE.md) 学习如何使用
3. 运行示例代码（`examples/` 目录）

### 开发者

1. 阅读 [ARCHITECTURE.md](ARCHITECTURE.md) 了解系统架构
2. 查看 [API.md](API.md) 了解 API 接口
3. 阅读 [modules/](modules/) 下的模块文档了解各 crate 的实现
4. 查看 [TEST_REPORT.md](TEST_REPORT.md) 了解测试覆盖情况

### 贡献者

1. 阅读 [CAPTURE_BROWSER_FINGERPRINTS.md](guides/CAPTURE_BROWSER_FINGERPRINTS.md) 了解如何添加新指纹
2. 查看 [ARCHITECTURE.md](ARCHITECTURE.md) 了解代码组织
3. 查看 [TEST_REPORT.md](TEST_REPORT.md) 了解测试覆盖情况

---

## 📦 Workspace 架构

项目采用 Cargo Workspace 架构，包含 7 个独立 crate：

1. **fingerprint-core** - 核心类型和工具
2. **fingerprint-tls** - TLS 配置、扩展和握手
3. **fingerprint-profiles** - 浏览器指纹配置
4. **fingerprint-headers** - HTTP Headers 和 User-Agent
5. **fingerprint-http** - HTTP 客户端实现
6. **fingerprint-dns** - DNS 预解析服务（可选）
7. **fingerprint** - 主库，重新导出所有功能

详细说明请查看 [ARCHITECTURE.md](ARCHITECTURE.md)

---

## 🔗 相关资源

### 脚本工具 (`../scripts/`)

- 测试脚本和工具脚本，参见 [scripts/](../scripts/) 目录

### 示例代码 (`../examples/`)

- 基础使用示例
- HTTP/2、HTTP/3 示例
- DNS 服务示例
- 配置导出示例

---

**文档版本**: v2.0.0  
**最后更新**: 2025-12-14
