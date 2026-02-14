# AI 开发者指南 (AI Developer Guide)

> **本文档专为 AI 辅助代码生成而创建。所有 AI 生成的代码和文档必须严格遵守本指南。**

---

## 🎯 首次阅读路线

### 第一步：理解项目的三项核心政策（必读）

1. **[COMMIT_POLICY.md](COMMIT_POLICY.md)** - 提交政策
   - 必须通过 7 项检查才能提交
   - 无任何例外

2. **[PROJECT_GOVERNANCE.md](PROJECT_GOVERNANCE.md)** - 完整治理规范
   - 项目结构规范
   - 文件放置规则
   - 代码风格指南
   - 文档模板

3. **[docs/AI_CODE_GENERATION_RULES.md](docs/AI_CODE_GENERATION_RULES.md)** - AI 代码生成规则
   - 绝对禁止事项
   - 必须做到的事项
   - 常见错误和修正

### 第二步：查看快速参考（实战查询）

**[QUICK_REFERENCE.md](QUICK_REFERENCE.md)** - 快速查询表
- 文件放置查询
- 检查清单
- 常见错误排查

### 第三步：开始工作

按照本指南的 **"工作流程"** 部分执行。

---

## 📋 工作流程（每次都要做这个）

### 阶段 1: 分析需求（IN -> OUT 思维）

在生成任何代码前，**必须明确回答**：

```
问题 1: 这是什么类型的工作？
- [ ] 新功能 (feature)
- [ ] 修复 bug (fix)
- [ ] 重构 (refactor)
- [ ] 文档 (docs)
- [ ] 性能优化 (perf)
- [ ] 测试 (test)

问题 2: 输出物是什么？
- [ ] Rust 代码 (.rs 文件)
- [ ] 文档 (.md 文件)
- [ ] 配置 (.toml, .yml 等)
- [ ] 脚本 (.sh 及其他)
- [ ] 多种文件组合

问题 3: 这些文件应该放在哪里？
- [ ] crates/*/src/              (Rust 源代码)
- [ ] crates/*/tests/            (集成测试)
- [ ] docs/                      (文档)
- [ ] scripts/                   (脚本)
- [ ] examples/                  (示例)
- [ ] 其他：_______

问题 4: 这个工作包含什么内容？
- [ ] 代码逻辑
- [ ] 单元测试
- [ ] 集成测试
- [ ] 文档注释
- [ ] 外部文档
- [ ] 配置更新
```

**如果无法回答以上任何问题，停止！询问用户或查阅现有代码。**

---

### 阶段 2: 规范检查（绝对禁止列表）

在生成**任何**代码前，检查：

#### 《严禁清单》

```
❌ 你是否计划在根目录创建新的目录或文件？
   → 停止！应该使用现有目录。

❌ 你是否计划创建 .backup, .disabled, .old 文件？
   → 停止！使用 git 历史，不要创建废弃文件。

❌ 你是否生成没有公开 API 文档的 pub fn？
   → 停止！添加 /// 文档注释。

❌ 你是否生成没有测试的新代码？
   → 停止！添加单元测试或集成测试。

❌ 你是否使用 unwrap(), panic!(), expect() 在库代码中？
   → 停止！使用 Result<T, E> 和 Option<T>。

❌ 你是否创建了一个 .md 文件但没有清晰的文件名？
   → 停止！使用 UPPERCASE_WITH_UNDERSCORES 格式。

❌ 你是否创建了文档但放在根目录而不是 docs/？
   → 停止！所有文档放在 docs/ 目录。

❌ 你是否在中文和英文之间混合？
   → 如果双语，必须有 _.md 和 _.en.md 两个文件。
   → 不要混在一个文件里。
```

**如果任何答案是 ❌，停止并修正。**

---

### 阶段 3: 文件结构规划（Location Planning）

对于每个要生成的文件，填写：

```
文件 1: [文件名和路径]
├─ 完整路径: crates/[name]/src/[file].rs
├─ 文件名规范: ✅ snake_case (for .rs)
├─ 需要更新的文件: 
│  ├─ src/lib.rs (添加 mod 声明)
│  ├─ src/mod.rs (添加导出)
│  └─ Cargo.toml (如有新依赖)
├─ 包含测试: ✅ #[cfg(test)] mod tests { ... }
├─ 包含文档: ✅ /// 和注释说明
└─ 检查点: 所有 pub 项都有文档? ✅

文件 2: [文件名和路径]
├─ 完整路径: docs/[FEATURE]_DESIGN.md
├─ 文件名规范: ✅ UPPERCASE_WITH_UNDERSCORES
├─ 包含中文版本: ✅
├─ 包含英文版本: ✅ (创建 .en.md)
├─ 遵循模板: ✅ (设计文档模板)
└─ 检查点: 是否在根目录? ❌ (应在 docs/)
```

---

### 阶段 4: 编码（Code Generation）

#### Code Style Checklist

在编写任何 Rust 代码前：

- [ ] 模块名: `snake_case` 
- [ ] 函数名: `snake_case` and private unless `pub`
- [ ] 结构体名: `PascalCase` and all public
- [ ] 常量名: `SCREAMING_SNAKE_CASE`
- [ ] 类型别名: `PascalCase`
- [ ] 所有 `pub` 有 `///` 文档
- [ ] 所有复杂逻辑有 `//` 说明注释
- [ ] 使用 `Result<T, E>` 替代 panic
- [ ] 所有 `pub` 函数有 `#[test]` 单元测试
- [ ] 添加了集成测试（`tests/` 目录）

#### Function Documentation Template

```rust
/// 简洁的一句话说明
///
/// # 详细说明
/// 更多说明（如果需要）
///
/// # 参数
/// - `param`: 说明
///
/// # 返回值
/// 说明返回值
///
/// # 错误
/// - `ErrorType`: 什么情况会发生这个错误
///
/// # 示例
/// ```
/// let result = function()?;
/// assert!(result.is_valid());
/// ```
pub fn function(param: Type) -> Result<Output, Error> { }
```

#### Test Pattern

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function_returns_correct_value() {
        // Arrange
        let input = setup_test_data();
        
        // Act
        let result = function(input)?;
        
        // Assert
        assert_eq!(result, expected);
    }

    #[test]
    fn test_function_handles_error() {
        // 测试错误情况
        let result = function(invalid_input)?;
        assert!(result.is_err());
    }
}
```

---

### 阶段 5: 文档生成（Documentation Generation）

对于文档文件，使用指定的模板：

#### 设计文档模板（Feature Design）

```markdown
# [功能名称] 设计文档

## 概述
简洁的功能描述。

## 设计目标
1. 目标 1
2. 目标 2

## 技术实现
### 方案描述
...

### 数据结构
```rust
// 关键数据结构
```

## API 定义
```rust
pub trait/fn ...
```

## 测试计划
- 单元测试：...
- 集成测试：...

## 风险评估
...
```

#### 完成报告模板（Completion Report）

```markdown
# Phase [N] 完成报告

## 📊 执行概览
- 时间范围：YYYY-MM-DD 到 YYYY-MM-DD
- 完成度：X%

## ✅ 已完成任务
- [x] 任务 1
- [x] 任务 2

## ❌ 遗留问题
(如果没有，写"无")

## 📈 关键指标
- 代码行数：X
- 测试覆盖率：X%
- 性能改进：X%

## 🔍 关键实现
### 实现 1 名称
...
```

---

### 阶段 6: 本地验证（Local Validation）

生成代码后，**必须在 AI 生成代码中的"实现计划"部分包含**：

```bash
# 运行这个命令
./scripts/pre_commit_test.sh

# 期望输出应该包含：
✅ 通过: 7
✅ 所有检查通过！
✅ 符合 GitHub Actions 规则，可以安全提交代码
```

**没有看到这个输出？修复所有问题直到看到为止。**

---

### 阶段 7: 提交验证（Commit Verification）

在生成"实现计划"或"提交指南"时，包含：

```bash
# 步骤 1: 添加文件
git add .

# 步骤 2: 验证 git 接受提交（不会被 pre-commit hook 拒绝）
git commit -m "type: subject"

# 步骤 3: GitHub Actions 会再次验证（应该全部通过）

# 步骤 4: 推送
git push
```

**如果任何步骤失败，返回阶段 4 修复代码。**

---

## 📄 代码类型和放置规范

### 新功能代码 (Feature)

```
放置位置： crates/[relevant-crate]/src/
文件：    [feature_name].rs
格式：    pub fn [name] + #[test] + ///
测试：    使用 mod tests { } 在源文件末尾
导出：    在 lib.rs 或 mod.rs 中添加 pub use
文档：    如有重大功能，在 docs/[FEATURE]_DESIGN.md
```

### Bug 修复 (Fix)

```
放置位置： 原始代码文件
文件：    修改现有文件
格式：    修改相关函数，保持风格一致
测试：    添加回归测试
文档：    更新相关说明
```

### 文档 (Documentation)

```
放置位置： docs/
文件名：   [DESCRIPTION]_[TYPE].md
          或 [FEATURE]_[TYPE].md
          或 PHASE_[N]_[TYPE].md
格式：     UPPERCASE_WITH_UNDERSCORES
模板：     使用指定的文档模板
语言：     提供 .md 和 .en.md 两个版本
```

### 测试 (Test)

```
放置位置（单元测试）： src/[file].rs 的 #[cfg(test)]
放置位置（集成测试）： crates/[crate]/tests/
命名：   test_[function]_[condition]_[expected]
格式：   使用 // Arrange // Act // Assert
覆盖：   正常情况 + 错误情况
```

---

## 🚫 严格禁止例表（Black List）

### 代码中的严禁

```rust
❌ fn function() { }       // 缺少文档和测试
❌ pub fn() { }            // pub 必须有 ///
❌ unwrap()                // 库代码中禁止
❌ todo!()                 // 不能提交
❌ dbg!(), println!()      // 不能在库代码中
❌ 硬编码数字              // 使用常量
❌ ignore 标记的测试      // 必须有相关问题追踪
```

### 文件结构中的严禁

```
❌ crates/src/            // 应该是 crates/*/src/
❌ src/tests/             // 应该是 tests/ 或 #[cfg(test)]
❌ /root/file.md          // 应该是 docs/file.md
❌ PHASE5.md              // 应该是 PHASE_5_[TYPE].md
❌ .backup, .disabled     // 不允许备份文件
❌ [file].md.bak          // 使用 git，不用备份
```

### 文档中的严禁

```
❌ 混合中英文            // 分为 .md 和 .en.md
❌ lowercase 文件名       // 必须 UPPERCASE
❌ 未结构化的内容        // 必须有标题和目录
❌ 损坏的链接            // 验证所有相对链接
❌ 缺少代码示例          // 复杂概念要有示例
```

---

## ✅ 必做事项检查表（Do List）

### 对于每个新的源文件

- [ ] 包含模块级文档（`//! module docs`）
- [ ] 所有 `pub` 项都有 `///` 文档
- [ ] 包含至少一个 `#[test]` 模块
- [ ] 所有公开函数都有测试
- [ ] 文档包含 `# 示例` 部分
- [ ] 错误使用 `Result<T, E>` 处理
- [ ] 常数使用 `const SCREAMING_CASE`

### 对于每个新的测试

- [ ] 用 `test_` 前缀命名
- [ ] 包含清晰的测试条件（test_x_with_y_returns_z）
- [ ] 有 Arrange / Act / Assert 结构
- [ ] 测试正常情况和错误情况

### 对于每个新的文档

- [ ] 文件在 `docs/` 目录
- [ ] 文件名使用 `UPPERCASE_WITH_UNDERSCORES.md`
- [ ] 包含一级标题 (`# Title`)
- [ ] 包含二级标题和目录
- [ ] 所有代码块有语言声明（```rust）
- [ ] 所有链接都是相对路径（../...）
- [ ] 提供 `_` 和 `_en` 两个版本

### 对于每个提交

- [ ] 运行 `./scripts/pre_commit_test.sh` 通过
- [ ] 所有 7 项检查都是 ✅
- [ ] 提交消息格式正确：`type: subject`
- [ ] 包含了所有修改的文件
- [ ] 提交消息描述了改动

---

## 🔄 常见场景的完整流程

### 场景 1: 添加新工具函数

```
1. 确定放置位置
   → crates/fingerprint-core/src/utils.rs (或新文件)

2. 编写代码
   ├─ /// 文档注释
   ├─ pub fn tool_name(...) -> Result<T, E>
   └─ #[cfg(test)] mod tests { ... }

3. 编写测试
   ├─ test_tool_works_with_valid_input
   └─ test_tool_handles_invalid_input

4. 更新导出
   └─ 在 lib.rs 或 mod.rs 添加 pub use

5. 运行验证
   └─ ./scripts/pre_commit_test.sh

6. 提交
   └─ git add . && git commit -m "feat: add tool_name function"
```

### 场景 2: 编写新特性文档

```
1. 确定文件位置
   → docs/[FEATURE]_DESIGN.md (中文)
   → docs/[FEATURE]_DESIGN.en.md (英文)

2. 选择模板
   └─ 使用 PROJECT_GOVERNANCE.md 中的设计文档模板

3. 编写结构
   ├─ # 一级标题
   ├─ ## 二级标题
   ├─ ### 三级标题
   ├─ 代码块 (```rust)
   └─ 相对链接 (../docs/...)

4. 验证格式
   └─ 所有标题正确，所有链接有效

5. 运行验证
   └─ ./scripts/pre_commit_test.sh (文档改动不会影响 7 项检查)

6. 提交
   └─ git add . && git commit -m "docs: add feature design documentation"
```

### 场景 3: 修复已知 Bug

```
1. 定位问题代码
   └─ 在 crates/[name]/src/[file].rs 中找到

2. 修复问题
   └─ 保持现有代码风格和模式

3. 添加回归测试
   └─ 在同一文件的 #[test] 中添加测试

4. 验证不引入新问题
   └─ cargo test --workspace

5. 运行完整验证
   └─ ./scripts/pre_commit_test.sh

6. 提交
   └─ git add . && git commit -m "fix: fix bug description"
```

---

## 🆘 问题排查

### 问题: "我不确定新代码应该放在哪个 crate"

**解决方案：**
1. 查看 [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) 了解各 crate 的职责
2. 询问用户或在现有相关 crate 中添加
3. 从不创建新的顶级目录

### 问题: "我需要添加新的 .md 文件"

**解决方案：**
1. 必须放在 `docs/` 目录
2. 使用 `UPPERCASE_WITH_UNDERSCORES.md` 格式
3. 如果是功能设计，遵循设计文档模板
4. 如果是阶段报告，遵循完成报告模板
5. 提供中英文两个版本（.md 和 .en.md）

### 问题: "本地检查失败"

**解决方案：**
```bash
# 查看具体是哪一项失败
./scripts/pre_commit_test.sh

# 根据失败项修复
cargo fmt --all              # 格式错误
cargo clippy --fix           # clippy 警告
cargo test --workspace       # 测试失败
cargo build --release        # 构建错误
```

### 问题: "GitHub Actions 失败但本地通过"

**可能原因：**
- 目标平台差异（macOS vs Ubuntu）
- 行尾符号问题（CRLF vs LF）
- 时区或浮点精度问题

**解决方案：**
1. 检查 GitHub Actions 日志的具体错误
2. 使用平台特定编译条件：`#[cfg(target_os = "...")]`
3. 查看测试是否有硬编码的路径或时间

---

## 📞 核心联系方式

| 问题 | 查找位置 |
|------|--------|
| 文件应该放在哪里 | [QUICK_REFERENCE.md#文件放置速查表](QUICK_REFERENCE.md) |
| 如何写注释和文档 | [PROJECT_GOVERNANCE.md#文档规范](PROJECT_GOVERNANCE.md) |
| 代码风格规范 | [PROJECT_GOVERNANCE.md#代码风格指南](PROJECT_GOVERNANCE.md) |
| AI 禁止做的事 | [docs/AI_CODE_GENERATION_RULES.md#绝对禁止](docs/AI_CODE_GENERATION_RULES.md) |
| 提交前检查清单 | [QUICK_REFERENCE.md#事前检查清单](QUICK_REFERENCE.md) |
| 常见错误解决 | [docs/AI_CODE_GENERATION_RULES.md#问题排查](docs/AI_CODE_GENERATION_RULES.md) |

---

## 前置条件：在你开始之前

### 清单 ✅

在任何工作开始前，确认：

- [ ] 我已阅读 [COMMIT_POLICY.md](COMMIT_POLICY.md)
- [ ] 我已阅读 [PROJECT_GOVERNANCE.md](PROJECT_GOVERNANCE.md)
- [ ] 我已阅读 [docs/AI_CODE_GENERATION_RULES.md](docs/AI_CODE_GENERATION_RULES.md)
- [ ] 我理解 7 项强制检查的含义
- [ ] 我知道文件应该放在哪些位置
- [ ] 我理解代码必须包含文档和测试
- [ ] 我知道如何使用提供的文档模板
- [ ] 我理解违反规则的后果

**如果任何项目是 ❌，停止并重新阅读相关文档。**

---

**最后更新：** 2026年2月14日  
**强制执行：** 所有 AI 辅助开发  
**适用范围：** fingerprint-rust 项目
