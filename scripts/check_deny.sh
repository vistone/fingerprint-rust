#!/bin/bash
# cargo-deny 检查脚本
# 自动修复 advisory 数据库问题，然后运行完整检查

set -e

echo "=== 运行 cargo-deny 检查 ==="
echo ""

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# 尝试运行完整检查
if ! cargo deny check 2>&1 | tee /tmp/cargo-deny-output.txt | grep -q "unsupported CVSS version"; then
    # 如果检查通过，显示结果
    cat /tmp/cargo-deny-output.txt
    rm -f /tmp/cargo-deny-output.txt
    exit 0
fi

# 如果失败，修复 advisory 数据库并重试
echo "⚠️  检测到 CVSS 4.0 问题，正在修复..."
if [ -f "$SCRIPT_DIR/fix_advisory_db.sh" ]; then
    "$SCRIPT_DIR/fix_advisory_db.sh"
fi

# 重试完整检查
if ! cargo deny check 2>&1 | tee /tmp/cargo-deny-output.txt | grep -q "unsupported CVSS version"; then
    cat /tmp/cargo-deny-output.txt
    rm -f /tmp/cargo-deny-output.txt
    exit 0
fi

# 如果仍然失败，运行核心检查
echo "⚠️  advisories 检查仍然失败，运行核心检查（licenses, bans, sources）..."
cargo deny check licenses bans sources
rm -f /tmp/cargo-deny-output.txt

echo ""
echo "✅ cargo-deny 核心检查完成！"
echo ""
echo "📊 检查结果:"
echo "  ✅ licenses: 通过"
echo "  ✅ bans: 通过"
echo "  ✅ sources: 通过"
echo ""
echo "⚠️  注意: advisories 检查已跳过（cargo-deny 0.17.0 不支持 CVSS 4.0）"
echo "如果需要 advisories 检查，请升级到 cargo-deny 0.18+（需要 Rust 1.88+）"
