# fingerprint-rust

**English** | [中文](#中文版本)

High-performance browser fingerprinting library written in Rust. Provides comprehensive browser fingerprint recognition and simulation capabilities, supporting TLS, HTTP/2, HTTP/3 and other modern protocols.

## 🎯 Key Features

- **66+ Real Browser Fingerprints** - Chrome, Firefox, Safari, Opera, Edge with accurate versions
- **Multi-Protocol Support** - HTTP/1.1, HTTP/2, HTTP/3 (QUIC) implementation
- **Advanced TLS Fingerprinting** - JA3, JA4+ generation and matching
- **Passive Recognition** - Network-level fingerprint identification
- **Active Protection** - Client-side fingerprint obfuscation and noise injection
- **Machine Learning** - Intelligent fingerprint classification and risk assessment

## 🚀 Quick Start

```bash
# Add to Cargo.toml
[dependencies]
fingerprint-core = "2.1"
fingerprint-tls = "2.1"
fingerprint-http = "2.1"
```

```rust
use fingerprint_core::{FingerprintClient, Profile};

let client = FingerprintClient::builder()
    .with_profile(Profile::Chrome120)
    .build()?;

let response = client.get("https://httpbin.org/headers").await?;
println!("Status: {}", response.status());
```

## 📚 Documentation

For detailed documentation, please refer to the [docs](docs/) directory:
- **[User Guide](docs/user-guides/)** - Getting started and usage guides
- **[API Reference](docs/reference/)** - Complete API documentation
- **[Architecture](docs/ARCHITECTURE.md)** - System architecture and design
- **[Developer Guides](docs/developer-guides/)** - Development guidelines
- **[Examples](examples/)** - Practical usage examples

## 📦 Module Structure

```
crates/
├── fingerprint-core/      # Core types and utilities
├── fingerprint-tls/       # TLS configuration and handshake
├── fingerprint-http/      # HTTP client (HTTP/1.1/2/3)
├── fingerprint-profiles/  # Browser fingerprint profiles
├── fingerprint-gateway/   # High-performance API gateway
└── fingerprint-defense/   # Passive detection and active protection
```

## 🔧 Building

```bash
# Build all crates
cargo build --workspace --release

# Run tests
cargo test --workspace

# Run examples
cargo run --example basic
```

## 🤝 Contributing

We welcome contributions! Please see [CONTRIBUTING](docs/CONTRIBUTING.md) for guidelines.

## 📄 License

MIT License - see [LICENSE](LICENSE) file for details.

---

# 中文版本

高性能浏览器指纹识别库，使用Rust编写。提供全面的浏览器指纹识别和模拟能力，支持TLS、HTTP/2、HTTP/3等现代协议。

## 🎯 核心功能

- **66+真实浏览器指纹** - Chrome、Firefox、Safari、Opera、Edge等准确版本
- **多协议支持** - HTTP/1.1、HTTP/2、HTTP/3(QUIC)完整实现
- **高级TLS指纹** - JA3、JA4+生成和匹配
- **被动识别** - 网络层面被动指纹识别
- **主动防护** - 客户端指纹混淆和噪声注入
- **机器学习** - 智能指纹分类和风险评估

## 🚀 快速开始

```bash
# 添加到Cargo.toml
[dependencies]
fingerprint-core = "2.1"
fingerprint-tls = "2.1"
fingerprint-http = "2.1"
```

```rust
use fingerprint_core::{FingerprintClient, Profile};

let client = FingerprintClient::builder()
    .with_profile(Profile::Chrome120)
    .build()?;

let response = client.get("https://httpbin.org/headers").await?;
println!("状态: {}", response.status());
```

## 📚 文档资源

详细文档请参考 [docs](docs/) 目录：
- **[用户指南](docs/user-guides/)** - 入门和使用指南
- **[API参考](docs/reference/)** - 完整API文档
- **[架构设计](docs/ARCHITECTURE.md)** - 系统架构和设计
- **[开发指南](docs/developer-guides/)** - 开发规范
- **[示例代码](examples/)** - 实际使用示例

## 📦 模块结构

```
crates/
├── fingerprint-core/      # 核心类型和工具
├── fingerprint-tls/       # TLS配置和握手
├── fingerprint-http/      # HTTP客户端(HTTP/1.1/2/3)
├── fingerprint-profiles/  # 浏览器指纹配置
├── fingerprint-gateway/   # 高性能API网关
└── fingerprint-defense/   # 被动检测和主动防护
```

## 🔧 构建

```bash
# 构建所有crate
cargo build --workspace --release

# 运行测试
cargo test --workspace

# 运行示例
cargo run --example basic
```

## 🤝 贡献指南

欢迎贡献！请查看 [CONTRIBUTING](docs/CONTRIBUTING.md)。

## 📄 许可证

MIT许可证 - 详见 [LICENSE](LICENSE)。

---
**Version**: 2.1.0  
**Last Updated**: 2026-02-14