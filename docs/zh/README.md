# 文档指南

欢迎来到 fingerprint-rust 中文文档！本目录包含了该项目的所有中文版本文档。

## 📍 从这里开始

- **[文档索引](INDEX.md)** ← 主要文档中心

## 📚 文档结构

```
zh/
├── INDEX.md                    # 中文文档导航中心
├── ORGANIZATION.md             # 文档组织说明
├── ARCHITECTURE.md             # 系统架构
├── CHANGELOG.md                # 变更日志
│
├── user-guides/                # 用户指南和教程
│   ├── getting-started.md       # 快速开始
│   ├── api-usage.md             # API 使用指南
│   └── fingerprint-guide.md     # 指纹指南
│
├── developer-guides/           # 开发文档
│   ├── FUZZING.md
│   ├── PROFILING.md
│   ├── TROUBLESHOOTING_GUIDE.md
│   ├── TUTORIALS.md
│   ├── contributing.md
│   ├── architecture.md
│   └── TEST_REPORT.md
│
├── modules/                    # 模块文档
│   └── [核心、TLS、HTTP 等模块]
│
├── guides/                     # 实现指南
│   ├── CAPTURE_BROWSER_FINGERPRINTS.md
│   ├── DNS_INTEGRATION_GUIDE.md
│   ├── HTTP2_INTEGRATION_GUIDE.md
│   ├── OPERATIONS_RUNBOOK.md
│   ├── TCP_FINGERPRINT.md
│   ├── UNIFIED_FINGERPRINT.md
│   ├── USAGE_GUIDE.md
│   └── README.md
│
├── reference/                  # 参考文档
│   ├── README.md
│   ├── document-management-tools.md
│   └── technical/              # 技术规范
│
├── architecture/               # 架构文档
├── http-client/                # HTTP 客户端文档
├── security/                   # 安全文档
└── [其他目录]
```

## 📖 如何使用此文档

1. **新用户**: 从 [快速开始指南](user-guides/) 开始基本设置
2. **API 用户**: 查看 [参考文档](reference/) 了解接口文档
3. **开发者**: 查看 [系统架构](ARCHITECTURE.md) 和 [开发指南](developer-guides/)
4. **运维人员**: 查看 [安全](https://github.com/vistone/fingerprint-rust/blob/main/docs/SECURITY.md) 和 [组织](ORGANIZATION.md)

## 🔍 查找所需内容

- **寻找代码示例？** → 查看 [examples/](../../examples/) 目录
- **想要性能建议？** → 查看开发指南
- **想要贡献？** → 阅读 [贡献指南](https://github.com/vistone/fingerprint-rust/blob/main/docs/CONTRIBUTING.md)
- **安全问题？** → 查看 [安全政策](https://github.com/vistone/fingerprint-rust/blob/main/docs/SECURITY.md)

## 🌐 其他语言版本

- **[English Documentation](../en/)** - 英文完整文档

---

**版本 (Version)**: 2.0  
**最后更新 (Last Updated)**: 2026-02-14  
**状态**: 活跃维护中
