# 文档中心

欢迎来到 fingerprint-rust 文档中心！这里是项目所有文档的统一入口。

## 📚 文档分类

### 🎯 核心文档（根目录）
- [INDEX.md](INDEX.md) / [INDEX.en.md](INDEX.en.md) - 文档总索引
- [README.md](README.md) - 本文档
- [ARCHITECTURE.md](ARCHITECTURE.md) / [ARCHITECTURE.en.md](ARCHITECTURE.en.md) - 架构概览
- [API.md](API.md) - API 概述
- [CHANGELOG.md](CHANGELOG.md) - 变更日志
- [CONTRIBUTING.md](CONTRIBUTING.md) - 贡献指南
- [SECURITY.md](SECURITY.md) - 安全说明

### 📖 用户指南 ([user-guides/](user-guides/))
面向最终用户和集成开发者：
- [快速开始](user-guides/getting-started.md) - 项目入门指南
- [指纹使用](user-guides/fingerprint-guide.md) - 浏览器指纹配置和使用
- [API调用](user-guides/api-usage.md) - REST API接口使用说明

### 👨‍💻 开发者指南 ([developer-guides/](developer-guides/))
面向项目贡献者和扩展开发者：
- [架构设计](developer-guides/architecture.md) - 系统架构详解
- [贡献指南](developer-guides/contributing.md) - 如何参与开发
- [模糊测试](developer-guides/FUZZING.md) - Fuzzing 测试指南
- [性能分析](developer-guides/PROFILING.md) - 性能优化指南
- [测试报告](developer-guides/TEST_REPORT.md) - 测试结果报告
- [故障排除](developer-guides/TROUBLESHOOTING.md) - 开发常见问题
- [教程](developer-guides/TUTORIALS.md) - 开发教程

### 🏗️ 架构文档 ([architecture/](architecture/))
系统架构和设计文档：
- [架构演进](architecture/ARCHITECTURE_EVOLUTION.md) - 架构演进历史
- [二进制格式设计](architecture/BINARY_FORMAT_DESIGN.md) - 数据格式设计
- [HTTP/2设置分析](architecture/HTTP2_SETTINGS_ANALYSIS_DESIGN.md) - HTTP/2 指纹
- [TLS ClientHello解析](architecture/TLS_CLIENTHELLO_PARSING_DESIGN.md) - TLS 解析设计
- [分类器设计](architecture/PHASE_7_3_CLASSIFIER_DESIGN.md) - ML 分类器架构

### 📋 技术规范 ([specifications/](specifications/))
技术实现规范和协议：
- [GREASE 规范化](specifications/GREASE_NORMALIZATION.md) - GREASE 处理
- [HPACK 指纹](specifications/HPACK_FINGERPRINTING.md) - HTTP/2 HPACK
- [TCP 握手指纹](specifications/TCP_HANDSHAKE_FINGERPRINTING.md) - TCP 指纹
- [PSK 0-RTT 实现](specifications/PSK_0RTT_IMPLEMENTATION.md) - TLS PSK
- [Rustls 集成](specifications/RUSTLS_FINGERPRINT_INTEGRATION.md) - Rustls 指纹
- [TLS ClientHello](specifications/TLS_CLIENTHELLO_INTEGRATION_COMPLETE.md) - TLS 集成
- [数据包捕获](specifications/PACKET_CAPTURE_IMPLEMENTATION.md) - 包捕获实现
- [TTL 评分优化](specifications/TTL_SCORING_OPTIMIZATION.md) - TTL 优化

### 📚 使用指南 ([guides/](guides/))
操作指南和教程：
- [浏览器指纹捕获](guides/CAPTURE_BROWSER_FINGERPRINTS.md) - 指纹捕获
- [DNS 集成](guides/DNS_INTEGRATION_GUIDE.md) - DNS 功能集成
- [TCP 指纹应用](guides/TCP_FINGERPRINT_APPLICATION.md) - TCP 应用
- [TCP 指纹同步](guides/TCP_FINGERPRINT_SYNC.md) - 数据同步
- [统一指纹](guides/UNIFIED_FINGERPRINT.md) - 统一指纹接口
- [统一指纹示例](guides/UNIFIED_FINGERPRINT_EXAMPLE.md) - 使用示例
- [使用指南](guides/USAGE_GUIDE.md) - 综合使用说明

### 🧩 模块文档 ([modules/](modules/))
各功能模块详细文档：
- [API Noise](modules/api-noise.md) - API 噪声模块
- [Core](modules/core.md) - 核心模块
- [Defense](modules/defense.md) - 防御模块
- [DNS](modules/dns.md) - DNS 指纹模块
- [Headers](modules/headers.md) - HTTP Headers 模块
- [HTTP](modules/http.md) - HTTP 指纹模块
- [HTTP Client](modules/http_client.md) - HTTP 客户端
- [ML](modules/ml.md) - 机器学习模块
- [Profiles](modules/profiles.md) - 配置文件模块
- [TLS](modules/tls.md) - TLS 指纹模块
- [TLS Config](modules/tls_config.md) - TLS 配置
- [TLS Handshake](modules/tls_handshake.md) - TLS 握手
- [User Agent](modules/useragent.md) - UA 模块

### 🌐 HTTP 客户端 ([http-client/](http-client/))
HTTP 客户端远程更新功能：
- [远程更新概述](http-client/REMOTE_UPDATE_SUMMARY.md)
- [远程更新索引](http-client/REMOTE_UPDATE_INDEX.md)
- [快速参考](http-client/REMOTE_UPDATE_QUICK_REFERENCE.md)
- [代码指南](http-client/REMOTE_UPDATE_CODE_GUIDE.md)
- [源码概览](http-client/REMOTE_UPDATE_SOURCE_CODE_OVERVIEW.md)

### 📊 项目管理 ([project-management/](project-management/))
项目历史和发展文档：
- **阶段文档** ([phases/](project-management/phases/)) - 各开发阶段记录
- **执行报告** ([reports/](project-management/reports/)) - 项目执行情况
- **归档文档** ([phases/archived/](project-management/phases/archived/)) - 历史阶段文档

### 📈 报告文档 ([reports/](reports/))
各类分析和总结报告：
- [代码对齐报告](reports/CODE_ALIGNMENT_FINAL_REPORT.md)
- [代码同步总结](reports/CODE_SYNC_COMPLETION_SUMMARY.md)
- [完整文件清单](reports/COMPLETE_FILE_MANIFEST.md)
- [综合分析计划](reports/COMPREHENSIVE_ANALYSIS_AND_FIX_PLAN.md)
- [项目分析](reports/PROJECT_ANALYSIS.md)
- [项目执行完成](reports/PROJECT_EXECUTION_COMPLETE.md)
- [翻译状态](reports/TRANSLATION_STATUS.md)

### 🔒 安全文档 ([security/](security/))
安全审计和改进文档：
- [审计报告](security/AUDIT_REPORT.md)
- [安全审计](security/SECURITY_AUDIT.md)
- [详细审计](security/SECURITY_AUDIT_DETAILED.md)
- [安全改进](security/SECURITY_IMPROVEMENTS.md)

### 📦 归档文档 ([archives/](archives/))
历史文档归档：
- **分析报告** ([analysis-reports/](archives/analysis-reports/))
- **完成报告** ([completion-reports/](archives/completion-reports/))
- **进度报告** ([progress-reports/](archives/progress-reports/))
- **项目文档** ([project-docs/](archives/project-docs/))
- **质量报告** ([quality-reports/](archives/quality-reports/))

## 🔍 快速查找

### 按需求查找
- **新手入门** → [快速开始](user-guides/getting-started.md) / [INDEX.md](INDEX.md)
- **集成开发** → [API调用指南](user-guides/api-usage.md) / [API.md](API.md)
- **贡献代码** → [贡献指南](CONTRIBUTING.md) / [开发者指南](developer-guides/)
- **解决问题** → [故障排除](developer-guides/TROUBLESHOOTING.md)
- **了解架构** → [架构文档](ARCHITECTURE.md) / [架构目录](architecture/)

### 按功能查找
- **指纹功能** → [模块文档](modules/) / [使用指南](guides/)
- **性能优化** → [性能分析](developer-guides/PROFILING.md)
- **部署运维** → [使用指南](guides/USAGE_GUIDE.md)
- **API开发** → [API文档](API.md) / [模块文档](modules/)

## 📝 文档维护规范

### 新增文档流程
1. 确定文档所属类别
2. 按照相应模板创建文档
3. 在[INDEX.md](INDEX.md)中添加导航链接
4. 经过评审后合并

### 文档更新要求
- 保持内容的准确性和时效性
- 使用清晰简洁的语言表达
- 配合适当的代码示例和图表
- 定期审查和更新过时内容

### 质量标准
- ✅ 内容准确无误
- ✅ 结构清晰合理
- ✅ 语言通俗易懂
- ✅ 示例完整可运行

## 🤝 贡献文档

欢迎为项目文档做出贡献！请参考：
- [贡献指南](developer-guides/contributing.md)
- [文档编写规范](developer-guides/documentation-style.md)

## 🆘 获取帮助

如果找不到所需信息：
- 查看[INDEX.md](INDEX.md)获取完整导航
- 提交GitHub Issue询问
- 加入社区讨论获取帮助

---
**主索引**: [INDEX.md](INDEX.md)  
**最后更新**: 2026-02-13