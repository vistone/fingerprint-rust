#!/bin/bash
# 修复 advisory 数据库中的 CVSS 4.0 问题
# cargo-deny 0.17.0 不支持 CVSS 4.0，需要修复或删除有问题的文件

ADVISORY_DB_DIR="$HOME/.cargo/advisory-dbs/github.com-9b36585d9d99f7b3/crates/deno"
ADVISORY_FILE="$ADVISORY_DB_DIR/RUSTSEC-0000-0000.md"

if [ -f "$ADVISORY_FILE" ]; then
    # 使用 Python 删除包含 CVSS 4.0 的行（更可靠）
    python3 << PYEOF
import re
import sys

file_path = "$ADVISORY_FILE"
try:
    with open(file_path, 'r') as f:
        content = f.read()
    
    # 删除包含 CVSS:4.0 的整行
    lines = content.split('\n')
    lines = [line for line in lines if 'CVSS:4.0' not in line]
    
    with open(file_path, 'w') as f:
        f.write('\n'.join(lines))
    
    print(f"✅ 已修复 advisory 文件: {file_path}")
except Exception as e:
    print(f"⚠️  修复失败: {e}", file=sys.stderr)
    sys.exit(1)
PYEOF
fi
