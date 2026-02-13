#!/bin/bash
# Phase 9.4 系统验证脚本
# 验证系统是否准备好运行负载测试

set -e

# 颜色输出
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}=== Phase 9.4 系统验证 ===${NC}"
echo ""

PASSED=0
FAILED=0
WARNINGS=0

# 1. 检查Python版本
echo -e "${YELLOW}[1/8] 检查 Python 版本...${NC}"
if command -v python3 &> /dev/null; then
    PYTHON_VERSION=$(python3 --version)
    echo -e "${GREEN}✓ Python 已安装: $PYTHON_VERSION${NC}"
    PASSED=$((PASSED + 1))
else
    echo -e "${RED}✗ Python3 未安装${NC}"
    FAILED=$((FAILED + 1))
fi

# 2. 检查虚拟环境
echo -e "${YELLOW}[2/8] 检查 Python 虚拟环境...${NC}"
if [ -d "venv" ] || [ -d ".venv" ] || [ -n "$VIRTUAL_ENV" ]; then
    echo -e "${GREEN}✓ 虚拟环境已配置${NC}"
    PASSED=$((PASSED + 1))
else
    echo -e "${YELLOW}⚠ 未检测到虚拟环境，建议创建: python3 -m venv venv${NC}"
    WARNINGS=$((WARNINGS + 1))
fi

# 3. 检查Python依赖
echo -e "${YELLOW}[3/8] 检查 Python 依赖...${NC}"
if [ -f "fingerprint_api/requirements.txt" ]; then
    echo -e "${GREEN}✓ requirements.txt 已存在${NC}"
    
    # 检查关键依赖是否已安装
    MISSING_DEPS=()
    for pkg in fastapi uvicorn redis aioredis pytest; do
        if python3 -c "import $pkg" 2>/dev/null; then
            echo -e "  ${GREEN}✓ $pkg 已安装${NC}"
        else
            echo -e "  ${RED}✗ $pkg 未安装${NC}"
            MISSING_DEPS+=($pkg)
        fi
    done
    
    if [ ${#MISSING_DEPS[@]} -eq 0 ]; then
        PASSED=$((PASSED + 1))
    else
        echo -e "${YELLOW}⚠ 缺少依赖，运行: pip install -r fingerprint_api/requirements.txt${NC}"
        WARNINGS=$((WARNINGS + 1))
    fi
else
    echo -e "${RED}✗ requirements.txt 不存在${NC}"
    FAILED=$((FAILED + 1))
fi

# 4. 检查Redis
echo -e "${YELLOW}[4/8] 检查 Redis 服务...${NC}"
if command -v redis-cli &> /dev/null; then
    if redis-cli ping 2>/dev/null | grep -q PONG; then
        echo -e "${GREEN}✓ Redis 正在运行${NC}"
        PASSED=$((PASSED + 1))
    else
        echo -e "${RED}✗ Redis 未运行，启动: redis-server${NC}"
        FAILED=$((FAILED + 1))
    fi
else
    echo -e "${YELLOW}⚠ redis-cli 未安装，无法验证Redis状态${NC}"
    WARNINGS=$((WARNINGS + 1))
fi

# 5. 检查负载测试工具 - k6
echo -e "${YELLOW}[5/8] 检查 k6 负载测试工具...${NC}"
if command -v k6 &> /dev/null; then
    K6_VERSION=$(k6 version | head -1)
    echo -e "${GREEN}✓ k6 已安装: $K6_VERSION${NC}"
    PASSED=$((PASSED + 1))
else
    echo -e "${YELLOW}⚠ k6 未安装${NC}"
    echo -e "  安装方式:"
    echo -e "  - Ubuntu: sudo apt-get install k6"
    echo -e "  - macOS: brew install k6"
    echo -e "  - Docker: docker pull grafana/k6"
    WARNINGS=$((WARNINGS + 1))
fi

# 6. 检查负载测试工具 - Apache Bench
echo -e "${YELLOW}[6/8] 检查 Apache Bench (ab)...${NC}"
if command -v ab &> /dev/null; then
    AB_VERSION=$(ab -V 2>&1 | head -1)
    echo -e "${GREEN}✓ Apache Bench 已安装: $AB_VERSION${NC}"
    PASSED=$((PASSED + 1))
else
    echo -e "${YELLOW}⚠ Apache Bench 未安装${NC}"
    echo -e "  安装方式:"
    echo -e "  - Ubuntu: sudo apt-get install apache2-utils"
    echo -e "  - macOS: 已预装"
    WARNINGS=$((WARNINGS + 1))
fi

# 7. 检查负载测试脚本
echo -e "${YELLOW}[7/8] 检查负载测试脚本...${NC}"
if [ -f "tests/load/k6_rate_limiting_test.js" ] && [ -f "tests/load/ab_rate_limiting_test.sh" ]; then
    echo -e "${GREEN}✓ 负载测试脚本已就绪${NC}"
    if [ -x "tests/load/ab_rate_limiting_test.sh" ]; then
        echo -e "  ${GREEN}✓ ab_rate_limiting_test.sh 可执行${NC}"
    else
        echo -e "  ${YELLOW}⚠ ab_rate_limiting_test.sh 不可执行，运行: chmod +x tests/load/ab_rate_limiting_test.sh${NC}"
    fi
    PASSED=$((PASSED + 1))
else
    echo -e "${RED}✗ 负载测试脚本缺失${NC}"
    FAILED=$((FAILED + 1))
fi

# 8. 检查测试负载文件
echo -e "${YELLOW}[8/8] 检查测试负载文件...${NC}"
if [ -f "tests/load/payloads/identify.json" ] && [ -f "tests/load/payloads/compare.json" ]; then
    echo -e "${GREEN}✓ 测试负载文件已就绪${NC}"
    PASSED=$((PASSED + 1))
else
    echo -e "${RED}✗ 测试负载文件缺失${NC}"
    FAILED=$((FAILED + 1))
fi

# 总结
echo ""
echo -e "${BLUE}=== 验证结果 ===${NC}"
echo -e "${GREEN}通过: $PASSED${NC}"
echo -e "${YELLOW}警告: $WARNINGS${NC}"
echo -e "${RED}失败: $FAILED${NC}"
echo ""

# 显示下一步操作
if [ $FAILED -eq 0 ]; then
    echo -e "${GREEN}✅ 系统已准备好运行负载测试!${NC}"
    echo ""
    echo -e "${BLUE}下一步操作:${NC}"
    echo ""
    echo "1. 安装 Python 依赖 (如果未安装):"
    echo "   pip install -r fingerprint_api/requirements.txt"
    echo ""
    echo "2. 启动 FastAPI 应用:"
    echo "   uvicorn fingerprint_api.main:app --host 0.0.0.0 --port 8000 --reload"
    echo ""
    echo "3. 在另一个终端运行快速测试:"
    echo "   bash tests/load/ab_rate_limiting_test.sh"
    echo ""
    echo "4. 运行完整 k6 负载测试:"
    echo "   k6 run tests/load/k6_rate_limiting_test.js"
    echo ""
    echo "5. 查看测试结果:"
    echo "   ls -lh load_test_results/"
    echo ""
    
    exit 0
else
    echo -e "${RED}❌ 系统尚未就绪，请修复上述问题${NC}"
    echo ""
    echo -e "${BLUE}常见问题解决:${NC}"
    echo ""
    echo "1. 安装 Python 依赖:"
    echo "   pip install -r fingerprint_api/requirements.txt"
    echo ""
    echo "2. 启动 Redis:"
    echo "   redis-server"
    echo ""
    echo "3. 安装 k6 (Ubuntu):"
    echo "   sudo apt-get install k6"
    echo ""
    echo "4. 安装 Apache Bench (Ubuntu):"
    echo "   sudo apt-get install apache2-utils"
    echo ""
    
    exit 1
fi
