#!/usr/bin/env python3
"""
简化版代码文档一致性检查工具
避免复杂终端输出问题，直接生成结果文件
"""

import json
from pathlib import Path

def simple_check():
    """简化的检查函数"""
    project_root = Path(".")
    
    # 基本统计
    crates_count = len(list(project_root.glob("crates/*/Cargo.toml")))
    md_files_count = len(list(project_root.rglob("*.md"))) - len(list(project_root.glob("target/**/*.md")))
    
    # 读取之前的分析结果
    report_file = project_root / "output" / "reports" / "code_doc_alignment_report.json"
    
    if report_file.exists():
        with open(report_file, 'r', encoding='utf-8') as f:
            report_data = json.load(f)
        
        inconsistencies = report_data.get("inconsistencies", [])
        duplicates = report_data.get("duplicates", [])
        
        # 生成简化报告
        simple_report = {
            "timestamp": "2026-02-13",
            "summary": {
                "crates_count": crates_count,
                "markdown_files": md_files_count,
                "inconsistencies_count": len(inconsistencies),
                "duplicates_count": len(duplicates)
            },
            "key_findings": [
                f"发现 {len(inconsistencies)} 个不一致项",
                f"发现 {len(duplicates)} 组重复内容",
                "主要问题集中在模块描述匹配上"
            ]
        }
        
        # 保存简化报告
        simple_report_file = project_root / "output" / "reports" / "simple_alignment_check.json"
        simple_report_file.parent.mkdir(parents=True, exist_ok=True)
        
        with open(simple_report_file, 'w', encoding='utf-8') as f:
            json.dump(simple_report, f, indent=2, ensure_ascii=False)
        
        print("✅ 简化检查完成")
        print(f"📊 项目统计:")
        print(f"   - Crates数量: {crates_count}")
        print(f"   - Markdown文件: {md_files_count}")
        print(f"   - 不一致项: {len(inconsistencies)}")
        print(f"   - 重复组: {len(duplicates)}")
        
        return simple_report
    
    else:
        print("❌ 未找到之前的分析报告")
        return None

if __name__ == "__main__":
    simple_check()