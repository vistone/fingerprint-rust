#!/bin/bash

echo "检查剩余的中文注释..."
echo "================================"

# 搜索包含中文字符的文件
find crates/ -name "*.rs" -exec grep -l $'[\\u4e00-\\u9fff]' {} \; 2>/dev/null | while read file; do
    echo "发现中文注释的文件: $file"
    # 显示具体的中文行
    grep -n $'[\\u4e00-\\u9fff]' "$file" 2>/dev/null | head -3
    echo "---"
done > /tmp/chinese_comments.txt

# 统计结果
if [ -f /tmp/chinese_comments.txt ]; then
    file_count=$(grep "发现中文注释的文件:" /tmp/chinese_comments.txt | wc -l)
    echo "总共发现 $file_count 个文件包含中文注释"
    
    if [ $file_count -eq 0 ]; then
        echo "✅ 没有发现中文注释，翻译完成！"
    else
        echo "❌ 仍有 $file_count 个文件需要处理"
        cat /tmp/chinese_comments.txt
    fi
else
    echo "检查完成，没有发现中文注释"
fi