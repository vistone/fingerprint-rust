#!/usr/bin/env python3
"""
文档质量自动化检查工具
用于检查项目文档的完整性、一致性和质量
"""

import os
import re
import json
from pathlib import Path
from typing import Dict, List, Set, Tuple
from datetime import datetime

class DocumentationChecker:
    def __init__(self, project_root: str = "."):
        self.project_root = Path(project_root)
        self.docs_dir = self.project_root / "docs"
        self.required_docs = {
            "核心文档": [
                "README.md",
                "docs/INDEX.md",
                "PROJECT_STRUCTURE.md",
                "CONTRIBUTING.md"
            ],
            "用户指南": [
                "docs/user-guides/getting-started.md",
                "docs/user-guides/fingerprint-guide.md",
                "docs/user-guides/api-usage.md"
            ],
            "开发者指南": [
                "docs/developer-guides/architecture.md",
                "docs/developer-guides/contributing.md"
            ]
        }
        
        self.quality_checks = [
            self.check_file_existence,
            self.check_links_validity,
            self.check_content_quality,
            self.check_naming_conventions,
            self.check_update_frequency
        ]

    def run_all_checks(self) -> Dict[str, any]:
        """运行所有文档检查"""
        results = {
            "timestamp": datetime.now().isoformat(),
            "summary": {},
            "details": {}
        }
        
        print("🔍 开始文档质量检查...")
        
        # 检查必需文档存在性
        existence_results = self.check_required_documents()
        results["details"]["existence"] = existence_results
        results["summary"]["missing_documents"] = len(existence_results.get("missing", []))
        
        # 检查文档质量
        quality_results = self.perform_quality_checks()
        results["details"]["quality"] = quality_results
        results["summary"]["quality_issues"] = sum(
            len(issues) for issues in quality_results.values()
        )
        
        # 生成报告
        self.generate_report(results)
        
        return results

    def check_required_documents(self) -> Dict[str, List[str]]:
        """检查必需文档是否存在"""
        missing = []
        present = []
        
        for category, docs in self.required_docs.items():
            for doc_path in docs:
                full_path = self.project_root / doc_path
                if full_path.exists():
                    present.append(doc_path)
                else:
                    missing.append(doc_path)
                    
        return {
            "present": present,
            "missing": missing
        }

    def perform_quality_checks(self) -> Dict[str, List[str]]:
        """执行文档质量检查"""
        issues = {
            "broken_links": [],
            "outdated_content": [],
            "poor_structure": [],
            "missing_metadata": []
        }
        
        # 遍历所有Markdown文件
        for md_file in self.project_root.rglob("*.md"):
            if "target/" in str(md_file) or ".git/" in str(md_file):
                continue
                
            try:
                content = md_file.read_text(encoding='utf-8')
                relative_path = md_file.relative_to(self.project_root)
                
                # 检查损坏的链接
                broken_links = self.find_broken_links(content, md_file)
                issues["broken_links"].extend([
                    f"{relative_path}: {link}" for link in broken_links
                ])
                
                # 检查内容质量问题
                quality_issues = self.analyze_content_quality(content, relative_path)
                issues["poor_structure"].extend(quality_issues)
                
                # 检查元数据
                if not self.has_proper_metadata(content):
                    issues["missing_metadata"].append(str(relative_path))
                    
            except Exception as e:
                issues["broken_links"].append(f"{md_file}: 读取错误 - {str(e)}")
        
        return issues

    def find_broken_links(self, content: str, file_path: Path) -> List[str]:
        """查找损坏的链接"""
        broken_links = []
        
        # 匹配Markdown链接 [text](url)
        link_pattern = r'\[([^\]]+)\]\(([^)]+)\)'
        links = re.findall(link_pattern, content)
        
        for text, url in links:
            # 跳过外部链接和锚点
            if url.startswith(('http://', 'https://', '#', 'mailto:')):
                continue
                
            # 检查相对链接
            if url.startswith('./') or url.startswith('../'):
                target_path = (file_path.parent / url).resolve()
            else:
                target_path = (self.project_root / url).resolve()
                
            if not target_path.exists():
                broken_links.append(f"损坏链接: {url}")
                
        return broken_links

    def analyze_content_quality(self, content: str, file_path: Path) -> List[str]:
        """分析内容质量"""
        issues = []
        
        # 检查标题结构
        headings = re.findall(r'^(#{1,6})\s+(.+)$', content, re.MULTILINE)
        if len(headings) == 0:
            issues.append("缺少标题结构")
            
        # 检查代码块
        code_blocks = content.count('```')
        if code_blocks % 2 != 0:
            issues.append("未闭合的代码块")
            
        # 检查列表格式
        list_items = re.findall(r'^(\s*)([-*+]|\d+\.)\s', content, re.MULTILINE)
        if list_items:
            # 检查嵌套列表的一致性
            pass
            
        return issues

    def has_proper_metadata(self, content: str) -> bool:
        """检查是否有适当的元数据"""
        # 检查是否包含更新日期
        if not re.search(r'最后更新[:\s]*20\d{2}', content):
            return False
            
        # 检查是否包含版本信息
        if not re.search(r'版本[:\s]*v\d+\.\d+', content):
            return False
            
        return True

    def check_file_existence(self, file_path: Path) -> bool:
        """检查文件是否存在"""
        return file_path.exists()

    def check_links_validity(self, content: str, file_path: Path) -> List[str]:
        """检查链接有效性"""
        return self.find_broken_links(content, file_path)

    def check_content_quality(self, content: str, file_path: Path) -> List[str]:
        """检查内容质量"""
        return self.analyze_content_quality(content, file_path)

    def check_naming_conventions(self, file_path: Path) -> bool:
        """检查命名约定"""
        filename = file_path.name.lower()
        # 检查是否使用英文命名
        return bool(re.match(r'^[a-z0-9_-]+\.md$', filename))

    def check_update_frequency(self, file_path: Path) -> str:
        """检查更新频率"""
        try:
            stat = file_path.stat()
            mtime = datetime.fromtimestamp(stat.st_mtime)
            days_since_update = (datetime.now() - mtime).days
            
            if days_since_update > 180:
                return "长期未更新"
            elif days_since_update > 90:
                return "较长时间未更新"
            else:
                return "近期更新"
        except:
            return "无法确定"

    def generate_report(self, results: Dict[str, any]):
        """生成检查报告"""
        report_file = self.project_root / "output" / "reports" / "documentation_quality_report.md"
        report_file.parent.mkdir(parents=True, exist_ok=True)
        
        with open(report_file, 'w', encoding='utf-8') as f:
            f.write("# 文档质量检查报告\n\n")
            f.write(f"**生成时间**: {results['timestamp']}\n\n")
            
            # 摘要
            f.write("## 📊 检查摘要\n\n")
            f.write(f"- 缺失文档: {results['summary']['missing_documents']} 个\n")
            f.write(f"- 质量问题: {results['summary']['quality_issues']} 个\n\n")
            
            # 详细结果
            f.write("## 📋 详细检查结果\n\n")
            
            # 存在性检查
            existence = results['details']['existence']
            f.write("### 📁 文档存在性检查\n\n")
            if existence['missing']:
                f.write("**缺失的文档**:\n")
                for doc in existence['missing']:
                    f.write(f"- `{doc}`\n")
                f.write("\n")
            
            # 质量检查
            quality = results['details']['quality']
            f.write("### 🔍 文档质量检查\n\n")
            
            for issue_type, issues in quality.items():
                if issues:
                    f.write(f"**{issue_type}**:\n")
                    for issue in issues[:10]:  # 限制显示数量
                        f.write(f"- {issue}\n")
                    if len(issues) > 10:
                        f.write(f"- ... 还有 {len(issues) - 10} 个问题\n")
                    f.write("\n")

        print(f"✅ 检查报告已生成: {report_file}")

def main():
    checker = DocumentationChecker()
    results = checker.run_all_checks()
    
    # 输出简要结果
    print(f"\n📋 检查完成!")
    print(f"📁 缺失文档: {results['summary']['missing_documents']} 个")
    print(f"🔍 质量问题: {results['summary']['quality_issues']} 个")
    
    if results['summary']['missing_documents'] == 0 and results['summary']['quality_issues'] == 0:
        print("🎉 所有文档检查通过!")
    else:
        print("⚠️  发现文档问题，请查看详细报告。")

if __name__ == "__main__":
    main()