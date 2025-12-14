# CI 故障排查指南

## 📋 常见 CI 错误及解决方案

### 1. 代码格式错误

**错误信息**:
```
Diff in /path/to/file.rs:XX
```

**解决方案**:
```bash
cargo fmt
git add -A
git commit -m "fix: 格式化代码"
```

### 2. Clippy 警告/错误

**错误信息**:
```
warning: unused variable: `xxx`
error: could not compile `fingerprint` due to previous errors
```

**解决方案**:
```bash
# 查看所有警告
cargo clippy --all-targets --features "rustls-tls,compression,http2,export" -- -D warnings

# 自动修复（如果可能）
cargo clippy --fix --all-targets --features "rustls-tls,compression,http2,export"

# 手动修复后验证
cargo clippy --all-targets --features "rustls-tls,compression,http2,export" -- -D warnings
```

### 3. 测试失败

**错误信息**:
```
test result: FAILED. X failed; Y passed
```

**解决方案**:
```bash
# 运行失败的测试
cargo test --lib --features "rustls-tls,compression,http2" -- --nocapture

# 运行特定测试
cargo test test_name --features "rustls-tls,compression,http2" -- --nocapture
```

### 4. 编译错误

**错误信息**:
```
error[EXXXX]: ...
```

**解决方案**:
```bash
# 检查编译错误
cargo check --features "rustls-tls,compression,http2"

# 查看详细错误信息
cargo build --features "rustls-tls,compression,http2" --verbose
```

### 5. 文档构建警告

**错误信息**:
```
warning: `fingerprint` (lib doc) generated X warnings
```

**解决方案**:
```bash
# 查看文档警告
cargo doc --features "rustls-tls,compression,http2" --no-deps

# 修复文档中的链接格式
# 将 `https://example.com` 改为 `<https://example.com>`
```

### 6. Feature 缺失错误

**错误信息**:
```
error: unexpected `cfg` condition value: `xxx`
```

**解决方案**:
1. 检查 `Cargo.toml` 中是否定义了该 feature
2. 如果未定义，添加：
```toml
[features]
xxx = []  # 或添加依赖
```

### 7. 依赖问题

**错误信息**:
```
error: failed to resolve: use of undeclared crate or module `xxx`
```

**解决方案**:
```bash
# 更新依赖
cargo update

# 检查 Cargo.toml 中的依赖声明
# 确保所有使用的 crate 都在 [dependencies] 中声明
```

## 🔍 本地验证 CI 步骤

在提交前，运行以下命令验证所有 CI 检查：

```bash
# 1. 格式化检查
cargo fmt -- --check

# 2. Clippy 检查
cargo clippy --all-targets --features "rustls-tls,compression,http2,export" -- -D warnings

# 3. 运行测试
cargo test --lib --features "rustls-tls,compression,http2"
cargo test --test integration_test --features "rustls-tls,compression,http2"

# 4. 构建检查
cargo build --features "rustls-tls,compression,http2"

# 5. 文档构建
cargo doc --features "rustls-tls,compression,http2" --no-deps
```

## 📊 CI 工作流步骤

当前 CI 工作流包含以下步骤：

1. **Check formatting** - 检查代码格式
2. **Install system dependencies** - 安装系统依赖（libssl-dev, pkg-config）
3. **Run Clippy** - 代码质量检查
4. **Run tests** - 运行单元测试和集成测试
5. **Build** - 编译项目
6. **Build documentation** - 构建文档

## 🛠️ 快速修复脚本

创建一个 `scripts/ci-check.sh` 脚本：

```bash
#!/bin/bash
set -e

echo "🔍 运行格式化检查..."
cargo fmt -- --check

echo "🔍 运行 Clippy 检查..."
cargo clippy --all-targets --features "rustls-tls,compression,http2,export" -- -D warnings

echo "🔍 运行测试..."
cargo test --lib --features "rustls-tls,compression,http2"
cargo test --test integration_test --features "rustls-tls,compression,http2"

echo "🔍 构建项目..."
cargo build --features "rustls-tls,compression,http2"

echo "✅ 所有检查通过！"
```

## 📝 提交前检查清单

- [ ] `cargo fmt -- --check` 通过
- [ ] `cargo clippy` 无警告
- [ ] 所有测试通过
- [ ] 项目能够编译
- [ ] 文档能够构建

## 🔗 相关资源

- [Rust 官方文档](https://doc.rust-lang.org/)
- [Clippy 文档](https://rust-lang.github.io/rust-clippy/)
- [GitHub Actions 文档](https://docs.github.com/en/actions)

---

**最后更新**: 2025-12-14
**维护者**: fingerprint-rust 团队

