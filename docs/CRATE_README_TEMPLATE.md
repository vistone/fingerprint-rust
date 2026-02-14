# Crate README 模板 (Crate README Template)

> 本文档提供了编写 crate README.md 的标准模板。每个 crate 必须包含一个符合此模板的 README.md。

---

## 使用方法

1. 复制下面的模板内容
2. 替换 `[模块名]` 为实际的 crate 名称
3. 根据 crate 的功能填写各个部分
4. 确保完整性和准确性
5. 可选：创建 `README.en.md` 英文版本

---

## 📝 标准模板

```markdown
# [模块名] (Module Name)

[简洁的一句话描述模块的主要功能]

## 功能特性 (Features)

- ✅ 功能特性 1：描述
- ✅ 功能特性 2：描述
- ✅ 功能特性 3：描述
- 🔧 可选特性 (feature name)：描述（如有）

## 快速开始 (Quick Start)

### 添加到 Cargo.toml

```toml
[dependencies]
fingerprint-[crate-name] = "0.1"
```

### 基本用法示例

```rust
use fingerprint_[crate_name]::{Type, function};

// 简单使用示例
let result = function(input)?;
println!("Result: {:?}", result);
```

## API 概览 (API Overview)

### 主要类型

| 类型 | 说明 |
|-----|------|
| `TypeName` | 主要数据类型说明 |
| `TraitName` | 主要特征说明 |
| `EnumName` | 枚举类型说明 |

### 主要函数

| 函数 | 说明 |
|-----|------|
| `pub fn function_name()` | 函数说明 |
| `pub async fn async_function()` | 异步函数说明 |

### 更多示例

```rust
// 示例 1：基础用法
let instance = Type::new();
let result = instance.method()?;

// 示例 2：高级用法
let config = Config::default()
    .with_option(value);
let instance = Type::with_config(config);
```

## 依赖关系 (Dependencies)

| 依赖 | 用途 | 版本 |
|-----|------|------|
| `serde` | 序列化/反序列化 | ^1.0 |
| `tokio` | 异步运行时 | ^1.0 |
| `log` | 日志记录 | ^0.4 |

## 可选特性 (Optional Features)

```toml
[features]
default = ["feature1"]
feature1 = ["dep1"]
feature2 = ["dep2"]
```

启用特性：
```toml
fingerprint-[crate-name] = { version = "0.1", features = ["feature1", "feature2"] }
```

## 架构说明 (Architecture)

### 模块结构

```
src/
├── lib.rs              # 模块入口
├── module1.rs          # 功能模块 1
├── module2.rs          # 功能模块 2
└── tests/              # 测试
```

### 主要组件

- **Component 1**: 说明
- **Component 2**: 说明
- **Component 3**: 说明

更详细的架构说明见：[docs/ARCHITECTURE.md](../../docs/ARCHITECTURE.md)

## 性能 (Performance)

- 处理速度：X ms/请求
- 内存使用：约 X MB
- 并发能力：支持 Y 个并发连接

## 局限性 (Limitations)

- 限制 1：说明
- 限制 2：说明

## 贡献 (Contributing)

欢迎提交 Issue 和 Pull Request！

详见：[CONTRIBUTING.md](../../CONTRIBUTING.md)

## 许可证 (License)

本项目采用 MIT 许可证。详见：[LICENSE](../../LICENSE)

## 相关文档 (Related Documentation)

- [完整 API 文档](https://docs.rs/fingerprint-[crate-name])
- [设计文档](../../docs/[CRATE]_DESIGN.md)（如有）
- [项目治理规范](../../PROJECT_GOVERNANCE.md)

---

**最后更新：** 2026年2月14日
```

---

## 撰写指南 (Writing Guidelines)

### 1. 功能特性部分

- 列出 3-5 个主要功能
- 使用 ✅ 或 🔧 标记
- 简洁明了，每项一行

### 2. 快速开始部分

- 包含清晰的导入语句
- 提供最简单的使用示例
- 确保代码能够编译

### 3. API 概览部分

- 只列出最重要的类型和函数
- 详细细节见文档注释
- 提供 2-3 个实际使用示例

### 4. 依赖关系部分

- 列出主要的外部依赖
- 标注版本要求
- 说明用途

### 5. 架构说明部分

- 简要描述模块结构
- 解释主要组件的职责
- 链接到详细的架构文档

---

## 检查清单 (Checklist)

编写完 README 后，确保：

- [ ] 文件名正确：`README.md`（中文）或 `README.en.md`（英文）
- [ ] 文件在正确位置：`crates/[crate-name]/README.md`
- [ ] 包含模块简介
- [ ] 包含至少 3 个功能特性
- [ ] 包含快速开始示例
- [ ] 包含示例代码且能编译
- [ ] 包含 API 概览表
- [ ] 包含依赖说明
- [ ] 包含架构简述
- [ ] 所有链接都有效（相对路径）
- [ ] 没有拼写错误
- [ ] 格式清晰易读

---

## 常见错误 (Common Mistakes)

❌ **错误示例 1：过于简洁**
```markdown
# MyModule

这是一个模块。它很有用。
```

✅ **正确示例 1：详细而清晰**
```markdown
# MyModule

提供 XXX 功能，用于 YYY 目的。

## 功能特性
- 特性 1 的详细说明
- 特性 2 的详细说明
- ...
```

❌ **错误示例 2：链接断裂**
```markdown
[详见](ARCHITECTURE.md)        # 相对于文件系统
[详见](/docs/ARCHITECTURE.md)  # 绝对路径（GitHub 上不工作）
```

✅ **正确示例 2：有效的相对链接**
```markdown
[详见](../../docs/ARCHITECTURE.md)  # 正确的相对路径
```

❌ **错误示例 3：示例代码无法编译**
```rust
// 这段代码缺少导入和设置
let result = function()?;  // ❌ function 从哪里来？
```

✅ **正确示例 3：完整的可运行示例**
```rust
use fingerprint_module::{Type, function};

let result = function()?;  // ✅ 清晰明确
```

---

## 实际例子 (Real Example)

### BadExample: 不符合规范

```
# Core Module

核心模块。
```

### GoodExample: 符合规范

```
# Core Module (fingerprint-core)

提供 fingerprint-rust 项目的核心功能，包括基础数据结构、算法实现和通用工具。

## 功能特性

- ✅ 高性能的指纹计算引擎
- ✅ 支持多种浏览器和设备类型
- ✅ 可扩展的模块化架构
- 🔧 可选的缓存支持（`redis` 特性）
- 🔧 可选的数据库支持（`database` 特性）

## 快速开始

### 添加到 Cargo.toml

```toml
[dependencies]
fingerprint-core = "0.1"
```

### 基本用法

```rust
use fingerprint_core::{FingerprintEngine, BrowserInfo};

let engine = FingerprintEngine::new();
let info = BrowserInfo::new("Chrome", "120", "Windows");
let fingerprint = engine.calculate(&info)?;

println!("Fingerprint: {}", fingerprint.id());
```

## API 概览

...（继续）
```

---

## 频繁更新

- 当 crate 添加新功能时，更新此 README
- 当 API 改变时，更新示例代码
- 定期检查链接有效性
- 保持示例代码能够编译

---

**创建日期：** 2026年2月14日  
**强制执行：** 所有 crate 必须有 README.md  
**验证方式：** 代码审查 + 自动检查
