# Rust Crates 工作区

这是 fingerprint-rust 项目的 Rust 工作区，包含稳定主链、预览模块和原型模块。

## Workspace Scope

当前 `Cargo.toml` 将稳定主链 crate 设为默认 member 集，用于根目录下的默认 `cargo check` / `cargo build`。预览和原型 crate 仍保留在 workspace 内，可通过 `--workspace` 或 `-p` 显式验证。以下 crate 仍保留在仓库中，但已在 workspace 中显式 `exclude`（实验/草稿状态，暂不纳入 CI 强制门禁）：

- `fingerprint-analysis`
- `fingerprint-anomaly`
- `fingerprint-config`
- `fingerprint-hardware-unified`
- `fingerprint-timing`

## 📦 Crate 结构

### 核心模块
- **fingerprint-core** - 核心类型和工具函数
- **fingerprint-tls** - TLS配置和握手实现
- **fingerprint-profiles** - 浏览器指纹配置管理
- **fingerprint-http** - HTTP客户端实现

### 扩展模块
- **fingerprint-canvas** - Canvas指纹识别
- **fingerprint-webgl** - WebGL指纹识别
- **fingerprint-audio** - Audio指纹识别
- **fingerprint-fonts** - Font指纹识别
- **fingerprint-storage** - Storage指纹识别

### 网络模块
- **fingerprint-dns** - DNS预解析服务
- **fingerprint-headers** - HTTP头部处理
- **fingerprint-gateway** - API网关实现

### 安全模块
- **fingerprint-defense** - 被动识别与主动防护
- **fingerprint-anomaly** - 异常检测模块
- **fingerprint-ml** - 机器学习组件

### 系统模块
- **fingerprint-hardware** - 硬件指纹识别
- **fingerprint-timing** - 时间特征分析
- **fingerprint-webrtc** - WebRTC指纹识别
- **fingerprint-api-noise** - API噪声生成

## 🎯 模块职责划分

### 核心层 (Core Layer)
```
fingerprint-core/
├── 基础类型定义
├── 工具函数库
├── 指纹抽象接口
└── 核心算法实现
```

### 协议层 (Protocol Layer)
```
fingerprint-tls/    # TLS协议实现
fingerprint-http/   # HTTP协议实现
fingerprint-dns/    # DNS协议实现
```

### 应用层 (Application Layer)
```
fingerprint-profiles/  # 浏览器配置管理
fingerprint-gateway/   # API网关服务
fingerprint-defense/   # 安全防护功能
```

### 扩展层 (Extension Layer)
```
fingerprint-canvas/    # Canvas指纹
fingerprint-webgl/     # WebGL指纹
fingerprint-audio/     # Audio指纹
...
```

## 🔧 开发指南

### 构建项目
```bash
# 构建默认稳定主链
cargo build

# 构建所有 workspace crate（含预览/原型）
cargo build --workspace

# 构建特定crate
cargo build -p fingerprint-core

# 带功能标志构建
cargo build --workspace --features "rustls-tls,http2,http3"
```

### 运行测试
```bash
# 运行所有 workspace 测试
cargo test --workspace

# 运行特定crate测试
cargo test -p fingerprint-core

# 运行文档测试
cargo test --doc
```

### 代码质量检查
```bash
# 检查默认稳定主链
cargo check

# 格式化代码
cargo fmt --all

# 检查代码风格
cargo clippy --workspace

# 生成文档
cargo doc --workspace --open
```

## 📊 依赖管理

### 工作区依赖
所有crate共享工作区级别的依赖配置：
- 统一的版本管理
- 共享的开发依赖
- 一致的编译配置

### Crate间依赖
```
fingerprint-tls ──┐
fingerprint-http ─┼── fingerprint-core
fingerprint-dns ──┘

fingerprint-profiles ── fingerprint-core
fingerprint-gateway ─── fingerprint-core
```

## 🚀 性能优化

### 编译优化
```toml
# Cargo.toml 配置
[profile.release]
opt-level = 3
lto = "fat"
codegen-units = 1
strip = true
```

### 内存管理
- 使用 `Box<T>` 减少栈分配
- 合理使用生命周期避免内存泄漏
- 利用 `Arc<T>` 实现安全的共享所有权

## 📈 监控和调试

### 性能监控
- 集成性能基准测试
- 实现运行时性能统计
- 提供详细的性能分析工具

### 调试支持
```rust
// 启用调试日志
RUST_LOG=debug cargo run

// 启用特定模块日志
RUST_LOG=fingerprint_core=trace cargo run
```

## 🤝 贡献指南

### 添加新功能
1. 在合适的crate中实现功能
2. 编写完整的单元测试
3. 更新相关文档
4. 通过所有质量检查

### 代码审查要点
- 遵循Rust编程规范
- 保持API的一致性
- 确保良好的错误处理
- 提供充分的文档注释

## 🔒 安全考虑

### 内存安全
- 充分利用Rust的所有权系统
- 避免使用不安全的代码块
- 定期进行安全审计

### 加密安全
- 使用经过验证的加密库
- 正确管理密钥和证书
- 实现安全的随机数生成

---
*最后更新: 2026-02-13*
