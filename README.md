# fingerprint-rust

**English** | [中文](#中文版本)

High-performance browser fingerprinting library written in Rust. Provides comprehensive browser fingerprint recognition and simulation capabilities, supporting TLS, HTTP/2, HTTP/3 and other modern protocols.

## 🎯 Key Features

- **90+ Real Browser Fingerprints** - Chrome, Firefox, Safari, Opera, Edge with accurate versions
- **Multi-Protocol Support** - HTTP/1.1, HTTP/2, HTTP/3 (QUIC) implementation
- **Advanced TLS Fingerprinting** - JA3, JA4+ generation and matching
- **Passive Recognition** - Network-level fingerprint identification
- **Active Protection** - Client-side fingerprint obfuscation and noise injection
- **Machine Learning** - Intelligent fingerprint classification and risk assessment

## 🚀 Quick Start

```bash
# Add to Cargo.toml
[dependencies]
fingerprint = "2.1"
```

```rust
use fingerprint::{get_random_fingerprint, mapped_tls_clients};

// Method 1: Get a random fingerprint with HTTP headers
let result = get_random_fingerprint().unwrap();
println!("Profile: {}", result.profile_id);
println!("User-Agent: {}", result.user_agent);
println!("Accept-Language: {}", result.headers.accept_language);

// Method 2: Use browser profiles directly
let profiles = mapped_tls_clients();
let chrome = profiles.get("chrome_133").unwrap();
let spec = chrome.get_client_hello_spec().unwrap();
println!("Cipher suites: {}", spec.cipher_suites.len());
```

## 📚 Documentation

For detailed documentation, please refer to the [docs](docs/) directory:
- **[User Guide](docs/en/user-guides/)** - Getting started and usage guides
- **[API Reference](docs/en/reference/)** - Complete API documentation
- **[Architecture](docs/en/ARCHITECTURE.md)** - System architecture and design
- **[Developer Guides](docs/en/developer-guides/)** - Development guidelines
- **[Examples](examples/)** - Practical usage examples

## 📦 Module Structure

```
crates/
├── fingerprint/           # Main facade crate (use this)
├── fingerprint-core/      # Core types and utilities
├── fingerprint-tls/       # TLS configuration and handshake
├── fingerprint-http/      # HTTP client (HTTP/1.1/2/3)
├── fingerprint-profiles/  # Browser fingerprint profiles
├── fingerprint-headers/   # HTTP headers generation
├── fingerprint-dns/       # DNS resolution
├── fingerprint-gateway/   # High-performance API gateway
├── fingerprint-defense/   # Passive detection and active protection
├── fingerprint-ml/        # Machine learning classification
├── fingerprint-canvas/    # Canvas fingerprinting
├── fingerprint-webgl/     # WebGL fingerprinting
├── fingerprint-audio/     # Audio fingerprinting
├── fingerprint-fonts/     # Font fingerprinting
├── fingerprint-storage/   # Storage fingerprinting
├── fingerprint-webrtc/    # WebRTC fingerprinting
├── fingerprint-hardware/  # Hardware fingerprinting
├── fingerprint-timing/    # Timing analysis
├── fingerprint-anomaly/   # Anomaly detection
└── fingerprint-api-noise/ # API noise injection
```

## 🔧 Building

```bash
# Build all crates
cargo build --workspace --release

# Run tests
cargo test --workspace
```

## 🤝 Contributing

We welcome contributions! Please see [CONTRIBUTING](docs/en/CONTRIBUTING.md) for guidelines.

## 📄 License

BSD-3-Clause License - see [LICENSE](LICENSE) file for details.

---

# 中文版本

高性能浏览器指纹识别库，使用Rust编写。提供全面的浏览器指纹识别和模拟能力，支持TLS、HTTP/2、HTTP/3等现代协议。

## 🎯 核心功能

- **90+真实浏览器指纹** - Chrome、Firefox、Safari、Opera、Edge等准确版本
- **多协议支持** - HTTP/1.1、HTTP/2、HTTP/3(QUIC)完整实现
- **高级TLS指纹** - JA3、JA4+生成和匹配
- **被动识别** - 网络层面被动指纹识别
- **主动防护** - 客户端指纹混淆和噪声注入
- **机器学习** - 智能指纹分类和风险评估

## 🚀 快速开始

```bash
# 添加到Cargo.toml
[dependencies]
fingerprint = "2.1"
```

```rust
use fingerprint::{get_random_fingerprint, mapped_tls_clients};

// 方法1：获取随机指纹和HTTP头
let result = get_random_fingerprint().unwrap();
println!("Profile: {}", result.profile_id);
println!("User-Agent: {}", result.user_agent);
println!("Accept-Language: {}", result.headers.accept_language);

// 方法2：直接使用浏览器配置
let profiles = mapped_tls_clients();
let chrome = profiles.get("chrome_133").unwrap();
let spec = chrome.get_client_hello_spec().unwrap();
println!("密码套件数量: {}", spec.cipher_suites.len());
```

## 📚 文档资源

详细文档请参考 [docs](docs/) 目录：
- **[用户指南](docs/zh/user-guides/)** - 入门和使用指南
- **[API参考](docs/zh/reference/)** - 完整API文档
- **[架构设计](docs/zh/ARCHITECTURE.md)** - 系统架构和设计
- **[开发指南](docs/zh/developer-guides/)** - 开发规范
- **[示例代码](examples/)** - 实际使用示例

## 📦 模块结构

```
crates/
├── fingerprint/           # 主入口crate（推荐使用）
├── fingerprint-core/      # 核心类型和工具
├── fingerprint-tls/       # TLS配置和握手
├── fingerprint-http/      # HTTP客户端(HTTP/1.1/2/3)
├── fingerprint-profiles/  # 浏览器指纹配置
├── fingerprint-headers/   # HTTP头生成
├── fingerprint-dns/       # DNS解析
├── fingerprint-gateway/   # 高性能API网关
├── fingerprint-defense/   # 被动检测和主动防护
├── fingerprint-ml/        # 机器学习分类
├── fingerprint-canvas/    # Canvas指纹
├── fingerprint-webgl/     # WebGL指纹
├── fingerprint-audio/     # 音频指纹
├── fingerprint-fonts/     # 字体指纹
├── fingerprint-storage/   # 存储指纹
├── fingerprint-webrtc/    # WebRTC指纹
├── fingerprint-hardware/  # 硬件指纹
├── fingerprint-timing/    # 时序分析
├── fingerprint-anomaly/   # 异常检测
└── fingerprint-api-noise/ # API噪声注入
```

## 🔧 构建

```bash
# 构建所有crate
cargo build --workspace --release

# 运行测试
cargo test --workspace
```

## 🤝 贡献指南

欢迎贡献！请查看 [CONTRIBUTING](docs/zh/CONTRIBUTING.md)。

## 📄 许可证

BSD-3-Clause许可证 - 详见 [LICENSE](LICENSE)。

---
**Version**: 2.1.0  
**Last Updated**: 2026-02-16