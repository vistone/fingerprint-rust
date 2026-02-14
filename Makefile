# fingerprint-rust 项目 Makefile
# 包含文档检查和维护相关的快捷命令

.PHONY: help docs-check docs-stats docs-remind docs-all clean-docs

# 默认目标
help:
	@echo "fingerprint-rust 文档维护工具"
	@echo ""
	@echo "可用命令:"
	@echo "  docs-check     - 运行文档质量检查"
	@echo "  docs-stats     - 生成文档统计报告"
	@echo "  docs-remind    - 检查文档更新状态"
	@echo "  docs-all       - 运行所有文档检查"
	@echo "  clean-docs     - 清理文档检查生成的文件"
	@echo ""
	@echo "使用示例:"
	@echo "  make docs-check    # 检查文档质量"
	@echo "  make docs-all      # 运行完整文档检查"

# 文档质量检查
docs-check:
	@echo "🔍 运行文档质量检查..."
	@python3 scripts/maintenance/check_documentation.py

# 文档统计
docs-stats:
	@echo "📊 生成文档统计报告..."
	@scripts/ci/check_docs.sh

# 文档更新提醒
docs-remind:
	@echo "📅 检查文档更新状态..."
	@python3 scripts/maintenance/update_reminder.py

# 运行所有文档检查
docs-all: docs-check docs-stats docs-remind
	@echo "✅ 所有文档检查完成!"

# 清理文档检查生成的文件
clean-docs:
	@echo "🧹 清理文档检查文件..."
	@rm -f output/reports/documentation_*.md
	@rm -f output/reports/documentation_*.json
	@rm -f output/data/document_tracking.json
	@echo "✅ 清理完成"

# 设置执行权限
setup-permissions:
	@echo "🔐 设置脚本执行权限..."
	@chmod +x scripts/maintenance/check_documentation.py
	@chmod +x scripts/maintenance/update_reminder.py
	@chmod +x scripts/ci/check_docs.sh
	@chmod +x scripts/project_cleanup.sh
	@echo "✅ 权限设置完成"

# 初始化文档检查环境
init-docs: setup-permissions
	@echo "🚀 初始化文档检查环境..."
	@mkdir -p output/{reports,data,logs,temp}
	@echo "✅ 环境初始化完成"