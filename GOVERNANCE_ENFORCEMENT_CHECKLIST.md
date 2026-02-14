# 规范执行检查清单 (Governance Enforcement Checklist)

> **本清单用于最后验证所有生成的代码和文档是否符合项目规范。**
> **这是提交前的最后一道关卡。**

---

## 🚀 使用方法

### 对于代码生成

1. 生成代码完成后
2. 按照下面的清单逐项检查
3. **必须全部通过**才能提交
4. 如有任何项目失败，返回代码生成阶段修正

### 对于代码审查

1. 接收到代码提交申请
2. 按照清单逐项验证
3. 任何不符合项目直接拒绝
4. 要求提交者按照清单修正

---

## ✅ Level 1: 文件结构检查 (File Structure Validation)

### L1.1 目录和文件位置

验证点：
```
- [ ] Rust 源代码在 crates/*/src/ 中
- [ ] 每个 crate 都有 Cargo.toml 文件
- [ ] 每个 crate 都有 src/lib.rs 文件
- [ ] 每个 crate 都有 README.md 文件（中文）✅ 必须
- [ ] README.md 描述了模块的功能点和主要 API
- [ ] 测试代码在 crates/*/tests/ 或 src/tests 模块中
- [ ] 文档在 docs/ 目录中
- [ ] 脚本在 scripts/ 目录中
- [ ] 示例在 examples/ 目录中
- [ ] 配置在 config/ 目录中
- [ ] 数据在 dataset/ 或 data/ 目录中
- [ ] 模型在 models/ 目录中
```

**失败处理：** 
- ❌ 如果任何文件在错误的位置，拒绝提交
- ✅ 修正后重新提交

### L1.2 文件命名规范

验证点：
```
- [ ] Rust 文件：snake_case.rs
- [ ] 文档文件：UPPERCASE_WITH_UNDERSCORES.md
- [ ] 脚本文件：snake_case.sh
- [ ] 配置文件：snake_case.toml 或 .yml
- [ ] 没有 .backup, .disabled, .old, .bak 文件
- [ ] 没有临时文件或测试文件在根目录
```

**失败处理：** 
- ❌ 不符合命名规范，拒绝提交
- ✅ 重命名后重新提交

### L1.3 项目结构完整性

验证点：
```
- [ ] 没有创建新的顶级目录（除非特别批准）
- [ ] 所有修改都在现有的目录结构内
- [ ] Cargo.toml 文件没有被意外修改
- [ ] .gitignore 包含所有临时文件和输出
```

**失败处理：** 
- ❌ 违反项目结构，拒绝提交
- ✅ 恢复原有结构后重新提交

---

## ✅ Level 2: 代码质量检查 (Code Quality Validation)

### L2.1 代码风格

验证点：
```
- [ ] cargo fmt --all -- --check 通过 (无格式错误)
- [ ] cargo clippy --workspace --all-targets --all-features 无警告
- [ ] 没有 TODO, FIXME 注释未处理
- [ ] 没有注释掉的代码
```

**失败处理：** 
```bash
# 自动修复
cargo fmt --all
cargo clippy --fix --workspace

# 验证修复
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features
```

### L2.2 文档注释

验证点：
```
对于所有 pub 项：
- [ ] 有 /// 文档注释（或 //! 对于模块）
- [ ] 文档注释解释了 *为什么*，不仅仅是 *是什么*
- [ ] 复杂函数包含 # 参数和 # 返回值
- [ ] 可能返回错误的函数包含 # 错误 部分
- [ ] 所有公开函数包含 # 示例 代码块
- [ ] 示例代码是可运行的（能通过 cargo test --doc）

对于模块：
- [ ] 模块顶部有 //! 文档注释
- [ ] 文档说明模块的用途和主要 API
```

**失败处理：** 
- ❌ 缺少文档注释，拒绝提交
- ✅ 添加文档后重新提交

### L2.3 代码安全性

验证点：
```
- [ ] 没有不必要的 unsafe 块
- [ ] 如果有 unsafe，有 // SAFETY: 说明为什么安全
- [ ] 没有 unwrap() 和 expect() 在库代码中
- [ ] 没有忽略的错误（所有 Result 都被处理）
- [ ] 没有 panic! 在公开 API 中
- [ ] 参数验证清晰（检查空、范围等）
```

**失败处理：** 
- ❌ 代码安全问题，拒绝提交
- ✅ 修正后重新提交

### L2.4 命名规范

验证点：
```
模块和函数：
- [ ] 函数名：snake_case
- [ ] 模块名：snake_case
- [ ] 结构体名：PascalCase
- [ ] 枚举名：PascalCase
- [ ] 常量名：SCREAMING_SNAKE_CASE
- [ ] 类型别名：PascalCase
- [ ] 特征名：PascalCase
- [ ] 没有单字母变量（除了 i, j 作为循环计数器）
```

**失败处理：** 
- ❌ 不符合命名规范，拒绝提交
- ✅ 重命名后重新提交

---

## ✅ Level 3: 测试检查 (Testing Validation)

### L3.1 单元测试

验证点：
```
对每个 pub fn：
- [ ] 有至少一个 #[test]
- [ ] 测试名称：test_[function]_[condition]_[expected]
- [ ] 测试包含正常情况（happy path）
- [ ] 测试包含错误情况（if applicable）
- [ ] 测试有 // Arrange // Act // Assert 注释
- [ ] 测试不依赖外部资源或系统状态

对于模块：
- [ ] 有 #[cfg(test)] mod tests { } 块
- [ ] 测试数量足够（通常最少 2-3 个）
- [ ] 测试覆盖主要代码路径
```

**失败处理：** 
```bash
# 验证测试
cargo test --lib --verbose
cargo test --workspace --verbose

# 检查覆盖率
cargo tarpaulin --workspace
```

### L3.2 集成测试

验证点：
```
对于新的公开 crate/模块：
- [ ] 在 crates/[name]/tests/ 中有集成测试
- [ ] 测试从"用户"角度使用 API
- [ ] 测试覆盖常见使用场景
- [ ] 测试包含错误处理场景
```

**失败处理：** 
- ❌ 缺少必要的集成测试，拒绝提交
- ✅ 添加后重新提交

### L3.3 测试执行

验证点：
```
- [ ] cargo test --lib 全部通过
- [ ] cargo test --workspace 全部通过
- [ ] cargo test --doc 全部通过（文档示例）
- [ ] 没有忽略的测试（#[ignore] 需有理由）
```

**失败处理：** 
```bash
# 运行测试查看失败
cargo test --workspace -- --nocapture

# 修复失败的测试
# （修正代码或测试预期值）

# 重新验证
cargo test --workspace --verbose
```

---

## ✅ Level 4: 编译检查 (Compilation Validation)

### L4.1 编译成功

验证点：
```
- [ ] cargo check --all-features 无错误
- [ ] cargo build --release 无错误
- [ ] 没有编译警告（在库代码中）
```

**失败处理：** 
```bash
# 检查具体错误
cargo check --all-features --verbose

# 修复错误

# 验证
cargo build --release --verbose
```

### L4.2 特性编译

验证点：
```
对于包含条件编译的代码：
- [ ] 支持所有指定的 features
- [ ] cargo build --all-features 成功
- [ ] cargo build --no-default-features 成功（如适用）
- [ ] 没有特性相关的编译错误
```

**失败处理：** 
```bash
# 检查特性配置
cargo build --all-features --verbose
cargo build --no-default-features --verbose
```

---

## ✅ Level 5: 文档检查 (Documentation Validation)

### L5.1 文档位置

验证点：
```
- [ ] 所有文档在 docs/ 目录
- [ ] 根目录只有：README.md, README.en.md, CONTRIBUTING.md, COMMIT_POLICY.md, PROJECT_GOVERNANCE.md, QUICK_REFERENCE.md
- [ ] 没有其他 .md 文件在根目录
```

**失败处理：** 
- ❌ 文档在错误的位置，拒绝提交
- ✅ 移动后重新提交

### L5.2 文档命名

验证点：
```
- [ ] 使用 UPPERCASE_WITH_UNDERSCORES.md 格式
- [ ] 对于双语文档，分为 NAME.md 和 NAME.en.md
- [ ] 文件名清晰反映内容
- [ ] 没有 lowercase 或 camelCase 文件名
```

**失败处理：** 
- ❌ 命名不符合，拒绝提交
- ✅ 重命名后重新提交

### L5.3 文档结构

验证点：
```
对于每个文档：
- [ ] 有一级标题 (# Title)
- [ ] 有清晰的二级标题 (## Section)
- [ ] 内容用三级标题或以下组织 (### Subsection)
- [ ] 有目录或导航（对于长文档）
- [ ] 所有代码块有语言声明（```rust, ```bash 等）
- [ ] 所有代码块是有效的
```

**失败处理：** 
- ❌ 结构不清晰，拒绝提交
- ✅ 重新组织后重新提交

### L5.4 文档内容

验证点：
```
- [ ] 没有拼写错误
- [ ] 没有语法错误
- [ ] 所有链接都是相对路径（../...）
- [ ] 没有损坏的链接
- [ ] 代码示例是准确的
- [ ] 包含相关的背景信息
- [ ] 长文档包含导航链接
```

**失败处理：** 
- ❌ 内容有问题，拒绝提交
- ✅ 修正后重新提交

### L5.5 文档模板遵守

验证点：

对于设计文档（Design Doc）：
```
- [ ] 包含 概述 章节
- [ ] 包含 设计目标 章节
- [ ] 包含 技术方案 章节
- [ ] 包含 API定义 或代码示例
- [ ] 包含 测试计划 章节
- [ ] 包含 风险评估 章节
```

对于完成报告（Completion Report）：
```
- [ ] 包含 执行概览 章节
- [ ] 包含 已完成任务 列表
- [ ] 包含 遗留问题 列表
- [ ] 包含 指标 章节
- [ ] 包含 关键实现 说明
```

对于执行计划（Execution Plan）：
```
- [ ] 包含 目标 说明
- [ ] 包含 任务分解 (WBS)
- [ ] 包含 时间表
- [ ] 包含 依赖关系
- [ ] 包含 风险评估
```

**失败处理：** 
- ❌ 不遵守模板，拒绝提交
- ✅ 按模板重写后重新提交

---

## ✅ Level 6: 提交检查 (Commit Validation)

### L6.1 本地测试

验证点：
```bash
# 运行完整的预提交检查
./scripts/pre_commit_test.sh

期望输出：
✅ 通过: 7
✅ 所有检查通过！
✅ 符合 GitHub Actions 规则，可以安全提交代码

- [ ] 代码格式化通过
- [ ] Lint 检查通过
- [ ] 编译检查通过
- [ ] 单元测试通过
- [ ] 集成测试通过
- [ ] 安全审计通过
- [ ] 发布构建通过
```

**失败处理：** 
```bash
# 查看失败项并修复
./scripts/pre_commit_test.sh | grep -A 5 "❌"

# 修复相应项

# 重新运行
./scripts/pre_commit_test.sh
```

### L6.2 提交消息

验证点：
```
- [ ] 格式遵循约定式提交：type: subject
- [ ] 类型正确：feat, fix, docs, style, refactor, perf, test, chore
- [ ] 主体描述清晰明了
- [ ] 没有"update"或"fix bug"这样笼统的消息
- [ ] 如有详细说明，用换行分隔
- [ ] 包含相关的 issue 号（如有）
```

**失败处理：** 
- ❌ 提交消息不符合，拒绝提交
- ✅ 修正消息后重新提交

### L6.3 提交内容

验证点：
```
- [ ] 包含所有相关的改动
- [ ] 没有包含不相关的文件
- [ ] 没有包含 target/, output/, .vscode/ 等临时目录
- [ ] 没有包含未追踪的配置文件
```

**失败处理：** 
```bash
# 检查要提交的文件
git status
git diff --cached

# 如有不必要的文件，取消暂存
git reset HEAD [unwanted_files]

# 重新提交
git add [correct_files]
git commit -m "..."
```

---

## ✅ Level 7: GitHub Actions 检查 (GitHub Actions Validation)

### L7.1 CI/CD 运行

验证点：
```
在 GitHub 上：
- [ ] 所有 GitHub Actions workflows 通过
- [ ] ci.yml 的所有检查通过（Ubuntu, macOS, Windows）
- [ ] security-audit.yml 通过（无已知漏洞）
- [ ] 没有部分通过（必须全部通过）
```

**失败处理：** 
1. 检查 GitHub Actions 的详细日志
2. 查看具体的失败步骤
3. 在本地重现问题
4. 修复代码
5. 重新推送

### L7.2 PR 合并准备

验证点：
```
如果这是 PR：
- [ ] 所有 CI/CD 检查通过
- [ ] 没有冲突
- [ ] 至少一个代码审查通过
- [ ] 提交消息清晰
```

**失败处理：** 
- ❌ 任何项目失败，不能合并
- ✅ 修正后再次推送

---

## 📋 快速检查清单（Fast Track）

### 对于代码提交：

```bash
# 运行这一条命令
./scripts/pre_commit_test.sh

# 如果看到这个：
✅ 所有检查通过！

# 那么继续：
git add .
git commit -m "type: description"
git push

# 完成！
```

### 对于文档提交：

```bash
# 检查清单：
- [ ] 文件在 docs/ 目录
- [ ] 文件名 = UPPERCASE_WITH_UNDERSCORES.md
- [ ] 有中英文版本（.md 和 .en.md）
- [ ] 遵循指定的模板
- [ ] 所有链接都是相对路径
- [ ] 没有拼写错误

# 提交：
git add docs/
git commit -m "docs: add documentation"
git push
```

---

## 🚫 绝对不允许的情况

### 绝对禁止 1: 跳过检查

```
❌ git commit --no-verify              不允许！
❌ 跳过 ./scripts/pre_commit_test.sh   不允许！
❌ 知道有错误但仍然推送                不允许！
```

**后果：** 提交会被 git hook 拒绝，GitHub Actions 会失败

### 绝对禁止 2: 不完整的工作

```
❌ 代码有 clippy 警告但声称通过       不允许！
❌ 缺少测试但声称完成                  不允许！
❌ 缺少文档注释但声称完成              不允许！
```

**后果：** 代码审查拒绝，要求修改

### 绝对禁止 3: 结构混乱

```
❌ 创建新的顶级目录                     不允许！
❌ 文件放在错误的位置                  不允许！
❌ 文档放在根目录                      不允许！
```

**后果：** 提交被拒绝，需要重组织

---

## ✨ 审查者指南

### 审查代码时的检查顺序

1. **L1**: 文件结构是否正确？
2. **L2**: 代码质量是否符合标准？
3. **L3**: 测试是否充分？
4. **L4**: 代码是否能编译？
5. **L5**: 文档是否完整？
6. **L6**: 提交消息是否清晰？
7. **L7**: GitHub Actions 是否全部通过？

### 拒绝的标准

任何 L1-L5 的项目不符合，**直接拒绝**：

```
审查意见：
- 该提交不符合项目规范的 L[N] - [具体项目]
- 需要修正后重新提交
- 具体要求见：[文档链接]
```

### 批准的标准

所有 L1-L7 都通过：

```
✅ 批准
所有检查通过，符合项目规范。
- L1 ✅ 文件结构正确
- L2 ✅ 代码质量符合标准
- L3 ✅ 测试充分
- L4 ✅ 编译无误
- L5 ✅ 文档完整
- L6 ✅ 提交清晰
- L7 ✅ GitHub Actions 通过
```

---

## 📞 常见问题

### Q: 如果代码通过了本地检查但 GitHub Actions 失败怎么办？

A: 
1. 检查 GitHub Actions 日志找出具体原因
2. 通常是平台差异（macOS vs Ubuntu）
3. 修复代码，确保跨平台兼容
4. 重新推送

### Q: 可以跳过某些检查吗？

A: **不可以。** 所有 7 项检查都是强制性的，无任何例外。

### Q: 文档可以只有中文吗？

A: 理想情况下应该有中英文两个版本。如果只有一种语言，需要在文件中清楚注明原因。

### Q: 过时的 TODO 注释怎么办？

A: 要么完成 TODO，要么删除注释，不允许在提交的代码中有 TODO 注释。

### Q: 单元测试和集成测试都是必需的吗？

A: 是的。单元测试测试单个函数的行为，集成测试测试整个模块的使用。

---

**最后更新：** 2026年2月14日  
**强制执行：** 所有代码和文档提交  
**执行权限：** 代码审查者和自动化工具  
**违规后果：** 提交拒绝，要求修改和重新提交
