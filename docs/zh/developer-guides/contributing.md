# 贡献指南

**版本 (Version)**: v1.0  
**最后更新 (Last Updated)**: 2026-02-13  
**贡献者**: 欢迎加入我们的开源社区！

---

## 🎯 欢迎贡献

感谢您对 fingerprint-rust 项目的关注！我们欢迎任何形式的贡献，包括但不限于：

- 🐛 Bug修复
- ✨ 新功能开发
- 📚 文档改进
- 🧪 测试用例
- 💡 功能建议
- 🌍 国际化支持

## 🚀 快速开始

### 1. 环境准备

```bash
# 克隆项目
git clone https://github.com/vistone/fingerprint-rust.git
cd fingerprint-rust

# 安装Rust工具链
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 验证安装
rustc --version
cargo --version
```

### 2. 项目结构熟悉

```
fingerprint-rust/
├── crates/                    # Rust工作区
│   ├── fingerprint-core/     # 核心库
│   ├── fingerprint-tls/      # TLS实现
│   └── ...                   # 其他模块
├── examples/                 # 使用示例 (Usage Examples)
├── tests/                    # 测试文件
├── docs/                     # 文档
└── Cargo.toml                # 工作区配置
```

### 3. 构建和测试

```bash
# 构建项目
cargo build --workspace

# 运行测试
cargo test --workspace

# 运行特定crate的测试
cargo test -p fingerprint-core

# 检查代码质量
cargo clippy --workspace
cargo fmt -- --check
```

## 📝 贡献流程

### 1. 选择合适的Issue

查看我们的 [Issues页面](https://github.com/vistone/fingerprint-rust/issues)：

- 🟢 **good first issue** - 适合新手的入门任务
- 🔵 **help wanted** - 需要帮助的任务
- 🟡 **enhancement** - 功能改进
- 🔴 **bug** - Bug修复

### 2. Fork和Clone

```bash
# Fork项目到您的GitHub账户
# 然后克隆到本地
git clone https://github.com/YOUR_USERNAME/fingerprint-rust.git
cd fingerprint-rust

# 添加上游仓库
git remote add upstream https://github.com/vistone/fingerprint-rust.git
```

### 3. 创建功能分支

```bash
# 同步最新代码
git fetch upstream
git checkout main
git merge upstream/main

# 创建功能分支
git checkout -b feature/your-feature-name
# 或
git checkout -b fix/your-bug-fix
```

### 4. 开发和测试

```bash
# 编写代码
# ... your code here ...

# 运行测试
cargo test

# 检查代码风格
cargo fmt
cargo clippy

# 构建验证
cargo build --release
```

### 5. 提交更改

```bash
# 添加更改文件
git add .

# 提交更改
git commit -m "feat: 添加新功能描述

- 详细说明实现的功能
- 解决的具体问题
- 相关的测试情况"

# 推送到远程
git push origin feature/your-feature-name
```

### 6. 创建Pull Request

在GitHub上创建Pull Request：

1. 填写PR标题和描述
2. 关联相关的Issue
3. 等待CI检查通过
4. 接受代码审查反馈

## 📋 代码规范

### Rust代码规范

#### 命名约定
```rust
// 结构体使用驼峰命名
struct HttpClient {}
struct TlsConfig {}

// 函数和方法使用蛇形命名
fn send_request() {}
fn parse_response() {}

// 常量使用大写蛇形命名
const DEFAULT_TIMEOUT: u64 = 30;
const MAX_RETRIES: usize = 3;

// 模块使用蛇形命名
mod http_client;
mod tls_config;
```

#### 错误处理
```rust
// 使用Result类型进行错误处理
pub fn connect(&self) -> Result<Connection, Error> {
    // ... implementation
}

// 自定义错误类型
#[derive(Debug)]
pub enum FingerprintError {
    Io(std::io::Error),
    Tls(rustls::Error),
    InvalidConfig(String),
}

impl std::fmt::Display for FingerprintError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FingerprintError::Io(e) => write!(f, "IO error: {}", e),
            FingerprintError::Tls(e) => write!(f, "TLS error: {}", e),
            FingerprintError::InvalidConfig(msg) => write!(f, "Invalid config: {}", msg),
        }
    }
}
```

#### 文档注释
```rust
/// 发送HTTP请求
/// 
/// # 参数
/// * `url` - 目标URL
/// * `method` - HTTP方法
/// 
/// # 返回值
/// 返回请求结果
/// 
/// # 错误
/// 当网络连接失败时返回错误
/// 
/// # 示例
/// ```
/// let client = HttpClient::new();
/// let response = client.request("https://example.com", "GET")?;
/// ```
pub fn request(&self, url: &str, method: &str) -> Result<Response, Error> {
    // ... implementation
}
```

### Git提交规范

使用 [Conventional Commits](https://www.conventionalcommits.org/) 格式：

```
<type>[optional scope]: <description>

[optional body]

[optional footer(s)]
```

**类型说明**:
- `feat`: 新功能
- `fix`: Bug修复
- `docs`: 文档更新
- `style`: 代码格式调整
- `refactor`: 代码重构
- `perf`: 性能优化
- `test`: 测试相关
- `chore`: 构建或辅助工具变动

**示例**:
```
feat(tls): 添加HTTP/3支持

- 实现QUIC协议客户端
- 支持RFC 9114标准
- 添加相关测试用例

Closes #123
```

## 🧪 测试要求

### 单元测试
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_user_agent() {
        let ua = "Mozilla/5.0 (Windows NT 10.0; Win64; x64)";
        let parsed = parse_user_agent(ua).unwrap();
        assert_eq!(parsed.browser, "Mozilla");
        assert_eq!(parsed.version, "5.0");
    }

    #[test]
    fn test_invalid_input() {
        let result = parse_user_agent("");
        assert!(result.is_err());
    }
}
```

### 集成测试
```rust
// tests/integration_test.rs
use fingerprint_core::HttpClient;

#[tokio::test]
async fn test_real_http_request() {
    let client = HttpClient::new();
    let response = client
        .get("https://httpbin.org/headers")
        .await
        .expect("Request should succeed");
    
    assert_eq!(response.status(), 200);
}
```

### 性能测试
```rust
// benches/performance.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_tls_handshake(c: &mut Criterion) {
    c.bench_function("tls_handshake_chrome", |b| {
        b.iter(|| {
            black_box(handshake_with_profile("chrome_120_win"))
        })
    });
}

criterion_group!(benches, benchmark_tls_handshake);
criterion_main!(benches);
```

## 📚 文档贡献

### 文档结构
```
docs/
├── user-guides/          # 用户指南
├── developer-guides/     # 开发者指南
├── reference/            # 技术参考
└── project-management/   # 项目管理
```

### Markdown规范
```markdown
# 一级标题

## 二级标题

### 三级标题

**粗体文本** 和 *斜体文本*

- 无序列表项1
- 无序列表项2

1. 有序列表项1
2. 有序列表项2

```rust
// 代码块示例
fn hello() {
    println!("Hello, world!");
}
```

[链接文本](链接地址)

![图片描述](图片地址)
```

## 🔧 开发工具推荐

### IDE/编辑器插件
- **VS Code**: 
  - rust-analyzer
  - CodeLLDB
  - Better TOML
- **IntelliJ IDEA**:
  - Rust plugin
  - TOML plugin

### 命令行工具
```bash
# 安装常用工具
cargo install cargo-watch    # 文件变化时自动重新构建
cargo install cargo-edit     # Cargo.toml编辑工具
cargo install cargo-audit    # 安全漏洞检查
cargo install cargo-udeps    # 未使用依赖检查
```

### Git工具
```bash
# 安装commitizen用于规范提交
npm install -g commitizen cz-conventional-changelog

# 配置项目使用conventional-changelog
echo '{ "path": "cz-conventional-changelog" }' > .czrc
```

## 🎯 质量标准

### 代码覆盖率要求
- **核心模块**: ≥ 90%
- **业务逻辑**: ≥ 85%
- **工具函数**: ≥ 80%

### 性能基准
- HTTP/3响应时间: ≤ 50ms
- HTTP/2响应时间: ≤ 60ms
- 内存使用: ≤ 100MB
- CPU使用: ≤ 50%

### 兼容性要求
- Rust版本: ≥ 1.75.0
- Linux: Ubuntu 20.04+
- macOS: 12.0+
- Windows: Windows 10+

## 🆘 获取帮助

### 社区支持
- **GitHub Discussions**: [讨论区](https://github.com/vistone/fingerprint-rust/discussions)
- **Issues**: [问题跟踪](https://github.com/vistone/fingerprint-rust/issues)
- **邮件列表**: dev@fingerprint-rust.org

### 开发者资源
- [架构文档](architecture.md)
- [API参考](../reference/api-reference.md)
- [测试指南](testing.md)

## 🏆 贡献者奖励

### 贡献等级
- **青铜贡献者**: ≥ 5次有效贡献
- **白银贡献者**: ≥ 20次有效贡献
- **黄金贡献者**: ≥ 50次有效贡献
- **钻石贡献者**: ≥ 100次有效贡献

### 奖励机制
- 项目纪念品
- GitHub贡献者徽章
- 技术分享机会
- 项目决策参与权

## 📜 行为准则

请遵守我们的[行为准则](CODE_OF_CONDUCT.md)，营造友善、包容的开源社区环境。

## 🙏 感谢贡献

每一位贡献者都是项目成功的重要组成部分。您的每一份贡献都将被记录在[贡献者名单](CONTRIBUTORS.md)中。

让我们一起打造更好的fingerprint-rust！

---
*最后更新 (Last Updated): 2026-02-13*  
*版本 (Version): v1.0*