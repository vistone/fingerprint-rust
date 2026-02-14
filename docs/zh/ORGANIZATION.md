# 文档组织说明

**版本 (Version)**: 2.0  
**日期**: 2026-02-13  
**状态**: 已完成

## 📋 概述

本文档说明 `docs/` 目录的组织结构和文档分类规则，确保文档管理有序、易于查找。

## 🗂️ 目录结构

```
docs/
├── README.md              # 文档中心首页
├── INDEX.md               # 文档总索引（中文）
├── INDEX.en.md            # 文档总索引（英文）
├── ARCHITECTURE.md        # 架构概览（中文）
├── ARCHITECTURE.en.md     # 架构概览（英文）
├── API.md                 # API 概述
├── CHANGELOG.md           # 变更日志
├── CONTRIBUTING.md        # 贡献指南
├── SECURITY.md            # 安全说明
│
├── architecture/          # 架构与设计文档
│   ├── ARCHITECTURE_EVOLUTION.md
│   ├── BINARY_FORMAT_DESIGN.md
│   ├── HTTP2_SETTINGS_ANALYSIS_DESIGN.md
│   ├── TLS_CLIENTHELLO_PARSING_DESIGN.md
│   └── PHASE_7_3_CLASSIFIER_DESIGN.md
│
├── specifications/        # 技术规范文档
│   ├── GREASE_NORMALIZATION.md
│   ├── HPACK_FINGERPRINTING.md
│   ├── TCP_HANDSHAKE_FINGERPRINTING.md
│   ├── PSK_0RTT_IMPLEMENTATION.md
│   ├── RUSTLS_FINGERPRINT_INTEGRATION.md
│   ├── TLS_CLIENTHELLO_INTEGRATION_COMPLETE.md
│   ├── PACKET_CAPTURE_IMPLEMENTATION.md
│   └── TTL_SCORING_OPTIMIZATION.md
│
├── guides/                # 操作指南
│   ├── CAPTURE_BROWSER_FINGERPRINTS.md
│   ├── DNS_INTEGRATION_GUIDE.md
│   ├── TCP_FINGERPRINT_APPLICATION.md
│   ├── TCP_FINGERPRINT_SYNC.md
│   ├── UNIFIED_FINGERPRINT.md
│   ├── UNIFIED_FINGERPRINT_EXAMPLE.md
│   └── USAGE_GUIDE.md
│
├── modules/               # 模块文档
│   ├── api-noise.md
│   ├── core.md
│   ├── defense.md
│   ├── dns.md
│   ├── headers.md
│   ├── http.md
│   ├── http_client.md
│   ├── ml.md
│   ├── profiles.md
│   ├── tls.md
│   ├── tls_config.md
│   ├── tls_handshake.md
│   └── useragent.md
│
├── developer-guides/      # 开发者指南
│   ├── architecture.md
│   ├── contributing.md
│   ├── FUZZING.md
│   ├── PROFILING.md
│   ├── TEST_REPORT.md
│   ├── TROUBLESHOOTING.md
│   └── TUTORIALS.md
│
├── user-guides/           # 用户指南
│   ├── getting-started.md
│   ├── fingerprint-guide.md
│   └── api-usage.md
│
├── http-client/           # HTTP 客户端文档
│   ├── REMOTE_UPDATE_SUMMARY.md
│   ├── REMOTE_UPDATE_INDEX.md
│   ├── REMOTE_UPDATE_QUICK_REFERENCE.md
│   ├── REMOTE_UPDATE_CODE_GUIDE.md
│   └── REMOTE_UPDATE_SOURCE_CODE_OVERVIEW.md
│
├── project-management/    # 项目管理文档
│   ├── phases/           # 阶段文档
│   │   ├── archived/     # 历史阶段（Phase 0-8）
│   │   ├── PHASE_1_EXECUTION_REPORT.md
│   │   ├── PHASE_7_4_COMPLETION_REPORT.md
│   │   ├── PHASE_8_DEPLOYMENT_GUIDE.md
│   │   ├── PHASE_8_EXECUTION_SUMMARY.md
│   │   ├── PHASE_8_FINAL_COMPLETION_REPORT.md
│   │   └── PHASE_9_*.md  # Phase 9 系列文档
│   ├── reports/          # 执行报告
│   │   ├── EXECUTION_SUMMARY.md
│   │   ├── PROJECT_ANALYSIS_REPORT.md
│   │   └── SESSION_3_*.md
│   └── unified-phase-9-4.md
│
├── reports/              # 分析报告
│   ├── CODE_ALIGNMENT_FINAL_REPORT.md
│   ├── CODE_SYNC_COMPLETION_SUMMARY.md
│   ├── COMPLETE_FILE_MANIFEST.md
│   ├── COMPREHENSIVE_ANALYSIS_AND_FIX_PLAN.md
│   ├── PROJECT_ANALYSIS.md
│   ├── PROJECT_EXECUTION_COMPLETE.md
│   ├── TRANSLATION_STATUS.md
│   └── ...
│
├── security/             # 安全文档
│   ├── AUDIT_REPORT.md
│   ├── SECURITY_AUDIT.md
│   ├── SECURITY_AUDIT_DETAILED.md
│   └── SECURITY_IMPROVEMENTS.md
│
├── archives/             # 历史归档
│   ├── analysis-reports/
│   ├── completion-reports/
│   ├── progress-reports/
│   ├── project-docs/
│   └── quality-reports/
│
├── archive/              # 旧版归档
│   ├── fingerprint_api_deprecated/
│   └── phase9.4/
│
└── reference/            # 参考文档
    ├── document-management-tools.md
    ├── guides/
    └── specifications/
```

## 📊 分类规则

### 1. 核心文档（根目录）
**存放位置**: `docs/`  
**文档类型 (Document Type)**:
- 主索引文件（INDEX.md）
- 总览文档（ARCHITECTURE.md, API.md）
- 项目元信息（README.md, CHANGELOG.md, CONTRIBUTING.md, SECURITY.md）

**命名规则**:
- 使用大写字母和下划线
- 支持多语言版本（.en.md, .zh.md）

### 2. 架构文档
**存放位置**: `docs/architecture/`  
**文档类型 (Document Type)**:
- 系统架构设计
- 数据结构设计
- 架构演进记录

**命名规则**:
- 描述性命名，如 `BINARY_FORMAT_DESIGN.md`
- 使用 `_DESIGN` 后缀表示设计文档

### 3. 技术规范
**存放位置**: `docs/specifications/`  
**文档类型 (Document Type)**:
- 协议实现规范
- 算法实现规范
- 技术标准文档

**命名规则**:
- 技术名称 + 功能描述
- 如 `TCP_HANDSHAKE_FINGERPRINTING.md`

### 4. 使用指南
**存放位置**: `docs/guides/`  
**文档类型 (Document Type)**:
- 操作指南
- 集成指南
- 最佳实践

**命名规则**:
- 使用 `_GUIDE` 后缀
- 描述清晰的功能名称

### 5. 模块文档
**存放位置**: `docs/modules/`  
**文档类型 (Document Type)**:
- 各功能模块的详细文档
- API 接口说明
- 使用示例 (Usage Examples)

**命名规则**:
- 使用小写字母和连字符
- 与模块名称保持一致（如 `fingerprint-ml` → `ml.md`）

### 6. 开发者文档
**存放位置**: `docs/developer-guides/`  
**文档类型 (Document Type)**:
- 开发指南
- 测试文档
- 调试文档

**命名规则**:
- 功能描述性命名
- 可使用大写（如 `FUZZING.md`）或小写（如 `contributing.md`）

### 7. 用户文档
**存放位置**: `docs/user-guides/`  
**文档类型 (Document Type)**:
- 快速入门
- 使用教程
- API 使用说明

**命名规则**:
- 使用小写字母和连字符
- 描述性命名，如 `getting-started.md`

### 8. 项目管理
**存放位置**: `docs/project-management/`  
**文档类型 (Document Type)**:
- 阶段规划和报告
- 项目执行记录
- 路线图

**分类规则**:
- `phases/` - 各阶段文档
- `phases/archived/` - 历史阶段归档
- `reports/` - 执行报告

### 9. 报告文档
**存放位置**: `docs/reports/`  
**文档类型 (Document Type)**:
- 分析报告
- 完成报告
- 状态总结

**命名规则**:
- 使用 `_REPORT` 或 `_SUMMARY` 后缀
- 如 `CODE_ALIGNMENT_FINAL_REPORT.md`

### 10. 安全文档
**存放位置**: `docs/security/`  
**文档类型 (Document Type)**:
- 安全审计
- 安全改进
- 漏洞报告

**命名规则**:
- 使用 `SECURITY_` 或 `AUDIT_` 前缀

### 11. 归档文档
**存放位置**: `docs/archives/` 或 `docs/archive/`  
**文档类型 (Document Type)**:
- 历史文档
- 废弃功能文档
- 已完成项目文档

**分类规则**:
- 按文档类型分子目录
- 保持原有文件名

## 🔄 整理历史

### 2026-02-13 - 全面整理
**变更内容**:
1. ✅ 创建 `architecture/` 目录，移入架构设计文档
2. ✅ 创建 `specifications/` 目录，移入技术规范文档
3. ✅ 整理 `guides/` 目录，统一操作指南文档
4. ✅ 整理 `developer-guides/` 目录，移入开发测试文档
5. ✅ 归档历史 Phase 报告到 `project-management/phases/archived/`
6. ✅ 整理 `reports/` 目录，移入各类报告文档
7. ✅ 更新 `README.md`，反映最新结构

**移动文件清单**:

**架构文档** → `architecture/`:
- ARCHITECTURE_EVOLUTION.md
- BINARY_FORMAT_DESIGN.md
- HTTP2_SETTINGS_ANALYSIS_DESIGN.md
- TLS_CLIENTHELLO_PARSING_DESIGN.md
- PHASE_7_3_CLASSIFIER_DESIGN.md

**技术规范** → `specifications/`:
- GREASE_NORMALIZATION.md
- HPACK_FINGERPRINTING.md
- TCP_HANDSHAKE_FINGERPRINTING.md
- PSK_0RTT_IMPLEMENTATION.md
- RUSTLS_FINGERPRINT_INTEGRATION.md
- TLS_CLIENTHELLO_INTEGRATION_COMPLETE.md
- PACKET_CAPTURE_IMPLEMENTATION.md
- TTL_SCORING_OPTIMIZATION.md

**开发文档** → `developer-guides/`:
- FUZZING.md
- PROFILING.md
- TEST_REPORT.md
- TROUBLESHOOTING.md
- TUTORIALS.md

**Phase 归档** → `project-management/phases/archived/`:
- P0_COMPLETION_REPORT.md
- PHASE2_COMPLETE.md
- PHASE2_INFRASTRUCTURE_SUMMARY.md
- PHASE2_VALIDATION_COMPLETE_REPORT.md
- PHASE_5B_COMPLETION_REPORT.md
- PHASE_6_COMPLETION_REPORT.md
- PHASE_6_PERFORMANCE_REPORT.md
- PHASE_7_1_COMPLETION_REPORT.md
- PHASE_7_2_EXECUTION_PLAN.md
- PHASE_7_2_EXECUTION_REPORT.md
- PHASE_7_3_COMPLETION_REPORT.md
- PHASE_7_3_EXECUTION_REPORT.md
- PHASE_7_4_DEVELOPMENT_PLAN.md
- PHASE_7_EXECUTION_SUMMARY.md
- PHASE_7_PLAN.md
- PHASE_7_PROJECT_SUMMARY.md
- PHASE_8_IMPLEMENTATION_REPORT.md
- FIREFOX_VALIDATION_COMPLETE.md
- HTTP2_INTEGRATION_COMPLETE.md
- JA3_DATABASE_MATCHING_COMPLETE.md

**报告文档** → `reports/`:
- BROWSER_VERSION_ADAPTATION_SUMMARY.md
- BROWSER_VERSION_ADAPTATION.md
- ENHANCEMENT_SUMMARY.md
- NEXT_STEPS_SUMMARY.md
- NEXT_STEP_PHASE_7_4_ACTION_PLAN.md
- PRODUCTION_DEPLOYMENT_SUMMARY.md
- SESSION_3_FINAL_SUMMARY.md
- SLA_AND_MONITORING_PROTOCOL.md
- LOGIC_REVIEW.md

**保持在根目录**:
- INDEX.md / INDEX.en.md
- ARCHITECTURE.md / ARCHITECTURE.en.md
- API.md
- README.md
- CHANGELOG.md
- CONTRIBUTING.md
- SECURITY.md

## 📝 维护指南

### 新增文档时
1. 确定文档类型和所属分类
2. 选择合适的目录
3. 遵循命名规则
4. 更新 README.md 和 INDEX.md

### 文档废弃时
1. 移动到 `archives/` 对应子目录
2. 在文档顶部添加 `[已归档]` 标记
3. 更新索引文件

### 定期审查
- 每季度审查文档结构
- 清理过时文档
- 更新索引和分类

## 🎯 最佳实践

### ✅ 推荐做法
- 文档命名清晰、描述准确
- 按功能和类型分类存放
- 保持目录层级不超过 3 层
- 及时更新索引文件
- 归档历史文档而非删除

### ❌ 避免做法
- 在根目录堆积大量文档
- 使用模糊的文件名
- 创建过深的目录层级
- 文档散落在多个位置
- 删除历史文档

## 📞 联系方式

如有文档组织相关问题，请：
- 查阅 [README.md](README.md)
- 提交 GitHub Issue
- 联系项目维护者

---
**最后更新 (Last Updated)**: 2026-02-13  
**维护者**: fingerprint-rust 团队
