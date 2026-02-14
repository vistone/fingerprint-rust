# fingerprint-rust 中文文档

欢迎来到 fingerprint-rust 文档!详细了解如何使用和开发此库。

## 🚀 快速开始

- **[快速开始指南](user-guides/getting-started.md)** - 安装和基本设置
- **[指纹指南](user-guides/fingerprint-guide.md)** - 浏览器指纹配置
- **[API 使用指南](user-guides/api-usage.md)** - REST API 使用

## 📚 核心文档

### 架构 & 设计
- **[系统架构](ARCHITECTURE.md)** - 完整的系统设计和架构
- **[模块设计](modules/)** - 详细的模块规范：
  - [核心模块](modules/core.md)
  - [TLS 模块](modules/tls.md)
  - [HTTP 模块](modules/http.md)
  - [配置文件模块](modules/profiles.md)
  - [防护模块](modules/defense.md)

### 开发
- **[贡献指南](https://github.com/vistone/fingerprint-rust/blob/main/docs/CONTRIBUTING.md)** - 如何为项目做贡献
- **[开发指南](developer-guides/)** - 开发文档
- **[API 参考](reference/)** - 完整的 API 文档

### 运维与安全
- **[安全](https://github.com/vistone/fingerprint-rust/blob/main/docs/SECURITY.md)** - 安全策略和最佳实践
- **[文档组织](ORGANIZATION.md)** - 文档组织指南
- **[变更日志](CHANGELOG.md)** - 版本历史和发布说明

## 📦 模块文档

每个 crate 都有详细的文档：
- **fingerprint-core** - 核心类型和工具
- **fingerprint-tls** - TLS 配置和握手
- **fingerprint-http** - HTTP 客户端实现
- **fingerprint-profiles** - 浏览器指纹配置文件
- **fingerprint-defense** - 被动检测和主动防护
- **fingerprint-gateway** - API 网关实现

## 📖 如何使用此文档

1. **新用户**: 从 [快速开始指南](user-guides/) 开始基本设置
2. **API 用户**: 查看 [API 参考](reference/) 了解接口文档
3. **开发者**: 查看 [系统架构](ARCHITECTURE.md) 和 [开发指南](developer-guides/)
4. **运维人员**: 查看 [安全](https://github.com/vistone/fingerprint-rust/blob/main/docs/SECURITY.md) 和 [文档组织](ORGANIZATION.md)

## 🔍 查找所需内容

- **寻找代码示例？** → 查看 [examples/](../../examples/) 目录
- **需要性能建议？** → 查看开发指南
- **想要贡献？** → 阅读 [贡献指南](https://github.com/vistone/fingerprint-rust/blob/main/docs/CONTRIBUTING.md)
- **有安全问题？** → 查看 [安全政策](https://github.com/vistone/fingerprint-rust/blob/main/docs/SECURITY.md)

## 📋 文档结构

```
docs/
├── zh/                      # 中文版本
├── en/                      # 英文版本
├── archives/                # 历史和已归档文档
└── README.md                # 语言选择入口
```

## 🎯 快速链接

- [项目仓库](https://github.com/vistone/fingerprint-rust)
- [问题跟踪](https://github.com/vistone/fingerprint-rust/issues)
- [发布版本](https://github.com/vistone/fingerprint-rust/releases)

---

**版本**: 2.1.0  
**最后更新**: 2026-02-14  
**状态**: 活跃维护中
