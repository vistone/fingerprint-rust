# AI 代码生成规则 (AI Code Generation Rules)

> **必读：** 这是使用 AI 辅助开发时的强制性规则。违反任何规则都会导致代码被拒绝。

---

## 🚫 绝对禁止

### 1. 文件放置

**❌ 禁止以下行为：**

| 禁止行为 | 原因 | 正确做法 |
|---------|------|--------|
| 在根目录创建新目录 | 破坏项目结构 | 在 `crates/` 中添加 |
| 在根目录创建 `*.md` 文件 | 文档混乱 | 放在 `docs/` 目录 |
| 将代码放在 `src/` 中 | `src/` 不存在 | 使用 `crates/*/src/` |
| 创建 `.backup`, `.disabled` 等文件 | 垃圾文件 | 使用 git 的历史而不是备份 |
| 将临时输出放在项目根目录 | 污染仓库 | 使用 `output/` 或 `phase7_results/` |
| 创建新 crate 但没有 README.md | 缺少文档说明 | 每个 crate 必须有 `README.md` |

### 2. 文档

**❌ 禁止以下行为：**

```
❌ 在根目录创建 PHASE_X_XXX.md
❌ 在根目录创建 [FEATURE]_DESIGN.md
❌ 创建无结构的 .md 文件
❌ 创建没有标题和目录的文档
❌ 只提供英文或中文，忽略另一种语言
❌ 使用 lowercase 或 camelCase 文件名
```

### 3. 代码

**❌ 禁止以下行为：**

```rust
// ❌ 禁止：没有文档的公开 API
pub fn new_fingerprint() { }

// ❌ 禁止：没有测试的新功能
pub fn process_data(input: Vec<u8>) -> Vec<u8> {
    // implementation
}

// ❌ 禁止：使用 unwrap/expect/panic
let value = optional.unwrap();

// ❌ 禁止：忽略错误
let result = operation().ok();

// ❌ 禁止：硬编码值和魔法数字
let timeout_ms = 5000;
for i in 0..10 { }

// ❌ 禁止：不遵循命名规范
fn NewFunction() { }
const max_retries = 3;
```

### 4. 提交

**❌ 禁止以下行为：**

```bash
❌ 提交未通过 cargo fmt 的代码
❌ 提交有 clippy 警告的代码
❌ 提交失败的测试
❌ 提交没有相关测试的功能
❌ 提交带有拼写错误的文档
❌ 提交没有文档的公开 API
```

### 5. 报告和文档生成

**❌ 禁止以下行为：**

| 禁止行为 | 原因 | 正确做法 |
|---------|------|--------|
| 乱生成报告（没有实际需求） | 污染仓库、增加无用文件 | 仅在明确需要时才生成报告 |
| 在根目录生成报告 | 破坏项目结构 | 所有报告放在 `docs/reports/` 目录 |
| 报告没有分类管理 | 难以维护 | 按报告类型使用子目录分类 |
| 生成无日期的报告 | 难以追踪版本 | 报告名称包含日期或版本号 |
| 生成不符合格式的报告 | 不专业 | 参考 [报告格式标准](#报告格式标准) |

```
❌ 禁止在根目录创建报告（如 FINAL_REPORT.md）
❌ 禁止生成未分类的报告
❌ 禁止生成"测试报告""临时报告"等无实际需求的报告
❌ 禁止混合不同类型的报告在同一文件中
❌ 禁止报告没有清晰的标题和日期
```

---

## ✅ 必须做到

### 1. 代码结构

**对于新功能，必须：**

```rust
// ✅ 必须：清晰的文档
/// 功能说明
///
/// # 参数
/// - `input`: 输入说明
///
/// # 返回值
/// 返回值说明
///
/// # 错误
/// - `Error::InvalidInput`: 当输入无效时
///
/// # 示例
/// ```
/// let result = my_function(input)?;
/// assert!(result.is_valid());
/// ```
pub fn my_function(input: String) -> Result<Data, Error> {
    // implementation with proper error handling
}

// ✅ 必须：配套的测试
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_my_function_with_valid_input() {
        let result = my_function("valid".to_string());
        assert!(result.is_ok());
    }

    #[test]
    fn test_my_function_with_invalid_input() {
        let result = my_function("".to_string());
        assert!(result.is_err());
    }
}
```

### 2. 文件结构

**所有文件必须：**

```
✅ 文件名使用 UPPERCASE_WITH_UNDERSCORES
✅ 放在正确的目录层级
✅ 遵守项目的模块组织
✅ 更新相关的 mod.rs 导入
✅ 在必要时更新 Cargo.toml
✅ 更新项目文档或 README

对于新 crate：
✅ 必须有 Cargo.toml 和 src/lib.rs
✅ 必须有 README.md（中文）描述模块功能
✅ 建议有 README.en.md（英文）版本
✅ README 应该包含：
   - 模块简介和功能说明
   - 主要 API 概览
   - 使用示例
   - 依赖和特性说明
```

### 3. 文档结构

**所有新增文档必须：**

```markdown
✅ 放在 docs/ 目录
✅ 使用正确的文件名格式
✅ 包含清晰的标题和目录
✅ 提供中英文版本（或标注只有一种语言的原因）
✅ 包含相关的代码示例
✅ 包含指向代码的正确链接
✅ 按照指定的模板（如有）
```

### 4. 代码质量

**所有代码必须：**

```bash
✅ 通过 cargo fmt --all -- --check
✅ 通过 cargo clippy --workspace --all-targets --all-features -- -D warnings
✅ 通过 cargo check --workspace --all-features
✅ 通过 cargo test --workspace
✅ 通过 cargo deny check advisories bans licenses sources
✅ 通过 cargo build --workspace --release
```

### 5. 提交流程

**任何提交前必须：**

```bash
✅ 运行 ./scripts/pre_commit_test.sh
✅ 所有 7 项检查都是 ✅
✅ 代码已 git add
✅ 提交消息遵循约定式提交
✅ 本地提交成功（git 未拒绝）
✅ 然后才能 git push
```

### 6. 报告和文档管理

**所有报告必须：**

```
✅ 仅在有明确需求时才生成
✅ 放在 docs/reports/ 目录中
✅ 按类型使用子目录分类（如 docs/reports/performance/、docs/reports/analysis/ 等）
✅ 文件名包含日期或版本号标识
✅ 使用 UPPERCASE_WITH_UNDERSCORES 命名规范
✅ 包含清晰的标题、日期和作者信息
✅ 不进行重复报告生成（检查历史是否已有类似报告）

报告分类目录示例：
docs/reports/
├── performance/           # 性能相关报告
├── security/              # 安全分析报告
├── analysis/              # 代码分析报告
├── architecture/          # 架构设计报告
├── completion/            # 完成度报告
└── evaluation/            # 评估报告
```

---

## 📋 提交前检查清单

**使用此清单确保代码符合规范：**

### 代码检查
- [ ] 代码在正确的文件中（`crates/*/src/`）
- [ ] 所有文件名遵循 snake_case
- [ ] 所有函数/变量名遵循命名规范
- [ ] 没有注释了的代码
- [ ] 没有 `println!`, `dbg!` 等调试代码
- [ ] 没有 `unwrap()`, `expect()`, 不合理的 `panic!`
- [ ] 正确的错误处理（Result/Option）
- [ ] 所有公开 API 有文档注释
- [ ] 文档包含 # 示例 部分

### 测试检查
- [ ] 添加了单元测试（`#[test]`）
- [ ] 添加了集成测试（`tests/`）
- [ ] 测试覆盖正常情况
- [ ] 测试覆盖错误情况
- [ ] 所有测试命名清晰（test_x_with_y_returns_z）
- [ ] 测试包含 // Arrange, // Act, // Assert 注释

### 文档检查（如有新增文档）
- [ ] 文件放在 `docs/` 目录
- [ ] 文件名使用 UPPERCASE_WITH_UNDERSCORES
- [ ] 包含一级标题
- [ ] 包含二级标题和目录
- [ ] 所有代码块有语言标记（```rust）
- [ ] 所有链接都是相对链接
- [ ] 提供中英文版本
- [ ] 没有拼写错误

### 质量检查
- [ ] `cargo fmt --all` 已运行
- [ ] `cargo clippy --workspace --all-targets --all-features` 通过
- [ ] `cargo check --workspace --all-features` 通过
- [ ] `cargo test --workspace` 全部通过
- [ ] `cargo deny check` 通过
- [ ] `cargo build --release` 通过
- [ ] `./scripts/pre_commit_test.sh` 显示 ✅ 所有检查通过

### 提交检查
- [ ] 提交消息遵循约定式提交：`type: subject`
- [ ] 提交消息清晰描述改动
- [ ] 没有包含无关的文件（`target/`, `output/` 等）

### 报告检查（如有新增报告）
- [ ] 报告确实有明确的需求（不是乱生成）
- [ ] 报告文件放在 `docs/reports/<category>/` 子目录中
- [ ] 报告文件名使用 UPPERCASE_WITH_UNDERSCORES 规范
- [ ] 报告文件名包含日期或版本号（如 `REPORT_20260214.md`）
- [ ] 报告包含标题、日期和作者信息
- [ ] 报告不是重复的（检查历史是否已有同类型报告）
- [ ] 报告在正确的分类目录中（不在 `docs/` 根目录）

---

## 📝 常见模式和示例

### 模式 1: 添加新的工具函数

```rust
// ✅ 正确示例
/// 计算输入的哈希值
///
/// # 参数
/// - `input`: 要散列的输入字符串
///
/// # 返回值
/// 输入的 SHA-256 哈希值（十六进制字符串）
///
/// # 错误
/// 不会返回错误；始终根据输入生成有效的哈希
///
/// # 示例
/// ```
/// use crate::utils::calculate_hash;
/// 
/// let hash = calculate_hash("test");
/// assert_eq!(hash.len(), 64); // SHA-256 是 64 个十六进制字符
/// ```
pub fn calculate_hash(input: &str) -> String {
    // implementation
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_hash_produces_64_char_string() {
        let hash = calculate_hash("test");
        assert_eq!(hash.len(), 64);
    }

    #[test]
    fn test_calculate_hash_is_deterministic() {
        let hash1 = calculate_hash("test");
        let hash2 = calculate_hash("test");
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_calculate_hash_different_for_different_inputs() {
        let hash1 = calculate_hash("test1");
        let hash2 = calculate_hash("test2");
        assert_ne!(hash1, hash2);
    }
}
```

### 模式 2: 添加新的错误类型

```rust
// ✅ 正确示例
/// 指纹处理可能的错误
#[derive(Debug)]
pub enum FingerprintError {
    /// 输入无效或格式不正确
    InvalidInput(String),
    /// 处理过程中出错
    ProcessingError(String),
    /// 依赖服务不可用
    ServiceUnavailable(String),
}

impl std::fmt::Display for FingerprintError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
            Self::ProcessingError(msg) => write!(f, "Processing error: {}", msg),
            Self::ServiceUnavailable(msg) => write!(f, "Service unavailable: {}", msg),
        }
    }
}

impl std::error::Error for FingerprintError {}

// 使用示例
pub fn process(input: String) -> Result<Data, FingerprintError> {
    if input.is_empty() {
        return Err(FingerprintError::InvalidInput(
            "Input cannot be empty".to_string()
        ));
    }
    // implementation
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_rejects_empty_input() {
        let result = process(String::new());
        assert!(matches!(result, Err(FingerprintError::InvalidInput(_))));
    }
}
```

### 模式 3: 添加新模块

```rust
// ✅ 在 crates/fingerprint-core/src/lib.rs 中
pub mod new_module;

// ✅ 创建 crates/fingerprint-core/src/new_module.rs
//! 新模块说明
//!
//! 这个模块提供...

/// 模块中的主要函数
///
/// # 说明
/// ...
pub fn main_function() -> Result<String, Error> {
    // implementation
}

#[cfg(test)]
mod tests {
    // tests
}

// ✅ 更新 Cargo.toml（如果需要新的依赖）
```

---

## 🚨 违规示例和修正

### 违规 1: 文件放置错误

```
❌ 错误：
  crates/fingerprint/src/new_module.rs
  ├── BrowserProfile ID
  ├── ...
  └── ADDED_FEATURE.md  // ❌ 代码和文档混合！

✅ 正确：
  crates/fingerprint-profiles/src/
  └── profiles.rs  // 代码在这里
  
  docs/
  └── FEATURE_NAME_DESIGN.md  // 文档在这里
```

### 违规 2: 缺少文档

```rust
❌ 错误：
pub fn new_fingerprint(data: Vec<u8>) -> Vec<u8> {
    // implementation
}

✅ 正确：
/// 从原始数据生成浏览器指纹
///
/// # 参数
/// - `data`: 浏览器信息的原始字节
///
/// # 返回值
/// 生成的指纹字节序列
///
/// # 错误
/// - `Error::InvalidData`: 当输入数据无效时
///
/// # 示例
/// ```
/// let fingerprint = new_fingerprint(vec![1, 2, 3])?;
/// assert!(!fingerprint.is_empty());
/// ```
pub fn new_fingerprint(data: Vec<u8>) -> Result<Vec<u8>, Error> {
    if data.is_empty() {
        return Err(Error::InvalidData("Data cannot be empty".to_string()));
    }
    // implementation
}
```

### 违规 3: 缺少测试

```rust
❌ 错误：
fn process_fingerprint(input: String) -> String {
    // 实现但没有测试
}

✅ 正确：
fn process_fingerprint(input: String) -> String {
    // 实现
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_fingerprint_with_valid_input() {
        let result = process_fingerprint("valid input".to_string());
        assert!(!result.is_empty());
    }

    #[test]
    fn test_process_fingerprint_with_empty_input() {
        let result = process_fingerprint(String::new());
        // 测试边界情况
    }
}
```

### 违规 4: 代码质量问题

```rust
❌ 错误：
pub fn get_data(id: usize) -> Option<Data> {
    DATA[id].clone()  // 可能 panic
}

pub fn process(input: String) -> String {
    let value = CONFIG.get("timeout").unwrap();  // 不安全
    // 使用 value
}

✅ 正确：
pub fn get_data(id: usize) -> Option<Data> {
    DATA.get(id).cloned()  // 安全的边界检查
}

pub fn process(input: String) -> Result<String, Error> {
    let value = CONFIG
        .get("timeout")
        .ok_or_else(|| Error::MissingConfig("timeout".to_string()))?;
    // 使用 value
}
```

### 违规 5: 乱生成报告

```
❌ 错误：
  项目根目录
  ├── FINAL_REPORT.md              // ❌ 报告在根目录！
  ├── PERFORMANCE_ANALYSIS.md      // ❌ 报告在根目录！
  ├── TEMPORARY_FINDINGS.md        // ❌ 没有日期标识！
  ├── RANDOM_TEST_REPORT.md        // ❌ 乱生成的报告！
  └── ...

✅ 正确：
  项目根目录
  └── docs/
      └── reports/
          ├── performance/
          │   └── PERFORMANCE_ANALYSIS_20260214.md    // ✓ 分类+日期
          ├── analysis/
          │   └── CODE_QUALITY_REPORT_20260214.md     // ✓ 有明确需求
          ├── completion/
          │   └── PHASE_7_COMPLETION_20260214.md      // ✓ 有实际信息
          ├── architecture/
          │   └── ARCHITECTURE_REVIEW_20260214.md     // ✓ 分类+日期
          └── security/
              └── SECURITY_AUDIT_20260214.md          // ✓ 分类+日期

原则：
✓ 仅在有明确需求时才生成
✓ 严格放入 docs/reports/<category>/ 子目录
✓ 文件名包含日期或版本号
✓ 按报告类型分类管理
✓ 不重复生成同类型报告
```

---

## 📞 问题排查

### "提交被 git hook 拒绝"

```bash
# 1. 检查 pre_commit_test.sh 的输出
./scripts/pre_commit_test.sh

# 2. 根据失败的检查修复
cargo fmt --all              # 如果是格式错误
cargo clippy --fix           # 如果是 clippy 警告
cargo test                   # 如果是测试失败

# 3. 重新运行 pre_commit_test.sh 验证
./scripts/pre_commit_test.sh

# 4. 如果还有问题，检查详细输出
RUST_BACKTRACE=1 cargo test --lib 2>&1 | head -100
```

### "代码在本地工作，但 GitHub Actions 失败"

这通常是因为：
- [ ] 未在 Linux 上测试（GitHub 默认 Ubuntu）
- [ ] 存在行尾符号问题（CRLF vs LF）
- [ ] 时区或系统相关的测试
- [ ] 依赖版本不同

**解决方案：**
```bash
# 确保使用 LF（Linux 风格）
git config core.autocrlf false

# 在 Linux VM 或 Docker 中测试
# 或使用 GitHub Codespaces 进行测试
```

---

## ✨ 最佳实践

### 1. 提交前的完整流程

```bash
# 1. 检查本地更改
git status

# 2. 查看具体改动
git diff

# 3. 添加文件
git add .

# 4. 运行完整的检查
./scripts/pre_commit_test.sh

# 5. 如果全部通过，提交
git commit -m "type: subject"

# 6. 推送到 GitHub
git push

# 7. 检查 GitHub Actions 结果
# 去 GitHub 查看 Actions 选项卡
```

### 2. 强制性规范回顾

在开始任何工作前：

- [ ] 阅读 [PROJECT_GOVERNANCE.md](PROJECT_GOVERNANCE.md)
- [ ] 阅读 [COMMIT_POLICY.md](COMMIT_POLICY.md)
- [ ] 阅读本文件（AI_CODE_GENERATION_RULES.md）
- [ ] 查看 [CONTRIBUTING.md](docs/CONTRIBUTING.md)

### 3. 报告生成指南

**生成报告前，必须回答以下问题：**

1. **这个报告有明确的需求吗？**
   - ✅ 需要：有人明确要求这个报告
   - ❌ 不需要：只是"顺便生成"一个报告

2. **这类型的报告已经存在吗？**
   - ✅ 检查 `docs/reports/` 目录中的现有报告
   - ❌ 不要生成重复的报告

3. **报告能放在正确的位置吗？**
   - ✅ 所有报告必须在 `docs/reports/<category>/` 目录
   - ❌ 不能在根目录或其他地方

4. **报告的命名和结构是否规范？**
   ```
   ✅ 正确：docs/reports/performance/PERFORMANCE_REPORT_20260214.md
   ✅ 正确：docs/reports/analysis/CODE_QUALITY_20260214.md
   ✅ 正确：docs/reports/completion/PHASE_7_STATUS_20260214.md
   ❌ 错误：docs/FINAL_REPORT.md
   ❌ 错误：docs/TEMP_REPORT.md
   ```

**报告模板头部（必须）：**

```markdown
# 报告标题

> **报告类型：** [performance/analysis/completion/security/architecture/evaluation]  
> **生成日期：** 2026-02-14  
> **版本：** 1.0  
> **作者：** [作者名称或 AI 系统名称]

## 报告摘要

[简明扼要的 2-3 句摘要，说明报告目的和主要发现]

---

## 目录

[自动生成或手动列出主要章节]

---

## 正文

[报告内容]

---

**最后更新：** [日期]
```

### 4. 有疑问时

- ✅ 查看现有代码作为参考
- ✅ 遵循项目中的现有模式
- ✅ 检查类似功能的实现方式
- ✅ 查看 `docs/reports/` 中的现有报告格式
- ❌ 不要"自己决定"文件位置或命名
- ❌ 不要乱生成不需要的报告

---

**最后更新：** 2026年2月14日  
**强制执行：** 所有 AI 辅助开发必须遵守  
**违规后果：** 代码被拒绝，要求修改并重新提交
