# fingerprint-rust Documentation

Welcome to the fingerprint-rust documentation! Here you'll find everything you need to use and develop with this library.

## 🚀 Getting Started

- **[Quick Start Guide](user-guides/getting-started.md)** - Installation and basic setup
- **[Fingerprint Guide](user-guides/fingerprint-guide.md)** - Browser fingerprint configuration
- **[API Usage Guide](user-guides/api-usage.md)** - REST API usage

## 📚 Core Documentation

### Architecture & Design
- **[System Architecture](ARCHITECTURE.md)** - Complete system design and architecture
- **[Module Design](modules/)** - Detailed module specifications:
  - [Core Module](modules/core.md)
  - [TLS Module](modules/tls.md)
  - [HTTP Module](modules/http.md)
  - [Profiles Module](modules/profiles.md)
  - [Defense Module](modules/defense.md)

### Development
- **[Contributing Guidelines](CONTRIBUTING.md)** - How to contribute to the project
- **[Developer Guides](developer-guides/)** - Development documentation
- **[API Reference](reference/)** - Complete API documentation

### Operations
- **[Security](SECURITY.md)** - Security policies and best practices
- **[Organization](ORGANIZATION.md)** - Documentation organization guide
- **[Changelog](CHANGELOG.md)** - Version history and release notes

## 📦 Module Documentation

Each crate has detailed documentation:
- **fingerprint-core** - Core types and utilities
- **fingerprint-tls** - TLS configuration and handshake
- **fingerprint-http** - HTTP client implementation
- **fingerprint-profiles** - Browser fingerprint profiles
- **fingerprint-defense** - Passive detection and active protection
- **fingerprint-gateway** - API gateway implementation

## 📖 How to Use This Documentation

1. **New Users**: Start with [Quick Start Guide](user-guides/) for basic setup
2. **API Users**: Check [API Reference](reference/) for interface documentation  
3. **Developers**: See [System Architecture](ARCHITECTURE.md) and [Developer Guides](developer-guides/)
4. **Operators**: Review [Security](SECURITY.md) and [Organization](ORGANIZATION.md)

## 🔍 Find What You Need

- **Looking for code examples?** → Check [examples/](../examples/) directory
- **Need performance tips?** → See Developer Guides
- **Want to contribute?** → Read [CONTRIBUTING.md](CONTRIBUTING.md)
- **Have security concerns?** → Review [SECURITY.md](SECURITY.md)

## 📋 Documentation Structure

```
docs/
├── INDEX.md                 # This file - documentation hub
├── ARCHITECTURE.md          # System architecture
├── CONTRIBUTING.md          # Contributing guidelines
├── SECURITY.md             # Security policies
├── ORGANIZATION.md         # Docs organization guide
├── CHANGELOG.md            # Version history
├── user-guides/            # User guides and tutorials
├── developer-guides/       # Development documentation
├── modules/                # Module-specific documentation
├── reference/              # API reference and specs
├── specifications/         # Technical specifications
├── guides/                 # Additional guides
└── archives/               # Historical and archived documents
```

## 🎯 Quick Links

- [Project Repository](https://github.com/vistone/fingerprint-rust)
- [Issue Tracker](https://github.com/vistone/fingerprint-rust/issues)
- [Releases](https://github.com/vistone/fingerprint-rust/releases)

---

**Version**: 2.1.0  
**Last Updated**: 2026-02-14  
**Status**: Actively Maintained
- [📊 进度报告](project-management/progress-reports.md) - 项目进度跟踪

### 阶段文档
- [✅ Phase 9.4 完整报告](project-management/phase-9-4-complete.md) - API网关和限速功能
- [📁 其他阶段文档](project-management/phases/) - 历史阶段文档归档
- [📝 项目报告](project-management/reports/) - 执行报告和总结

### 历史文档
- [📋 变更日志](project-management/changelog.md) - 版本变更历史
- [🔍 架构演进](project-management/architecture-evolution.md) - 架构发展历程
- [🎉 里程碑](project-management/milestones.md) - 重要里程碑记录

## 🔍 快速查找

### 按功能查找
- **想了解指纹功能？** → [指纹使用指南](user-guides/fingerprint-guide.md)
- **想集成API？** → [API调用指南](user-guides/api-usage.md)
- **想参与开发？** → [贡献指南](developer-guides/contributing.md)
- **遇到问题？** → [故障排除](user-guides/troubleshooting.md)

### 按角色查找
- **新用户** → [快速开始](user-guides/getting-started.md)
- **开发者** → [架构设计](developer-guides/architecture.md)
- **运维人员** → [部署手册](reference/deployment-manual.md)
- **项目管理者** → [项目路线图](project-management/roadmap.md)

## 🆘 获取帮助

- **GitHub Issues**: [提交问题](https://github.com/vistone/fingerprint-rust/issues)
- **讨论区**: [社区讨论](https://github.com/vistone/fingerprint-rust/discussions)
- **邮件列表**: project@fingerprint-rust.org
- **实时聊天**: [Discord频道](https://discord.gg/fingerprint-rust)

---
**最后更新**: 2026-02-13  
**文档版本**: v2.1.0