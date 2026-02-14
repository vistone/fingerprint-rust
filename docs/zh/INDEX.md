# fingerprint-rust 文档

欢迎来到 fingerprint-rust 文档！您可以在这里找到使用和开发此库所需的一切。

## 🚀 快速开始

- **[快速开始指南](user-guides/getting-started.md)** - 安装和基础设置
- **[指纹识别指南](user-guides/fingerprint-guide.md)** - 浏览器指纹识别配置
- **[API 使用指南](user-guides/api-usage.md)** - REST API 使用方法

## 📚 核心文档

### 架构与设计
- **[系统架构](ARCHITECTURE.md)** - 完整的系统设计和架构
- **[模块设计](modules/)** - 详细的模块规范：
  - [核心模块](modules/core.md)
  - [TLS 模块](modules/tls.md)
  - [HTTP 模块](modules/http.md)
  - [配置文件模块](modules/profiles.md)
  - [防护模块](modules/defense.md)

### 开发
- **[贡献指南](CONTRIBUTING.md)** - 如何为项目做出贡献
- **[开发者指南](developer-guides/)** - 开发文档
- **[API 参考](reference/)** - 完整的 API 文档

### 运维
- **[安全性](SECURITY.md)** - 安全策略和最佳实践
- **[组织结构](ORGANIZATION.md)** - 文档组织指南
- **[Changelog](CHANGELOG.md)** - 版本历史和发布说明

## 📦 模块文档

每个 crate 都有详细的文档：
- **fingerprint-core** - 核心类型和实用工具
- **fingerprint-tls** - TLS 配置和握手
- **fingerprint-http** - HTTP 客户端实现
- **fingerprint-profiles** - 浏览器指纹识别配置文件
- **fingerprint-defense** - 被动检测和主动防护
- **fingerprint-gateway** - API 网关实现

## 📖 如何使用此文档

1. **新用户**：从[快速开始指南](user-guides/)开始进行基础设置
2. **API 用户**：查看 [API 参考](reference/)获取接口文档
3. **开发者**：参阅[系统架构](ARCHITECTURE.md)和[开发者指南](developer-guides/)
4. **运维人员**：查阅[安全性](SECURITY.md)和[组织结构](ORGANIZATION.md)

## 🔍 查找您需要的内容

- **正在寻找代码示例？** → 查看 [examples/](../examples/) 目录
- **需要性能提示？** → 参阅开发者指南
- **想要做出贡献？** → 阅读 [CONTRIBUTING.md](CONTRIBUTING.md)
- **有安全方面的疑虑？** → 查阅 [SECURITY.md](SECURITY.md)

## 📋 文档结构

```
docs/
├── INDEX.md                 # 此文件 - 文档中心
├── ARCHITECTURE.md          # 系统架构
├── CONTRIBUTING.md          # 贡献指南
├── SECURITY.md             # 安全策略
├── ORGANIZATION.md         # 文档组织指南
├── CHANGELOG.md            # 版本历史
├── user-guides/            # 用户指南和教程
├── developer-guides/       # 开发文档
├── modules/                # 模块特定文档
├── reference/              # API 参考和规范
├── guides/                 # 实现指南
├── http-client/            # HTTP 客户端文档
├── security/               # 安全审计文档
└── archives/               # 历史文档和报告
```

## 🎯 关键资源

- [实现指南](guides/) - 协议和功能实现
  - [浏览器指纹识别](guides/CAPTURE_BROWSER_FINGERPRINTS.md)
  - [TCP 指纹识别](guides/TCP_FINGERPRINT.md)
  - [统一指纹识别](guides/UNIFIED_FINGERPRINT.md)
  - [DNS 集成](guides/DNS_INTEGRATION_GUIDE.md)
  - [HTTP/2 集成](guides/HTTP2_INTEGRATION_GUIDE.md)

- [技术参考](reference/technical/)
  - TLS、HTTP/2、TCP、DNS 规范
  - GREASE、HPACK、PSK/0RTT 实现细节

- [HTTP 客户端文档](http-client/)
  - 远程更新指南和参考

- [安全文档](security/)
  - 安全审计报告和改进方案

- [存档资源](archives/)
  - 所有历史报告和阶段文档

### 按主题快速链接

- **想了解指纹识别功能？** → [指纹识别使用指南](user-guides/fingerprint-guide.md)
- **想集成 API？** → [API 集成指南](user-guides/api-usage.md)
- **想为开发做出贡献？** → [贡献指南](developer-guides/contributing.md)
- **遇到问题？** → [故障排除指南](user-guides/troubleshooting.md)

### 按角色查找

- **新用户** → [快速开始](user-guides/getting-started.md)
- **开发者** → [架构设计](developer-guides/architecture.md)
- **运维团队** → [部署手册](reference/deployment-manual.md)
- **项目管理** → [项目路线图](project-management/roadmap.md)

## 🆘 获取帮助

- **GitHub Issues**: [提交问题](https://github.com/vistone/fingerprint-rust/issues)
- **讨论论坛**: [社区讨论](https://github.com/vistone/fingerprint-rust/discussions)
- **邮件列表**: project@fingerprint-rust.org
- **实时聊天**: [Discord 频道](https://discord.gg/fingerprint-rust)

---
**最后更新时间**: 2026-02-13  
**文档版本**: v2.1.0
