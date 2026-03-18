#!/bin/bash
# 提交前全面测试脚本
# 完全遵循 GitHub Actions 的规则运行本地检查
# 同步 .github/workflows/ci.yml 和 security-audit.yml 的检查项

set -euo pipefail

echo "=========================================="
echo "🔍 提交前检查（遵循 GitHub Actions 规则）"
echo "=========================================="
echo ""

# 获取脚本所在目录
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$PROJECT_ROOT"

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# 测试计数器
PASSED=0
FAILED=0
TEST_OUTPUT_LOG="$(mktemp /tmp/fingerprint-pre-commit.XXXXXX.log)"
TEST_FEATURES="rustls-tls,compression,http2,connection-pool,dns"

cleanup() {
    rm -f "$TEST_OUTPUT_LOG"
}

trap cleanup EXIT

# 测试函数
run_test() {
    local test_name="$1"
    shift
    
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "🧪 $test_name"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    
    # 执行测试命令
    if "$@" > "$TEST_OUTPUT_LOG" 2>&1; then
        echo -e "${GREEN}✅ 通过${NC}"
        PASSED=$((PASSED + 1))
        echo ""
        return 0
    else
        echo -e "${RED}❌ 失败${NC}"
        # 显示最后几行输出以便调试
        if [ -f "$TEST_OUTPUT_LOG" ]; then
            echo "错误输出:"
            tail -20 "$TEST_OUTPUT_LOG" | sed 's/^/  /'
        fi
        FAILED=$((FAILED + 1))
        echo ""
        return 1
    fi
}

# ========== LINT 检查（来自 ci.yml:lint job）==========

# 1. 代码格式化检查（对应 GitHub Actions: Check formatting）
run_test "格式化检查 (cargo fmt)" cargo fmt --all -- --check

# 2. Clippy 检查（对应 GitHub Actions: Run clippy）
# 注意：不使用 --all-features 避免 http3 版本兼容性问题，而是使用特定的特性集合
run_test "Linter 检查 (cargo clippy)" cargo clippy --workspace --all-targets --features "$TEST_FEATURES" -- -D warnings

# ========== 编译检查（来自 ci.yml:test job）==========

# 3. 编译检查（对应 GitHub Actions: Check workspace）
# 在默认情况下不包含 http3（由于版本兼容性问题）
# 使用完整的特性集合，与 TEST_FEATURES 环变量相同
run_test "编译检查 (cargo check)" cargo check --workspace --features "$TEST_FEATURES"

# ========== 测试（来自 ci.yml:test job）==========

# 5. 库单元测试（对应 GitHub Actions: Test workspace with nextest）
# 首先尝试使用 nextest（更快），如果不可用则回退到 cargo test
if command -v cargo-nextest &> /dev/null; then
    run_test "单元测试 (cargo nextest --lib)" cargo nextest run --workspace --features "$TEST_FEATURES" --lib --no-fail-fast
else
    run_test "单元测试 (cargo test --lib)" cargo test --workspace --lib --features "$TEST_FEATURES"
fi

# 6. 集成测试（对应 GitHub Actions: Test workspace 测试完整套件）
# 使用 --skip examples 来排除编译示例（这些应该在单独的构建步骤中测试）
if command -v cargo-nextest &> /dev/null; then
    run_test "集成测试 (cargo nextest)" cargo nextest run --workspace --features "$TEST_FEATURES" --no-fail-fast
else
    run_test "集成测试 (cargo test)" cargo test --workspace --features "$TEST_FEATURES" --lib --tests
fi

# ========== 安全审计（来自 security-audit.yml）==========

# 6. cargo-deny 检查（对应 GitHub Actions: cargo deny check）
if command -v cargo-deny &> /dev/null; then
    run_test "安全审计 (cargo-deny)" cargo deny check advisories bans licenses sources
else
    echo -e "${YELLOW}⚠️  cargo-deny 未安装，跳过此检查${NC}"
    echo "  安装: cargo install cargo-deny"
    echo ""
fi

# ========== 构建检查（来自 ci.yml:build job，可选）==========

# 7. 构建发布版本（主要特性组合）
run_test "构建检查 (cargo build --release)" cargo build --workspace --features "$TEST_FEATURES" --release

# 总结
echo "=========================================="
echo "📊 本地检查总结"
echo "=========================================="
echo -e "${GREEN}✅ 通过: $PASSED${NC}"
if [ $FAILED -gt 0 ]; then
    echo -e "${RED}❌ 失败: $FAILED${NC}"
    echo ""
    echo -e "${RED}❌ 检查失败，请修复问题后再提交${NC}"
    echo ""
    echo "📌 提示："
    echo "  - 格式化:  cargo fmt --all"
    echo "  - Clippy:  cargo clippy --workspace --all-targets --all-features -D warnings"
    echo "  - 测试:    cargo test --workspace"
    echo ""
    exit 1
else
    echo -e "${GREEN}✅ 所有检查通过！${NC}"
    echo -e "${GREEN}✅ 符合 GitHub Actions 规则，可以安全提交代码${NC}"
    exit 0
fi
