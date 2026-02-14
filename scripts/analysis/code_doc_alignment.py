#!/usr/bin/env python3
"""
代码文档一致性分析工具
检查代码实现与文档描述的一致性，识别重复和不一致的内容
"""

import os
import re
import json
from pathlib import Path
from typing import Dict, List, Set, Tuple
from dataclasses import dataclass

@dataclass
class CodeModule:
    """代码模块信息"""
    name: str
    path: str
    description: str
    functions: List[str]
    structs: List[str]
    impl_blocks: List[str]

@dataclass
class Documentation:
    """文档信息"""
    path: str
    title: str
    content: str
    mentioned_modules: List[str]
    mentioned_functions: List[str]

class CodeDocAnalyzer:
    """代码文档一致性分析器"""
    
    def __init__(self, project_root: str = "."):
        self.project_root = Path(project_root)
        self.code_modules: Dict[str, CodeModule] = {}
        self.documents: List[Documentation] = []
        self.inconsistencies: List[Dict] = []
        self.duplicates: List[Dict] = []
        
    def analyze_project(self) -> Dict:
        """执行完整的代码文档一致性分析"""
        print("🔍 开始代码文档一致性分析...")
        
        # 1. 扫描代码模块
        self._scan_code_modules()
        
        # 2. 扫描文档
        self._scan_documents()
        
        # 3. 检查一致性
        self._check_consistency()
        
        # 4. 识别重复内容
        self._find_duplicates()
        
        # 5. 生成报告
        return self._generate_report()
    
    def _scan_code_modules(self):
        """扫描所有Rust代码模块"""
        print("📦 扫描代码模块...")
        
        crate_dirs = list(self.project_root.glob("crates/*/src"))
        example_files = list(self.project_root.glob("examples/*.rs"))
        
        # 扫描crate模块
        for crate_dir in crate_dirs:
            self._parse_crate_module(crate_dir)
        
        # 扫描示例文件
        for example_file in example_files:
            self._parse_example_file(example_file)
    
    def _parse_crate_module(self, crate_dir: Path):
        """解析crate模块"""
        lib_rs = crate_dir / "lib.rs"
        if not lib_rs.exists():
            return
            
        try:
            content = lib_rs.read_text(encoding='utf-8')
            module_name = crate_dir.parent.name
            
            # 提取模块描述（从文档注释）
            description = self._extract_module_description(content)
            
            # 提取函数和结构体
            functions = self._extract_functions(content)
            structs = self._extract_structs(content)
            impl_blocks = self._extract_impl_blocks(content)
            
            self.code_modules[module_name] = CodeModule(
                name=module_name,
                path=str(lib_rs.relative_to(self.project_root)),
                description=description,
                functions=functions,
                structs=structs,
                impl_blocks=impl_blocks
            )
            
        except Exception as e:
            print(f"警告: 无法解析 {crate_dir}: {e}")
    
    def _parse_example_file(self, example_file: Path):
        """解析示例文件"""
        try:
            content = example_file.read_text(encoding='utf-8')
            module_name = example_file.stem
            
            # 提取主要功能描述
            description = self._extract_example_description(content)
            functions = self._extract_functions(content)
            
            self.code_modules[f"example_{module_name}"] = CodeModule(
                name=f"example_{module_name}",
                path=str(example_file.relative_to(self.project_root)),
                description=description,
                functions=functions,
                structs=[],
                impl_blocks=[]
            )
            
        except Exception as e:
            print(f"警告: 无法解析示例 {example_file}: {e}")
    
    def _extract_module_description(self, content: str) -> str:
        """从代码中提取模块描述"""
        # 查找模块级文档注释
        module_doc_pattern = r'/\*!(.*?)\*/'
        match = re.search(module_doc_pattern, content, re.DOTALL)
        if match:
            return match.group(1).strip()
        
        # 查找 //! 注释
        line_doc_pattern = r'//!(.*?)$'
        matches = re.findall(line_doc_pattern, content, re.MULTILINE)
        if matches:
            return ' '.join(match.strip() for match in matches)
        
        return "未找到模块描述"
    
    def _extract_example_description(self, content: str) -> str:
        """从示例代码中提取描述"""
        # 查找main函数上方的注释
        main_pattern = r'(/\*.*?fn\s+main|mains*fn\s+main)'
        match = re.search(main_pattern, content, re.DOTALL)
        if match:
            # 提取前面的注释
            comment_pattern = r'/\*(.*?)\*/'
            comment_match = re.search(comment_pattern, content[:match.start()], re.DOTALL)
            if comment_match:
                return comment_match.group(1).strip()
        
        return "示例代码"
    
    def _extract_functions(self, content: str) -> List[str]:
        """提取函数名"""
        # 匹配pub fn声明
        fn_pattern = r'pub\s+fn\s+(\w+)'
        return re.findall(fn_pattern, content)
    
    def _extract_structs(self, content: str) -> List[str]:
        """提取结构体名"""
        struct_pattern = r'(?:pub\s+)?struct\s+(\w+)'
        return re.findall(struct_pattern, content)
    
    def _extract_impl_blocks(self, content: str) -> List[str]:
        """提取impl块"""
        impl_pattern = r'impl(?:\s+<.*?>)?\s+(\w+)'
        return re.findall(impl_pattern, content)
    
    def _scan_documents(self):
        """扫描所有文档文件"""
        print("📚 扫描文档文件...")
        
        # 查找所有Markdown文件
        md_files = list(self.project_root.rglob("*.md"))
        
        for md_file in md_files:
            if self._should_scan_document(md_file):
                self._parse_document(md_file)
    
    def _should_scan_document(self, file_path: Path) -> bool:
        """判断是否应该扫描此文档"""
        excluded_patterns = [
            "target/", ".git/", "vendor/", "venv/",
            "output/temp/", "output/logs/",
            "site-packages/"  # 排除Python包
        ]
        
        path_str = str(file_path)
        return not any(pattern in path_str for pattern in excluded_patterns)
    
    def _parse_document(self, md_file: Path):
        """解析文档文件"""
        try:
            content = md_file.read_text(encoding='utf-8')
            
            # 提取标题
            title_match = re.search(r'^#\s+(.+)$', content, re.MULTILINE)
            title = title_match.group(1) if title_match else md_file.stem
            
            # 提取提及的模块和函数
            mentioned_modules = self._extract_mentioned_modules(content)
            mentioned_functions = self._extract_mentioned_functions(content)
            
            self.documents.append(Documentation(
                path=str(md_file.relative_to(self.project_root)),
                title=title,
                content=content,
                mentioned_modules=mentioned_modules,
                mentioned_functions=mentioned_functions
            ))
            
        except Exception as e:
            print(f"警告: 无法解析文档 {md_file}: {e}")
    
    def _extract_mentioned_modules(self, content: str) -> List[str]:
        """从文档中提取提及的模块名"""
        modules = []
        # 匹配crate名称模式
        crate_pattern = r'(?:`)?(fingerprint-\w+)(?:`)'
        matches = re.findall(crate_pattern, content)
        modules.extend(matches)
        
        # 匹配示例文件名
        example_pattern = r'(?:`)?example_(\w+)(?:`)'
        matches = re.findall(example_pattern, content)
        modules.extend([f"example_{match}" for match in matches])
        
        return list(set(modules))  # 去重
    
    def _extract_mentioned_functions(self, content: str) -> List[str]:
        """从文档中提取提及的函数名"""
        # 匹配函数名模式
        fn_pattern = r'(?:`)?(\w+::\w+)(?:`)'
        return re.findall(fn_pattern, content)
    
    def _check_consistency(self):
        """检查代码与文档的一致性"""
        print("🔍 检查代码文档一致性...")
        
        for doc in self.documents:
            # 检查文档中提及的模块是否真实存在
            for module_name in doc.mentioned_modules:
                if module_name not in self.code_modules:
                    self.inconsistencies.append({
                        "type": "missing_module",
                        "document": doc.path,
                        "module": module_name,
                        "description": f"文档中提及的模块 '{module_name}' 在代码中未找到"
                    })
            
            # 检查文档描述与代码实现是否匹配
            self._check_module_descriptions(doc)
    
    def _check_module_descriptions(self, doc: Documentation):
        """检查模块描述的一致性"""
        for module_name in doc.mentioned_modules:
            if module_name in self.code_modules:
                code_module = self.code_modules[module_name]
                doc_description = self._get_module_description_from_doc(doc, module_name)
                
                if doc_description and code_module.description != "未找到模块描述":
                    # 简单的相似度检查
                    if not self._descriptions_similar(doc_description, code_module.description):
                        self.inconsistencies.append({
                            "type": "description_mismatch",
                            "document": doc.path,
                            "module": module_name,
                            "doc_description": doc_description,
                            "code_description": code_module.description
                        })
    
    def _get_module_description_from_doc(self, doc: Documentation, module_name: str) -> str:
        """从文档中提取特定模块的描述"""
        # 简化实现，实际应该更精确地匹配
        return f"关于{module_name}的描述"
    
    def _descriptions_similar(self, desc1: str, desc2: str) -> bool:
        """检查两个描述是否相似"""
        # 简单的关键词匹配
        keywords1 = set(re.findall(r'\w+', desc1.lower()))
        keywords2 = set(re.findall(r'\w+', desc2.lower()))
        
        if not keywords1 or not keywords2:
            return False
            
        intersection = len(keywords1.intersection(keywords2))
        union = len(keywords1.union(keywords2))
        
        return intersection / union > 0.3  # 30%相似度阈值
    
    def _find_duplicates(self):
        """识别重复内容"""
        print("🔄 识别重复内容...")
        
        # 按内容相似度分组文档
        content_groups = {}
        
        for i, doc1 in enumerate(self.documents):
            for j, doc2 in enumerate(self.documents[i+1:], i+1):
                similarity = self._calculate_similarity(doc1.content, doc2.content)
                if similarity > 0.8:  # 80%相似度认为是重复
                    group_key = tuple(sorted([doc1.path, doc2.path]))
                    if group_key not in content_groups:
                        content_groups[group_key] = {
                            "similarity": similarity,
                            "documents": [doc1.path, doc2.path],
                            "common_words": self._get_common_words(doc1.content, doc2.content)
                        }
        
        self.duplicates = list(content_groups.values())
    
    def _calculate_similarity(self, content1: str, content2: str) -> float:
        """计算两个文档的相似度"""
        # 移除Markdown格式符号
        clean1 = re.sub(r'[#*\-_`]', '', content1.lower())
        clean2 = re.sub(r'[#*\-_`]', '', content2.lower())
        
        words1 = set(clean1.split())
        words2 = set(clean2.split())
        
        if not words1 or not words2:
            return 0.0
            
        intersection = len(words1.intersection(words2))
        union = len(words1.union(words2))
        
        return intersection / union
    
    def _get_common_words(self, content1: str, content2: str) -> List[str]:
        """获取两个文档的共同词汇"""
        words1 = set(re.findall(r'\w+', content1.lower()))
        words2 = set(re.findall(r'\w+', content2.lower()))
        return list(words1.intersection(words2))
    
    def _generate_report(self) -> Dict:
        """生成分析报告"""
        report = {
            "summary": {
                "total_modules": len(self.code_modules),
                "total_documents": len(self.documents),
                "inconsistencies": len(self.inconsistencies),
                "duplicates": len(self.duplicates)
            },
            "modules": {name: {
                "path": module.path,
                "description": module.description,
                "functions": module.functions,
                "structs": module.structs
            } for name, module in self.code_modules.items()},
            "inconsistencies": self.inconsistencies,
            "duplicates": self.duplicates
        }
        
        # 保存报告
        report_file = self.project_root / "output" / "reports" / "code_doc_alignment_report.json"
        report_file.parent.mkdir(parents=True, exist_ok=True)
        
        with open(report_file, 'w', encoding='utf-8') as f:
            json.dump(report, f, indent=2, ensure_ascii=False)
        
        print(f"✅ 分析报告已生成: {report_file}")
        return report

def main():
    analyzer = CodeDocAnalyzer()
    report = analyzer.analyze_project()
    
    print("\n📊 分析结果摘要:")
    print(f"📦 代码模块数: {report['summary']['total_modules']}")
    print(f"📚 文档文件数: {report['summary']['total_documents']}")
    print(f"❌ 不一致项数: {report['summary']['inconsistencies']}")
    print(f"🔄 重复组数: {report['summary']['duplicates']}")
    
    if report['inconsistencies']:
        print("\n⚠️  发现的不一致项:")
        for item in report['inconsistencies'][:5]:
            print(f"  - {item['type']}: {item['document']} -> {item.get('module', '')}")
    
    if report['duplicates']:
        print("\n🔄 发现的重复内容:")
        for dup in report['duplicates'][:3]:
            print(f"  - 相似度 {dup['similarity']:.2f}: {dup['documents']}")

if __name__ == "__main__":
    main()