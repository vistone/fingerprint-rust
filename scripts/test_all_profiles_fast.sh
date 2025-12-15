#!/bin/bash
# 快速测试所有浏览器指纹的导出功能

set -e

echo "=========================================="
echo "测试所有浏览器指纹的导出功能"
echo "=========================================="
echo ""

# 先编译一次
echo "编译 export_config 示例..."
cargo build --example export_config --features export --release > /dev/null 2>&1
echo "编译完成"
echo ""

# 获取所有支持的浏览器指纹列表
PROFILES=(
    "chrome_103" "chrome_104" "chrome_105" "chrome_106" "chrome_107" "chrome_108"
    "chrome_109" "chrome_110" "chrome_111" "chrome_112" "chrome_116_PSK" "chrome_116_PSK_PQ"
    "chrome_117" "chrome_120" "chrome_124" "chrome_130_PSK" "chrome_131" "chrome_131_PSK"
    "chrome_133" "chrome_133_PSK"
    "firefox_102" "firefox_104" "firefox_105" "firefox_106" "firefox_108" "firefox_110"
    "firefox_117" "firefox_120" "firefox_123" "firefox_132" "firefox_133" "firefox_135"
    "safari_15_6_1" "safari_16_0" "safari_ipad_15_6" "safari_ios_15_5" "safari_ios_15_6"
    "safari_ios_16_0" "safari_ios_17_0" "safari_ios_18_0" "safari_ios_18_5"
    "opera_89" "opera_90" "opera_91"
    "zalando_android_mobile" "zalando_ios_mobile" "nike_ios_mobile" "nike_android_mobile"
    "mms_ios" "mms_ios_2" "mms_ios_3" "mesh_ios" "mesh_android" "mesh_ios_2" "mesh_android_2"
    "confirmed_ios" "confirmed_android" "confirmed_android_2"
    "okhttp4_android_7" "okhttp4_android_8" "okhttp4_android_9" "okhttp4_android_10"
    "okhttp4_android_11" "okhttp4_android_12" "okhttp4_android_13"
    "cloudflare_custom"
)

TOTAL=${#PROFILES[@]}
SUCCESS=0
FAILED=0
FAILED_LIST=()

echo "总共 ${TOTAL} 个浏览器指纹需要测试"
echo ""

# 创建输出目录
mkdir -p exported_profiles

# 测试每个指纹
for profile in "${PROFILES[@]}"; do
    printf "测试 %-35s " "${profile}..."
    
    # 使用已编译的 release 版本，更快
    if ./target/release/examples/export_config "$profile" "exported_profiles/${profile}.json" > /dev/null 2>&1; then
        # 验证 JSON 文件是否有效且非空
        if [ -f "exported_profiles/${profile}.json" ] && [ -s "exported_profiles/${profile}.json" ]; then
            # 简单验证 JSON 格式（检查是否包含基本字段）
            if grep -q '"cipher_suites"' "exported_profiles/${profile}.json" 2>/dev/null; then
                echo "✅"
                ((SUCCESS++))
            else
                echo "❌ JSON格式错误"
                ((FAILED++))
                FAILED_LIST+=("${profile} (JSON格式错误)")
            fi
        else
            echo "❌ 文件为空"
            ((FAILED++))
            FAILED_LIST+=("${profile} (文件为空)")
        fi
    else
        echo "❌ 导出失败"
        ((FAILED++))
        FAILED_LIST+=("${profile} (导出失败)")
    fi
done

echo ""
echo "=========================================="
echo "测试结果汇总"
echo "=========================================="
echo "总测试数: ${TOTAL}"
echo "成功: ${SUCCESS}"
echo "失败: ${FAILED}"
echo ""

if [ ${FAILED} -gt 0 ]; then
    echo "失败的指纹:"
    for failed in "${FAILED_LIST[@]}"; do
        echo "  - ${failed}"
    done
    echo ""
fi

# 显示导出的文件统计
FILE_COUNT=$(ls -1 exported_profiles/*.json 2>/dev/null | wc -l)
TOTAL_SIZE=$(du -sh exported_profiles 2>/dev/null | awk '{print $1}')

echo "导出的文件:"
echo "  文件数量: ${FILE_COUNT}"
echo "  总大小: ${TOTAL_SIZE}"

echo ""
if [ ${FAILED} -eq 0 ]; then
    echo "✅ 所有浏览器指纹测试通过！"
    exit 0
else
    echo "❌ 有 ${FAILED} 个指纹测试失败"
    exit 1
fi

