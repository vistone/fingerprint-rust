#!/usr/bin/env python3
"""
文档分析和去重工具
用于分析项目中的重复文档并提供合并建议
"""

import os
import re
from collections import defaultdict
from pathlib import Path
from typing import Dict, List, Set, Tuple

def find_markdown_files(root_dir: str = ".") -> List[Path]:
    """查找所有Markdown文件"""
    md_files = []
    exclude_dirs = {'.git', 'target', 'venv', 'vendor'}
    
    for root, dirs, files in os.walk(root_dir):
        # 跳过排除的目录
        dirs[:] = [d for d in dirs if d not in exclude_dirs]
        
        for file in files:
            if file.endswith('.md'):
                md_files.append(Path(root) / file)
    
    return md_files

def extract_keywords(content: str) -> Set[str]:
    """提取文档关键词"""
    # 移除代码块和链接
    content = re.sub(r'```.*?```', '', content, flags=re.DOTALL)
    content = re.sub(r'\[.*?\]\(.*?\)', '', content)
    
    # 提取重要词汇
    words = re.findall(r'\b(?:Phase|API|Gateway|Rust|Python|TLS|HTTP|fingerprint|browser)\b', content, re.IGNORECASE)
    return set(words)

def calculate_similarity(doc1_content: str, doc2_content: str) -> float:
    """计算两文档的相似度"""
    keywords1 = extract_keywords(doc1_content)
    keywords2 = extract_keywords(doc2_content)
    
    if not keywords1 and not keywords2:
        return 0.0
    
    intersection = len(keywords1.intersection(keywords2))
    union = len(keywords1.union(keywords2))
    
    return intersection / union if union > 0 else 0.0

def analyze_document_groups(md_files: List[Path]) -> Dict[str, List[Tuple[Path, float]]]:
    """分析文档分组"""
    # 按主题分组
    groups = defaultdict(list)
    
    phase_pattern = re.compile(r'Phase\s*[0-9.]+', re.IGNORECASE)
    api_pattern = re.compile(r'(?:API|Gateway)', re.IGNORECASE)
    architecture_pattern = re.compile(r'(?:Architecture|Design)', re.IGNORECASE)
    
    for file_path in md_files:
        try:
            with open(file_path, 'r', encoding='utf-8') as f:
                content = f.read()
        except Exception as e:
            print(f"警告: 无法读取 {file_path}: {e}")
            continue
        
        # 确定文档主题
        content_lower = content.lower()
        
        if phase_pattern.search(content):
            phase_match = phase_pattern.search(content)
            phase_num = phase_match.group().replace(' ', '')
            groups[f"Phase_{phase_num}"].append((file_path, len(content)))
        elif api_pattern.search(content) and 'gateway' in content_lower:
            groups["API_Gateway"].append((file_path, len(content)))
        elif architecture_pattern.search(content):
            groups["Architecture"].append((file_path, len(content)))
        elif 'readme' in file_path.name.lower():
            groups["README"].append((file_path, len(content)))
        else:
            # 按文件名分组
            name_parts = file_path.stem.split('_')
            if len(name_parts) > 1:
                group_key = name_parts[0].capitalize()
                groups[group_key].append((file_path, len(content)))
            else:
                groups["Other"].append((file_path, len(content)))
    
    return groups

def find_similar_documents(md_files: List[Path], threshold: float = 0.7) -> List[Tuple[str, List[Path]]]:
    """查找相似文档"""
    similar_groups = []
    
    # 读取所有文档内容
    doc_contents = {}
    for file_path in md_files:
        try:
            with open(file_path, 'r', encoding='utf-8') as f:
                doc_contents[file_path] = f.read()
        except Exception:
            continue
    
    # 比较文档相似度
    processed = set()
    
    for i, file1 in enumerate(md_files):
        if file1 in processed:
            continue
            
        similar_group = [file1]
        processed.add(file1)
        
        for file2 in md_files[i+1:]:
            if file2 in processed:
                continue
                
            if file1 in doc_contents and file2 in doc_contents:
                similarity = calculate_similarity(doc_contents[file1], doc_contents[file2])
                
                if similarity >= threshold:
                    similar_group.append(file2)
                    processed.add(file2)
        
        if len(similar_group) > 1:
            group_name = f"Similar_Group_{len(similar_groups) + 1}"
            similar_groups.append((group_name, similar_group))
    
    return similar_groups

def generate_report(groups: Dict[str, List[Tuple[Path, float]]], 
                   similar_docs: List[Tuple[str, List[Path]]]):
    """生成分析报告"""
    
    print("=" * 80)
    print("📊 项目文档分析报告")
    print("=" * 80)
    print()
    
    # 按组统计
    print("📂 文档分组统计:")
    print("-" * 40)
    total_docs = 0
    
    for group_name, files in sorted(groups.items()):
        count = len(files)
        total_docs += count
        avg_size = sum(size for _, size in files) // count if count > 0 else 0
        
        print(f"{group_name:20} | {count:3} 个文件 | 平均大小: {avg_size:,} 字符")
        
        # 显示前3个文件
        for file_path, size in sorted(files, key=lambda x: x[1], reverse=True)[:3]:
            rel_path = file_path.relative_to('.')
            print(f"  - {rel_path} ({size:,} 字符)")
        print()
    
    print(f"总计: {total_docs} 个文档文件")
    print()
    
    # 相似文档分析
    if similar_docs:
        print("🔄 发现相似文档组:")
        print("-" * 40)
        
        for group_name, files in similar_docs:
            print(f"\n{group_name}:")
            for file_path in files:
                rel_path = file_path.relative_to('.')
                print(f"  • {rel_path}")
            
            # 显示合并建议
            print("  建议操作:")
            print("  1. 保留内容最完整的一个")
            print("  2. 将其他文档的重要信息整合进去")
            print("  3. 在保留文档中添加指向已删除文档的引用")
    else:
        print("✅ 未发现高度相似的文档")
    
    print()
    print("=" * 80)

def main():
    """主函数"""
    print("🔍 开始分析项目文档...")
    
    # 查找所有Markdown文件
    md_files = find_markdown_files()
    print(f"找到 {len(md_files)} 个Markdown文件")
    
    # 分析文档分组
    groups = analyze_document_groups(md_files)
    
    # 查找相似文档
    similar_docs = find_similar_documents(md_files, threshold=0.6)
    
    # 生成报告
    generate_report(groups, similar_docs)
    
    # 保存详细报告
    report_file = "DOCUMENT_ANALYSIS_REPORT.md"
    print(f"\n📄 详细报告已保存到: {report_file}")

if __name__ == "__main__":
    main()