# 文档管理工具使用指南

本文档介绍 fingerprint-rust 项目的智能文档管理工具集。

## 📦 工具集概述

文档管理工具集包含以下核心组件：

- **文档搜索系统** - 全文搜索和智能推荐
- **版本控制系统** - 文档变更跟踪和历史管理
- **质量检查工具** - 自动化文档质量评估
- **统一管理界面** - 集成所有功能的命令行工具

## 🚀 快速开始

### 安装和初始化

```bash
# 确保必要的依赖已安装
pip install sqlite3  # 通常Python内置

# 设置执行权限
chmod +x scripts/tools/*.py

# 初始化文档索引
python3 scripts/tools/document_search.py index
```

### 交互式使用

```bash
# 启动交互式文档管理器
python3 scripts/tools/document_manager.py --interactive
```

## 🔍 文档搜索系统

### 基本搜索

```bash
# 命令行搜索
python3 scripts/tools/document_search.py search --query "TLS指纹" --limit 5

# 交互式搜索
python3 scripts/tools/document_manager.py --interactive
# 然后选择搜索选项
```

### 搜索功能特点

- **全文搜索**: 支持文档内容的全文检索
- **模糊匹配**: 智能的关键词匹配算法
- **分类筛选**: 按文档类型和标签过滤
- **相关推荐**: 基于内容相似性的文档推荐

### 搜索示例

```bash
# 搜索特定技术
python3 scripts/tools/document_search.py search --query "HTTP/2 配置"

# 搜索用户指南
python3 scripts/tools/document_search.py search --query "user guide" --limit 3

# 搜索API相关文档
python3 scripts/tools/document_search.py search --query "API 接口"
```

## 📚 文档版本控制

### 跟踪文档变更

```bash
# 跟踪所有文档变更
python3 scripts/tools/document_version_control.py track

# 强制重新跟踪所有文档
python3 scripts/tools/document_version_control.py track --force

# 指定作者
python3 scripts/tools/document_version_control.py track --author "张三"
```

### 查看文档历史

```bash
# 查看特定文档的历史
python3 scripts/tools/document_version_control.py history --document "docs/user-guides/getting-started.md"

# 通过管理器查看
python3 scripts/tools/document_manager.py history docs/user-guides/getting-started.md
```

### 版本控制特性

- **自动版本跟踪**: 基于内容哈希的智能版本识别
- **变更历史**: 完整的文档修改历史记录
- **作者追踪**: 记录每次修改的作者信息
- **提交信息**: 自动生成有意义的变更说明

## 📊 文档质量检查

### 运行质量检查

```bash
# 检查所有文档质量
python3 scripts/maintenance/check_documentation.py

# 通过Makefile运行
make docs-check

# 通过管理器运行
python3 scripts/tools/document_manager.py check
```

### 检查内容

质量检查工具会评估以下方面：

- **文档完整性**: 检查必需文档是否存在
- **链接有效性**: 验证内部链接是否有效
- **内容质量**: 分析文档结构和格式
- **元数据完整性**: 检查更新日期和版本信息

### 检查报告

检查结果会生成详细的报告文件：
- `output/reports/documentation_quality_report.md` - 质量检查报告
- `output/reports/documentation_stats.json` - 统计数据

## 🤖 智能文档管理

### 统一管理界面

```bash
# 启动交互式管理器
python3 scripts/tools/document_manager.py --interactive

# 或者直接运行
python3 scripts/tools/document_manager.py
```

交互式界面提供以下功能：
1. 📝 文档搜索
2. 📚 查看文档历史
3. 🔍 文档质量检查
4. 🔄 跟踪文档变更

### 命令行使用

```bash
# 搜索文档
python3 scripts/tools/document_manager.py search "关键词"

# 检查质量
python3 scripts/tools/document_manager.py check

# 跟踪变更
python3 scripts/tools/document_manager.py track
```

## ⚙️ 配置和自定义

### 配置文件

工具使用以下配置位置：

```bash
# 数据库存储
output/data/document_index.db          # 搜索索引
output/data/document_versions.db       # 版本控制
output/data/document_tracking.json     # 跟踪数据

# 报告输出
output/reports/documentation_*.md      # 各种报告
output/reports/documentation_*.json    # 统计数据
```

### 自定义设置

可以通过修改源代码来调整行为：

```python
# 在 document_search.py 中调整索引策略
class DocumentIndexer:
    def _should_index_file(self, file_path: Path) -> bool:
        # 自定义文件过滤规则
        pass

# 在 check_documentation.py 中调整检查规则
class DocumentationChecker:
    def check_content_quality(self, content: str, file_path: Path) -> List[str]:
        # 自定义质量检查规则
        pass
```

## 🛠️ 集成到开发流程

### CI/CD 集成

```yaml
# .github/workflows/docs-check.yml
name: Documentation Check
on: [push, pull_request]

jobs:
  docs-check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - name: Check documentation
        run: |
          chmod +x scripts/ci/check_docs.sh
          scripts/ci/check_docs.sh
```

### 定期维护

```bash
# 添加到 crontab 进行定期检查
0 9 * * 1 /path/to/project/scripts/maintenance/update_reminder.py
0 9 * * 5 /path/to/project/scripts/maintenance/check_documentation.py
```

## 📈 最佳实践

### 文档编写规范

1. **使用清晰的标题结构**
2. **添加适当的元数据**（更新日期、版本等）
3. **包含相关标签**便于搜索
4. **定期更新内容**保持时效性

### 维护建议

1. **每周运行一次质量检查**
2. **每月审查文档更新状态**
3. **及时处理破损链接**
4. **保持文档结构一致性**

### 团队协作

1. **建立文档贡献流程**
2. **设置文档审查机制**
3. **分配文档维护责任**
4. **定期培训团队成员**

## 🆘 故障排除

### 常见问题

**Q: 搜索不到预期的文档？**
A: 确保已运行索引命令：`python3 scripts/tools/document_search.py index`

**Q: 版本控制不工作？**
A: 检查数据库权限和磁盘空间

**Q: 质量检查报错？**
A: 确保所有必需的Python包已安装

### 日志和调试

```bash
# 启用详细输出
export DEBUG=1
python3 scripts/tools/document_manager.py --interactive

# 查看详细日志
tail -f output/logs/scripts/document_tools.log
```

## 📅 未来计划

- [ ] Web界面支持
- [ ] 更智能的语义搜索
- [ ] 文档自动生成
- [ ] 多语言支持
- [ ] 集成AI辅助写作

---
*最后更新: 2026-02-13*  
*版本: v1.0*