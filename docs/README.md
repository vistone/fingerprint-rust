# Fingerprint-Rust 文档

## 📚 Documentation / 文档

欢迎来到 fingerprint-rust 文档中心！ 
Welcome to fingerprint-rust documentation center!

---

### 🌍 语言选择 / Language Selection

#### 中文 / Chinese  
**[→ 进入中文文档](zh/)**

主要指南：
- 📖 [快速开始](zh/guides/QUICKSTART.md) - 5分钟快速入门
- 📖 [开发者指南](zh/guides/DEVELOPMENT.md) - 开发环境和贡献
- 📖 [常见问题](zh/FAQ.md) - 常见问题解答
- 📖 [完整目录](zh/INDEX.md) - 所有文档索引

#### English  
**[→ Go to English Documentation](en/)**

Main Guides:
- 📖 [Quick Start](en/guides/QUICKSTART.md) - 5-minute setup
- 📖 [Developer Guide](en/guides/DEVELOPMENT.md) - Development & contribution
- 📖 [FAQ](en/FAQ.md) - Frequently asked questions
- 📖 [Full Index](en/INDEX.md) - All documentation

---

## 📋 文档版本说明 / Version Information

| 版本 | 内容 | 状态 | 最后更新 |
|------|------|------|--------|
| 中文 (Chinese) | 完整 | ✅ 活跃维护 | 2026-02-24 |
| English | 完整 | ✅ 活跃维护 | 2026-02-24 |
| 共享 (Shared) | Project governance | ✅ Active | 2026-02-24 |

---

## 🎯 文档组织结构 / Documentation Structure

### 中文文档 (Chinese Docs)
```
docs/zh/
├── README.md                   # 中文文档说明
├── INDEX.md                    # 文档索引（从这里开始）
├── FAQ.md                      # 常见问题解答
├── ARCHITECTURE.md             # 系统架构
├── CONTRIBUTING.md             # 贡献指南
├── SECURITY.md                 # 安全策略
│
├── guides/                     # 实现指南
│   ├── README.md
│   ├── QUICKSTART.md          # ⭐ 5分钟快速开始
│   ├── DEVELOPMENT.md         # ⭐ 开发者指南
│   ├── CAPTURE_BROWSER_FINGERPRINTS.md
│   ├── DNS_INTEGRATION_GUIDE.md
│   └── ...
│
├── user-guides/                # 用户指南
│   ├── getting-started.md
│   ├── api-usage.md
│   └── fingerprint-guide.md
│
├── developer-guides/           # 开发文档
│   ├── FUZZING.md
│   ├── PROFILING.md
│   └── ...
│
├── reference/                  # API参考
│   ├── README.md
│   └── technical/
│
└── security/                   # 安全相关
```

### English Documentation
```
docs/en/
├── README.md                   # English docs guide
├── INDEX.md                    # Documentation index (START HERE)
├── FAQ.md                      # Frequently asked questions
├── ARCHITECTURE.md             # System architecture
├── CONTRIBUTING.md             # Contributing guide
├── SECURITY.md                 # Security policies
│
├── guides/                     # Implementation guides
│   ├── README.md
│   ├── QUICKSTART.md          # ⭐ 5-minute quick start
│   ├── DEVELOPMENT.md         # ⭐ Developer guide
│   ├── CAPTURE_BROWSER_FINGERPRINTS.md
│   ├── DNS_INTEGRATION_GUIDE.md
│   └── ...
│
├── user-guides/                # User guides
│   ├── getting-started.md
│   ├── api-usage.md
│   └── fingerprint-guide.md
│
├── developer-guides/           # Development docs
│   ├── FUZZING.md
│   ├── PROFILING.md
│   └── ...
│
├── reference/                  # API reference
│   ├── README.md
│   └── technical/
│
└── security/                   # Security related
```

---

## 🔗 核心快速链接 / Quick Links

### 新用户 / New Users

| 语言 | 快速开始 | 常见问题 | 完整文档 |
|------|---------|---------|---------|
| 中文 | [→ 5分钟入门](zh/guides/QUICKSTART.md) | [→ FAQ](zh/FAQ.md) | [→ 所有文档](zh/INDEX.md) |
| English | [→ 5-min start](en/guides/QUICKSTART.md) | [→ FAQ](en/FAQ.md) | [→ All docs](en/INDEX.md) |

### 开发者 / Developers

| 语言 | 开发指南 | 架构设计 | 贡献指南 |
|------|---------|---------|---------|
| 中文 | [→ 开发者指南](zh/guides/DEVELOPMENT.md) | [→ 架构](zh/ARCHITECTURE.md) | [→ 贡献](zh/CONTRIBUTING.md) |
| English | [→ Dev guide](en/guides/DEVELOPMENT.md) | [→ Architecture](en/ARCHITECTURE.md) | [→ Contributing](en/CONTRIBUTING.md) |

---

## 📚 文档类别 / Documentation Categories

### 📖 用户文档 (User Documentation)
- **快速开始** (Quick Start) - 5分钟入门教程
- **API使用** (API Usage) - REST API和库使用
- **实现指南** (Implementation Guides) - 特定功能的集成指南
- **常见问题** (FAQ) - 回答常见问题

### 💻 开发者文档 (Developer Documentation)
- **开发指南** (Developer Guide) - 开发环境、编码规范、工作流
- **架构设计** (Architecture) - 系统设计和模块架构
- **API参考** (API Reference) - 完整的API文档
- **贡献指南** (Contributing) - 如何为项目做贡献

### 🔒 运维文档 (Operations Documentation)
- **安全政策** (Security) - 安全指南和最佳实践
- **组织指南** (Organization) - 文档组织结构
- **变更日志** (Changelog) - 版本历史

---

## 🎓 推荐阅读顺序

### 对于新用户 / For New Users
1. 选择语言 → Select a language
2. [快速开始 / Quick Start](##-快速开始)
3. [API使用指南](##-user-guides-用户指南)
4. [常见问题解答 / FAQ](##-frequently-asked-questions)

### 对于开发者 / For Developers
1. [开发指南 / Developer Guide](##-developer-guides-开发文档)
2. [系统架构 / Architecture](##-architecture--design)
3. [贡献指南 / Contributing](##-contributing)
4. [API参考 / API Reference](##-api-reference)

---

## ✨ 文档特点 / Documentation Features

✅ **完全双语** - Chinese + English  
✅ **实践示例** - 可运行的代码示例  
✅ **系统分类** - 按用途清晰分类  
✅ **链接对齐** - 中英文链接一一对应  
✅ **实时更新** - 与代码同步更新  
✅ **易于导航** - 清晰的索引和导航  

---

## 📞 获取帮助 / Getting Help

- 📖 **查看文档** - Read documentation (links above)
- 🐛 **报告问题** - [GitHub Issues](https://github.com/vistone/fingerprint-rust/issues)
- 💬 **讨论交流** - [GitHub Discussions](https://github.com/vistone/fingerprint-rust/discussions)
- 📧 **联系我们** - Contact contributors

---

**项目**: fingerprint-rust  
**当前版本**: v2.1.0  
**最后更新**: 2026-02-24
**类型**: 浏览器指纹识别库 / Browser Fingerprinting Library  
**仓库**: [GitHub](https://github.com/vistone/fingerprint-rust)  
**许可**: BSD-3-Clause
