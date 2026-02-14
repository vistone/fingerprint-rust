#!/usr/bin/env python3
"""
Verify Chinese comment translation results
验证中文注释翻译结果
"""

import os
import re

def check_chinese_comments(directory: str) -> dict:
    """检查目录中仍存在的中文注释"""
    results = {
        'files_with_chinese': [],
        'total_chinese_lines': 0,
        'chinese_snippets': []
    }
    
    # 中文字符的Unicode范围
    chinese_pattern = re.compile(r'[\u4e00-\u9fff]')
    
    for root, dirs, files in os.walk(directory):
        for file in files:
            if file.endswith('.rs'):
                file_path = os.path.join(root, file)
                try:
                    with open(file_path, 'r', encoding='utf-8') as f:
                        for line_num, line in enumerate(f, 1):
                            if '//' in line and chinese_pattern.search(line):
                                results['files_with_chinese'].append(file_path)
                                results['total_chinese_lines'] += 1
                                results['chinese_snippets'].append({
                                    'file': file_path,
                                    'line': line_num,
                                    'content': line.strip()
                                })
                                break  # 每个文件只记录一次
                except Exception as e:
                    print(f"Error reading {file_path}: {e}")
    
    # 去重文件列表
    results['files_with_chinese'] = list(set(results['files_with_chinese']))
    
    return results

def main():
    """主函数"""
    target_dir = "crates"
    
    print("Checking for remaining Chinese comments...")
    print("=" * 50)
    
    results = check_chinese_comments(target_dir)
    
    print(f"Files with Chinese comments: {len(results['files_with_chinese'])}")
    print(f"Total lines with Chinese: {results['total_chinese_lines']}")
    
    if results['chinese_snippets']:
        print("\nSample Chinese comments found:")
        print("-" * 30)
        for snippet in results['chinese_snippets'][:10]:  # 显示前10个
            print(f"{snippet['file']}:{snippet['line']}")
            print(f"  {snippet['content'][:60]}{'...' if len(snippet['content']) > 60 else ''}")
    
    if len(results['files_with_chinese']) == 0:
        print("\n🎉 Excellent! No Chinese comments found. Translation complete!")
    else:
        print(f"\n⚠️  Found Chinese comments in {len(results['files_with_chinese'])} files")
        print("Files containing Chinese comments:")
        for file_path in sorted(results['files_with_chinese']):
            print(f"  - {file_path}")

if __name__ == "__main__":
    main()