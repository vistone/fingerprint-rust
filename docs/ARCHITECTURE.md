# 架构设计文档

**版本**: v2.0.0 (Workspace)  
**最后更新**: 2025-12-14

---

## 📋 目录

1. [项目概述](#项目概述)
2. [Workspace 架构](#workspace-架构)
3. [Crate 职责划分](#crate-职责划分)
4. [依赖关系](#依赖关系)
5. [设计原则](#设计原则)
6. [文件组织](#文件组织)
7. [测试策略](#测试策略)
8. [性能考虑](#性能考虑)
9. [扩展性](#扩展性)

---

## 1. 项目概述

### 1.1 项目定位

`fingerprint-rust` 是一个**生产级**的浏览器 TLS 指纹库，采用 Cargo Workspace 架构，提供：

- **69+ 浏览器指纹配置**：Chrome、Firefox、Safari、Opera、Edge 等主流浏览器
- **完整 TLS 指纹生成**：ClientHello Spec、密码套件、扩展等
- **高性能 HTTP 客户端**：支持 HTTP/1.1、HTTP/2、HTTP/3
- **真实环境验证**：Google Earth API 端到端测试，100% 通过率

### 1.2 技术栈

- **语言**: Rust 2021 Edition
- **架构**: Cargo Workspace（7 个独立 crate）
- **TLS 实现**: rustls 0.21（可选），自研 TLS Handshake Builder
- **HTTP/2**: h2 0.4
- **HTTP/3**: quinn 0.10 + h3 0.0.4
- **异步运行时**: tokio 1.40
- **密码学库**: ring 0.17.14（真实密钥生成）
- **连接池**: netconnpool-rust（自定义）
- **DNS 解析**: hickory-resolver 0.24（可选）

---

## 2. Workspace 架构

### 2.1 目录结构

```
fingerprint-rust/
├── Cargo.toml                    # Workspace 根配置
├── crates/                        # 所有 crate 代码
│   ├── fingerprint-core/          # 核心类型和工具
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── types.rs           # 浏览器类型、操作系统类型
│   │       ├── utils.rs           # 工具函数
│   │       └── dicttls/           # TLS 字典（密码套件、扩展类型等）
│   │
│   ├── fingerprint-tls/          # TLS 配置、扩展和握手
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── tls_config/        # TLS ClientHello Spec
│   │       ├── tls_extensions.rs  # TLS 扩展实现
│   │       └── tls_handshake/     # TLS 握手消息构建
│   │
│   ├── fingerprint-profiles/     # 浏览器指纹配置
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       └── profiles.rs        # 69+ 个浏览器指纹配置
│   │
│   ├── fingerprint-headers/      # HTTP Headers 和 User-Agent
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── headers.rs         # HTTP 请求头生成
│   │       ├── useragent.rs        # User-Agent 生成
│   │       └── http2_config.rs    # HTTP/2 配置
│   │
│   ├── fingerprint-http/         # HTTP 客户端实现
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       └── http_client/       # HTTP/1.1、HTTP/2、HTTP/3
│   │
│   ├── fingerprint-dns/          # DNS 预解析服务（可选）
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       └── dns/               # DNS 解析器、服务器池等
│   │
│   └── fingerprint/              # 主库，重新导出所有功能
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs             # 重新导出所有公共 API
│           ├── random.rs           # 随机指纹生成
│           └── export.rs          # 配置导出功能
│
├── tests/                        # 集成测试
├── examples/                     # 示例代码
└── docs/                        # 文档
```

### 2.2 Workspace 配置

根目录 `Cargo.toml` 配置：

```toml
[workspace]
members = [
    "crates/fingerprint-core",
    "crates/fingerprint-tls",
    "crates/fingerprint-profiles",
    "crates/fingerprint-headers",
    "crates/fingerprint-http",
    "crates/fingerprint-dns",
    "crates/fingerprint",
]
resolver = "2"

[workspace.package]
version = "1.0.0"
edition = "2021"
# ... 其他公共配置

[workspace.dependencies]
# 所有依赖定义在这里，子 crate 通过 workspace = true 引用
rand = "0.8"
# ...
```

### 2.3 为什么使用 `crates/` 目录？

使用 `crates/` 目录是 Rust Workspace 项目的**标准实践**：

- ✅ **社区约定**：tokio、serde、hyper 等知名项目都使用 `crates/`
- ✅ **语义清晰**：直接表示"多个 crate 的集合"
- ✅ **结构清晰**：与根目录文件区分明显
- ✅ **易于扩展**：添加新 crate 不会让根目录变乱

详细说明请查看 [WHY_CRATES_DIRECTORY.md](WHY_CRATES_DIRECTORY.md)

---

## 3. Crate 职责划分

### 3.1 fingerprint-core

**职责**: 核心类型和工具函数

**代码位置**: `crates/fingerprint-core/src/`

**包含模块**:
- `types.rs`: 浏览器类型、操作系统类型等核心类型
  - `BrowserType`: 浏览器类型枚举
  - `OperatingSystem`: 操作系统类型枚举
  - `UserAgentTemplate`: User-Agent 模板结构
- `utils.rs`: 工具函数
  - `random_choice`: 线程安全的随机选择
  - `extract_chrome_version`: 从 User-Agent 提取 Chrome 版本
  - `extract_platform`: 从 User-Agent 提取平台信息
  - `infer_browser_from_profile_name`: 从 profile 名称推断浏览器类型
  - `is_mobile_profile`: 判断是否为移动端 profile
- `dicttls/`: TLS 字典模块
  - `cipher_suites.rs`: 密码套件常量
  - `extensions.rs`: 扩展类型常量
  - `signature_schemes.rs`: 签名算法常量
  - `supported_groups.rs`: 椭圆曲线常量

**依赖**:
- `rand`: 随机数生成
- `once_cell`: 延迟初始化

**公共 API**:
```rust
pub use types::{BrowserType, OperatingSystem, OperatingSystems, UserAgentTemplate};
pub use utils::{random_choice, extract_chrome_version, extract_platform, ...};
pub use dicttls::*;
```

### 3.2 fingerprint-tls

**职责**: TLS 配置、扩展和握手

**代码位置**: `crates/fingerprint-tls/src/`

**包含模块**:
- `tls_config/`: TLS ClientHello Spec 生成、比较、分析
  - `spec.rs`: ClientHelloSpec 定义
  - `builder.rs`: Builder 模式构建器
  - `ja4.rs`: JA4 指纹生成
  - `comparison.rs`: 指纹比较
  - `extract.rs`: 签名提取
  - `grease.rs`: GREASE 值处理
  - `signature.rs`: 签名结构
  - `stats.rs`: 统计信息
  - `observable.rs`: 可观察性
  - `metadata.rs`: 元数据
  - `version.rs`: TLS 版本
- `tls_extensions.rs`: TLS 扩展实现
  - `SNIExtension`: SNI 扩展
  - `KeyShareExtension`: KeyShare 扩展
  - `SupportedVersionsExtension`: 支持的 TLS 版本
  - 等等...
- `tls_handshake/`: TLS 握手消息构建
  - `builder.rs`: TLS Handshake Builder
  - `messages.rs`: ClientHello 消息结构
  - `handshake.rs`: 握手消息
  - `record.rs`: TLS 记录层

**依赖**:
- `fingerprint-core`: 核心类型和字典
- `sha2`: 哈希函数（JA4 指纹）
- `thiserror`: 错误处理
- `ring` (optional): 真实密钥生成
- `rand` (optional): 随机数生成（用于握手）

**公共 API**:
```rust
pub use tls_config::*;
pub use tls_extensions::*;
pub use tls_handshake::TLSHandshakeBuilder;
```

### 3.3 fingerprint-profiles

**职责**: 浏览器指纹配置管理

**代码位置**: `crates/fingerprint-profiles/src/`

**包含模块**:
- `profiles.rs`: 69+ 个浏览器指纹配置
  - `ClientProfile`: TLS 指纹配置结构
  - `ClientHelloID`: 浏览器标识
  - `mapped_tls_clients()`: 全局指纹配置映射表
  - 各种浏览器的指纹配置函数（chrome_103, chrome_133, firefox_133, 等）

**依赖**:
- `fingerprint-core`: 核心类型
- `fingerprint-tls`: TLS 配置
- `fingerprint-headers`: HTTP/2 配置

**公共 API**:
```rust
pub use profiles::{
    chrome_103, chrome_133, firefox_133, safari_16_0, opera_91,
    edge_120, edge_124, edge_133, ClientHelloID, ClientProfile,
    mapped_tls_clients, default_client_profile,
};
```

### 3.4 fingerprint-headers

**职责**: HTTP Headers 和 User-Agent 生成

**代码位置**: `crates/fingerprint-headers/src/`

**包含模块**:
- `headers.rs`: HTTP 请求头生成
  - `HTTPHeaders`: HTTP 请求头结构
  - `generate_headers`: 根据浏览器类型生成标准 Headers
  - `random_language`: 随机选择语言（30+ 种语言）
- `useragent.rs`: User-Agent 生成
  - `UserAgentGenerator`: User-Agent 生成器
  - `get_user_agent_by_profile_name`: 根据 profile 名称获取 User-Agent
  - `random_os`: 随机选择操作系统
- `http2_config.rs`: HTTP/2 配置
  - `HTTP2Settings`: HTTP/2 Settings
  - `chrome_http2_settings`: Chrome HTTP/2 配置
  - `firefox_http2_settings`: Firefox HTTP/2 配置
  - `safari_http2_settings`: Safari HTTP/2 配置
  - `chrome_pseudo_header_order`: Chrome 伪头部顺序
  - `chrome_header_priority`: Chrome 头部优先级

**依赖**:
- `fingerprint-core`: 核心类型和工具
- `rand`: 随机数生成
- `once_cell`: 延迟初始化

**公共 API**:
```rust
pub use headers::{generate_headers, random_language, HTTPHeaders};
pub use useragent::{get_user_agent_by_profile_name, random_os, UserAgentGenerator};
pub use http2_config::{chrome_http2_settings, HTTP2Settings, ...};
```

### 3.5 fingerprint-http

**职责**: HTTP 客户端实现（HTTP/1.1、HTTP/2、HTTP/3）

**代码位置**: `crates/fingerprint-http/src/http_client/`

**包含模块**:
- `mod.rs`: HTTP 客户端主类
  - `HttpClient`: HTTP 客户端主类
  - `HttpClientConfig`: 客户端配置
  - `HttpClientError`: 错误类型
- `http1.rs`: HTTP/1.1 实现
  - TCP 连接管理
  - TLS 支持（rustls）
  - Chunked encoding 处理
  - Gzip/Deflate/Brotli 解压
  - HTTP 重定向
  - Keep-Alive
- `http2.rs`: HTTP/2 实现
  - ALPN 协议协商
  - 多路复用
  - HPACK 压缩
  - Server Push
  - 浏览器特定的 Settings 和 Priority
- `http3.rs`: HTTP/3 实现
  - QUIC 协议
  - UDP 传输
  - TLS 1.3
  - 0-RTT 连接
  - 连接迁移
- `http1_pool.rs`, `http2_pool.rs`, `http3_pool.rs`: 连接池实现
- `pool.rs`: 连接池管理（与 netconnpool 集成）
- `response.rs`: HTTP 响应解析
- `request.rs`: HTTP 请求构建器
- `cookie.rs`: Cookie 管理
- `proxy.rs`: 代理支持
- `rustls_client_hello_customizer.rs`: 通过 ClientHelloCustomizer 应用浏览器指纹
- `rustls_utils.rs`: rustls 工具函数
- `tls.rs`: TLS 连接器
- `io.rs`: IO 工具
- `reporter.rs`: 报告生成

**依赖**:
- `fingerprint-core`: 核心类型
- `fingerprint-tls`: TLS 配置
- `fingerprint-profiles`: 指纹配置
- `fingerprint-headers`: HTTP Headers
- `rustls`, `h2`, `quinn`, `h3` (optional): HTTP 协议实现
- `netconnpool` (optional): 连接池

**公共 API**:
```rust
pub use http_client::{
    HttpClient, HttpClientConfig, HttpClientError,
    HttpMethod, HttpRequest, HttpResponse,
    Cookie, CookieStore, ProxyConfig, TlsConnector,
    ValidationReport, ReportFormat, ReportSection,
};
```

### 3.6 fingerprint-dns

**职责**: DNS 预解析服务（可选功能）

**代码位置**: `crates/fingerprint-dns/src/dns/`

**包含模块**:
- `service.rs`: DNS 服务主接口
  - `Service`: DNS 服务（start/stop）
- `resolver.rs`: DNS 解析器
  - `DNSResolver`: 高并发 DNS 查询
- `serverpool.rs`: DNS 服务器池管理
  - `ServerPool`: DNS 服务器池
- `collector.rs`: DNS 服务器收集器
  - `ServerCollector`: 自动收集 DNS 服务器
- `ipinfo.rs`: IP 地理信息客户端
  - `IPInfoClient`: IPInfo.io 客户端
- `storage.rs`: 数据存储
  - 多格式支持（JSON/YAML/TOML）
  - 原子性写入
- `config.rs`: 配置加载
- `types.rs`: 类型定义

**依赖**:
- `fingerprint-core`: 核心类型
- `fingerprint-http`: HTTP 客户端（用于 IPInfo API）
- `hickory-resolver`: DNS 解析
- `serde`, `toml`, `serde_yaml`: 配置解析
- `tokio`, `futures`: 异步运行时

**公共 API**:
```rust
pub use dns::{
    Service as DNSService, DNSResolver, ServerCollector,
    ServerPool, IPInfoClient, DNSConfig, DNSResult,
    DomainIPs, IPInfo, DNSError,
};
```

### 3.7 fingerprint

**职责**: 主库，重新导出所有功能

**代码位置**: `crates/fingerprint/src/`

**包含模块**:
- `lib.rs`: 重新导出所有公共 API
- `random.rs`: 随机指纹生成
  - `get_random_fingerprint`: 随机获取指纹
  - `get_random_fingerprint_by_browser`: 根据浏览器类型获取指纹
  - `FingerprintResult`: 指纹结果结构
- `export.rs`: 配置导出功能
  - `export_config_json`: 导出配置为 JSON

**依赖**:
- 所有其他 crate

**公共 API**:
```rust
// 重新导出所有功能，保持向后兼容
pub use fingerprint_core::*;
pub use fingerprint_tls::*;
pub use fingerprint_profiles::*;
pub use fingerprint_headers::*;
pub use fingerprint_http::*;
pub use random::*;
```

---

## 4. 依赖关系

### 4.1 依赖图

```
fingerprint (主库)
├── fingerprint-core (核心)
│   ├── rand
│   └── once_cell
│
├── fingerprint-tls
│   ├── fingerprint-core
│   ├── sha2
│   ├── thiserror
│   └── ring (optional)
│
├── fingerprint-profiles
│   ├── fingerprint-core
│   ├── fingerprint-tls
│   └── fingerprint-headers
│
├── fingerprint-headers
│   ├── fingerprint-core
│   ├── rand
│   └── once_cell
│
├── fingerprint-http
│   ├── fingerprint-core
│   ├── fingerprint-tls
│   ├── fingerprint-profiles
│   ├── fingerprint-headers
│   ├── rustls (optional)
│   ├── h2 (optional)
│   ├── quinn (optional)
│   └── netconnpool (optional)
│
└── fingerprint-dns (可选)
    ├── fingerprint-core
    ├── fingerprint-http
    ├── hickory-resolver
    └── serde, toml, serde_yaml
```

### 4.2 依赖管理

**Workspace 依赖**:
- 所有依赖定义在根 `Cargo.toml` 的 `[workspace.dependencies]` 中
- 子 crate 通过 `dependency.workspace = true` 引用

**示例**:
```toml
# 根 Cargo.toml
[workspace.dependencies]
rand = "0.8"

# 子 crate Cargo.toml
[dependencies]
rand.workspace = true
```

---

## 5. 设计原则

### 5.1 职责单一

- 每个 crate 只负责一个明确的功能领域
- Crate 之间保持相互独立
- 仅在业务整合层（fingerprint crate）进行组合

### 5.2 输入输出清晰

- 每个函数都有明确的输入参数和返回值
- 使用 Rust 的类型系统确保类型安全
- 错误处理使用 `Result` 类型

### 5.3 避免不必要的嵌套与耦合

- Crate 之间通过公共接口交互
- 使用 trait 和枚举实现多态
- 避免深层嵌套结构

### 5.4 线程安全

- 使用 `OnceLock` 实现线程安全的单例
- 随机数生成使用线程本地随机数生成器
- 所有公共 API 都是线程安全的

### 5.5 性能优化

- 使用 `HashMap` 进行快速查找
- 避免不必要的克隆
- 使用引用传递减少内存分配
- 支持并行编译（Workspace 架构）

---

## 6. 文件组织

### 6.1 源代码组织

```
crates/
├── fingerprint-core/src/
│   ├── lib.rs
│   ├── types.rs
│   ├── utils.rs
│   └── dicttls/
│
├── fingerprint-tls/src/
│   ├── lib.rs
│   ├── tls_config/
│   ├── tls_extensions.rs
│   └── tls_handshake/
│
├── fingerprint-profiles/src/
│   ├── lib.rs
│   └── profiles.rs
│
├── fingerprint-headers/src/
│   ├── lib.rs
│   ├── headers.rs
│   ├── useragent.rs
│   └── http2_config.rs
│
├── fingerprint-http/src/
│   ├── lib.rs
│   └── http_client/
│
├── fingerprint-dns/src/
│   ├── lib.rs
│   └── dns/
│
└── fingerprint/src/
    ├── lib.rs
    ├── random.rs
    └── export.rs
```

### 6.2 测试组织

```
tests/
├── integration_test.rs          # 集成测试
├── http_client_test.rs          # HTTP 客户端测试
├── dns_service_test.rs          # DNS 服务测试
└── ...
```

### 6.3 示例组织

```
examples/
├── basic.rs                     # 基础使用示例
├── custom_tls_fingerprint.rs    # 自定义 TLS 指纹
├── http2_with_pool.rs           # HTTP/2 + 连接池
├── http3_with_pool.rs           # HTTP/3 + 连接池
├── dns_service.rs               # DNS 服务示例
└── ...
```

---

## 7. 测试策略

### 7.1 单元测试

- 每个 crate 都包含单元测试
- 测试覆盖核心功能
- 使用 `#[cfg(test)]` 标记测试代码

### 7.2 集成测试

- `tests/` 目录包含全面的集成测试
- 测试所有公共 API
- 测试并发安全性
- 测试边界情况

### 7.3 测试覆盖

- ✅ 随机指纹获取
- ✅ 指定浏览器类型获取指纹
- ✅ User-Agent 生成
- ✅ HTTP Headers 生成和管理
- ✅ TLS 指纹生成和比较
- ✅ HTTP/1.1、HTTP/2、HTTP/3 客户端
- ✅ 连接池功能
- ✅ DNS 预解析服务
- ✅ 并发访问安全性
- ✅ 错误处理

### 7.4 测试结果

- **总测试数**: 74 个
- **通过**: 74 个
- **失败**: 0 个
- **成功率**: 100%

---

## 8. 性能考虑

### 8.1 编译性能

- **并行编译**: Workspace 支持并行编译多个 crate
- **增量编译**: 只重新编译修改的 crate
- **预计提升**: 30-50% 编译速度提升

### 8.2 运行时性能

- **零分配操作**: 关键路径避免不必要的内存分配
- **快速查找**: 使用 HashMap 进行 O(1) 查找
- **线程安全**: 使用线程本地随机数生成器，避免锁竞争
- **延迟初始化**: 使用 `OnceLock` 实现延迟初始化

### 8.3 HTTP 客户端性能

| 协议 | 平均响应时间 | 最小 | 最大 | 成功率 |
|------|--------------|------|------|--------|
| **HTTP/3** | 40.3ms | 35ms | 48ms | 100% 🥇 |
| **HTTP/1.1** | 44.4ms | 37ms | 79ms | 100% 🥈 |
| **HTTP/2** | 48.0ms | 43ms | 60ms | 100% 🥉 |

---

## 9. 扩展性

项目设计支持以下扩展：

### 9.1 添加新浏览器指纹

在 `crates/fingerprint-profiles/src/profiles.rs` 中添加新的配置函数：

```rust
pub fn chrome_134() -> ClientProfile {
    // ...
}
```

### 9.2 添加新 User-Agent 模板

在 `crates/fingerprint-headers/src/useragent.rs` 的 `init_templates` 中添加。

### 9.3 添加新语言

在 `crates/fingerprint-headers/src/headers.rs` 的 `LANGUAGES` 数组中添加。

### 9.4 添加新操作系统

在 `crates/fingerprint-core/src/types.rs` 的 `OperatingSystem` 枚举中添加。

### 9.5 添加新 Crate

1. 在 `crates/` 目录下创建新 crate
2. 在根 `Cargo.toml` 的 `[workspace]` 中添加成员
3. 配置依赖关系

---

## 10. 构建和测试

### 10.1 构建所有 crate

```bash
# 构建整个 workspace
cargo build --workspace

# 构建特定 crate
cargo build -p fingerprint-core
cargo build -p fingerprint-http --features "rustls-tls,http2"
```

### 10.2 运行测试

```bash
# 测试整个 workspace
cargo test --workspace

# 测试特定 crate
cargo test -p fingerprint-core
cargo test -p fingerprint-http --features "rustls-tls,http2"
```

### 10.3 检查编译

```bash
# 检查整个 workspace
cargo check --workspace

# 检查特定 crate
cargo check -p fingerprint-tls
```

---

**文档版本**: v2.0.0  
**最后更新**: 2025-12-14
