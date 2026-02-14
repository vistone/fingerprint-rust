#!/bin/bash
# 提交前全面测试脚本
# 在提交代码前运行所有必要的测试和检查

echo "=========================================="
echo "🔍 提交前全面测试"
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

# 测试函数
run_test() {
    local test_name="$1"
    local test_command="$2"
    
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "🧪 测试: $test_name"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    
    # 执行测试命令
    if eval "$test_command" > /tmp/test_output.log 2>&1; then
        echo -e "${GREEN}✅ $test_name 通过${NC}"
        ((PASSED++))
        echo ""
        return 0
    else
        echo -e "${RED}❌ $test_name 失败${NC}"
        # 显示最后几行输出以便调试
        if [ -f /tmp/test_output.log ]; then
            echo "错误输出:"
            tail -15 /tmp/test_output.log | sed 's/^/  /'
        fi
        ((FAILED++))
        echo ""
        return 1
    fi
}

# 1. 代码格式化检查
run_test "代码格式化检查" "cargo fmt --all -- --check"

# 2. 文档规制检查
run_test "文档规制检查" "python3 scripts/verify_doc_pairs.py"

# 3. 编译检查
run_test "编译检查 (cargo check)" "cargo check --workspace"

# 4. Clippy 检查
run_test "Clippy 检查" "cargo clippy --workspace --all-targets --features 'rustls-tls,compression,http2,http3,connection-pool,dns' -- -D warnings"

# 5. 单元测试
run_test "单元测试" "cargo test --workspace --lib --quiet"

# 6. 集成测试（测试 tests/ 目录下的所有测试文件）
run_test "集成测试" "cargo test --workspace --quiet 2>&1 | grep -E '(test result|error)' || cargo test --workspace --quiet"

# 7. 安全审计 (cargo-deny)
if command -v cargo-deny &> /dev/null; then
    run_test "安全审计 (cargo-deny)" "cargo-deny check"
else
    echo -e "${YELLOW}⚠️  cargo-deny 未安装，跳过安全审计${NC}"
    echo ""
fi

# 总结
echo "=========================================="
echo "📊 测试总结"
echo "=========================================="
echo -e "${GREEN}✅ 通过: $PASSED${NC}"
if [ $FAILED -gt 0 ]; then
    echo -e "${RED}❌ 失败: $FAILED${NC}"
    echo ""
    echo -e "${RED}❌ 测试失败，请修复问题后再提交${NC}"
    exit 1
else
    echo -e "${GREEN}✅ 所有测试通过！可以安全提交代码${NC}"
    exit 0
fi
