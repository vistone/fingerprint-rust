# fingerprint-rust 贡献指南

**版本**: v1.0  
**最后更新**: 2026-02-13  
**文档类型**: 技术文档

---

感谢你对 fingerprint-rust 项目的贡献兴趣！本文档提供了为项目贡献的指南和最佳实践。

## 目录

- [行为准则](#行为准则)
- [入门指南](#入门指南)
- [开发工作流](#开发工作流)
- [编码标准](#编码标准)
- [测试指南](#测试指南)
- [文档编写](#文档编写)
- [拉取请求流程](#拉取请求流程)
- [安全](#安全)

## 行为准则

### 我们的承诺

我们致力于为所有贡献者提供一个热烈欢迎和包容的环境，不论其背景或经验水平如何。

### 预期行为

- 相互尊重和体贴
- 欢迎新手，帮助他们快速上手
- 虚心接受建设性批评
- 关注对项目最有利的事情
- 对其他贡献者表示同情

### 不可接受的行为

- 骚扰、歧视或冒犯性评论
- 网络暴力或侮辱性言论
- 公开或私下骚扰
- 发布他人的私人信息
- 其他合理认为不当的行为

## 入门指南

### 前置条件

- **Rust**: 1.92.0 或更高版本。请使用 `rustup` 安装
- **Git**: 用于版本控制
- **Cargo**: 随 Rust 自动安装

### Fork 和克隆项目

1. 在 GitHub 上 Fork 项目仓库
2. 克隆你的 Fork：
   ```bash
   git clone https://github.com/YOUR_USERNAME/fingerprint-rust.git
   cd fingerprint-rust
   ```

3. 添加上游深层：
   ```bash
   git remote add upstream https://github.com/vistone/fingerprint-rust.git
   ```

### 构建项目

```bash
# 构建所有工作区 Crate
cargo build --workspace

# 启用所有特性进行构建
cargo build --workspace --all-features

# 构建特定 Crate
cargo build -p fingerprint-core
```

### 运行测试

```bash
# 运行所有测试
cargo test --workspace --lib

# 使用所有特性运行测试
cargo test --workspace --all-features

# 运行特定测试
cargo test -p fingerprint-core test_name
```

## 开发工作流

### 1. 创建分支

```bash
git checkout -b feature/your-feature-name
# 或
git checkout -b fix/issue-number-description
```

分支命名约定：
- `feature/` - 新功能
- `fix/` - 错误修复
- `docs/` - 文档更新
- `refactor/` - 代码重构
- `test/` - 测试添加或改进
- `perf/` - 性能改进

### 2. 进行更改

遵循下面[编码标准](#编码标准)部分的要求。

### 3. 测试你的更改

```bash
# 运行测试
cargo test --workspace --lib

# 运行 Clippy
cargo clippy --workspace --all-targets --all-features -- -D warnings

# 格式化代码
cargo fmt --all

# 检查文档
cargo doc --workspace --no-deps --all-features
```

### 4. 提交更改

编写清晰、具有描述性的提交消息：

```bash
git commit -m "feat: 为 Chrome 135 添加新的浏览器指纹"
git commit -m "fix: 修复数据包解析中的缓冲区溢出"
git commit -m "docs: 更新 HTTP 客户端 API 文档"
```

提交消息格式：
- `feat:` - 新功能
- `fix:` - 错误修复
- `docs:` - 文档更改
- `style:` - 代码风格更改（格式等）
- `refactor:` - 代码重构
- `test:` - 添加或更新测试
- `perf:` - 性能改进
- `chore:` - 维护任务

### 5. 推送和创建拉取请求

```bash
git push origin feature/your-feature-name
```

然后在 GitHub 上创建拉取请求。

## 编码标准

### 基础原则

1. **安全第一**：避免 `unsafe` 代码，除非绝对必要
2. **错误处理**：使用 `Result` 和 `?` 操作符，避免在生产代码中使用 `unwrap()`
3. **文档**：记录所有公开 API
4. **测试**：为新功能编写测试
5. **性能**：考虑更改的性能影响

### Rust 编码风格指南

遵循 [Rust API 指南](https://rust-lang.github.io/api-guidelines/)：

```rust
// ✅ 正确：适当的错误处理
pub fn parse_packet(data: &[u8]) -> Result<Packet, PacketError> {
    if data.len() < MIN_SIZE {
        return Err(PacketError::TooShort);
    }
    // ...
    Ok(packet)
}

// ❌ 错误：在生产代码中使用 unwrap()
pub fn parse_packet(data: &[u8]) -> Packet {
    let value = data.get(0).unwrap(); // 可能会 panic！
    // ...
}
```

### 错误处理

使用 `thiserror` 定义错误类型：

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MyError {
    #[error("无效的数据包大小：{0}")]
    InvalidSize(usize),
    
    #[error("IO 错误：{0}")]
    Io(#[from] std::io::Error),
}
```

### 文档编写

记录所有公开 API：

```rust
/// 从原始字节解析 IPv4 数据包。
///
/// # 参数
///
/// * `data` - 原始数据包数据
///
/// # 返回值
///
/// * `Ok(Packet)` - 成功解析的数据包
/// * `Err(PacketError)` - 如果数据包格式不正确
///
/// # 示例
///
/// ```
/// use fingerprint::parse_packet;
///
/// let data = vec![0x45, 0x00, /* ... */];
/// let packet = parse_packet(&data)?;
/// ```
///
/// # 错误
///
/// 如果数据包小于最小大小，返回 `PacketError::TooShort`。
/// 如果 IHL 字段无效，返回 `PacketError::InvalidIhl`。
pub fn parse_packet(data: &[u8]) -> Result<Packet, PacketError> {
    // ...
}
```

### 代码组织结构

```rust
// 1. 模块文档在顶部
//! # 模块名称
//!
//! 模块的简要描述。

// 2. 导入
use std::io;
use crate::types::*;

// 3. 常量
const MAX_SIZE: usize = 1024;

// 4. 类型定义
pub struct MyStruct {
    // 字段
}

// 5. 特征实现
impl MyTrait for MyStruct {
    // ...
}

// 6. 方法
impl MyStruct {
    pub fn new() -> Self {
        // ...
    }
}

// 7. 测试
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_something() {
        // ...
    }
}
```

### 性能考虑

```rust
// ✅ 正确：避免不必要的分配
pub fn process_data(data: &[u8]) -> Result<(), Error> {
    // 使用引用，而不是克隆
}

// ❌ 错误：不必要的克隆
pub fn process_data(data: Vec<u8>) -> Result<(), Error> {
    let copied = data.clone(); // 如果可能，应避免
}

// ✅ 正确：重用分配
let mut buffer = Vec::with_capacity(1024);
for item in items {
    buffer.clear();
    // 重用缓冲区
}

// ❌ 错误：在循环中分配
for item in items {
    let buffer = Vec::new(); // 每次迭代都分配
}
```

## 测试指南

### 单元测试

为所有公开函数编写单元测试：

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_packet() {
        let data = vec![/* 有效的数据包 */];
        let result = parse_packet(&data);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_invalid_packet() {
        let data = vec![/* 无效的数据包 */];
        let result = parse_packet(&data);
        assert!(result.is_err());
    }

    #[test]
    #[should_panic(expected = "buffer overflow")]
    fn test_panic_on_overflow() {
        // 测试 panic 是否如预期发生
    }

    #[test]
    #[ignore] // 标记需要网络访问的测试
    fn test_network_operation() {
        // 需要网络的测试
    }
}
```

### 集成测试

在 `tests/` 目录中放置集成测试：

```rust
// tests/integration_test.rs
use fingerprint::*;

#[test]
fn test_end_to_end() {
    // 测试完整工作流
}
```

### 属性测试

考虑使用 `proptest` 进行属性测试：

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_parse_never_panics(data in prop::collection::vec(any::<u8>(), 0..1024)) {
        // 即使有随机数据也不应该 panic
        let _ = parse_packet(&data);
    }
}
```

## 文档编写

### 代码文档

- 使用 `///` 注释记录所有公开 API
- 在文档中包含使用示例
- 解释错误和边界情况
- 为模块级文档使用 `//!`

### 用户文档

- 更新用户相关更改的 README.md
- 向 `examples/` 目录添加示例
- 更新 `docs/` 目录中的相关指南
- 保持 CHANGELOG.md 最新

### API 文档

生成并查看文档：

```bash
cargo doc --workspace --no-deps --all-features --open
```

## 拉取请求流程

### 提交之前

1. **更新你的分支**：
   ```bash
   git fetch upstream
   git rebase upstream/main
   ```

2. **运行所有检查**：
   ```bash
   cargo test --workspace --lib
   cargo clippy --workspace --all-targets --all-features -- -D warnings
   cargo fmt --all -- --check
   cargo doc --workspace --no-deps --all-features
   ```

3. **更新文档**（如需要）

4. **为新功能添加测试**

### 拉取请求描述模板

```markdown
## 描述

更改的简要描述。

## 更改类型

- [ ] 错误修复（非破坏性更改修复问题）
- [ ] 新功能（非破坏性更改添加功能）
- [ ] 破坏性更改（修复或功能导致现有功能改变）
- [ ] 文档更新

## 测试

- [ ] 添加或更新了单元测试
- [ ] 添加或更新了集成测试
- [ ] 所有测试在本地通过

## 清单

- [ ] 代码遵循项目风格指南
- [ ] 已完成自审查
- [ ] 为复杂代码添加了注释
- [ ] 更新了文档
- [ ] 没有引入新的警告
- [ ] 添加了测试且通过

## 相关 Issue

关闭 #123
关联 #456
```

### 审查流程

1. 维护者将审查你的拉取请求
2. 处理反馈并进行所需的更改
3. 一旦批准，你的拉取请求将被合并

### 合并后

1. 删除你的分支：
   ```bash
   git branch -d feature/your-feature-name
   git push origin --delete feature/your-feature-name
   ```

2. 更新你的主分支：
   ```bash
   git checkout main
   git pull upstream main
   ```

## 安全

### 报告安全问题

**不要**通过公开 Issue 报告安全漏洞。

相反，请：
1. 使用 GitHub 安全公告（首选）
2. 查看 [SECURITY.md](SECURITY.md) 了解详情

### 安全考虑事项

贡献时，请考虑：
- 输入验证
- 缓冲区溢出防防
- 整数溢出处理
- 拒绝服务防护
- 信息泄露风险

### 代码审查清单

- [ ] 没有 `unsafe` 代码（除非有正当理由）
- [ ] 适当的错误处理（生产代码中没有 `unwrap()`）
- [ ] 对外部数据的输入验证
- [ ] 数组访问的界限检查
- [ ] 没有整数溢出可能性
- [ ] 适当的资源清理（RAII）
- [ ] 错误消息中没有信息泄露

## 致谢

贡献者将被：
- 列在发布说明中
- 在 README.md 中被认可（对于重大贡献）
- 在提交历史中被记录

## 有问题？

- **GitHub Discussions**：提出一般问题
- **GitHub Issues**：报告 Bug 和功能请求
- **拉取请求**：进行代码贡献

## 资源

- [Rust 编程书](https://doc.rust-lang.org/book/)
- [Rust API 指南](https://rust-lang.github.io/api-guidelines/)
- [Cargo 手册](https://doc.rust-lang.org/cargo/)
- [Clippy Lints](https://rust-lang.github.io/rust-clippy/)

感谢你对 fingerprint-rust 的贡献！🦀
