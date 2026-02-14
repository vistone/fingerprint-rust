# 提交政策 (Commit Policy)

## 🔒 强制性规则 (Mandatory Rule)

**所有代码必须通过GitHub Actions的所有检查才能提交！**

> **NO EXCEPTIONS** - 没有任何例外

## 📋 强制检查项 (Required Checks)

在提交代码之前，以下7项检查**必须全部通过**：

### 1️⃣ 代码格式化 (Code Formatting)
```bash
cargo fmt --all -- --check
```
- 所有Rust代码必须符合 rustfmt 标准
- 使用 `cargo fmt --all` 自动格式化

### 2️⃣ Linter检查 (Linting)
```bash
cargo clippy --workspace --all-targets --all-features -- -D warnings
```
- 必须通过所有 clippy 检查
- 不能有任何警告级别的问题

### 3️⃣ 编译检查 (Compilation)
```bash
cargo check --workspace --all-features
```
- 代码必须能够编译通过
- 包括所有可选特性

### 4️⃣ 单元测试 (Unit Tests)
```bash
cargo test --workspace --lib
```
- 所有单元测试必须通过
- 没有跳过或忽略的测试

### 5️⃣ 集成测试 (Integration Tests)
```bash
cargo test --workspace --lib --tests
```
- 所有集成测试必须通过
- 测试覆盖率应该保持或提高

### 6️⃣ 安全审计 (Security Audit)
```bash
cargo deny check advisories bans licenses sources
```
- 没有已知的安全漏洞
- 依赖许可证合规

### 7️⃣ 发布构建 (Release Build)
```bash
cargo build --workspace --release
```
- 必须能构建发布版本
- 不能有任何编译警告

## 🚀 提交流程 (Commit Workflow)

### 方法 1: 自动检查（推荐）
```bash
git add .
git commit -m "message"
```
提交时会自动运行pre-commit hook，检查全部通过才能提交。

### 方法 2: 手动运行检查
```bash
# 方法 A: 运行所有检查
./scripts/pre_commit_test.sh

# 方法 B: 单个检查
cargo fmt --all
cargo clippy --workspace --all-targets --all-features
cargo check --workspace --all-features
cargo test --workspace
cargo deny check
cargo build --release
```

## ✅ 快速修复指南 (Quick Fix Guide)

### 格式化失败
```bash
cargo fmt --all
git add .
```

### Clippy警告
```bash
# 查看具体问题
cargo clippy --workspace --all-targets --all-features

# 通常可以自动修复
cargo clippy --fix --workspace
```

### 测试失败
```bash
# 运行失败的测试
cargo test --workspace -- --nocapture

# 查看具体输出
RUST_BACKTRACE=1 cargo test &lt;test_name&gt; -- --nocapture
```

### 安全问题
```bash
# 检查具体问题
cargo deny check advisories

# 更新 Cargo.lock
cargo update
```

## 📝 提交消息规范 (Commit Message Convention)

遵循约定式提交 (Conventional Commits)：

```
<type>: <subject>

<body>

<footer>
```

### Type 类型
- `feat`: 新功能
- `fix`: 修复bug
- `docs`: 文档更新
- `style`: 代码风格（不影响功能）
- `refactor`: 重构代码
- `perf`: 性能优化
- `test`: 添加/修改测试
- `chore`: 构建、依赖、工具相关

### 示例
```
fix: correct metrics test to use correct gateway metric names

- Fix test_gather_metrics to check for 'fingerprint_gateway_*' metric names
- The metrics registry actually includes '_gateway' prefix in metric names
- Update assertion to check for actual registered metrics

Fixes: #123
```

## 🚫 被拒绝的提交 (Rejected Commits)

以下情况的提交**会被拒绝**：

- ❌ 代码格式不符合 rustfmt 标准
- ❌ 有 clippy 警告或错误
- ❌ 编译失败
- ❌ 任何测试失败
- ❌ 安全检查不通过（未知依赖、漏洞等）
- ❌ 发布构建失败

## 🔄 强制执行机制 (Enforcement)

### Git Hook
- **Pre-commit hook** 在本地强制执行所有检查
- 如果任何检查失败，提交会被阻止
- 无法跳过这些检查（没有 `--no-verify` 选项）

### GitHub Actions
- 所有分支上的PR都会运行完整的CI/CD流程
- 必须通过所有检查才能合并

## 📞 遇到问题 (Troubleshooting)

### "Cannot start a runtime from within a runtime"
- 这通常是测试中的异步代码问题
- 查看 test_collector_standalone.rs

### 平台特定失败
- 某些测试在不同操作系统上可能有差异
- 使用 `#[cfg(target_os = "...")]` 处理

### 缓存问题
```bash
cargo clean
cargo build
```

## 📚 相关文档

- [GitHub Actions Workflows](.github/workflows/)
- [Pre-commit Test Script](./scripts/pre_commit_test.sh)
- [Contributing Guide](./CONTRIBUTING.md)

---

**最后更新**: 2026年2月14日  
**适用范围**: fingerprint-rust 项目所有分支
