# 文档指南

欢迎来到 fingerprint-rust 文档！此目录包含项目的所有文档。

## 📍 从这里开始

- **[文档索引](INDEX.md)** ← 主文档中心

## 📚 文档结构

```
docs/
├── INDEX.md                    # 主文档中心（从这里开始）
├── ARCHITECTURE.md             # 系统架构和设计
├── CONTRIBUTING.md             # 如何为项目做贡献
├── SECURITY.md                 # 安全策略和指引
├── ORGANIZATION.md             # 文档组织指南
├── CHANGELOG.md                # 版本历史和发布说明
├── API.md                      # API 概览
│
├── user-guides/                # 用户指南和教程
│   ├── README.md
│   ├── getting-started.md
│   ├── api-usage.md
│   └── fingerprint-guide.md
│
├── developer-guides/           # 开发和故障排除
│   ├── README.md
│   ├── FUZZING.md
│   ├── PROFILING.md
│   ├── TROUBLESHOOTING_GUIDE.md
│   ├── TUTORIALS.md
│   ├── contributing.md
│   ├── architecture.md
│   └── TEST_REPORT.md
│
├── guides/                     # 实现指南
│   ├── README.md
│   ├── CAPTURE_BROWSER_FINGERPRINTS.md
│   ├── DNS_INTEGRATION_GUIDE.md
│   ├── HTTP2_INTEGRATION_GUIDE.md
│   ├── OPERATIONS_RUNBOOK.md
│   ├── TCP_FINGERPRINT.md
│   ├── UNIFIED_FINGERPRINT.md
│   └── USAGE_GUIDE.md
│
├── reference/                  # 参考文档
│   ├── README.md
│   ├── document-management-tools.md
│   ├── technical/              # 技术规范
│   │   ├── GREASE_NORMALIZATION.md
│   │   ├── HPACK_FINGERPRINTING.md
│   │   ├── PACKET_CAPTURE_IMPLEMENTATION.md
│   │   ├── PSK_0RTT_IMPLEMENTATION.md
│   │   ├── RUSTLS_FINGERPRINT_INTEGRATION.md
│   │   ├── TCP_HANDSHAKE_FINGERPRINTING.md
│   │   ├── TLS_CLIENTHELLO_INTEGRATION_COMPLETE.md
│   │   └── TTL_SCORING_OPTIMIZATION.md
│
├── architecture/               # 架构文档
├── modules/                    # 模块特定指南
├── http-client/                # HTTP 客户端文档
├── security/                   # 安全文档
│
└── archives/                   # 历史文档和报告
    └── 各种存档文档
```

## 🎯 快速导航

### 针对不同用户类型

**👤 项目用户**
- 从 [快速开始](user-guides/getting-started.md) 开始
- 查看 [API 使用](user-guides/api-usage.md) 进行集成

**👨‍💻 开发者**
- 阅读 [架构](developer-guides/architecture.md)
- 查看 [故障排除](developer-guides/TROUBLESHOOTING_GUIDE.md)
- 参考 [贡献指南](CONTRIBUTING.md)

**🏢 DevOps/运维**
- 查阅 [运维工作手册](guides/OPERATIONS_RUNBOOK.md)
- 查看 [安全](SECURITY.md)
- 参考 [组织](ORGANIZATION.md)

**🔬 贡献者**
- 阅读 [CONTRIBUTING.md](CONTRIBUTING.md)
- 查看 [ARCHITECTURE.md](ARCHITECTURE.md)
- 审查 [故障排除指南](developer-guides/TROUBLESHOOTING_GUIDE.md)

## ✨ 关键文档文件

| 文档 | 用途 | 目标用户 |
|------|------|---------|
| INDEX.md | 文档中心 | 所有人 |
| ARCHITECTURE.md | 系统设计 | 开发者、架构师 |
| CONTRIBUTING.md | 贡献指南 | 贡献者 |
| SECURITY.md | 安全策略 | 安全、运维 |
| CHANGELOG.md | 发布说明 | 所有人 |
| ORGANIZATION.md | 文档结构 | 维护人员 |

## 📖 文档状态

- ✅ **核心文档** - 维护良好且最新
- ✅ **用户指南** - 完整且当前
- ✅ **开发者指南** - 全面
- ✅ **技术规范** - 详细准确
- 📦 **存档** - 历史文档供参考

## 🔄 为文档做贡献

如需贡献或报告文档问题：

1. 阅读 [CONTRIBUTING.md](CONTRIBUTING.md)
2. 查看现有 [问题](https://github.com/vistone/fingerprint-rust/issues)
3. 通过拉取请求提交改进

## 📞 获取帮助

- **常见问题** → 查看 [user-guides/](user-guides/) 目录
- **技术问题** → 查看 [故障排除指南](developer-guides/TROUBLESHOOTING_GUIDE.md)
- **API 问题** → 查看 [API 参考](reference/)
- **缺陷/功能** → 打开 [问题](https://github.com/vistone/fingerprint-rust/issues)

---

**最后更新**: 2026-02-14  
**版本**: 2.1.0  
**维护者**: fingerprint-rust 贡献者
