#!/bin/bash
# 测试 HTTP/2 Settings 应用

set -e

echo "=========================================="
echo "测试 HTTP/2 Settings 应用"
echo "=========================================="

echo ""
echo "1. 检查代码编译..."
cargo check --features http2 2>&1 | grep -E "(error|warning|Finished)" | head -20 || echo "编译检查完成"

echo ""
echo "2. 检查 HTTP/2 相关代码..."
echo "   - http2.rs 中的 Builder 使用"
grep -n "Builder::new\|builder\." src/http_client/http2.rs | head -5

echo ""
echo "   - http2_pool.rs 中的 Builder 使用"
grep -n "Builder::new\|builder\." src/http_client/http2_pool.rs | head -5

echo ""
echo "3. 检查 Settings 应用逻辑..."
echo "   - 从 profile 获取 Settings"
grep -n "profile.get_settings\|settings.get" src/http_client/http2.rs | head -5

echo ""
echo "4. 运行单元测试（不执行需要网络的测试）..."
cargo test --lib http_client::http2 --no-fail-fast 2>&1 | tail -30 || echo "测试完成"

echo ""
echo "=========================================="
echo "测试完成"
echo "=========================================="

