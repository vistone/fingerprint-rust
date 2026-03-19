# fingerprint-rust

**English** | [中文](#中文版本)

`fingerprint-rust` is a Rust workspace for browser fingerprint generation, protocol-level fingerprint analysis, passive detection, and related gateway/service tooling.

The current repository has a clear split between:
- stable, documented crates that are part of the default workspace member set and release path
- preview/prototype crates that compile today but are excluded from the default member set while APIs continue to evolve
- experimental crates that are intentionally excluded from the workspace

## Key Features

- 90+ browser profile presets for Chrome, Firefox, Safari, Opera, and Edge
- TLS fingerprinting primitives including JA3, JA4, JARM, GREASE handling, and handshake builders
- HTTP client support for HTTP/1.1, HTTP/2, and optional HTTP/3
- Browser profile, header, and User-Agent generation
- Optional DNS, passive defense, API noise, and gateway modules

## Quick Start

```toml
[dependencies]
fingerprint = "2.1"
```

```rust
use fingerprint::{get_random_fingerprint, mapped_tls_clients};

let result = get_random_fingerprint().unwrap();
println!("Profile: {}", result.profile_id);
println!("User-Agent: {}", result.user_agent);
println!("Accept-Language: {}", result.headers.accept_language);

let profiles = mapped_tls_clients();
let chrome = profiles.get("chrome_133").unwrap();
println!("Profile ID: {}", chrome.id());
println!("Cipher suites: {}", chrome.tls_config.cipher_suites.len());
println!("HTTP/2 settings: {}", chrome.http2_settings.len());
```

## Support Matrix

| Scope | Status | Notes |
| --- | --- | --- |
| `fingerprint` | Stable | Recommended public entry point for consumers |
| `fingerprint-core` | Stable | Core types, parsers, JA3/JA4/JARM, cache, metrics, packet parsing |
| `fingerprint-tls` | Stable | TLS config, extensions, handshake building |
| `fingerprint-http` | Stable | HTTP client and protocol fingerprinting; `http3` remains optional |
| `fingerprint-profiles` | Stable | Browser profile catalog and version helpers |
| `fingerprint-headers` | Stable | Header and User-Agent generation |
| `fingerprint-dns` | Stable | Network-aware DNS service module; external network/API behavior depends on runtime environment |
| `fingerprint-defense` | Stable | Passive analysis and defense-oriented helpers |
| `fingerprint-gateway` | Stable | Service crate with Redis-backed rate limiting and Actix integration |
| `fingerprint-api-noise` | Stable | Optional API noise helpers |
| `fingerprint-ai-models` | Preview | In workspace, but excluded from the default member set because it is adjacent to the main browser-fingerprint surface |
| `fingerprint-audio` / `canvas` / `fonts` / `hardware` / `storage` / `webgl` / `webrtc` / `ml` | Prototype | Stay in the workspace for explicit validation, but are excluded from the default member set while APIs and heuristics evolve |
| `fingerprint-analysis` / `anomaly` / `config` / `hardware-unified` / `timing` | Experimental | Present in repo, intentionally excluded from the workspace |

## Workspace Layout

### Recommended crates

```text
crates/
├── fingerprint/           # Main facade crate
├── fingerprint-core/      # Shared core types and algorithms
├── fingerprint-tls/       # TLS specs, extensions, handshake builders
├── fingerprint-http/      # HTTP client and protocol support
├── fingerprint-profiles/  # Browser profile catalog
├── fingerprint-headers/   # HTTP headers and User-Agent helpers
├── fingerprint-dns/       # DNS service and resolver helpers
├── fingerprint-defense/   # Passive analysis and defense helpers
├── fingerprint-gateway/   # Actix gateway service
├── fingerprint-api-noise/ # Optional anti-fingerprinting helpers
└── fingerprint-ai-models/ # Adjacent provider/content detection crate
```

### Prototype crates in workspace

```text
crates/
├── fingerprint-audio/
├── fingerprint-canvas/
├── fingerprint-fonts/
├── fingerprint-hardware/
├── fingerprint-ml/
├── fingerprint-storage/
├── fingerprint-webgl/
└── fingerprint-webrtc/
```

### Experimental crates excluded from workspace

The following crates are kept in-repo but excluded from the workspace manifest: `fingerprint-analysis`, `fingerprint-anomaly`, `fingerprint-config`, `fingerprint-hardware-unified`, and `fingerprint-timing`.

## Build And Verification

```bash
cargo check
cargo build --workspace --release
cargo test --workspace --no-run
cargo test --workspace --lib
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
```

Notes:
- `cargo check` at the repository root now targets the stable default member set.
- Use `cargo check --workspace` or `cargo test --workspace ...` when you want full workspace validation, including preview/prototype crates.
- Some network-facing tests are intentionally ignored unless the required environment is available.
- `fingerprint-gateway` and some cache/rate-limit paths require Redis at runtime.
- `fingerprint-dns` and selected examples depend on network access or external APIs.

## Documentation

See the [docs](docs/) directory for:
- [User Guide](docs/en/user-guides/)
- [API Reference](docs/en/reference/)
- [Architecture](docs/en/ARCHITECTURE.md)
- [Developer Guides](docs/en/developer-guides/)
- [Examples](examples/)

## Contributing

See [CONTRIBUTING](docs/en/CONTRIBUTING.md).

## License

BSD-3-Clause. See [LICENSE](LICENSE).

---

# 中文版本

`fingerprint-rust` 是一个面向浏览器指纹生成、协议层指纹分析、被动检测以及相关网关/服务工具的 Rust workspace。

当前仓库分为三层：
- 已纳入默认 member 集和发布主路径的稳定 crate
- 已编译可用、但暂不纳入默认 member 集的预览/原型 crate
- 保留在仓库中、但暂未纳入 workspace 的实验 crate

## 核心能力

- 90+ 浏览器配置，覆盖 Chrome、Firefox、Safari、Opera、Edge
- TLS 指纹能力，包括 JA3、JA4、JARM、GREASE 处理与握手构建
- HTTP/1.1、HTTP/2，以及可选 HTTP/3 支持
- 浏览器配置、HTTP 头与 User-Agent 生成
- 可选 DNS、被动防护、API 噪声注入和网关模块

## 快速开始

```toml
[dependencies]
fingerprint = "2.1"
```

```rust
use fingerprint::{get_random_fingerprint, mapped_tls_clients};

let result = get_random_fingerprint().unwrap();
println!("Profile: {}", result.profile_id);
println!("User-Agent: {}", result.user_agent);
println!("Accept-Language: {}", result.headers.accept_language);

let profiles = mapped_tls_clients();
let chrome = profiles.get("chrome_133").unwrap();
println!("Profile ID: {}", chrome.id());
println!("密码套件数量: {}", chrome.tls_config.cipher_suites.len());
println!("HTTP/2 设置数: {}", chrome.http2_settings.len());
```

## 支持矩阵

| 范围 | 状态 | 说明 |
| --- | --- | --- |
| `fingerprint` | 稳定 | 推荐的公共入口 |
| `fingerprint-core` | 稳定 | 核心类型、解析器、JA3/JA4/JARM、缓存、指标、报文解析 |
| `fingerprint-tls` | 稳定 | TLS 配置、扩展与握手构建 |
| `fingerprint-http` | 稳定 | HTTP 客户端与协议指纹；`http3` 仍为可选能力 |
| `fingerprint-profiles` | 稳定 | 浏览器配置目录与版本适配 |
| `fingerprint-headers` | 稳定 | HTTP 头与 User-Agent 生成 |
| `fingerprint-dns` | 稳定 | 面向网络环境的 DNS 服务模块；真实行为依赖运行时网络/API |
| `fingerprint-defense` | 稳定 | 被动分析与防护辅助模块 |
| `fingerprint-gateway` | 稳定 | 基于 Actix 的服务 crate，带 Redis 限流 |
| `fingerprint-api-noise` | 稳定 | 可选的 API 噪声注入能力 |
| `fingerprint-ai-models` | 预览 | 已纳入 workspace，但不在默认 member 集内；和主浏览器指纹链路是相邻领域 |
| `fingerprint-audio` / `canvas` / `fonts` / `hardware` / `storage` / `webgl` / `webrtc` / `ml` | 原型 | 保留在 workspace 中供显式验证，但不在默认 member 集内；API 和启发式规则仍在演进 |
| `fingerprint-analysis` / `anomaly` / `config` / `hardware-unified` / `timing` | 实验 | 保留在仓库中，但未纳入 workspace |

## 工作区结构

### 推荐使用的 crate

```text
crates/
├── fingerprint/           # 主 facade crate
├── fingerprint-core/      # 共享核心类型和算法
├── fingerprint-tls/       # TLS 配置、扩展、握手构建
├── fingerprint-http/      # HTTP 客户端与协议支持
├── fingerprint-profiles/  # 浏览器配置目录
├── fingerprint-headers/   # HTTP 头和 User-Agent
├── fingerprint-dns/       # DNS 服务与解析辅助
├── fingerprint-defense/   # 被动分析与防护
├── fingerprint-gateway/   # Actix 网关服务
├── fingerprint-api-noise/ # 可选反指纹辅助
└── fingerprint-ai-models/ # 相邻领域的提供商/内容检测 crate
```

### workspace 中的原型 crate

```text
crates/
├── fingerprint-audio/
├── fingerprint-canvas/
├── fingerprint-fonts/
├── fingerprint-hardware/
├── fingerprint-ml/
├── fingerprint-storage/
├── fingerprint-webgl/
└── fingerprint-webrtc/
```

### 未纳入 workspace 的实验 crate

以下 crate 当前保留在仓库中，但在 workspace manifest 中被排除：`fingerprint-analysis`、`fingerprint-anomaly`、`fingerprint-config`、`fingerprint-hardware-unified`、`fingerprint-timing`。

## 构建与验证

```bash
cargo check
cargo build --workspace --release
cargo test --workspace --no-run
cargo test --workspace --lib
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
```

说明：
- 仓库根目录直接执行 `cargo check` 时，现在只会验证稳定的默认 member 集。
- 如果需要覆盖预览/原型 crate，请显式使用 `cargo check --workspace` 或 `cargo test --workspace ...`。
- 一部分真实联网测试会在缺少外部环境时被显式忽略。
- `fingerprint-gateway` 以及部分缓存/限流能力运行时需要 Redis。
- `fingerprint-dns` 和部分示例依赖网络访问或外部 API。

## 文档资源

详见 [docs](docs/) 目录：
- [用户指南](docs/zh/user-guides/)
- [API 参考](docs/zh/reference/)
- [架构设计](docs/zh/ARCHITECTURE.md)
- [开发指南](docs/zh/developer-guides/)
- [示例代码](examples/)

## 贡献指南

请查看 [CONTRIBUTING](docs/zh/CONTRIBUTING.md)。

## 许可证

BSD-3-Clause，详见 [LICENSE](LICENSE)。

---
**Version**: 2.1.0  
**Last Updated**: 2026-03-18
