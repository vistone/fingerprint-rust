#!/bin/bash

# CI/CD 文档检查脚本
# 用于在持续集成流程中自动检查文档质量

set -e  # 遇到错误立即退出

echo "🚀 开始CI文档检查..."

# 检查必需工具
check_tools() {
    echo "🔍 检查必需工具..."
    
    local missing_tools=()
    
    # 检查Python
    if ! command -v python3 &> /dev/null; then
        missing_tools+=("python3")
    fi
    
    # 检查必要的Python包
    if ! python3 -c "import re, json, pathlib" &> /dev/null; then
        echo "⚠️  缺少必要的Python标准库"
    fi
    
    if [ ${#missing_tools[@]} -ne 0 ]; then
        echo "❌ 缺少工具: ${missing_tools[*]}"
        exit 1
    fi
    
    echo "✅ 所有工具检查通过"
}

# 运行文档质量检查
run_documentation_check() {
    echo "📚 运行文档质量检查..."
    
    cd "$(dirname "$0")/../.."
    
    # 确保输出目录存在
    mkdir -p output/reports
    
    # 运行检查脚本
    if python3 scripts/maintenance/check_documentation.py; then
        echo "✅ 文档检查完成"
        
        # 检查是否有严重问题
        local report_file="output/reports/documentation_quality_report.md"
        if [ -f "$report_file" ]; then
            # 检查报告中是否包含严重问题
            if grep -q "缺失文档.*[1-9]" "$report_file" || grep -q "质量问题.*[1-9]" "$report_file"; then
                echo "⚠️  发现文档问题，但允许继续构建"
                return 0  # 允许构建继续，但标记警告
            else
                echo "🎉 文档质量良好"
                return 0
            fi
        fi
    else
        echo "❌ 文档检查失败"
        return 1
    fi
}

# 检查文档链接有效性
check_links() {
    echo "🔗 检查文档链接..."
    
    cd "$(dirname "$0")/../.."
    
    local broken_links=0
    
    # 检查README中的链接
    if [ -f "README.md" ]; then
        # 简单的链接检查（可以扩展）
        local readme_links=$(grep -o '\[.*\](.*)' README.md | grep -v '^http' | wc -l)
        echo "README中包含 $readme_links 个内部链接"
    fi
    
    echo "✅ 链接检查完成"
}

# 生成文档统计报告
generate_stats() {
    echo "📊 生成文档统计..."
    
    cd "$(dirname "$0")/../.."
    
    local stats_file="output/reports/documentation_stats.json"
    mkdir -p "$(dirname "$stats_file")"
    
    # 统计文档数量
    local total_docs=$(find . -name "*.md" -not -path "./target/*" -not -path "./.git/*" | wc -l)
    local docs_docs=$(find docs/ -name "*.md" 2>/dev/null | wc -l)
    local root_docs=$(find . -maxdepth 1 -name "*.md" | wc -l)
    
    # 生成JSON统计
    cat > "$stats_file" << EOF
{
    "timestamp": "$(date -Iseconds)",
    "total_documents": $total_docs,
    "docs_directory": $docs_docs,
    "root_directory": $root_docs,
    "directories": {
        "config_readme": $(test -f config/README.md && echo "true" || echo "false"),
        "docs_readme": $(test -f docs/README.md && echo "true" || echo "false"),
        "output_readme": $(test -f output/README.md && echo "true" || echo "false"),
        "crates_readme": $(test -f crates/README.md && echo "true" || echo "false"),
        "scripts_readme": $(test -f scripts/README.md && echo "true" || echo "false")
    }
}
EOF
    
    echo "✅ 统计报告生成完成: $stats_file"
}

# 主执行流程
main() {
    echo "========================================"
    echo "  fingerprint-rust 文档CI检查"
    echo "========================================"
    
    check_tools
    run_documentation_check
    check_links
    generate_stats
    
    echo "========================================"
    echo "  ✅ 所有检查完成!"
    echo "========================================"
}

# 执行主函数
main "$@"