# 项目规范体系导航 (Project Governance System Map)

> **本文档是整个项目规范体系的导航中心。所有成员和 AI 开发者必须理解并遵守这些规范。**

---

## 🎯 5 秒速记

```
✅ 严格遵守项目结构（不能乱放文件）
✅ 必须编写文档和测试（不能偷懒）
✅ 必须通过 7 项本地检查（没有例外）
✅ 不允许使用 --no-verify（规则是强制的）
```

---

## 📚 规范文档体系（Complete Governance Ecosystem）

### Tier 1: 核心政策文件（Core Policy）

这些文件定义了项目的基本规则：

#### 📄 [COMMIT_POLICY.md](COMMIT_POLICY.md) - 提交政策
- **对谁：** 所有提交者
- **用途：** 定义提交前必须满足的条件
- **关键内容：**
  - 7 项强制检查（fmt, clippy, check, test-lib, test, deny, build）
  - 快速修复指南
  - 规范化的提交消息格式
  - 被拒绝提交的原因说明

**何时查看：** 
- 准备提交代码前
- 提交被拒绝时
- 需要理解 CI/CD 流程时

---

#### 📄 [PROJECT_GOVERNANCE.md](PROJECT_GOVERNANCE.md) - 完整项目治理规范
- **对谁：** 所有开发者和 AI 辅助开发
- **用途：** 定义项目的完整结构和标准
- **关键内容：**
  - 第一部分：项目结构规范（crates, docs, scripts 等目录）
  - 第二部分：文件放置规范（代码、文档、配置、数据）
  - 第三部分：文档规范（模板、命名、写作规范）
  - 第四部分：代码风格指南（命名、注释、测试、错误处理）
  - 第五部分：AI 代码生成规则（禁止项、必做项）
  - 第六部分：提交流程（提交、审查、违规处理）
  - 第七部分：版本和依赖管理
  - 第八部分：执行和监督机制

**何时查看：** 
- 创建新文件或目录时
- 不确定文件应该放在哪里时
- 需要了解完整的编码标准时
- 需要理解文档模板时

---

### Tier 2: 快速参考和指南（Quick Reference & Guides）

这些文件提供快速查询和实战指导：

#### 📄 [QUICK_REFERENCE.md](QUICK_REFERENCE.md) - 快速参考
- **对谁：** 需要快速查询的开发者
- **用途：** 快速找到答案，不需要阅读完整文档
- **关键内容：**
  - 5 秒钟总结
  - 文件放置速查表
  - 文档命名规范
  - 7 项检查清单
  - 代码模板
  - 常见错误排查

**何时查看：** 
- 需要快速答案时
- 不确定文件放在哪里时（查看表格）
- 需要修复常见错误时

---

#### 📄 [docs/AI_CODE_GENERATION_RULES.md](docs/AI_CODE_GENERATION_RULES.md) - AI 代码生成规则
- **对谁：** AI 辅助开发和代码审查者
- **用途：** 定义 AI 生成代码必须遵守的规则
- **关键内容：**
  - 7 项绝对禁止
  - 必须做到的内容
  - 有效性检查清单
  - 常见违规例子和修正
  - 问题排查指南

**何时查看：** 
- 使用 AI 生成代码前
- 要求 AI 做某事但不确定规则时
- 代码审查时检查是否符合 AI 规则

---

#### 📄 [docs/AI_DEVELOPER_GUIDE.md](docs/AI_DEVELOPER_GUIDE.md) - AI 开发者综合指南
- **对谁：** 专为 AI 辅助开发而设计
- **用途：** 提供完整的工作流程和指导
- **关键内容：**
  - 首次阅读路线
  - 7 阶段工作流程（分析 → 规范检查 → 规划 → 编码 → 文档 → 验证 → 提交）
  - 代码类型和放置规范
  - 严禁清单和必做清单
  - 提交前检查清单
  - 常见场景的完整流程
  - 问题排查和最佳实践

**何时查看：** 
- AI 开始任何工作前（必读）
- 不清楚工作流程时
- 需要了解具体的代码模式时

---

### Tier 3: 执行检查和验证（Enforcement & Validation）

这些文件定义了如何验证和执行规范：

#### 📄 [GOVERNANCE_ENFORCEMENT_CHECKLIST.md](GOVERNANCE_ENFORCEMENT_CHECKLIST.md) - 执行检查清单
- **对谁：** 代码审查者和自动化工具
- **用途：** 提供客观的验证标准
- **关键内容：**
  - Level 1: 文件结构检查
  - Level 2: 代码质量检查
  - Level 3: 测试检查
  - Level 4: 编译检查
  - Level 5: 文档检查
  - Level 6: 提交检查
  - Level 7: GitHub Actions 检查
  - 快速检查清单
  - 审查者指南
  - 批准/拒绝的标准

**何时查看：** 
- 进行代码审查时
- 需要了解一个提交是否符合规范时
- 做最终的质量验证时

---

## 🔄 使用场景 (Usage Scenarios)

### 场景 1: "我要写一个新函数"

```
1. 查看：[QUICK_REFERENCE.md#代码示例模板](QUICK_REFERENCE.md)
   ↓
2. 遵循：[PROJECT_GOVERNANCE.md#代码风格指南](PROJECT_GOVERNANCE.md)
   ↓
3. 包含：文档注释 + 单元测试
   ↓
4. 验证：./scripts/pre_commit_test.sh
   ↓
5. 提交：遵循 [COMMIT_POLICY.md](COMMIT_POLICY.md)
```

### 场景 2: "我需要写项目文档"

```
1. 查看：[QUICK_REFERENCE.md#文件放置](QUICK_REFERENCE.md)
   ↓
2. 确认：文件应在 docs/ 目录
   ↓
3. 查看：[PROJECT_GOVERNANCE.md#文档模板](PROJECT_GOVERNANCE.md)
   ↓
4. 选择：设计文档/完成报告/执行计划模板
   ↓
5. 提供：中文（.md）和英文（.en.md）版本
   ↓
6. 提交：通过 [COMMIT_POLICY.md](COMMIT_POLICY.md)
```

### 场景 3: "我是代码审查者，需要检查提交"

```
1. 获取：完整的修改列表
   ↓
2. 检查：[GOVERNANCE_ENFORCEMENT_CHECKLIST.md#Level1](GOVERNANCE_ENFORCEMENT_CHECKLIST.md)
   ↓
3. 检查：[GOVERNANCE_ENFORCEMENT_CHECKLIST.md#Level2-5](GOVERNANCE_ENFORCEMENT_CHECKLIST.md)
   ↓
4. 判断：按照 [GOVERNANCE_ENFORCEMENT_CHECKLIST.md#审查者指南](GOVERNANCE_ENFORCEMENT_CHECKLIST.md)
   ↓
5. 结论：✅ 批准 或 ❌ 拒绝（需修正）
```

### 场景 4: "AI 要生成代码"

```
1. 读取：[docs/AI_DEVELOPER_GUIDE.md](docs/AI_DEVELOPER_GUIDE.md) (必读)
   ↓
2. 进行：7 阶段工作流程
   ↓
3. 参考：[docs/AI_CODE_GENERATION_RULES.md](docs/AI_CODE_GENERATION_RULES.md)
   ↓
4. 核对：[GOVERNANCE_ENFORCEMENT_CHECKLIST.md](GOVERNANCE_ENFORCEMENT_CHECKLIST.md)
   ↓
5. 验证：./scripts/pre_commit_test.sh
   ↓
6. 提交：遵循所有政策
```

---

## 📋 文档快速查询（Quick Lookup）

### 我想知道...

| 问题 | 查看文档 | 快速链接 |
|------|---------|--------|
| **文件应该放在哪里？** | QUICK_REFERENCE.md | [文件放置速查表](QUICK_REFERENCE.md#文件放置速查表) |
| **如何命名文件？** | PROJECT_GOVERNANCE.md | [文件放置规范](PROJECT_GOVERNANCE.md#第二部分文件放置规范) |
| **如何写代码？** | PROJECT_GOVERNANCE.md | [代码风格指南](PROJECT_GOVERNANCE.md#第四部分代码风格指南) |
| **如何写文档？** | PROJECT_GOVERNANCE.md | [文档规范](PROJECT_GOVERNANCE.md#第三部分文档规范) |
| **提交前要做什么？** | COMMIT_POLICY.md | [提交流程](COMMIT_POLICY.md#提交流程) |
| **AI 禁止做什么？** | AI_CODE_GENERATION_RULES.md | [绝对禁止](docs/AI_CODE_GENERATION_RULES.md#绝对禁止) |
| **7 项检查是什么？** | QUICK_REFERENCE.md | [7 项强制检查](QUICK_REFERENCE.md#7-项强制检查清单) |
| **如何做代码审查？** | GOVERNANCE_ENFORCEMENT_CHECKLIST.md | [审查者指南](GOVERNANCE_ENFORCEMENT_CHECKLIST.md#审查者指南) |
| **提交被拒绝了？** | COMMIT_POLICY.md | [快速修复指南](COMMIT_POLICY.md#快速修复指南) |
| **AI 工作流程？** | AI_DEVELOPER_GUIDE.md | [7 阶段工作流](docs/AI_DEVELOPER_GUIDE.md#阶段-1-分析需求in---out-思维) |

---

## ✅ 学习路径（Learning Path）

### 对于新开发者（First-time Developer）

1. **第 1 天：理解基本规则**
   - 阅读 [QUICK_REFERENCE.md](QUICK_REFERENCE.md) - 5 分钟速记
   - 扫过 [PROJECT_GOVERNANCE.md](PROJECT_GOVERNANCE.md) - 了解结构

2. **第 2 天：准备工作**
   - 详读 [COMMIT_POLICY.md](COMMIT_POLICY.md) - 了解提交规则
   - 详读 [PROJECT_GOVERNANCE.md#代码风格指南](PROJECT_GOVERNANCE.md#第四部分代码风格指南) - 学习编码标准

3. **第 3 天：开始工作**
   - 查看现有代码找参考
   - 使用 [PROJECT_GOVERNANCE.md](PROJECT_GOVERNANCE.md) 中的模板
   - 按照 [QUICK_REFERENCE.md](QUICK_REFERENCE.md) 的清单验证

### 对于代码审查者（Code Reviewer）

1. 精读 [GOVERNANCE_ENFORCEMENT_CHECKLIST.md](GOVERNANCE_ENFORCEMENT_CHECKLIST.md)
2. 理解 7 个检查级别和验证点
3. 使用快速检查清单审查提交
4. 根据审查者指南批准或拒绝

### 对于 AI 辅助开发（AI-Assisted Development）

1. 必读：[docs/AI_DEVELOPER_GUIDE.md](docs/AI_DEVELOPER_GUIDE.md) - 完整工作流程
2. 参考：[docs/AI_CODE_GENERATION_RULES.md](docs/AI_CODE_GENERATION_RULES.md) - 禁止和要求
3. 使用：[GOVERNANCE_ENFORCEMENT_CHECKLIST.md](GOVERNANCE_ENFORCEMENT_CHECKLIST.md) - 最终验证

---

## 🔐 规范的强制执行（Enforcement Mechanisms）

### 自动执行（Automatic）

```
┌─────────────────────────────────────┐
│ 1. Git Pre-commit Hook              │
│    └─> 运行 ./scripts/pre_commit_test.sh
│        └─> 7 项检查全部通过才能提交
│
│ 2. GitHub Actions                   │
│    └─> 再次运行完整检查
│        └─> 任何失败则 PR 失败
│
│ 3. 代码审查                          │
│    └─> 手动验证 7 层级检查
│        └─> 不符合则拒绝合并
└─────────────────────────────────────┘
```

### 三重防守（Triple Defense）

1. **本地防守：** Git Hook 阻止不合规代码提交
2. **远程防守：** GitHub Actions 验证所有规则
3. **人工防守：** 代码审查最终决定

**结果：** 没有办法绕过这些规则

---

## 📞 获取帮助（Getting Help）

### 如果你：

| 情况 | 做什么 |
|------|-------|
| 不知道文件放在哪里 | 查看 [QUICK_REFERENCE.md#文件放置](QUICK_REFERENCE.md) |
| 不知道如何开始 | 查看 [docs/AI_DEVELOPER_GUIDE.md#首次阅读路线](docs/AI_DEVELOPER_GUIDE.md) |
| 提交被拒绝了 | 查看 [COMMIT_POLICY.md#快速修复指南](COMMIT_POLICY.md) |
| 不确定代码风格 | 查看 [PROJECT_GOVERNANCE.md#代码风格指南](PROJECT_GOVERNANCE.md) |
| 在做代码审查 | 查看 [GOVERNANCE_ENFORCEMENT_CHECKLIST.md](GOVERNANCE_ENFORCEMENT_CHECKLIST.md) |
| AI 生成失败 | 查看 [docs/AI_CODE_GENERATION_RULES.md#问题排查](docs/AI_CODE_GENERATION_RULES.md) |

---

## 🎓 相关文档

### 项目的其他重要文档

| 文档 | 用途 |
|------|------|
| [README.md](README.md) | 项目简介和快速开始 |
| [CONTRIBUTING.md](CONTRIBUTING.md) | 贡献指南 |
| [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) | 系统架构设计 |
| [docs/API.md](docs/API.md) | API 参考文档 |

---

## 📊 规范体系结构（Governance Hierarchy）

```
PROJECT_GOVERNANCE.md (核心规范)
├─ COMMIT_POLICY.md (提交规则)
│  └─ ./scripts/pre_commit_test.sh (自动执行)
│
├─ QUICK_REFERENCE.md (快速查询)
│  └─ 所有规范的快速索引
│
├─ docs/AI_CODE_GENERATION_RULES.md (AI 禁止项)
│  └─ docs/AI_DEVELOPER_GUIDE.md (AI 工作流程)
│
└─ GOVERNANCE_ENFORCEMENT_CHECKLIST.md (验证标准)
   └─ GitHub Actions (自动验证)
      └─ Code Review (人工审查)
```

---

## ✨ 总结

| 方面 | 规范 | 执行 | 验证 |
|------|------|------|------|
| **代码** | PROJECT_GOVERNANCE.md | Git Hook | Tests + Clippy |
| **文档** | PROJECT_GOVERNANCE.md | 模板 | 审查 |
| **提交** | COMMIT_POLICY.md | Git Hook | GitHub Actions |
| **审查** | GOVERNANCE_ENFORCEMENT_CHECKLIST.md | 清单 | 人工 + 自动 |

---

## 🚀 快速开始

### 第一次读什么？

```bash
# 5 分钟快速理解
cat QUICK_REFERENCE.md

# 15 分钟理解完整规范
cat PROJECT_GOVERNANCE.md

# 如果使用 AI：30 分钟搞定工作流程
cat docs/AI_DEVELOPER_GUIDE.md
```

### 如何提交代码？

```bash
# 1. 编写代码
# 2. 运行检查
./scripts/pre_commit_test.sh

# 3. 看到 ✅ 全部通过
# 4. 提交
git add .
git commit -m "type: subject"
git push
```

---

**最后更新：** 2026年2月14日  
**适用范围：** fingerprint-rust 项目的所有开发활동  
**强制执行：** Git Hook + GitHub Actions + Code Review + AI Restrictions  
**规范状态：** ✅ 完整、全面、无例外
