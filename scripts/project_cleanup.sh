#!/bin/bash

# 项目清理和重组脚本
# 用于整理fingerprint-rust项目的文件结构

set -e  # 遇到错误立即退出

echo "🚀 开始项目清理和重组..."

# 创建必要的目录结构
echo "📁 创建标准目录结构..."
mkdir -p config/{build,deployment,monitoring,services}
mkdir -p output/{logs,data/{dns,pcap,analysis},temp,reports}
mkdir -p docs/{user-guides,developer-guides,reference,project-management}
mkdir -p tests/{unit,integration,e2e,performance}

# 移动日志和临时文件
echo "🧹 清理输出文件..."
if [ -d "dns_output" ]; then
    mv dns_output output/data/dns/
    echo "  ✓ 移动 dns_output → output/data/dns/"
fi

if [ -f "phase-9-3-deployment.log" ]; then
    mv phase-9-3-deployment.log output/logs/
    echo "  ✓ 移动部署日志到 output/logs/"
fi

if [ -d "tmp" ]; then
    mv tmp/* output/temp/ 2>/dev/null || true
    rmdir tmp
    echo "  ✓ 移动临时文件到 output/temp/"
fi

# 归类配置文件
echo "⚙️  归类配置文件..."
# 移动Kubernetes配置
if [ -d "k8s" ]; then
    mv k8s config/deployment/
    echo "  ✓ 移动 k8s → config/deployment/k8s/"
fi

# 移动监控配置
if [ -d "monitoring" ]; then
    mv monitoring config/monitoring/
    echo "  ✓ 移动 monitoring → config/monitoring/"
fi

# 移动部署脚本
if [ -f "deploy.sh" ]; then
    mkdir -p config/deployment/scripts
    mv deploy.sh config/deployment/scripts/
    echo "  ✓ 移动 deploy.sh → config/deployment/scripts/"
fi

# 清理已废弃的目录
echo "🗑️  清理废弃目录..."
if [ -d "fingerprint_api" ]; then
    echo "  ⚠️  fingerprint_api 目录包含已废弃的Python实现"
    echo "     建议备份后删除此目录"
    # 可选：mv fingerprint_api fingerprint_api_backup
fi

# 生成文档索引
echo "📚 生成文档索引..."
cat > docs/INDEX.md << 'EOF'
# 文档中心

欢迎来到 fingerprint-rust 文档中心！

## 📚 用户指南
- [快速开始](user-guides/getting-started.md) - 项目入门指南
- [指纹使用](user-guides/fingerprint-guide.md) - 浏览器指纹使用说明
- [API使用](user-guides/api-usage.md) - API调用指南

## 👨‍💻 开发者指南
- [架构设计](developer-guides/architecture.md) - 系统架构详解
- [贡献指南](developer-guides/contributing.md) - 如何参与开发
- [测试指南](developer-guides/testing.md) - 测试策略和方法

## 📖 参考文档
- [API参考](reference/api-reference.md) - 完整API文档
- [配置说明](reference/configuration.md) - 配置参数详解
- [故障排除](reference/troubleshooting.md) - 常见问题解决

## 📋 项目管理
- [路线图](project-management/roadmap.md) - 项目发展规划
- [发布记录](project-management/release-notes.md) - 版本更新历史

---
*最后更新: $(date)*
EOF

echo "  ✓ 生成 docs/INDEX.md"

# 显示清理结果
echo ""
echo "✅ 项目清理完成！"
echo ""
echo "📋 清理结果摘要:"
echo "  • 创建了标准化的目录结构"
echo "  • 移动了输出文件到 output/ 目录"
echo "  • 归类了配置文件到 config/ 目录"
echo "  • 生成了统一的文档索引"
echo ""
echo "📝 下一步建议:"
echo "  1. 检查移动的文件是否正确"
echo "  2. 更新相关的路径引用"
echo "  3. 删除已废弃的 fingerprint_api 目录（如果确认不再需要）"
echo "  4. 重新组织 docs/ 目录下的文档"
echo ""
echo "⚠️  重要提醒:"
echo "  • 建议先在测试环境中运行此脚本"
echo "  • 备份重要数据后再执行"
echo "  • 检查是否有硬编码的路径需要更新"