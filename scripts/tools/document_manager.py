#!/usr/bin/env python3
"""
文档管理工具集统一入口
集成搜索、版本控制、质量检查等功能
"""

import sys
from pathlib import Path

# 添加脚本目录到Python路径
script_dir = Path(__file__).parent
sys.path.insert(0, str(script_dir))

from document_search import DocumentSearcher
from document_version_control import DocumentVersionControl
from maintenance.check_documentation import DocumentationChecker

class DocumentManager:
    """文档管理器主类"""
    
    def __init__(self, project_root: str = "."):
        self.project_root = Path(project_root)
        self.searcher = DocumentSearcher(project_root)
        self.version_control = DocumentVersionControl(project_root)
        self.checker = DocumentationChecker(project_root)
    
    def interactive_mode(self):
        """交互式模式"""
        print("📚 fingerprint-rust 文档管理系统")
        print("=" * 40)
        
        while True:
            print("\n请选择操作:")
            print("1. 搜索文档")
            print("2. 查看文档历史")
            print("3. 检查文档质量")
            print("4. 跟踪文档变更")
            print("5. 退出")
            
            choice = input("\n请输入选择 (1-5): ").strip()
            
            if choice == '1':
                self._search_documents()
            elif choice == '2':
                self._view_document_history()
            elif choice == '3':
                self._check_document_quality()
            elif choice == '4':
                self._track_document_changes()
            elif choice == '5':
                print("👋 再见!")
                break
            else:
                print("❌ 无效选择，请重新输入")
    
    def _search_documents(self):
        """搜索文档"""
        query = input("请输入搜索关键词: ").strip()
        if not query:
            print("❌ 查询不能为空")
            return
            
        limit = input("请输入结果数量限制 (默认10): ").strip()
        limit = int(limit) if limit.isdigit() else 10
        
        print(f"\n🔍 搜索 '{query}'...")
        results = self.searcher.search(query, limit)
        
        if results:
            print(f"\n找到 {len(results)} 个结果:")
            for i, doc in enumerate(results, 1):
                print(f"\n{i}. {doc.title}")
                print(f"   路径: {doc.path}")
                print(f"   分类: {doc.category}")
                print(f"   标签: {', '.join(doc.tags) if doc.tags else '无'}")
                print(f"   字数: {doc.word_count}")
                
                # 显示部分内容预览
                preview = doc.content[:200].replace('\n', ' ')
                if len(doc.content) > 200:
                    preview += "..."
                print(f"   预览: {preview}")
        else:
            print("❌ 未找到相关文档")
    
    def _view_document_history(self):
        """查看文档历史"""
        doc_path = input("请输入文档相对路径: ").strip()
        if not doc_path:
            print("❌ 路径不能为空")
            return
            
        history = self.version_control.get_document_history(doc_path)
        
        if history:
            print(f"\n{doc_path} 的版本历史:")
            print("-" * 50)
            for version in history:
                print(f"版本: {version.version_id}")
                print(f"时间: {version.timestamp.strftime('%Y-%m-%d %H:%M:%S')}")
                print(f"作者: {version.author}")
                print(f"说明: {version.commit_message}")
                print(f"大小: {version.content_length} 字节")
                if version.parent_version:
                    print(f"父版本: {version.parent_version}")
                print("-" * 30)
        else:
            print(f"❌ 未找到文档 {doc_path} 的历史记录")
    
    def _check_document_quality(self):
        """检查文档质量"""
        print("🔍 开始文档质量检查...")
        results = self.checker.run_all_checks()
        
        print(f"\n📊 检查结果摘要:")
        print(f"  缺失文档: {results['summary']['missing_documents']} 个")
        print(f"  质量问题: {results['summary']['quality_issues']} 个")
        
        if results['summary']['missing_documents'] == 0 and results['summary']['quality_issues'] == 0:
            print("🎉 所有文档检查通过!")
        else:
            print("⚠️  发现文档问题，请查看详细报告。")
    
    def _track_document_changes(self):
        """跟踪文档变更"""
        force = input("是否强制跟踪所有文档? (y/N): ").strip().lower() == 'y'
        print("🔍 开始跟踪文档变更...")
        count = self.version_control.track_changes(force=force)
        print(f"✅ 跟踪了 {count} 个文档变更")

def main():
    """主函数"""
    import argparse
    
    parser = argparse.ArgumentParser(description='文档管理工具集')
    parser.add_argument('--interactive', '-i', action='store_true', 
                       help='进入交互式模式')
    parser.add_argument('--project-root', '-p', default='.', 
                       help='项目根目录')
    
    # 子命令
    subparsers = parser.add_subparsers(dest='command', help='可用命令')
    
    # 搜索命令
    search_parser = subparsers.add_parser('search', help='搜索文档')
    search_parser.add_argument('query', help='搜索关键词')
    search_parser.add_argument('--limit', '-l', type=int, default=10, 
                              help='结果数量限制')
    
    # 历史命令
    history_parser = subparsers.add_parser('history', help='查看文档历史')
    history_parser.add_argument('document', help='文档路径')
    
    # 检查命令
    subparsers.add_parser('check', help='检查文档质量')
    
    # 跟踪命令
    track_parser = subparsers.add_parser('track', help='跟踪文档变更')
    track_parser.add_argument('--force', '-f', action='store_true', 
                             help='强制跟踪所有文档')
    
    args = parser.parse_args()
    
    manager = DocumentManager(args.project_root)
    
    if args.interactive or not args.command:
        manager.interactive_mode()
    elif args.command == 'search':
        results = manager.searcher.search(args.query, args.limit)
        if results:
            print(f"找到 {len(results)} 个结果:")
            for i, doc in enumerate(results, 1):
                print(f"{i}. {doc.path} - {doc.title}")
        else:
            print("未找到相关文档")
    elif args.command == 'history':
        history = manager.version_control.get_document_history(args.document)
        for version in history:
            print(f"{version.timestamp}: {version.commit_message}")
    elif args.command == 'check':
        manager.checker.run_all_checks()
    elif args.command == 'track':
        manager.version_control.track_changes(force=args.force)

if __name__ == "__main__":
    main()