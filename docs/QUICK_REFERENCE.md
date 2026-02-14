# 项目规范快速参考 (Quick Reference)

> 本文档是对完整规范的快速查询。详细内容请参考完整文档。

---

## 🎯 5 秒钟规范总结

### 核心原则

```
✅ 严格按照项目结构放置文件
✅ 必须有测试和文档
✅ 必须通过所有 7 项本地检查
✅ 不允许任何例外或绕过
```

### 提交流程

```bash
# 1. 修改代码
# 2. 运行检查
./scripts/pre_commit_test.sh

# 3. 如果看到 ✅ 所有检查通过
# 4. 提交并推送
git add .
git commit -m "type: message"
git push
```

### 如果检查失败

```bash
# 自动修复格式
cargo fmt --all

# 自动修复 clippy
cargo clippy --fix --workspace

# 运行测试找出问题
cargo test --workspace

# 重新运行检查脚本
./scripts/pre_commit_test.sh
```

---

## 📁 文件放置速查表

### 正确的位置

| 文件类型 | 位置 | 示例 |
|---------|------|------|
| **Crate 配置** | `crates/*/Cargo.toml` | `crates/fingerprint-core/Cargo.toml` |
| **Crate 说明** | `crates/*/README.md` | `crates/fingerprint-core/README.md` ✅ **必须** |
| **Rust 源代码** | `crates/*/src/` | `crates/fingerprint-core/src/lib.rs` |
| **单元测试** | 源文件中 (inline) | `#[cfg(test)] mod tests {` |
| **集成测试** | `crates/*/tests/` | `crates/fingerprint/tests/integration.rs` |
| **示例代码** | `examples/` | `examples/basic.rs` |
| **项目文档** | `docs/` | `docs/ARCHITECTURE.md` |
| **API文档** | `docs/API.md` | （单一文件） |
| **配置** | `config/` | `config/deployment/production.toml` |
| **数据集** | `dataset/` | `dataset/training_data.csv` |
| **模型** | `models/` | `models/trained_model.pkl` |
| **脚本** | `scripts/` | `scripts/build.sh` |

### ❌ 错误的位置

```
❌ crates/src/            (应该是 crates/*/src/)
❌ src/                   (项目没有顶级 src/)
❌ data/                  (应该是 dataset/ 或 models/)
❌ reports/               (应该是 docs/)
❌ PHASE_X_XXX.md         (应该是 docs/PHASE_X_XXX.md)
```

---

## 📝 文档命名规范

### 正确的文件名

```
✅ ARCHITECTURE.md                      一级标题：一个文件
✅ PHASE_5_COMPLETION_REPORT.md         PHASE_[N]_[TYPE].md
✅ HTTP2_INTEGRATION_GUIDE.md           [FEATURE]_[TYPE].md
✅ BROWSER_VERSION_ADAPTATION.md        [TOPIC].md
✅ AI_CODE_GENERATION_RULES.md          [DESCRIPTION].md
```

### ❌ 错误的文件名

```
❌ architecture.md                      (应该大写)
❌ Phase_5_report.md                    (应该 PHASE_5)
❌ phase5.md                            (应该完整名称)
❌ Report_Phase5.md                     (应该 PHASE_5 开头)
❌ TODO.md, NOTES.md, TEMP.md           (不允许，缺乏结构)
```

---

## ✅ 7 项强制检查清单

```bash
# 检查工具已包含在：./scripts/pre_commit_test.sh

✅ 1. 代码格式化
   cargo fmt --all -- --check

✅ 2. Lint 检查  
   cargo clippy --workspace --all-targets --all-features -- -D warnings

✅ 3. 编译检查
   cargo check --workspace --all-features

✅ 4. 单元测试
   cargo test --workspace --lib

✅ 5. 集成测试
   cargo test --workspace

✅ 6. 安全审计
   cargo deny check advisories bans licenses sources

✅ 7. 发布构建
   cargo build --workspace --release
```

**规则：** 全部通过才能提交（无例外）

---

## 🚫 AI 代码生成绝对禁止

### ❌ 禁止 1: 乱放文件

```
❌ 在根目录创建新目录
❌ 创建 src/ 目录
❌ 将 .md 文件放在根目录
❌ 创建 .backup, .disabled, .old 等文件
```

**正确做法：** 使用指定的目录结构

### ❌ 禁止 2: 乱写文档

```
❌ 没有模板的 .md 文件
❌ 只有一种语言（中文或英文）
❌ 使用 lowercase 或 camelCase 文件名
❌ 在根目录创建文档文件
```

**正确做法：** 参考文档模板，放在 `docs/`

### ❌ 禁止 3: 低质量代码

```
❌ pub fn 没有文档注释
❌ 新功能没有测试
❌ 使用 unwrap(), panic! 等不安全操作
❌ 硬编码值
❌ 没有错误处理
```

**正确做法：** 文档 + 测试 + 错误处理

### ❌ 禁止 4: 跳过检查

```
❌ 代码有 clippy 警告
❌ 测试失败
❌ 使用 git commit --no-verify
❌ 未运行 pre_commit_test.sh
```

**正确做法：** 全部通过后才提交

---

## 💻 代码示例模板

### 新函数模板

```rust
/// 函数简短说明
///
/// # 详细说明（如需要）
/// 更多细节...
///
/// # 参数
/// - `param1`: 说明
/// - `param2`: 说明
///
/// # 返回值
/// 返回值说明
///
/// # 错误
/// - `Error::Type1`: 错误说明
/// - `Error::Type2`: 错误说明
///
/// # 示例
/// ```
/// let result = my_function(param)?;
/// assert!(result.is_valid());
/// ```
pub fn my_function(param1: String, param2: usize) -> Result<Data, Error> {
    // implementation
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_my_function_valid_input() {
        // arrange
        let input = prepare_test_data();
        
        // act
        let result = my_function(input, 10)?;
        
        // assert
        assert!(result.is_valid());
    }

    #[test]
    fn test_my_function_invalid_input() {
        let result = my_function(String::new(), 0);
        assert!(result.is_err());
    }
}
```

### 新 crate 模板

```
crates/fingerprint-new-feature/
├── Cargo.toml
└── src/
    ├── lib.rs           (模块入口 + 文档)
    ├── module1.rs       (功能模块)
    └── module2.rs       (功能模块)

docs/
├── NEW_FEATURE_DESIGN.md        (中文设计文档)
├── NEW_FEATURE_DESIGN.en.md     (英文设计文档)
```

---

## 🔄 完整的事前检查清单

在运行 `git commit` 前：

- [ ] `./scripts/pre_commit_test.sh` 显示 ✅ 所有检查通过
- [ ] 所有 `pub` 函数都有 `///` 文档
- [ ] 添加了单元测试（`#[test]`）
- [ ] 添加了集成测试（如适用）
- [ ] 文件在正确的目录
- [ ] 文件名遵循规范
- [ ] 代码无 `println!`, `dbg!`, `unwrap()` 等
- [ ] 错误使用了 `Result<T, E>`
- [ ] 文档（若有）放在 `docs/` 且符合命名规范
- [ ] 提交消息格式：`type: subject`，例如 `feat: add new feature`

---

## 📚 完整文档导航

| 文档 | 用途 | 链接 |
|------|------|------|
| **项目治理** | 完整的规范体系 | [PROJECT_GOVERNANCE.md](PROJECT_GOVERNANCE.md) |
| **提交政策** | 提交和检查规则 | [COMMIT_POLICY.md](COMMIT_POLICY.md) |
| **AI 代码生成** | AI 辅助开发规则 | [docs/AI_CODE_GENERATION_RULES.md](docs/AI_CODE_GENERATION_RULES.md) |
| **快速参考** | 本文件 | [QUICK_REFERENCE.md](QUICK_REFERENCE.md) |
| **贡献指南** | 参与项目 | [CONTRIBUTING.md](CONTRIBUTING.md) |
| **API 文档** | API 参考 | [docs/API.md](docs/API.md) |
| **架构设计** | 系统架构 | [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) |

---

## ⚠️ 最常见的错误

### 错误 1: "测试失败"

```bash
# ❌ 常见原因
- 在 Ubuntu 上工作但未测试 macOS 兼容性
- 使用系统路径（如 /home/）而不是相对路径
- 时区或时间相关的问题

# ✅ 解决方案
- 本地运行 ./scripts/pre_commit_test.sh (Ubuntu 环境)
- 查看 GitHub Actions 的具体错误
- 添加 #[cfg(target_os = "...")] 条件编译
```

### 错误 2: "Clippy 仍有警告"

```bash
# ❌ 常见的不行
cargo clippy --fix

# ✅ 应该
cargo clippy --fix --workspace --all-targets --all-features
# 然后手动检查修改
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

### 错误 3: "找不到测试文件"

```bash
# ❌ 错误的位置
crates/my-crate/src/tests/       # ❌ tests 应该和 src 平级
crates/my-crate/src/test_mod.rs  # ❌ 单元测试应该在源文件中

# ✅ 正确的位置
crates/my-crate/src/lib.rs       # 单元测试在这里
  #[cfg(test)]
  mod tests { }
  
crates/my-crate/tests/            # 集成测试在这里
  integration_test.rs
```

### 错误 4: "文档链接损坏"

```markdown
# ❌ 错误的链接
[链接](../../docs/ARCHITECTURE.md)    # 包含 ../../
[链接](/docs/ARCHITECTURE.md)         # 绝对路径
[链接](docs/ARCHITECTURE.md)          # Windows 路径问题

# ✅ 正确的链接
[链接](../docs/ARCHITECTURE.md)       # 相对路径，使用 /
[代码链接](../crates/core/src/file.rs#L10)  # 包含行号
```

---

## 🆘 需要帮助？

### 问题：代码检查失败，但不知道为什么

```bash
# 1. 查看详细的失败信息
./scripts/pre_commit_test.sh | head -50

# 2. 针对性修复某一项
cargo fmt --all
cargo clippy --workspace --all-targets --all-features
cargo test --workspace

# 3. 查看具体的测试失败
cargo test --lib -- --nocapture | grep -A 20 "test failures"
```

### 问题：不确定文件应该放在哪里

查看本页面的"📁 文件放置速查表"，或完整文档 [PROJECT_GOVERNANCE.md#文件放置规范](PROJECT_GOVERNANCE.md)

### 问题：不确定如何写文档

查看 [docs/AI_CODE_GENERATION_RULES.md#模式-1-添加新的工具函数](docs/AI_CODE_GENERATION_RULES.md)

---

**最后更新：** 2026年2月14日  
**有效范围：** 所有项目成员和 AI 辅助开发  
**强制执行：** Git Hook + GitHub Actions + Code Review
