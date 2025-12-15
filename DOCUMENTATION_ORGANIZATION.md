# 文档组织结构

## 📁 目录结构

```
fingerprint-rust/
├── README.md                    # 项目主文档（保留在根目录）
├── CHANGELOG.md                 # 更新日志（保留在根目录）
│
├── docs/                        # 所有文档
│   ├── INDEX.md                # 文档索引
│   ├── README.md               # 文档目录说明
│   │
│   ├── guides/                 # 使用指南
│   │   ├── USAGE_GUIDE.md      # 使用指南
│   │   └── CAPTURE_BROWSER_FINGERPRINTS.md  # 抓取浏览器指纹指南
│   │
│   ├── reports/                 # 测试报告
│   │   ├── ALL_PROFILES_TEST_REPORT.md
│   │   ├── TEST_RESULTS.md
│   │   ├── CURRENT_IMPLEMENTATION_STATUS.md
│   │   └── IMPLEMENTATION_SUMMARY.md
│   │
│   ├── modules/                 # 模块文档
│   │   ├── tls_config.md
│   │   ├── http_client.md
│   │   └── ...
│   │
│   ├── archive/                # 历史文档
│   │   └── ...
│   │
│   └── *.md                    # 技术文档
│       ├── RUSTLS_FINGERPRINT_INTEGRATION.md
│       ├── CUSTOM_TLS_IMPLEMENTATION.md
│       ├── CLIENTHELLO_ANALYSIS.md
│       └── UTLS_STYLE_API.md
│
├── scripts/                     # 测试脚本
│   ├── test_all_profiles.sh
│   ├── test_all_profiles_fast.sh
│   ├── test_all_profiles_simple.sh
│   └── test_http2_settings.sh
│
└── exported_profiles/          # 导出的配置文件
    └── *.json                  # 66 个浏览器指纹配置
```

## 📚 文档分类

### 根目录（仅保留标准文件）
- `README.md` - 项目主文档
- `CHANGELOG.md` - 更新日志

### docs/guides/ - 使用指南
- 用户如何使用项目的指南
- 如何抓取浏览器指纹的教程

### docs/reports/ - 测试报告
- 测试结果和统计
- 实现状态报告

### docs/ - 技术文档
- 技术实现细节
- API 文档
- 架构说明

### docs/modules/ - 模块文档
- 各个模块的详细文档

### scripts/ - 脚本工具
- 测试脚本
- 自动化工具

## 🔍 快速查找

- **如何使用**: `docs/guides/USAGE_GUIDE.md`
- **如何抓取指纹**: `docs/guides/CAPTURE_BROWSER_FINGERPRINTS.md`
- **测试报告**: `docs/reports/`
- **技术文档**: `docs/INDEX.md`

## ✅ 整理完成

所有文档已按类别整理到相应目录，根目录保持整洁。

