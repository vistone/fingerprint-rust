#!/usr/bin/env python3
"""
智能文档检索系统
提供全文搜索、语义搜索和智能推荐功能
"""

import os
import re
import json
import pickle
from pathlib import Path
from typing import Dict, List, Tuple, Optional
from dataclasses import dataclass
from datetime import datetime
import sqlite3

@dataclass
class DocumentInfo:
    """文档信息数据类"""
    path: str
    title: str
    content: str
    word_count: int
    last_modified: datetime
    tags: List[str]
    category: str
    similarity_score: float = 0.0

class DocumentIndexer:
    """文档索引器"""
    
    def __init__(self, project_root: str = "."):
        self.project_root = Path(project_root)
        self.index_db = self.project_root / "output" / "data" / "document_index.db"
        self._init_database()
        
    def _init_database(self):
        """初始化数据库"""
        self.index_db.parent.mkdir(parents=True, exist_ok=True)
        
        with sqlite3.connect(self.index_db) as conn:
            conn.execute('''
                CREATE TABLE IF NOT EXISTS documents (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    path TEXT UNIQUE,
                    title TEXT,
                    content TEXT,
                    word_count INTEGER,
                    last_modified TIMESTAMP,
                    tags TEXT,
                    category TEXT,
                    indexed_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
                )
            ''')
            
            conn.execute('''
                CREATE VIRTUAL TABLE IF NOT EXISTS document_search 
                USING fts5(path, title, content, tags, category)
            ''')
            
    def index_documents(self):
        """索引所有文档"""
        print(".CreateIndexing documents...")
        
        documents = self._scan_documents()
        indexed_count = 0
        
        with sqlite3.connect(self.index_db) as conn:
            for doc_info in documents:
                try:
                    # 插入或更新文档信息
                    conn.execute('''
                        INSERT OR REPLACE INTO documents 
                        (path, title, content, word_count, last_modified, tags, category)
                        VALUES (?, ?, ?, ?, ?, ?, ?)
                    ''', (
                        doc_info.path,
                        doc_info.title,
                        doc_info.content,
                        doc_info.word_count,
                        doc_info.last_modified.isoformat(),
                        ','.join(doc_info.tags),
                        doc_info.category
                    ))
                    
                    # 更新全文搜索索引
                    conn.execute('''
                        INSERT OR REPLACE INTO document_search 
                        (path, title, content, tags, category)
                        VALUES (?, ?, ?, ?, ?)
                    ''', (
                        doc_info.path,
                        doc_info.title,
                        doc_info.content,
                        ' '.join(doc_info.tags),
                        doc_info.category
                    ))
                    
                    indexed_count += 1
                    
                except Exception as e:
                    print(f"Warning: Failed to index {doc_info.path}: {e}")
        
        print(f"✅ Indexed {indexed_count} documents")
        
    def _scan_documents(self) -> List[DocumentInfo]:
        """扫描文档"""
        documents = []
        
        for md_file in self.project_root.rglob("*.md"):
            if self._should_index_file(md_file):
                try:
                    doc_info = self._parse_document(md_file)
                    documents.append(doc_info)
                except Exception as e:
                    print(f"Warning: Failed to parse {md_file}: {e}")
                    
        return documents
    
    def _should_index_file(self, file_path: Path) -> bool:
        """判断是否应该索引文件"""
        excluded_patterns = [
            "target/", ".git/", "vendor/", "venv/",
            "output/temp/", "output/logs/"
        ]
        
        path_str = str(file_path)
        return not any(pattern in path_str for pattern in excluded_patterns)
    
    def _parse_document(self, file_path: Path) -> DocumentInfo:
        """解析文档内容"""
        content = file_path.read_text(encoding='utf-8')
        
        # 提取标题
        title_match = re.search(r'^#\s+(.+)$', content, re.MULTILINE)
        title = title_match.group(1) if title_match else file_path.stem
        
        # 提取标签
        tags = self._extract_tags(content)
        
        # 确定分类
        category = self._determine_category(file_path)
        
        # 统计字数
        word_count = len(re.findall(r'\S+', content))
        
        # 获取修改时间
        mtime = datetime.fromtimestamp(file_path.stat().st_mtime)
        
        return DocumentInfo(
            path=str(file_path.relative_to(self.project_root)),
            title=title.strip(),
            content=content,
            word_count=word_count,
            last_modified=mtime,
            tags=tags,
            category=category
        )
    
    def _extract_tags(self, content: str) -> List[str]:
        """从内容中提取标签"""
        tags = []
        
        # 从标题级别提取
        headings = re.findall(r'^#{1,6}\s+(.+)$', content, re.MULTILINE)
        tags.extend([h.lower().replace(' ', '_') for h in headings[:3]])
        
        # 从特定关键词提取
        keywords = ['rust', 'tls', 'http', 'fingerprint', 'api', 'security']
        content_lower = content.lower()
        tags.extend([kw for kw in keywords if kw in content_lower])
        
        return list(set(tags))  # 去重
    
    def _determine_category(self, file_path: Path) -> str:
        """确定文档分类"""
        path_parts = str(file_path.relative_to(self.project_root)).split('/')
        
        if 'docs' in path_parts:
            if 'user-guides' in path_parts:
                return 'user_guide'
            elif 'developer-guides' in path_parts:
                return 'developer_guide'
            elif 'reference' in path_parts:
                return 'reference'
            elif 'project-management' in path_parts:
                return 'project_management'
            else:
                return 'documentation'
        elif path_parts[0] in ['README.md', 'CONTRIBUTING.md', 'SECURITY.md']:
            return 'root'
        else:
            return 'other'

class DocumentSearcher:
    """文档搜索引擎"""
    
    def __init__(self, project_root: str = "."):
        self.project_root = Path(project_root)
        self.indexer = DocumentIndexer(project_root)
        self.index_db = self.indexer.index_db
    
    def search(self, query: str, limit: int = 10) -> List[DocumentInfo]:
        """执行搜索"""
        # 首先确保索引是最新的
        self.indexer.index_documents()
        
        results = []
        
        with sqlite3.connect(self.index_db) as conn:
            # 使用全文搜索
            cursor = conn.execute('''
                SELECT d.path, d.title, d.content, d.word_count, 
                       d.last_modified, d.tags, d.category
                FROM documents d
                JOIN document_search s ON d.path = s.path
                WHERE document_search MATCH ?
                ORDER BY rank
                LIMIT ?
            ''', (query, limit))
            
            for row in cursor.fetchall():
                doc_info = DocumentInfo(
                    path=row[0],
                    title=row[1],
                    content=row[2][:500] + "..." if len(row[2]) > 500 else row[2],
                    word_count=row[3],
                    last_modified=datetime.fromisoformat(row[4]),
                    tags=row[5].split(',') if row[5] else [],
                    category=row[6]
                )
                results.append(doc_info)
        
        return results
    
    def recommend_similar(self, document_path: str, limit: int = 5) -> List[DocumentInfo]:
        """推荐相似文档"""
        # 简单的基于标签和分类的推荐
        target_doc = None
        
        with sqlite3.connect(self.index_db) as conn:
            cursor = conn.execute('''
                SELECT tags, category FROM documents WHERE path = ?
            ''', (document_path,))
            
            row = cursor.fetchone()
            if row:
                target_tags = set(row[0].split(',') if row[0] else [])
                target_category = row[1]
            else:
                return []
        
        # 查找相似文档
        similar_docs = []
        with sqlite3.connect(self.index_db) as conn:
            cursor = conn.execute('''
                SELECT path, title, content, word_count, last_modified, tags, category
                FROM documents 
                WHERE path != ?
                ORDER BY 
                    CASE WHEN category = ? THEN 1 ELSE 0 END DESC,
                    LENGTH(tags) DESC
                LIMIT ?
            ''', (document_path, target_category, limit * 2))
            
            for row in cursor.fetchall():
                doc_tags = set(row[5].split(',') if row[5] else [])
                similarity_score = len(target_tags.intersection(doc_tags))
                
                if similarity_score > 0:
                    doc_info = DocumentInfo(
                        path=row[0],
                        title=row[1],
                        content=row[2][:300] + "..." if len(row[2]) > 300 else row[2],
                        word_count=row[3],
                        last_modified=datetime.fromisoformat(row[4]),
                        tags=row[5].split(',') if row[5] else [],
                        category=row[6],
                        similarity_score=similarity_score
                    )
                    similar_docs.append(doc_info)
        
        # 按相似度排序并限制数量
        similar_docs.sort(key=lambda x: x.similarity_score, reverse=True)
        return similar_docs[:limit]

def main():
    """主函数 - 提供命令行接口"""
    import argparse
    
    parser = argparse.ArgumentParser(description='智能文档检索系统')
    parser.add_argument('action', choices=['search', 'recommend', 'index'], 
                       help='执行的操作')
    parser.add_argument('--query', '-q', help='搜索查询')
    parser.add_argument('--document', '-d', help='文档路径（用于推荐）')
    parser.add_argument('--limit', '-l', type=int, default=10, help='结果数量限制')
    
    args = parser.parse_args()
    
    searcher = DocumentSearcher()
    
    if args.action == 'index':
        print(".CreateIndexing all documents...")
        searcher.indexer.index_documents()
        print("✅ Indexing complete")
        
    elif args.action == 'search':
        if not args.query:
            print("错误: 搜索操作需要提供查询参数")
            return
            
        print(f"🔍 搜索: {args.query}")
        results = searcher.search(args.query, args.limit)
        
        if results:
            print(f"\n找到 {len(results)} 个结果:")
            for i, doc in enumerate(results, 1):
                print(f"\n{i}. {doc.title}")
                print(f"   路径: {doc.path}")
                print(f"   分类: {doc.category}")
                print(f"   标签: {', '.join(doc.tags)}")
                print(f"   字数: {doc.word_count}")
        else:
            print("未找到相关文档")
            
    elif args.action == 'recommend':
        if not args.document:
            print("错误: 推荐操作需要提供文档路径")
            return
            
        print(f"🤖 为文档推荐相关内容: {args.document}")
        recommendations = searcher.recommend_similar(args.document, args.limit)
        
        if recommendations:
            print(f"\n推荐 {len(recommendations)} 个相关文档:")
            for i, doc in enumerate(recommendations, 1):
                print(f"\n{i}. {doc.title}")
                print(f"   路径: {doc.path}")
                print(f"   相似度得分: {doc.similarity_score}")
                print(f"   分类: {doc.category}")
        else:
            print("未找到相关推荐")

if __name__ == "__main__":
    main()