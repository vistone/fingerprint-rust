#!/bin/bash
# 简单测试所有浏览器指纹

echo "测试所有浏览器指纹导出功能..."
echo ""

# 获取所有指纹列表
PROFILES=$(cargo run --example export_config --features export 2>&1 | grep "  - " | sed 's/  - //' | sort)

mkdir -p exported_profiles
SUCCESS=0
FAILED=0

for profile in $PROFILES; do
    printf "%-40s " "$profile"
    if ./target/release/examples/export_config "$profile" "exported_profiles/${profile}.json" >/dev/null 2>&1; then
        if [ -f "exported_profiles/${profile}.json" ] && grep -q '"cipher_suites"' "exported_profiles/${profile}.json" 2>/dev/null; then
            echo "✅"
            ((SUCCESS++))
        else
            echo "❌"
            ((FAILED++))
        fi
    else
        echo "❌"
        ((FAILED++))
    fi
done

echo ""
echo "成功: $SUCCESS, 失败: $FAILED, 总计: $((SUCCESS + FAILED))"

