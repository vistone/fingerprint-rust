# 对 fingerprint-rust 的贡献

**版本**: v1.0  
**最后更新**: 2026-02-13  
**文档类型**: 技术文档

---

感谢您对 fingerprint-rust 的贡献感兴趣！本文档提供了为项目做贡献的指南和最佳实践。

## 目录

- [行为准则](#行为准则)
- [入门指南](#入门指南)
- [开发工作流](#开发工作流)
- [编码标准](#编码标准)
- [测试指南](#测试指南)
- [文档](#文档)
- [拉取请求流程](#拉取请求流程)
- [安全性](#安全性)

## 行为准则

### 我们的承诺

我们致力于为所有贡献者提供一个热烈和包容的环境，无论其背景或经验水平如何。

### 预期行为

- 保持尊重和体贴
- 欢迎新手并帮助他们入门
- 虚心接受建设性的批评
- 关注对项目最有利的事项
- 对其他贡献者表示同情

### 不可接受的行为

- 骚扰、歧视或冒犯性评论
- 拖旧账或辱骂性言辞
- 公开或私下骚扰
- 发布他人的私人信息
- 其他可合理认为不适当的行为

## 入门指南

### 先决条件

- **Rust**: 1.92.0 或更高版本（使用 `rustup` 进行安装）
- **Git**: 用于版本控制
- **Cargo**: 随 Rust 安装一起提供

### 分叉和克隆

1. 在 GitHub 上分叉本仓库
2. 克隆您的分叉：
   ```bash
   git clone https://github.com/YOUR_USERNAME/fingerprint-rust.git
   cd fingerprint-rust
   ```

3. 添加上游远程：
   ```bash
   git remote add upstream https://github.com/vistone/fingerprint-rust.git
   ```

### 构建项目

```bash
# 构建所有工作区 crates
cargo build --workspace

# 使用所有功能构建
cargo build --workspace --all-features

# 构建特定 crate
cargo build -p fingerprint-core
```

### 运行测试

```bash
# 运行所有测试
cargo test --workspace --lib

# 使用所有功能运行测试
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

遵循下面的[编码标准](#编码标准)部分。

### 3. 测试您的更改

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

编写清晰、描述性的提交消息：

```bash
git commit -m "feat: 添加 Chrome 135 新浏览器指纹"
git commit -m "fix: 解决数据包解析中的缓冲区溢出"
git commit -m "docs: 更新 HTTP 客户端 API 文档"
```

提交消息格式：
- `feat:` - 新功能
- `fix:` - 错误修复
- `docs:` - 文档更改
- `style:` - 代码风格更改（格式化等）
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

### 一般原则

1. **安全第一**: 避免使用 `unsafe` 代码，除非绝对必要
2. **错误处理**: 使用 `Result` 和 `?` 操作符，避免在生产代码中使用 `unwrap()`
3. **文档**: 记录所有公开的 API
4. **测试**: 为新功能编写测试
5. **性能**: 考虑更改对性能的影响

### Rust 风格指南

遵循 [Rust API 指南](https://rust-lang.github.io/api-guidelines/)：

```rust
// ✅ 好的：正确的错误处理
pub fn parse_packet(data: &[u8]) -> Result<Packet, PacketError> {
    if data.len() < MIN_SIZE {
        return Err(PacketError::TooShort);
    }
    // ...
    Ok(packet)
}

// ❌ 不好的：在生产中使用 unwrap()
pub fn parse_packet(data: &[u8]) -> Packet {
    let value = data.get(0).unwrap(); // 可能会崩溃！
    // ...
}
```

### 错误处理

使用 `thiserror` 来定义错误类型：

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MyError {
    #[error("Invalid packet size: {0}")]
    InvalidSize(usize),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
```

### 文档

记录所有公开的 API：

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
/// * `Err(PacketError)` - 如果数据包格式错误
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

### 代码组织

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

// 5. 特质实现
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
// ✅ 好的：避免不必要的分配
pub fn process_data(data: &[u8]) -> Result<(), Error> {
    // 使用引用，而不是克隆
}

// ❌ 不好的：不必要的克隆
pub fn process_data(data: Vec<u8>) -> Result<(), Error> {
    let copied = data.clone(); // 如果可能，避免
}

// ✅ 好的：重用分配
let mut buffer = Vec::with_capacity(1024);
for item in items {
    buffer.clear();
    // 重用缓冲区
}

// ❌ 不好的：在循环中分配
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
        let data = vec![/* 有效数据包 */];
        let result = parse_packet(&data);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_invalid_packet() {
        let data = vec![/* 无效数据包 */];
        let result = parse_packet(&data);
        assert!(result.is_err());
    }

    #[test]
    #[should_panic(expected = "buffer overflow")]
    fn test_panic_on_overflow() {
        // 测试预期的崩溃发生
    }

    #[test]
    #[ignore] // 标记需要网络访问的测试
    fn test_network_operation() {
        // 需要网络的测试
    }
}
```

### 集成测试

将集成测试放在 `tests/` 目录中：

```rust
// tests/integration_test.rs
use fingerprint::*;

#[test]
fn test_end_to_end() {
    // 测试完整工作流
}
```

### 基于属性的测试

考虑使用 `proptest` 进行基于属性的测试：

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_parse_never_panics(data in prop::collection::vec(any::<u8>(), 0..1024)) {
        // 即使使用随机数据也不应该崩溃
        let _ = parse_packet(&data);
    }
}
```

## 文档

### 代码文档

- 使用 `///` 注释记录所有公开的 API
- 在文档中包括示例
- 解释错误和边界情况
- 使用 `//!` 进行模块级文档

### 用户文档

- 为面向用户的更改更新 README.md
- 在 `examples/` 目录中添加示例
- 更新 `docs/` 目录中的相关指南
- 保持 CHANGELOG.md 最新

### API 文档

生成和检查文档：

```bash
cargo doc --workspace --no-deps --all-features --open
```

## 拉取请求流程

### 提交前

1. **更新您的分支**：
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

3. **更新文档**（如果需要）

4. **为新功能添加测试**

### 拉取请求描述模板

```markdown
## 说明

更改的简要说明。

## 更改类型

- [ ] 错误修复（解决问题的非破坏性更改）
- [ ] 新功能（添加功能的非破坏性更改）
- [ ] 破坏性更改（导致现有功能变更的修复或功能）
- [ ] 文档更新

## 测试

- [ ] 单元测试已添加/更新
- [ ] 集成测试已添加/更新
- [ ] 所有测试在本地通过

## 检查清单

- [ ] 代码遵循项目风格指南
- [ ] 已完成自我审查
- [ ] 为复杂代码添加了注释
- [ ] 文档已更新
- [ ] 未引入新警告
- [ ] 测试已添加并通过

## 相关问题

关闭 #123
相关 #456
```

### 审查流程

1. 维护者将审查您的拉取请求
2. 解决反馈并进行请求的更改
3. 获得批准后，您的拉取请求将被合并

### 合并后

1. 删除您的分支：
   ```bash
   git branch -d feature/your-feature-name
   git push origin --delete feature/your-feature-name
   ```

2. 更新您的主分支：
   ```bash
   git checkout main
   git pull upstream main
   ```

## 安全性

### 报告安全问题

**不要**通过公开问题报告安全漏洞。

相反：
1. 使用 GitHub 安全公告（首选）
2. 详见 [SECURITY.md](SECURITY.md)

### 安全考虑

贡献时，请考虑：
- 输入验证
- 缓冲区溢出防止
- 整数溢出处理
- 拒绝服务防止
- 信息泄露风险

### 代码审查检查清单

- [ ] 没有没有正当理由的 `unsafe` 代码
- [ ] 正确的错误处理（生产中没有 `unwrap()`）
- [ ] 对外部数据的输入验证
- [ ] 数组访问的边界检查
- [ ] 没有整数溢出的可能性
- [ ] 正确的资源清理（RAII）
- [ ] 错误消息中没有信息泄露

## 致谢

贡献者将被：
- 列在发布说明中
- 在 README.md 中得到认可（针对重要贡献）
- 在提交历史中获得记录

## 有问题吗？

- **GitHub 讨论**: 用于一般问题
- **GitHub 问题**: 用于错误报告和功能请求
- **拉取请求**: 用于代码贡献

## 资源

- [Rust 书籍](https://doc.rust-lang.org/book/)
- [Rust API 指南](https://rust-lang.github.io/api-guidelines/)
- [Cargo 书籍](https://doc.rust-lang.org/cargo/)
- [Clippy 链接检查](https://rust-lang.github.io/rust-clippy/)

感谢您对 fingerprint-rust 的贡献！🦀
