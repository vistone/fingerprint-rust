# 开发者指南

**[English](../../en/developer-guides/DEVELOPMENT.md)** | [中文](#中文)

---

## 中文

### 🚀 开发环境设置

#### 前置条件

- Rust 1.92.0 或更高版本
- Cargo
- Git

#### 安装Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

#### 克隆仓库

```bash
git clone https://github.com/vistone/fingerprint-rust.git
cd fingerprint-rust
```

#### 安装开发工具

```bash
# 安装格式化工具
rustup component add rustfmt

# 安装lint工具
rustup component add clippy

# 安装快速测试工具
cargo install cargo-nextest

# 安装代码覆盖工具
cargo install cargo-tarpaulin
```

### 📝 代码贡献规范

#### 命名规范

```rust
// ✅ 正确: snake_case 函数
pub fn parse_fingerprint(data: &[u8]) -> Result<Fingerprint> { }

// ✅ 正确: PascalCase 结构体
pub struct FingerprintData {
    pub browser: BrowserType,
}

// ✅ 正确: UPPER_CASE 常量
pub const MAX_RETRY_COUNT: usize = 3;

// ❌ 错误: 混合命名
pub fn ParseFingerprint() { }
pub struct fingerprint_data { }
```

#### 文档注释

所有public API必须有文档注释：

```rust
/// 解析浏览器指纹数据
///
/// 此函数从原始字节数据中提取指纹信息。
///
/// # 参数
///
/// * `data` - 指纹数据字节数组
/// * `flags` - 解析选项标志
///
/// # 返回值
///
/// 成功时返回解析后的`Fingerprint`结构体，失败时返回`FingerprintError`。
///
/// # 错误
///
/// 如果数据格式不正确或不完整，将返回`ParseError`。
///
/// # 示例
///
/// ```
/// use fingerprint_core::{parse_fingerprint, FingerprintError};
///
/// let data = vec![0x01, 0x02, 0x03];
/// match parse_fingerprint(&data, 0) {
///     Ok(fp) => println!("浏览器: {:?}", fp.browser),
///     Err(e) => eprintln!("错误: {}", e),
/// }
/// ```
pub fn parse_fingerprint(data: &[u8], flags: u8) -> Result<Fingerprint, FingerprintError> {
    // implementation
}
```

### 🧪 测试

#### 运行所有测试

```bash
# 快速测试（推荐）
cargo nextest run --workspace

# 标准测试
cargo test --workspace

# 包含文档测试
cargo test --workspace --doc

# 特定crate的测试
cd crates/fingerprint-core
cargo test
```

#### 编写测试

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_fingerprint() {
        let data = vec![/* ... */];
        let result = parse_fingerprint(&data, 0);
        
        assert!(result.is_ok());
        let fp = result.unwrap();
        assert_eq!(fp.browser, BrowserType::Chrome);
    }

    #[test]
    #[should_panic(expected = "invalid format")]
    fn test_parse_invalid_fingerprint() {
        let invalid_data = vec![];
        let _ = parse_fingerprint(&invalid_data, 0);
    }

    #[tokio::test]
    async fn test_async_operation() {
        let result = async_parse_fingerprint(&data).await;
        assert!(result.is_ok());
    }
}
```

#### 基准测试

```bash
# 运行基准测试
cargo bench --workspace

# 只运行特定基准
cargo bench --workspace fingerprint_parsing
```

### 📊 代码质量检查

#### 格式化检查

```bash
# 检查格式
cargo fmt --all -- --check

# 自动格式化
cargo fmt --all
```

#### Lint检查

```bash
# 运行clippy
cargo clippy --workspace --all-targets --all-features -- -D warnings

# 自动修复问题
cargo clippy --workspace --fix
```

#### 测试覆盖率

```bash
# 生成覆盖率报告
cargo tarpaulin --workspace --all-features --out Html --output-dir coverage

# 查看报告
open coverage/index.html
```

#### 安全审计

```bash
# 检查依赖安全性
cargo audit

# 检查依赖许可证
cargo deny check
```

### 🏗️ 项目结构

```
crates/
├── fingerprint/               # 主库（最上层API）
├── fingerprint-core/          # 核心类型和工具
├── fingerprint-tls/           # TLS指纹识别
├── fingerprint-http/          # HTTP客户端
├── fingerprint-profiles/      # 浏览器配置
├── fingerprint-headers/       # 请求头生成
├── fingerprint-defense/       # 防御机制
├── fingerprint-gateway/       # API网关
├── fingerprint-ml/            # 机器学习模块
└── ...
```

#### 添加新功能

1. **创建新模块**：在现有crate中添加新文件
2. **添加测试**：为每个新功能编写测试
3. **更新文档**：添加doc注释和README
4. **运行检查**：确保所有测试和lint通过

示例：

```rust
// crates/fingerprint-core/src/new_feature.rs
//! 新功能模块

/// 新功能的主要类型
#[derive(Debug, Clone)]
pub struct NewFeature {
    // ...
}

impl NewFeature {
    /// 创建新实例
    pub fn new() -> Self {
        Self { /* ... */ }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_feature() {
        let feature = NewFeature::new();
        // ...
    }
}
```

### 🔄 工作流

#### 提交代码前

```bash
# 1. 更新到最新代码
git pull origin main

# 2. 创建功能分支
git checkout -b feature/new-feature

# 3. 进行更改和提交
git add .
git commit -m "feat: add new feature"

# 4. 运行所有检查
./scripts/pre_commit_test.sh

# 5. 推送到远程
git push origin feature/new-feature
```

#### 完整检查清单

提交前运行：

```bash
# 格式化
cargo fmt --all

# Lint
cargo clippy --workspace --all-features -- -D warnings

# 编译
cargo check --workspace --all-features

# 测试
cargo test --workspace --all-features

# 文档
cargo doc --workspace --no-deps

# 安全审计
cargo audit
```

### 📈 性能优化

#### 分析性能

```bash
# 运行性能基准
cargo bench --workspace

# 生成火焰图
cargo install flamegraph
cargo flamegraph --bin fingerprint
```

#### 常见优化

1. **使用不可变引用**: 避免不必要的复制
2. **缓存常用值**: 使用LRU缓存
3. **异步操作**: 使用tokio处理I/O
4. **SIMD优化**: 使用vectorized操作（如可用）

### 🐛 调试

#### 启用日志

```rust
// 代码中
use log::{info, warn, error};

fn process_fingerprint(data: &[u8]) {
    info!("开始处理指纹");
    warn!("某个警告");
    error!("发生错误");
}
```

运行时：

```bash
RUST_LOG=debug cargo run
RUST_LOG=fingerprint_core=trace cargo test --lib
```

#### 使用调试器

```bash
# 使用rust-gdb
rust-gdb target/debug/fingerprint

# 使用rust-lldb (macOS)
rust-lldb target/debug/fingerprint
```

### 📚 相关资源

- [Rust官方文档](https://doc.rust-lang.org/)
- [Cargo文档](https://doc.rust-lang.org/cargo/)
- [Rust API指南](https://rust-lang.github.io/api-guidelines/)
- [Clippy文档](https://doc.rust-lang.org/clippy/)

### 🤝 获取帮助

- 📖 查看現有文档
- 🐛 检查GitHub Issues
- 💬 参与Discussions
- 📧 联系维护者
