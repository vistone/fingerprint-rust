#!/usr/bin/env python3
"""
批量为文档添加元数据脚本
自动为缺少更新时间和版本信息的文档添加标准元数据
"""

import os
import re
from pathlib import Path
from datetime import datetime

def add_metadata_to_file(file_path: Path):
    """为单个文件添加元数据"""
    try:
        content = file_path.read_text(encoding='utf-8')
        
        # 检查是否已有元数据
        if re.search(r'(最后更新|版本|Last updated|Version)', content):
            return False, "已有元数据"
        
        # 查找第一个标题
        title_match = re.search(r'^#\s+(.+)$', content, re.MULTILINE)
        if not title_match:
            return False, "未找到标题"
        
        title = title_match.group(1).strip()
        
        # 创建元数据块
        metadata = f"""**版本**: v1.0  
**最后更新**: {datetime.now().strftime('%Y-%m-%d')}  
**文档类型**: 技术文档

---

"""
        
        # 在标题后插入元数据
        new_content = content.replace(
            f"# {title}", 
            f"# {title}\n\n{metadata}", 
            1
        )
        
        # 写入文件
        file_path.write_text(new_content, encoding='utf-8')
        return True, f"已添加元数据到 {title}"
        
    except Exception as e:
        return False, f"处理失败: {str(e)}"

def main():
    """主函数"""
    project_root = Path(".")
    processed_count = 0
    skipped_count = 0
    error_count = 0
    
    print("🔍 开始批量添加文档元数据...")
    
    # 查找所有Markdown文件
    for md_file in project_root.rglob("*.md"):
        # 跳过不需要处理的目录
        if any(skip_dir in str(md_file) for skip_dir in [
            "target/", ".git/", "vendor/", "venv/", 
            "output/temp/", "output/logs/"
        ]):
            continue
            
        success, message = add_metadata_to_file(md_file)
        
        if success:
            print(f"✅ {md_file}: {message}")
            processed_count += 1
        elif "已有元数据" in message:
            print(f"ℹ️  {md_file}: {message}")
            skipped_count += 1
        else:
            print(f"❌ {md_file}: {message}")
            error_count += 1
    
    print(f"\n📊 处理完成!")
    print(f"✅ 处理成功: {processed_count} 个文件")
    print(f"ℹ️  已跳过: {skipped_count} 个文件")
    print(f"❌ 处理失败: {error_count} 个文件")

if __name__ == "__main__":
    main()