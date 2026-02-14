# 项目报告管理 / Project Reports Directory

> **重要：** 本目录用于存放项目进展中生成的各类报告。请遵守 [AI Code Generation Rules](../AI_CODE_GENERATION_RULES.md) 中关于报告生成的规范。

---

## 📋 目录结构 / Directory Structure

```
reports/
├── performance/      # 性能相关报告 / Performance Reports
├── security/         # 安全分析报告 / Security Analysis Reports
├── analysis/         # 代码分析报告 / Code Analysis Reports
├── architecture/     # 架构设计报告 / Architecture Design Reports
├── completion/       # 完成度报告 / Completion Reports
├── evaluation/       # 评估报告 / Evaluation Reports
└── archives/         # 历史报告 / Historical Reports
```

---

## 🚫 规范要求 / Compliance Requirements

### 报告生成的强制规则：

✅ **必须遵守：**
- [ ] 仅在有 **明确需求** 时才生成报告
- [ ] 所有报告必须放在此目录下的 **子目录** 中
- [ ] 文件名使用 `UPPERCASE_WITH_UNDERSCORES` 规范
- [ ] 文件名必须包含 **日期或版本号**（如：`REPORT_20260214.md`）
- [ ] 报告必须包含以下头部信息：
  - 报告标题（一级标题）
  - 报告类型（metadata）
  - 生成日期
  - 版本号
  - 作者信息

❌ **严格禁止：**
- ~~在根目录创建报告~~
- ~~生成无日期的报告~~
- ~~生成不分类的报告~~
- ~~报告没有标题和元信息~~
- ~~生成重复的报告~~
- ~~生成临时或测试性报告~~

---

## 📝 报告标准模板 / Standard Template

所有报告开头应包含以下格式：

```markdown
# [报告标题]

> **报告类型：** [performance|security|analysis|architecture|completion|evaluation]  
> **生成日期：** YYYY-MM-DD  
> **版本：** x.x  
> **作者：** [作者名称或 AI 系统名称]

## 报告摘要

[简明扼要的 2-3 句摘要，说明报告目的和主要发现]

---

## 目录

- [章节 1](#section1)
- [章节 2](#section2)
- [章节 3](#section3)

---

## 正文

[具体报告内容...]

---

**最后更新：** YYYY-MM-DD
```

---

## 📂 分类指南 / Categorization Guide

| 目录 | 用途 | 示例 |
|------|------|------|
| **performance/** | 性能测试、基准测试、优化分析 | `PERFORMANCE_BENCHMARK_20260214.md` |
| **security/** | 安全审计、漏洞分析、风险评估 | `SECURITY_AUDIT_20260214.md` |
| **analysis/** | 代码质量、静态分析、复杂度分析 | `CODE_QUALITY_ANALYSIS_20260214.md` |
| **architecture/** | 架构设计、结构优化、技术方案 | `ARCHITECTURE_REVIEW_20260214.md` |
| **completion/** | 完成度统计、进度报告、验收报告 | `PHASE_7_COMPLETION_20260214.md` |
| **evaluation/** | 评估报告、对标分析、效果评价 | `FEATURE_EVALUATION_20260214.md` |
| **archives/** | 历史报告、旧版本、参考资料 | `HISTORICAL_ANALYSIS_20260201.md` |

---

## ✅ 生成报告前检查清单 / Pre-Generation Checklist

生成任何报告前，请回答这 4 个问题：

1. **这个报告有明确的需求吗？**
   - ✅ 有人明确要求这份报告
   - ❌ 我只是想生成一份报告

2. **这类型的报告已经存在吗？**
   - ✅ 检查本目录中是否有同类型的现有报告
   - ❌ 不要生成重复报告

3. **报告能放在正确的分类目录中吗？**
   - ✅ 确认报告属于哪个分类（performance/security/等）
   - ❌ 报告不能放在其他地方

4. **报告的命名和格式是否规范？**
   - ✅ 文件名遵循规范：`TYPE_DESCRIPTION_DATE.md`
   - ✅ 包含标准头部信息

---

## 📌 特别提醒 / Important Notes

- **不要在根目录创建报告** - 所有报告必须使用子目录
- **不要生成"临时"报告** - 所有报告都应具有永久参考价值
- **不要生成重复报告** - 如需更新，修改现有报告而非生成新报告
- **不要混淆报告和文档** - 文档放在 `docs/zh/`、`docs/en/`，报告放在此目录
- **定期归档** - 旧报告应移到 `archives/` 目录以保持清洁

---

**更新日期：** 2026年2月14日  
**相关文件：** [AI Code Generation Rules](../AI_CODE_GENERATION_RULES.md)
