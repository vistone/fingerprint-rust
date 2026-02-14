# 项目治理规范 (Project Governance)

## 📋 文档概述

本文档规定了 fingerprint-rust 项目的所有开发规范。**所有代码、文档和工作流都必须遵守本规范。**

---

## 🏗️ 第一部分：项目结构规范

### 1.1 顶级目录结构

```
fingerprint-rust/
├── benches/                     # 性能基准测试
├── crates/                      # 所有 Rust crates
├── config/                      # 配置文件
├── data/                        # 静态数据
├── dataset/                     # 训练数据集
├── docs/                        # 项目文档
├── examples/                    # 示例代码
├── fuzz/                        # Fuzzing 测试
├── models/                      # 机器学习模型
├── output/                      # 生成的输出（临时）
├── phase7_api/                  # API 定义和生成
├── phase7_results/              # 执行结果（临时）
├── scripts/                     # 实用脚本
├── target/                      # cargo 构建输出（.gitignore）
├── tests/                       # 集成测试
├── Cargo.toml                   # Workspace 定义
├── Cargo.lock                   # 依赖版本锁定
├── rust-toolchain.toml          # Rust 版本定义
├── Makefile                     # 开发任务
├── COMMIT_POLICY.md             # 提交政策
├── PROJECT_GOVERNANCE.md        # 本文件
├── CONTRIBUTING.md              # 贡献指南
├── README.md                    # 中文说明
├── README.en.md                 # 英文说明
└── deny.toml                    # 安全审计配置
```

**规则：**
- ⚠️ 不允许在顶级目录创建新目录，除非经过代码审查
- ⚠️ 临时输出必须放在 `output/` 或 `phase7_results/` 中
- ⚠️ 所有生成的文件都应该在 `.gitignore` 中

### 1.2 Crates 目录结构

```
crates/
├── README.md                    # Crates 总体说明
├── fingerprint/                 # 主 crate
│   ├── Cargo.toml               # ✅ 必须
│   ├── README.md                # ✅ 必须：模块功能说明
│   └── src/
│       ├── lib.rs               # ✅ 必须：库入口
│       ├── main.rs              # 二进制入口（如果有）
│       └── ...
├── fingerprint-core/            # 核心库
│   ├── Cargo.toml               # ✅ 必须
│   ├── README.md                # ✅ 必须：模块功能说明
│   └── src/
│       └── lib.rs               # ✅ 必须
├── fingerprint-gateway/         # 网关相关
│   ├── Cargo.toml               # ✅ 必须
│   ├── README.md                # ✅ 必须：模块功能说明
│   └── src/
│       └── lib.rs               # ✅ 必须
├── fingerprint-defense/         # 防护机制
├── fingerprint-ml/              # 机器学习模块
├── fingerprint-profiles/        # 浏览器配置文件
├── fingerprint-[feature]/       # 其他功能模块
└── [其他 crates]
```

**强制规则（必须遵守）：**
- ✅ **必须有** `Cargo.toml` - crate 的配置文件
- ✅ **必须有** `src/lib.rs` - 库的主入口
- ✅ **必须有** `README.md` - 模块功能说明（中文）
- ✅ **建议有** `README.en.md` - 模块功能说明（英文）
- ⚠️ crate 名称必须使用 `fingerprint-*` 前缀
- ⚠️ 不允许创建 `src/bin/` 目录用于多个二进制，应该创建独立的 crate
- ⚠️ 每个 crate 的 `src/lib.rs` 开头必须包含模块文档注释（`//!`）

### 1.3 Docs 目录结构

```
docs/
├── README.md                    # 文档索引（中文）
├── INDEX.md                     # 文档索引（中文）
├── INDEX.en.md                  # 文档索引（英文）
├── ARCHITECTURE.md              # 架构设计（中文）
├── ARCHITECTURE.en.md           # 架构设计（英文）
├── API.md                       # API 文档
├── CONTRIBUTING.md              # 贡献指南
│
├── PHASE_[N]/                   # 分阶段文档
│   ├── PHASE_N_COMPLETION_REPORT.md
│   ├── PHASE_N_EXECUTION_PLAN.md
│   └── PHASE_N_*_SUMMARY.md
│
├── FEATURE_[NAME]/              # 功能相关文档
│   ├── [FEATURE]_DESIGN.md
│   ├── [FEATURE]_IMPLEMENTATION.md
│   └── [FEATURE]_TESTING.md
│
└── CONFIG/                      # 配置相关
    ├── BROWSER_VERSION_ADAPTATION.md
    ├── FIREFOX_CAPTURE_GUIDE.md
    └── HTTP2_INTEGRATION_GUIDE.md
```

**规则：**
- ⚠️ 所有项目相关文档必须放在 `docs/` 目录
- ⚠️ 文档应提供中文和英文版本
- ⚠️ 文件名使用 UPPERCASE_WITH_UNDERSCORES 格式
- ⚠️ 不允许在根目录创建 `*.md` 文件（除了 README, LICENSE, COMMIT_POLICY, PROJECT_GOVERNANCE）

### 1.4 Scripts 目录结构

```
scripts/
├── pre_commit_test.sh           # 强制性：提交前检查
├── build.sh                     # 构建脚本
├── test.sh                      # 测试脚本
├── deploy.sh                    # 部署脚本
├── setup.sh                     # 环境设置
└── [其他实用脚本]
```

**规则：**
- ⚠️ 每个脚本必须有清晰的头部注释（目的、使用方法）
- ⚠️ 脚本必须是可执行的（chmod +x）
- ⚠️ 脚本应该支持跨平台（Linux, macOS, Windows）

---

## 📄 第二部分：文件放置规范

### 2.1 代码文件

| 文件类型 | 位置 | 说明 |
|---------|------|------|
| 源代码 | `crates/*/src/` | Rust 源代码 |
| 单元测试 | `crates/*/src/` (inline) | 在源文件中使用 `#[cfg(test)]` |
| 集成测试 | `crates/*/tests/` | 独立的测试文件 |
| 示例代码 | `examples/` | 演示用途的可运行代码 |
| 基准测试 | `benches/` | 性能基准 |
| Fuzzing | `fuzz/` | Fuzzing 测试用例 |

**规则：**
- ⚠️ 单元测试应该在源文件末尾，不应单独成文件
- ⚠️ 集成测试应该放在 `tests/` 目录
- ⚠️ 所有示例代码必须是可运行的（cargo run --example...）
- ⚠️ 禁止在 `.rs` 文件中放置中文注释（除非是文档注释）

### 2.2 数据文件

| 数据类型 | 位置 | 说明 |
|---------|------|------|
| 配置文件 | `config/` | 部署、监控、服务配置 |
| 训练数据 | `dataset/` | ML 训练数据集 |
| 静态数据 | `data/` | 应用使用的静态数据 |
| 模型文件 | `models/` | 训练好的 ML 模型 |
| 生成的输出 | `output/` | 临时生成的文件 |

**规则：**
- ⚠️ 大型数据文件应该使用 Git LFS
- ⚠️ 不允许将数据文件放在 `src/` 中
- ⚠️ 临时输出必须在 `.gitignore` 中

### 2.3 文档文件

| 文档类型 | 位置 | 命名格式 |
|---------|------|---------|
| API 文档 | `docs/API.md` | `API.md` |
| 架构文档 | `docs/ARCHITECTURE*.md` | `ARCHITECTURE*.md` |
| 阶段报告 | `docs/PHASE_*_*.md` | `PHASE_[N]_COMPLETION_REPORT.md` |
| 功能设计 | `docs/[FEATURE]_DESIGN.md` | `[FEATURE]_DESIGN.md` |
| 贡献指南 | `CONTRIBUTING.md` 或 `docs/CONTRIBUTING.md` | `CONTRIBUTING.md` |
| 项目说明 | `README.md`, `README.en.md` | `README*.md` |

**规则：**
- ⚠️ 文档必须放在 `docs/` 目录（README 和 CONTRIBUTING 除外）
- ⚠️ 所有文件名使用 UPPERCASE_WITH_UNDERSCORES
- ⚠️ 提供中文和英文版本时，使用 `_.md` 和 `_.en.md` 后缀

---

## 📝 第三部分：文档规范

### 3.1 文档类型和模板

#### 3.1.1 完成报告 (Completion Report)

**位置：** `docs/PHASE_[N]_COMPLETION_REPORT.md`

**内容：**
```markdown
# Phase [N] 完成报告

## 📊 执行概览
- 时间范围：YYYY-MM-DD 到 YYYY-MM-DD
- 完成度：X%
- 主要成果：列表

## ✅ 已完成任务
- [ ] 任务 1
- [ ] 任务 2

## ❌ 遗留问题
- [ ] 问题 1
- [ ] 问题 2

## 📈 指标
- 代码行数：X
- 测试覆盖率：X%
- 性能提升：X%

## 🔍 关键实现
### 实现 1 名称
详细描述...

## 📚 相关文档
- 链接 1
- 链接 2
```

#### 3.1.2 设计文档 (Design Document)

**位置：** `docs/[FEATURE]_DESIGN.md`

**内容：**
```markdown
# [功能名称] 设计文档

## 概述
简洁的功能描述...

## 设计目标
1. 目标 1
2. 目标 2

## 技术方案
### 方案描述
详细说明...

### 数据结构
```rust
// 关键数据结构
```

### 流程图
ASCII 或 Mermaid 图...

## API 定义
```rust
pub trait/fn ...
```

## 测试计划
- 单元测试：...
- 集成测试：...
- 性能测试：...

## 风险和缓解
| 风险 | 影响 | 缓解方案 |
|-----|------|--------|
```

#### 3.1.3 执行计划 (Execution Plan)

**位置：** `docs/PHASE_[N]_EXECUTION_PLAN.md`

**内容：**
```markdown
# Phase [N] 执行计划

## 目标
清晰列出本阶段目标...

## 任务分解 (WBS)
1. 任务组 1
   - [ ] 子任务 1.1
   - [ ] 子任务 1.2
2. 任务组 2
   ...

## 时间表
| 任务 | 开始 | 结束 | 负责人 |
|-----|------|------|--------|

## 依赖关系
```
任务A → 任务B → 任务C
```

## 风险评估
- 风险 1：...
```

### 3.2 文档写作规范

#### 3.2.1 标题等级
```markdown
# 一级标题 (文档标题)
## 二级标题 (主要章节)
### 三级标题 (小节)
#### 四级标题 (细节)
```

#### 3.2.2 列表格式
```markdown
# 有序列表
1. 第一项
2. 第二项
3. 第三项

# 无序列表
- 项目 1
- 项目 2
  - 子项目 2.1
  - 子项目 2.2

# 复选列表
- [ ] 未完成任务
- [x] 已完成任务
```

#### 3.2.3 代码块
```markdown
# 简单代码
`code`

# 代码块
​```rust
fn main() {
    println!("Hello");
}
​```

# 带标题的代码块
​```rust
// src/lib.rs
pub fn function() {}
​```
```

#### 3.2.4 表格
```markdown
| 列1 | 列2 | 列3 |
|-----|-----|-----|
| 值1 | 值2 | 值3 |
```

#### 3.2.5 链接和引用
```markdown
# 相对链接
[链接文本](../docs/FILE.md)
[代码链接](../crates/crate-name/src/file.rs#L10-L20)

# 绝对链接
[GitHub](https://github.com/vistone/fingerprint-rust)

# 脚注
文本[^1]
[^1]: 脚注内容
```

#### 3.2.6 强调和格式
```markdown
**粗体** / __粗体__
*斜体* / _斜体_
***粗斜体***

~~删除线~~

`代码` 内联

> 引用文本
> 多行引用
```

### 3.3 文档审查清单

任何文档提交前必须检查：

- [ ] 文件放在正确的目录
- [ ] 文件名使用正确的格式（UPPERCASE_WITH_UNDERSCORES）
- [ ] 包含清晰的标题和描述
- [ ] 所有代码块语法正确
- [ ] 链接都有效（相对链接）
- [ ] 表格格式正确且增值
- [ ] 没有拼写或语法错误
- [ ] 如需要，提供中文和英文版本
- [ ] 符合二级标题的结构

---

## 💻 第四部分：代码风格指南

### 4.1 Rust 代码风格

#### 4.1.1 基本规则
- 遵循 [Rust Style Guide](https://doc.rust-lang.org/1.0.0/style/)
- 使用 `cargo fmt` 自动格式化
- 所有代码必须通过 `cargo clippy` 检查

#### 4.1.2 命名规范

```rust
// 模块：snake_case
mod http_fingerprint;

// 类型/结构体：PascalCase
struct BrowserFingerprint {}
trait FingerprintProvider {}
enum FingerprintType {}

// 函数/方法：snake_case
fn calculate_fingerprint() {}
pub fn get_metrics() {}

// 常量：SCREAMING_SNAKE_CASE
const MAX_RETRIES: u32 = 3;
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

// 类型别名：PascalCase
type FingerprintId = String;
```

#### 4.1.3 文档注释

```rust
/// 模块级文档
//! 这是模块文档

/// 函数文档
///
/// # 说明
/// 详细说明...
///
/// # 参数
/// - `param1`: 参数说明
///
/// # 返回值
/// 返回值说明
///
/// # 错误
/// 可能的错误...
///
/// # 示例
/// ```
/// let result = function();
/// assert_eq!(result, expected);
/// ```
pub fn function(param1: String) -> Result<String, Error> {
    todo!()
}
```

#### 4.1.4 测试规范

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_describes_what_it_tests() {
        // Arrange
        let input = prepare_test_data();
        
        // Act
        let result = function_under_test(input);
        
        // Assert
        assert_eq!(result, expected_value);
    }

    #[test]
    #[should_panic(expected = "error message")]
    fn test_panic_case() {
        function_that_panics();
    }

    #[tokio::test]
    async fn test_async_function() {
        let result = async_function().await;
        assert!(result.is_ok());
    }

    #[ignore]
    #[test]
    fn test_expensive_operation() {
        // This test is ignored by default
        // Run with: cargo test -- --ignored
    }
}
```

#### 4.1.5 错误处理

```rust
// 使用 Result 和 ?
fn operation() -> Result<String, Box<dyn std::error::Error>> {
    let data = read_file()?;
    let processed = process(data)?;
    Ok(processed)
}

// 自定义错误类型
#[derive(Debug)]
pub enum FingerprintError {
    InvalidInput(String),
    ProcessingFailed(String),
}

impl std::fmt::Display for FingerprintError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
            Self::ProcessingFailed(msg) => write!(f, "Processing failed: {}", msg),
        }
    }
}

impl std::error::Error for FingerprintError {}
```

### 4.2 注释规范

#### 4.2.1 何时写注释
- ✅ **应该写：** 解释 *为什么* 而不是 *做什么*
- ✅ **应该写：** 复杂算法的高层说明
- ✅ **应该写：** 重要的设计决策和权衡
- ❌ **不应该写：** 重复代码本身说明的内容
- ❌ **不应该写：** 过时的注释

#### 4.2.2 注释格式
```rust
// 单行注释：空格后跟内容

/// 文档注释：用于公开 API
pub fn public_function() {}

//! 模块文档：在模块开头

/* 块注释：仅在特殊情况使用 */

// TODO: 待完成的任务
// FIXME: 需要修复的问题
// SAFETY: 说明为什么 unsafe 代码是安全的
unsafe {
    // implementation
}
```

---

## ✅ 第五部分：AI 代码生成规则

### 5.1 强制性规则

#### 规则 1：必须遵守项目结构
- ❌ **禁止：** 创建新的顶级目录
- ❌ **禁止：** 将代码放在错误的位置
- ✅ **必须：** 在现有的 `crates/` 中添加代码
- ✅ **必须：** 遵守文件命名规范

#### 规则 2：不允许乱写报告
- ❌ **禁止：** 在根目录创建 `*.md` 文件
- ❌ **禁止：** 创建无结构的文档
- ✅ **必须：** 使用指定的模板和位置
- ✅ **必须：** 提供完整的中英文版本

#### 规则 3：必须编写文档
- ❌ **禁止：** 没有 doc comments 的公开 API
- ❌ **禁止：** 没有说明的复杂函数
- ✅ **必须：** 为所有 `pub` 函数添加文档
- ✅ **必须：** 包含示例和用法说明

#### 规则 4：必须包含测试
- ❌ **禁止：** 没有测试的功能代码
- ❌ **禁止：** 忽略失败的测试
- ✅ **必须：** 为新功能添加单元测试
- ✅ **必须：** 为公开 API 添加集成测试

#### 规则 5：必须通过所有检查
- ❌ **禁止：** 提交未通过 `cargo fmt` 的代码
- ❌ **禁止：** 提交有 clippy 警告的代码
- ❌ **禁止：** 提交失败的测试
- ✅ **必须：** 运行 `./scripts/pre_commit_test.sh` 确保全部通过

### 5.2 提交前检查清单

在提交任何代码前，AI 必须确保：

- [ ] ✅ 所有文件在正确的目录
- [ ] ✅ 所有文件名符合命名规范
- [ ] ✅ 代码通过 `cargo fmt` 格式化
- [ ] ✅ 代码通过 `cargo clippy` 检查
- [ ] ✅ 代码成功编译（`cargo check`）
- [ ] ✅ 所有测试通过（`cargo test`）
- [ ] ✅ 安全审计通过（`cargo deny`）
- [ ] ✅ 发布构建通过（`cargo build --release`）
- [ ] ✅ 所有公开 API 有文档注释
- [ ] ✅ 添加了适当的单元/集成测试
- [ ] ✅ 文档放在 `docs/` 目录（如有）
- [ ] ✅ 使用了指定的模板和格式（如有）
- [ ] ✅ 提交消息遵循约定式提交

### 5.3 常见错误和修正

#### ❌ 错误 1: 文件放置不当
```
错误：在根目录创建 PHASE_X_XXX.md
正确：放在 docs/PHASE_X_XXX.md
```

#### ❌ 错误 2: 创建不必要的目录
```
错误：创建 crates/new-feature/
正确：在现有 crate 中添加模块
```

#### ❌ 错误 3: 缺少文档
```
错误：pub fn new_function() { }
正确：
/// 函数说明
///
/// # 示例
/// ```
/// let result = new_function();
/// ```
pub fn new_function() { }
```

#### ❌ 错误 4: 缺少测试
```
错误：实现新功能但没有测试
正确：包含 #[test] 或 tests/ 文件
```

#### ❌ 错误 5: 代码质量问题
```
错误：使用 unwrap()，硬编码值，忽略错误
正确：使用 Result/Option，常量，适当的错误处理
```

---

## 🔄 第六部分：提交和审查流程

### 6.1 提交流程

```
1. 修改代码
   ↓
2. 运行 ./scripts/pre_commit_test.sh
   ↓
3a. 所有通过 → 继续
3b. 有失败 → 修复并重新运行
   ↓
4. git add .
   ↓
5. git commit -m "..."
   ↓
6. git-hook 自动运行检查
   ↓
6a. 全部通过 → 提交成功
6b. 有失败 → 提交被拒绝，返回第2步
   ↓
7. git push
   ↓
8. GitHub Actions 再次验证
```

### 6.2 代码审查标准

任何代码必须满足以下标准：

#### 功能性 (Functionality)
- [ ] 代码实现了预期的功能
- [ ] 代码不引入新的 bug
- [ ] 代码与现有代码兼容

#### 代码质量 (Code Quality)
- [ ] 代码风格一致
- [ ] 命名清晰明确
- [ ] 注释适当且有帮助
- [ ] 没有重复代码
- [ ] 函数/模块职责单一

#### 文档 (Documentation)
- [ ] 公开 API 有文档注释
- [ ] 复杂逻辑有说明注释
- [ ] 有使用示例
- [ ] 文档准确无误

#### 测试 (Testing)
- [ ] 有单元测试
- [ ] 有集成测试（如适用）
- [ ] 测试覆盖关键路径
- [ ] 所有测试通过

#### 安全性 (Security)
- [ ] 没有 unsafe 代码（或有充分理由）
- [ ] 没有 unwrap/panic（或有充分理由）
- [ ] 适当的错误处理
- [ ] 没有安全漏洞

#### 性能 (Performance)
- [ ] 没有明显的性能回退
- [ ] 大型操作有优化
- [ ] 内存使用合理

---

## 📊 第七部分：版本和依赖管理

### 7.1 Rust 版本

```toml
# rust-toolchain.toml
[toolchain]
channel = "stable"
```

**规则：**
- ✅ 使用稳定版本的 Rust
- ❌ 禁止使用 nightly（除了必要的特性）
- ✅ 定期更新到最新稳定版

### 7.2 依赖管理

#### 7.2.1 添加依赖

```bash
# 在对应的 crate 目录
cargo add package_name

# 添加具体版本
cargo add package_name@1.2.3
```

#### 7.2.2 依赖审批

添加任何外部依赖前，必须考虑：

- [ ] 是否有维护的替代方案？
- [ ] 依赖有多少传递依赖？
- [ ] 依赖有已知的安全问题？
- [ ] 依赖的许可证与项目兼容？
- [ ] 依赖是否增加过多编译时间？

#### 7.2.3 安全审计

```bash
# 检查安全漏洞
cargo audit

# 检查依赖许可证和来源
cargo deny check
```

**规则：**
- ✅ 必须通过 `cargo deny` 检查
- ❌ 禁止使用有已知漏洞的版本
- ✅ 定期更新依赖

---

## 🎯 第八部分：执行和监督

### 8.1 自动化执行

#### Git Hook（本地）
- 位置：`.git/hooks/pre-commit`
- 行为：在提交前自动运行 `scripts/pre_commit_test.sh`
- 结果：如果失败则阻止提交

#### GitHub Actions（远程）
- 位置：`.github/workflows/ci.yml`
- 触发：Push 或 PR
- 检查：
  - 代码格式 (fmt)
  - Lint 检查 (clippy)
  - 编译检查 (check)
  - 单元测试 (test)
  - 集成测试 (test)
  - 安全审计 (deny)
  - 发布构建 (build)

### 8.2 违规后果

#### 提交违规
| 违规行为 | 处理方式 |
|---------|---------|
| 文件放置不当 | 要求重构并重新提交 |
| 缺少文档 | 要求添加文档 |
| 缺少测试 | 要求添加测试 |
| 代码质量问题 | 要求修复警告/错误 |
| 未通过检查 | 提交被拒绝 |

#### 持续违规
- 第1次：警告 + 要求修正
- 第2次：暂停 commit 权限 + 强制 code review
- 第3次：暂停 commit 权限，需要管理员审批

---

## 📚 附录 A：快速参考

### 常见任务

#### 添加新功能
```bash
# 1. 在适当的 crate 中添加代码
# 2. 添加文档注释
# 3. 添加测试
# 4. 运行检查
./scripts/pre_commit_test.sh
# 5. 提交
git add .
git commit -m "feat: add new feature"
```

#### 修复 Bug
```bash
# 1. 定位问题代码
# 2. 修复 bug
# 3. 添加回归测试
# 4. 运行检查
./scripts/pre_commit_test.sh
# 5. 提交
git add .
git commit -m "fix: fix bug description"
```

#### 添加文档
```bash
# 1. 在 docs/ 中创建文件
# 2. 遵循文档模板
# 3. 提供中英文版本
# 4. 在索引中链接
# 5. 提交
git add docs/
git commit -m "docs: add documentation"
```

---

## 📞 支持和反馈

如遇问题：

1. **检查本文档** - 答案可能已在此
2. **查看现有代码** - 参考类似的实现
3. **运行本地测试** - 使用 `./scripts/pre_commit_test.sh`
4. **查看 CI/CD 日志** - GitHub Actions 提供详细错误

---

**最后更新：** 2026年2月14日  
**适用范围：** fingerprint-rust 项目所有分支、所有代码和文档  
**强制执行：** Git Hook + GitHub Actions + Code Review
