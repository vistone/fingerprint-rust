#!/bin/bash

# DNS 模块测试脚本
# 测试 DNS 预解析库的功能

set -e

echo "🔍 DNS 模块测试脚本"
echo "===================="
echo ""

# 颜色定义
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 功能标志
FEATURES="dns,rustls-tls"

# 测试计数器
TESTS_PASSED=0
TESTS_FAILED=0

# 测试函数
run_test() {
    local test_name=$1
    local test_command=$2
    
    echo -e "${BLUE}📡 测试: ${test_name}${NC}"
    echo "----------------------------------------"
    
    if eval "$test_command" 2>&1; then
        echo -e "${GREEN}✅ ${test_name} - 通过${NC}"
        ((TESTS_PASSED++))
        return 0
    else
        echo -e "${RED}❌ ${test_name} - 失败${NC}"
        ((TESTS_FAILED++))
        return 1
    fi
}

# 测试 1: IPInfo.io 集成
run_test "IPInfo.io 集成" \
    "cargo run --example test_ipinfo --features ${FEATURES}"

echo ""

# 测试 2: DNS 服务器收集器（从 public-dns.info 获取）
run_test "DNS 服务器收集器 (public-dns.info)" \
    "cargo run --example test_collector_only --features ${FEATURES}"

echo ""

# 测试 3: 单元测试
echo -e "${BLUE}📡 测试: DNS 模块单元测试${NC}"
echo "----------------------------------------"
if cargo test --features ${FEATURES} --lib dns 2>&1; then
    echo -e "${GREEN}✅ DNS 模块单元测试 - 通过${NC}"
    ((TESTS_PASSED++))
else
    echo -e "${RED}❌ DNS 模块单元测试 - 失败${NC}"
    ((TESTS_FAILED++))
fi

echo ""

# 测试 4: 编译检查
echo -e "${BLUE}📡 测试: 编译检查${NC}"
echo "----------------------------------------"
if cargo check --features ${FEATURES} 2>&1 | grep -q "Finished"; then
    echo -e "${GREEN}✅ 编译检查 - 通过${NC}"
    ((TESTS_PASSED++))
else
    echo -e "${YELLOW}⚠️  编译检查 - 有警告或错误${NC}"
    ((TESTS_FAILED++))
fi

echo ""
echo "========================================="
echo -e "${BLUE}📊 测试总结${NC}"
echo "========================================="
echo -e "通过: ${GREEN}${TESTS_PASSED}${NC}"
echo -e "失败: ${RED}${TESTS_FAILED}${NC}"
echo ""

if [ $TESTS_FAILED -eq 0 ]; then
    echo -e "${GREEN}✅ 所有测试通过！${NC}"
    exit 0
else
    echo -e "${RED}❌ 有测试失败${NC}"
    exit 1
fi
