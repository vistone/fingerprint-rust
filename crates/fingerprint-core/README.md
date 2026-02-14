# fingerprint-core

核心类型和基础工具函数库，提供 fingerprint-rust 项目的所有基础设施。

## 功能特性

- ✅ 基础数据结构定义（FingerprintData, BrowserInfo, etc.）
- ✅ 通用工具函数库
- ✅ 指纹计算的抽象接口
- ✅ 核心算法实现
- 🔧 可选的 Redis 缓存支持（`redis` 特性）
- 🔧 可选的数据库支持（`database` 特性）

## 快速开始

### 添加到 Cargo.toml

```toml
[dependencies]
fingerprint-core = { path = "../fingerprint-core" }
```

### 基本用法

```rust
use fingerprint_core::{FingerprintData, BrowserInfo};

// 创建浏览器信息
let info = BrowserInfo::new("Chrome", "120.0", "Windows");

// 创建指纹数据
let fingerprint = FingerprintData::new(info);
println!("Fingerprint ID: {}", fingerprint.id);
```

## API 概览

### 主要类型

| 类型 | 说明 |
|-----|------|
| `FingerprintData` | 指纹数据的核心结构 |
| `BrowserInfo` | 浏览器信息容器 |
| `FingerprintError` | 统一错误类型 |
| `FingerprintResult` | 操作结果类型别名 |

### 主要函数

| 函数 | 说明 |
|-----|------|
| `hash_fingerprint()` | 计算指纹哈希值 |
| `normalize_data()` | 标准化指纹数据 |
| `validate_fingerprint()` | 验证指纹有效性 |

## 项目结构

```
src/
├── lib.rs              # 库入口，包含模块声明
├── types.rs            # 基础数据类型定义
├── error.rs            # 错误类型和处理
├── utils.rs            # 工具函数库
├── hash.rs             # 哈希算法实现
└── cache.rs            # 缓存支持（可选）
```

## 依赖关系

| 依赖 | 用途 | 版本 |
|-----|------|------|
| `serde` | 序列化/反序列化 | ^1.0 |
| `sha2` | SHA-256 哈希 | ^0.10 |
| `redis` | Redis 缓存（可选） | ^0.23 |

## 可选特性

```toml
[features]
default = []
redis = ["dep:redis"]
database = ["sqlx"]
connection-pool = ["deadpool"]
```

启用特性示例：

```toml
fingerprint-core = { path = "../fingerprint-core", features = ["redis", "database"] }
```

## 使用示例

### 示例 1：基础指纹计算

```rust
use fingerprint_core::{FingerprintData, BrowserInfo};

let info = BrowserInfo {
    user_agent: "Mozilla/5.0...".to_string(),
    browser: "Chrome".to_string(),
    version: "120.0".to_string(),
    language: "en-US".to_string(),
};

let fingerprint = FingerprintData::from_browser_info(&info)?;
println!("Fingerprint: {:?}", fingerprint);
```

### 示例 2：使用缓存

```rust
use fingerprint_core::cache::{Cache, InMemoryCache};

let cache = InMemoryCache::new();
let key = "browser_fp_123";

// 存储
cache.set(key, fingerprint_data)?;

// 检索
let cached = cache.get(key)?;
```

## 架构设计

### 模块关系

```
┌─────────────────────────────┐
│    fingerprint-core         │
├─────────────────────────────┤
│  Types Module               │
│  ├─ FingerprintData        │
│  ├─ BrowserInfo            │
│  └─ Error Types            │
├─────────────────────────────┤
│  Utils Module               │
│  ├─ Hash Functions         │
│  ├─ Validation             │
│  └─ Conversion             │
├─────────────────────────────┤
│  Cache Module (Optional)    │
│  ├─ In-Memory Cache        │
│  └─ Redis Cache            │
└─────────────────────────────┘
```

## 性能指标

- 指纹计算速度：< 1ms per fingerprint
- 内存使用：约 2MB steady state
- 缓存命中率：>95% (with caching enabled)

## 局限性

- 不支持动态 JavaScript 执行
- 仅基于静态特征计算指纹
- 对时间戳敏感，需要定期更新

## 贡献指南

欢迎提交 Issue 和 Pull Request！

详见：[CONTRIBUTING.md](../../CONTRIBUTING.md)

## 许可证

本项目采用 MIT 许可证。详见：[LICENSE](../../LICENSE)

## 相关文档

- [Core API 文档](https://docs.rs/fingerprint-core)
- [架构设计](../../docs/ARCHITECTURE.md)
- [项目治理规范](../../PROJECT_GOVERNANCE.md)

---

**最后更新：** 2026年2月14日
