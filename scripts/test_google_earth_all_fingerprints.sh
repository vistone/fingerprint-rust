#!/bin/bash
# 全面测试 Google Earth API - 所有浏览器指纹
# 测试地址: https://kh.google.com/rt/earth/PlanetoidMetadata

set -e

echo "=========================================="
echo "Google Earth API 全面测试"
echo "地址: https://kh.google.com/rt/earth/PlanetoidMetadata"
echo "=========================================="
echo ""

echo "测试所有 66 个浏览器指纹..."
echo ""

# 测试 HTTP/1.1
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "1. 测试 HTTP/1.1"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
cargo test --test comprehensive_google_earth_test test_all_fingerprints_http1 \
    --features rustls-tls,http2,http3 \
    -- --ignored --nocapture 2>&1 | tee /tmp/google_earth_http1_test.log

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "2. 测试 HTTP/2"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
cargo test --test comprehensive_google_earth_test test_all_fingerprints_http2 \
    --features rustls-tls,http2,http3 \
    -- --ignored --nocapture 2>&1 | tee /tmp/google_earth_http2_test.log

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "3. 测试 HTTP/3"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
cargo test --test comprehensive_google_earth_test test_all_fingerprints_http3 \
    --features rustls-tls,http2,http3 \
    -- --ignored --nocapture 2>&1 | tee /tmp/google_earth_http3_test.log

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "4. 全面测试（所有协议）"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "⚠️  这个测试会运行较长时间（66 个指纹 × 3 个协议 = 198 个测试）"
echo ""
read -p "是否继续？(y/N) " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    cargo test --test comprehensive_google_earth_test test_all_fingerprints_all_protocols \
        --features rustls-tls,http2,http3 \
        -- --ignored --nocapture 2>&1 | tee /tmp/google_earth_all_test.log
fi

echo ""
echo "=========================================="
echo "测试完成！"
echo "日志文件保存在 /tmp/google_earth_*.log"
echo "=========================================="

