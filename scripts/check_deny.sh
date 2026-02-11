#!/bin/bash
# cargo-deny 检查脚本
# 强制使用支持 CVSS 4.0 的 cargo-deny 版本，并执行完整检查

set -e

echo "=== 运行 cargo-deny 检查 ==="
echo ""

# Ensure cargo-deny supports CVSS 4.0 (>= 0.18)
REQUIRED_VERSION="0.18.0"
if command -v cargo-deny &> /dev/null; then
    DENY_VERSION=$(cargo-deny --version | awk '{print $2}')
    if [ "$(printf '%s\n' "$REQUIRED_VERSION" "$DENY_VERSION" | sort -V | head -n1)" != "$REQUIRED_VERSION" ]; then
        echo "⚠️  cargo-deny $DENY_VERSION detected, upgrading to >= $REQUIRED_VERSION..."
        cargo install cargo-deny --version 0.18.1 --locked
    fi
else
    echo "⚠️  cargo-deny not found, installing 0.18.1..."
    cargo install cargo-deny --version 0.18.1 --locked
fi

# Run full checks (do not skip advisories)
cargo deny check advisories bans licenses sources

echo ""
echo "✅ cargo-deny 完整检查通过！"
