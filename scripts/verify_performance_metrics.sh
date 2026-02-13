#!/bin/bash
# 性能指标验证脚本
# 自动验证文档中声称的性能数据

echo "=========================================="
echo "🚀 性能指标验证测试"
echo "=========================================="
echo ""

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 测试配置
TEST_URL="https://httpbin.org/get"
ITERATIONS=20
TIMEOUT=30

# 性能指标标准（来自文档）
EXPECTED_HTTP3_TIME=40.3  # ms
EXPECTED_HTTP1_TIME=44.4  # ms  
EXPECTED_HTTP2_TIME=48.0  # ms
SUCCESS_RATE_THRESHOLD=100  # %

# 结果收集数组
declare -a HTTP1_TIMES
declare -a HTTP2_TIMES  
declare -a HTTP3_TIMES

# 测试函数
test_protocol_performance() {
    local protocol=$1
    local prefer_http3=$2
    local prefer_http2=$3
    local times_array=$4
    
    echo "🧪 测试 ${protocol} 性能..."
    
    for i in $(seq 1 $ITERATIONS); do
        # 使用超时命令防止hang住
        local start_time=$(date +%s%3N)  # 毫秒时间戳
        
        # 执行HTTP请求测试
        timeout ${TIMEOUT}s cargo run --example test_http_performance \
            -- "$TEST_URL" "$prefer_http3" "$prefer_http2" > /tmp/perf_test_$$.log 2>&1
        
        local exit_code=$?
        local end_time=$(date +%s%3N)
        local duration=$((end_time - start_time))
        
        if [ $exit_code -eq 0 ] && [ $duration -lt $((TIMEOUT * 1000)) ]; then
            echo -e "${GREEN}✓${NC} ${protocol} 请求 #$i: ${duration}ms"
            eval "$times_array+=(\$duration)"
        else
            echo -e "${RED}✗${NC} ${protocol} 请求 #$i: 失败或超时 (${duration}ms)"
        fi
        
        sleep 0.1  # 避免过于频繁的请求
    done
    
    echo ""
}

# 计算统计数据
calculate_stats() {
    local times_array=("$@")
    local sum=0
    local count=${#times_array[@]}
    
    if [ $count -eq 0 ]; then
        echo "0 0 0 0"  # 如果没有有效数据，返回零值
        return
    fi
    
    # 计算总和和最小值
    local min=${times_array[0]}
    local max=${times_array[0]}
    
    for time in "${times_array[@]}"; do
        sum=$((sum + time))
        if [ $time -lt $min ]; then
            min=$time
        fi
        if [ $time -gt $max ]; then
            max=$time
        fi
    done
    
    # 计算平均值
    local avg=$((sum / count))
    
    # 计算中位数
    local sorted_times=($(printf '%s\n' "${times_array[@]}" | sort -n))
    local median
    if [ $((count % 2)) -eq 1 ]; then
        median=${sorted_times[$((count / 2))]}
    else
        local mid1=${sorted_times[$((count / 2 - 1))]}
        local mid2=${sorted_times[$((count / 2))]}
        median=$(((mid1 + mid2) / 2))
    fi
    
    echo "$avg $min $max $median"
}

# 验证性能指标
verify_performance() {
    local protocol=$1
    local actual_avg=$2
    local expected_avg=$3
    local tolerance=5  # 5ms容差
    
    local lower_bound=$((expected_avg - tolerance))
    local upper_bound=$((expected_avg + tolerance))
    
    if [ $actual_avg -le $upper_bound ] && [ $actual_avg -ge $lower_bound ]; then
        echo -e "${GREEN}✅ ${protocol} 性能达标: ${actual_avg}ms (期望: ${expected_avg}±${tolerance}ms)${NC}"
        return 0
    else
        echo -e "${RED}❌ ${protocol} 性能未达标: ${actual_avg}ms (期望: ${expected_avg}±${tolerance}ms)${NC}"
        return 1
    fi
}

# 主测试流程
echo "📋 测试配置:"
echo "   测试URL: $TEST_URL"
echo "   迭代次数: $ITERATIONS"
echo "   超时设置: ${TIMEOUT}s"
echo ""

# 创建测试示例程序
cat > examples/test_http_performance.rs << 'EOF'
use fingerprint::{HttpClient, HttpClientConfig};
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 4 {
        eprintln!("Usage: {} <url> <prefer_http3> <prefer_http2>", args[0]);
        std::process::exit(1);
    }
    
    let url = &args[1];
    let prefer_http3 = args[2] == "true";
    let prefer_http2 = args[3] == "true";
    
    let config = HttpClientConfig {
        user_agent: "Mozilla/5.0 (X11; Linux x86_64) Chrome/133.0.0.0".to_string(),
        prefer_http3,
        prefer_http2,
        ..Default::default()
    };
    
    let client = HttpClient::new(config)?;
    let response = client.get(url)?;
    
    println!("Status: {}", response.status_code);
    println!("Version: {:?}", response.http_version);
    
    Ok(())
}
EOF

# 编译测试程序
echo "🔨 编译测试程序..."
cargo build --example test_http_performance --quiet
if [ $? -ne 0 ]; then
    echo -e "${RED}❌ 编译失败${NC}"
    exit 1
fi

# 执行性能测试
test_protocol_performance "HTTP/3" "true" "true" "HTTP3_TIMES"
test_protocol_performance "HTTP/1.1" "false" "false" "HTTP1_TIMES"  
test_protocol_performance "HTTP/2" "false" "true" "HTTP2_TIMES"

# 计算并验证结果
echo "📊 性能测试结果分析:"
echo "=========================================="

# HTTP/3结果
if [ ${#HTTP3_TIMES[@]} -gt 0 ]; then
    read http3_avg http3_min http3_max http3_median <<< $(calculate_stats "${HTTP3_TIMES[@]}")
    echo -e "${BLUE}HTTP/3:${NC}"
    echo "   平均响应时间: ${http3_avg}ms"
    echo "   最快响应时间: ${http3_min}ms"  
    echo "   最慢响应时间: ${http3_max}ms"
    echo "   中位数响应时间: ${http3_median}ms"
    echo "   成功率: $((100 * ${#HTTP3_TIMES[@]} / ITERATIONS))%"
    verify_performance "HTTP/3" $http3_avg $EXPECTED_HTTP3_TIME
    http3_result=$?
else
    echo -e "${RED}HTTP/3: 无有效测试数据${NC}"
    http3_result=1
fi

echo ""

# HTTP/1.1结果
if [ ${#HTTP1_TIMES[@]} -gt 0 ]; then
    read http1_avg http1_min http1_max http1_median <<< $(calculate_stats "${HTTP1_TIMES[@]}")
    echo -e "${BLUE}HTTP/1.1:${NC}"
    echo "   平均响应时间: ${http1_avg}ms"
    echo "   最快响应时间: ${http1_min}ms"
    echo "   最慢响应时间: ${http1_max}ms"  
    echo "   中位数响应时间: ${http1_median}ms"
    echo "   成功率: $((100 * ${#HTTP1_TIMES[@]} / ITERATIONS))%"
    verify_performance "HTTP/1.1" $http1_avg $EXPECTED_HTTP1_TIME
    http1_result=$?
else
    echo -e "${RED}HTTP/1.1: 无有效测试数据${NC}"
    http1_result=1
fi

echo ""

# HTTP/2结果
if [ ${#HTTP2_TIMES[@]} -gt 0 ]; then
    read http2_avg http2_min http2_max http2_median <<< $(calculate_stats "${HTTP2_TIMES[@]}")
    echo -e "${BLUE}HTTP/2:${NC}"
    echo "   平均响应时间: ${http2_avg}ms"
    echo "   最快响应时间: ${http2_min}ms"
    echo "   最慢响应时间: ${http2_max}ms"
    echo "   中位数响应时间: ${http2_median}ms"
    echo "   成功率: $((100 * ${#HTTP2_TIMES[@]} / ITERATIONS))%"
    verify_performance "HTTP/2" $http2_avg $EXPECTED_HTTP2_TIME
    http2_result=$?
else
    echo -e "${RED}HTTP/2: 无有效测试数据${NC}"
    http2_result=1
fi

echo ""
echo "=========================================="

# 总体评估
passed_tests=0
total_tests=3

[ $http3_result -eq 0 ] && ((passed_tests++))
[ $http1_result -eq 0 ] && ((passed_tests++))  
[ $http2_result -eq 0 ] && ((passed_tests++))

echo "🎯 性能验证总结:"
echo "   通过测试: $passed_tests/$total_tests"
echo "   整体通过率: $((100 * passed_tests / total_tests))%"

if [ $passed_tests -eq $total_tests ]; then
    echo -e "${GREEN}🎉 所有性能指标验证通过！${NC}"
    exit 0
else
    echo -e "${YELLOW}⚠️  部分性能指标未达标，请检查网络环境或调整期望值${NC}"
    exit 1
fi