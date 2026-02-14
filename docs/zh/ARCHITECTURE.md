# 架构设计文档

**版本**: v2.1.0（具有主动/被动防御的工作区）  
**最后更新**: 2026-02-13

---

## 📋 目录

1. [项目概述](#项目概述)
2. [工作区架构](#工作区架构)
3. [Crate 职责](#crate-职责)
4. [依赖关系](#依赖关系)
5. [设计原则](#设计原则)
6. [文件组织](#文件组织)
7. [测试策略](#测试策略)
8. [性能考虑](#性能考虑)
9. [可扩展性](#可扩展性)

---

## 1. 项目概述

### 1.1 项目定位

`fingerprint-rust` 是一个**生产就绪**的浏览器指纹库，使用 Cargo 工作区架构，提供：

- **97+ 浏览器指纹配置文件**: Chrome、Firefox、Safari、Opera、Edge 以及主流浏览器及其移动变体
- **完整的 TLS 指纹生成**: ClientHello 规范、密码套件、扩展等
- **高性能 HTTP 客户端**: 支持 HTTP/1.1、HTTP/2、HTTP/3（QUIC）
- **真实环境验证**: Google Earth API 端到端测试，100% 通过率
- **机器学习分类**: 三层分层分类器架构，准确率 95% 以上
- **被动识别防御**: JA4+ 全栈指纹识别和威胁检测

### 1.2 技术栈

- **语言**: Rust 1.92.0+
- **架构**: Cargo 工作区（20 个独立 crate）
- **TLS实现**: rustls 0.23（可选）、自研 TLS Handshake Builder
- **HTTP/2**: h2 0.4
- **HTTP/3**: quinn 0.11 + h3 0.0.8
- **异步运行时**: tokio 1.40
- **密码库**: ring 0.17.14（真实密钥生成）
- **连接池**: netconnpool-rust（自定义）
- **DNS 解析**: hickory-resolver 0.24（可选）
- **机器学习**: candle-core 0.8（Rust ML 框架）

---

## 2. 工作区架构

### 2.1 目录结构

```
fingerprint-rust/
├── Cargo.toml                    # 工作区根配置
├── crates/                        # 所有 crate 代码
│   ├── fingerprint-core/          # 系统级保护核心抽象层
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── types.rs           # 核心类型定义
│   │       ├── utils.rs           # 实用函数
│   │       └── traits.rs          # 核心特质定义
│   │
│   ├── fingerprint-tls/          # TLS 配置、扩展和握手
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── tls_config/        # TLS ClientHello 规范
│   │       ├── tls_extensions.rs  # TLS 扩展实现
│   │       └── tls_handshake/     # TLS 握手消息构建
│   │
│   ├── fingerprint-profiles/     # 浏览器指纹配置模块
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       └── profiles.rs        # 97+ 浏览器指纹配置函数
│   │
│   ├── fingerprint-headers/      # HTTP 头部和用户代理生成
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── headers.rs         # HTTP 请求头生成
│   │       ├── useragent.rs       # 用户代理生成
│   │       └── http2_config.rs    # HTTP/2 配置
│   │
│   ├── fingerprint-http/         # HTTP 客户端实现
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       └── http_client/       # HTTP/1.1、HTTP/2、HTTP/3 支持
│   │
│   ├── fingerprint-dns/          # DNS 预解析服务
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       └── dns_resolver.rs    # DNS 解析器实现
│   │
│   ├── fingerprint-defense/      # 系统级保护实现层
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── passive/           # 被动分析器（TCP/HTTP/TLS/JA4+）
│   │       ├── consistency/       # 跨层一致性审计
│   │       ├── database/          # 指纹数据库（SQLite）
│   │       ├── learner/           # 自学习机制
│   │       └── capture/           # 数据包捕获
│   │
│   ├── fingerprint-anomaly/      # 异常检测模块
│   │   └── src/ - ML 异常检测实现
│   │
│   ├── fingerprint-canvas/       # Canvas 指纹识别
│   ├── fingerprint-webgl/        # WebGL 指纹识别
│   ├── fingerprint-audio/        # 音频上下文指纹
│   ├── fingerprint-fonts/        # 字体枚举检测
│   ├── fingerprint-webrtc/       # WebRTC IP 泄露检测
│   ├── fingerprint-hardware/     # 硬件能力检测
│   ├── fingerprint-timing/       # 时序攻击保护
│   ├── fingerprint-storage/      # 存储指纹识别
│   ├── fingerprint-ml/           # 机器学习指纹匹配
│   ├── fingerprint-api-noise/    # API 噪声注入
│   │
│   ├── fingerprint-gateway/      # 高性能 API 网关
│   │
│   └── fingerprint/              # 独立浏览器 TLS 指纹库
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs
│           └── fingerprint.rs     # 统一公共 API
│
├── examples/                      # 使用示例
├── tests/                         # 集成测试
├── docs/                          # 文档
├── config/                        # 配置文件
└── output/                        # 输出文件
```

### 2.2 工作区配置

根 `Cargo.toml` 配置：

```toml
[workspace]
members = [
    "crates/fingerprint-core",
    "crates/fingerprint-tls",
    "crates/fingerprint-profiles",
    "crates/fingerprint-headers",
    "crates/fingerprint-http",
    "crates/fingerprint-dns",
    "crates/fingerprint-defense",
    "crates/fingerprint-api-noise",
    "crates/fingerprint-gateway",
    "crates/fingerprint",
    "crates/fingerprint-canvas",
    "crates/fingerprint-webgl",
    "crates/fingerprint-audio",
    "crates/fingerprint-fonts",
    "crates/fingerprint-storage",
    "crates/fingerprint-webrtc",
    "crates/fingerprint-hardware",
    "crates/fingerprint-timing",
    "crates/fingerprint-ml",
    "crates/fingerprint-anomaly",
]
resolver = "2"

[workspace.package]
version = "1.0.0"
edition = "2021"
# ... 其他常见配置

[workspace.dependencies]
# 所有依赖在此定义，子 crate 通过 workspace = true 引用
rand = "0.8"
# ...
```

### 2.3 为什么使用 `crates/` 目录？

在 Rust 工作区项目中使用 `crates/` 目录是**标准做法**：

- ✅ **社区约定**: 流行项目如 tokio、serde、hyper 都使用 `crates/`
- ✅ **语义明确**: 直接表示"多个 crate 的集合"
- ✅ **结构清晰**: 从视觉上区分根目录文件
- ✅ **易于扩展**: 添加新 crate 不会混乱根目录

---

## 3. Crate 职责

### 3.1 fingerprint-core

**职责**: 核心类型和实用函数  
**代码位置**: `crates/fingerprint-core/src/`  
**包含模块**: types、utils、dicttls

### 3.2 fingerprint-tls

**职责**: TLS 配置、扩展和握手  
**代码位置**: `crates/fingerprint-tls/src/`  
**包含模块**: tls_config、tls_extensions、tls_handshake

### 3.3 fingerprint-profiles

**职责**: 浏览器指纹配置管理  
**代码位置**: `crates/fingerprint-profiles/src/`  
**包含模块**: profiles.rs，包含 69+ 浏览器指纹配置

### 3.4 fingerprint-headers

**职责**: HTTP 头部和用户代理生成  
**代码位置**: `crates/fingerprint-headers/src/`  
**包含模块**: headers、useragent、http2_config

### 3.5 fingerprint-http

**职责**: HTTP 客户端实现（HTTP/1.1、HTTP/2、HTTP/3）  
**代码位置**: `crates/fingerprint-http/src/http_client/`  
**包含模块**: http1、http2、http3、连接池管理、响应解析

### 3.6 fingerprint-dns

**职责**: DNS 预解析服务（可选功能）  
**代码位置**: `crates/fingerprint-dns/src/dns/`  
**包含模块**: service、resolver、服务器池、收集器、IP 信息

### 3.7 fingerprint-defense

**职责**: 全栈被动指纹识别和主动一致性审计  
**代码位置**: `crates/fingerprint-defense/src/`  
**包含模块**:
- `passive/`: TCP、TLS、HTTP 分析
- `database.rs`: 基于 SQLite 的流量持久化
- `learner.rs`: 自学习机制
- `capture/`: 数据包捕获引擎

### 3.8 其他扩展 Crate

补充前端和特征维度指纹识别能力：
- `fingerprint-api-noise`: API 噪声生成
- `fingerprint-gateway`: Rust API 网关
- `fingerprint-canvas`: Canvas 指纹识别
- `fingerprint-webgl`: WebGL 指纹识别
- `fingerprint-audio`: 音频指纹识别
- `fingerprint-fonts`: 字体指纹识别
- `fingerprint-storage`: 存储指纹识别
- `fingerprint-webrtc`: WebRTC 指纹识别
- `fingerprint-hardware`: 硬件指纹识别
- `fingerprint-timing`: 时序指纹识别
- `fingerprint-ml`: ML 指纹分析
- `fingerprint-anomaly`: 异常检测

### 3.9 fingerprint

**职责**: 主库，重新导出所有功能  
**代码位置**: `crates/fingerprint/src/`  
**函数**: 随机指纹生成、配置导出

---

## 4. 依赖关系

### 4.1 依赖图

```
fingerprint（主库）
├── fingerprint-core
├── fingerprint-tls
├── fingerprint-profiles
├── fingerprint-headers
├── fingerprint-http
└── fingerprint-dns（可选）
└── fingerprint-defense（可选）
```

### 4.2 依赖管理

- 所有依赖在根 `Cargo.toml` 的 `[workspace.dependencies]` 下定义
- 子 crate 通过 `dependency.workspace = true` 引用

---

## 5. 设计原则

### 5.1 单一职责
每个 crate 仅负责一个清晰的功能域

### 5.2 清晰的输入和输出
每个函数都有清晰的输入参数和返回值

### 5.3 避免不必要的嵌套和耦合
Crate 通过使用特质和枚举的公共接口进行交互

### 5.4 线程安全
所有公共 API 都使用适当的同步原语实现线程安全

### 5.5 性能优化
- 使用 HashMap 进行快速查找
- 避免不必要的克隆
- 支持并行编译

---

## 6. 文件组织

### 6.1 源代码组织

```
crates/
├── fingerprint-core/src/
├── fingerprint-tls/src/
├── fingerprint-profiles/src/
├── fingerprint-headers/src/
├── fingerprint-http/src/
├── fingerprint-dns/src/
└── fingerprint/src/
```

### 6.2 测试组织

```
tests/
├── integration_test.rs
├── http_client_test.rs
├── dns_service_test.rs
└── ...
```

### 6.3 示例组织

```
examples/
├── basic.rs
├── custom_tls_fingerprint.rs
├── http2_with_pool.rs
├── http3_with_pool.rs
└── dns_service.rs
```

---

## 7. 测试策略

### 7.1 单元测试
每个 crate 都包括覆盖核心功能的单元测试

### 7.2 集成测试
在 `tests/` 目录中进行全面测试，覆盖所有公共 API

### 7.3 测试覆盖
- ✅ 随机指纹检索
- ✅ 按浏览器类型检索指纹
- ✅ 用户代理生成
- ✅ HTTP 头部生成
- ✅ TLS 指纹生成
- ✅ HTTP/1.1、HTTP/2、HTTP/3 客户端
- ✅ 连接池功能
- ✅ DNS 服务
- ✅ 并发访问安全
- ✅ 错误处理

### 7.4 测试结果
- **总测试数**: 74
- **通过**: 74
- **失败**: 0
- **成功率**: 100%

---

## 8. 性能考虑

### 8.1 编译性能
- **并行编译**: 工作区支持多个 crate 的并行编译
- **增量编译**: 仅重新编译修改的 crate
- **预期改进**: 编译速度提升 30-50%

### 8.2 运行时性能
- **零分配操作**: 关键路径避免不必要的内存分配
- **快速查找**: 使用 HashMap 进行 O(1) 查找
- **线程安全**: 使用线程本地随机数生成器
- **延迟初始化**: 使用 `OnceLock` 进行延迟初始化

### 8.3 HTTP 客户端性能

| 协议 | 平均响应时间 | 最小值 | 最大值 | 成功率 |
|----------|----------------------|-----|-----|--------------|
| **HTTP/3** | 40.3ms | 35ms | 48ms | 100% 🥇 |
| **HTTP/1.1** | 44.4ms | 37ms | 79ms | 100% 🥈 |
| **HTTP/2** | 48.0ms | 43ms | 60ms | 100% 🥉 |

---

## 9. 可扩展性

项目设计支持以下扩展：

### 9.1 添加新浏览器指纹
在 `crates/fingerprint-profiles/src/profiles.rs` 中添加函数

### 9.2 添加新用户代理模板
更新 `crates/fingerprint-headers/src/useragent.rs`

### 9.3 添加新语言
添加到 `crates/fingerprint-headers/src/headers.rs` 中的 `LANGUAGES` 数组

### 9.4 添加新操作系统
添加到 `crates/fingerprint-core/src/types.rs` 中的 `OperatingSystem` 枚举

### 9.5 添加新 Crate
1. 在 `crates/` 目录下创建新 crate
2. 在根 `Cargo.toml` 的 `[workspace]` 中添加成员
3. 配置依赖关系

---

## 10. 构建和测试

### 10.1 构建所有 Crate

```bash
# 构建整个工作区
cargo build --workspace

# 构建特定 crate
cargo build -p fingerprint-core
cargo build -p fingerprint-http --features "rustls-tls,http2"
```

### 10.2 运行测试

```bash
# 测试整个工作区
cargo test --workspace

# 测试特定 crate
cargo test -p fingerprint-core
```

### 10.3 检查编译

```bash
# 检查整个工作区
cargo check --workspace
```

---

**文档版本**: v2.1.0  
**最后更新**: 2026-02-13
