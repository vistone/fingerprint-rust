#!/usr/bin/env python3
"""
文档版本控制系统
跟踪文档变更历史，提供版本比较和恢复功能
"""

import os
import json
import hashlib
import difflib
from pathlib import Path
from typing import Dict, List, Optional
from dataclasses import dataclass, asdict
from datetime import datetime
import sqlite3

@dataclass
class DocumentVersion:
    """文档版本信息"""
    version_id: str
    document_path: str
    content_hash: str
    content_length: int
    author: str
    timestamp: datetime
    commit_message: str
    parent_version: Optional[str] = None

class DocumentVersionControl:
    """文档版本控制器"""
    
    def __init__(self, project_root: str = ".", author: str = "system"):
        self.project_root = Path(project_root)
        self.author = author
        self.version_db = self.project_root / "output" / "data" / "document_versions.db"
        self._init_database()
        
    def _init_database(self):
        """初始化版本数据库"""
        self.version_db.parent.mkdir(parents=True, exist_ok=True)
        
        with sqlite3.connect(self.version_db) as conn:
            # 文档版本表
            conn.execute('''
                CREATE TABLE IF NOT EXISTS document_versions (
                    version_id TEXT PRIMARY KEY,
                    document_path TEXT NOT NULL,
                    content_hash TEXT NOT NULL,
                    content_length INTEGER NOT NULL,
                    author TEXT NOT NULL,
                    timestamp TIMESTAMP NOT NULL,
                    commit_message TEXT,
                    parent_version TEXT,
                    FOREIGN KEY (parent_version) REFERENCES document_versions(version_id)
                )
            ''')
            
            # 创建索引
            conn.execute('''
                CREATE INDEX IF NOT EXISTS idx_document_path 
                ON document_versions(document_path)
            ''')
            
            conn.execute('''
                CREATE INDEX IF NOT EXISTS idx_timestamp 
                ON document_versions(timestamp)
            ''')
    
    def track_changes(self, force: bool = False):
        """跟踪文档变更"""
        print("🔍 检查文档变更...")
        
        tracked_count = 0
        changed_docs = []
        
        # 查找所有Markdown文档
        for md_file in self.project_root.rglob("*.md"):
            if self._should_track_file(md_file):
                if self._check_and_record_change(md_file, force):
                    tracked_count += 1
                    changed_docs.append(str(md_file.relative_to(self.project_root)))
        
        print(f"✅ 跟踪了 {tracked_count} 个文档变更")
        
        if changed_docs:
            print("\n变更的文档:")
            for doc in changed_docs:
                print(f"  - {doc}")
        
        return tracked_count
    
    def _should_track_file(self, file_path: Path) -> bool:
        """判断是否应该跟踪文件"""
        excluded_patterns = [
            "target/", ".git/", "vendor/", "venv/",
            "output/temp/", "output/logs/"
        ]
        
        path_str = str(file_path)
        return not any(pattern in path_str for pattern in excluded_patterns)
    
    def _check_and_record_change(self, file_path: Path, force: bool = False) -> bool:
        """检查并记录文件变更"""
        try:
            # 计算文件哈希
            content_hash = self._calculate_file_hash(file_path)
            content_length = file_path.stat().st_size
            
            # 检查是否已有记录
            latest_version = self._get_latest_version(str(file_path.relative_to(self.project_root)))
            
            # 如果内容未改变且非强制模式，则跳过
            if not force and latest_version and latest_version.content_hash == content_hash:
                return False
            
            # 创建新版本
            version_id = self._generate_version_id(file_path, content_hash)
            parent_version = latest_version.version_id if latest_version else None
            
            # 记录新版本
            new_version = DocumentVersion(
                version_id=version_id,
                document_path=str(file_path.relative_to(self.project_root)),
                content_hash=content_hash,
                content_length=content_length,
                author=self.author,
                timestamp=datetime.now(),
                commit_message=self._generate_commit_message(latest_version, content_length),
                parent_version=parent_version
            )
            
            self._save_version(new_version)
            return True
            
        except Exception as e:
            print(f"警告: 无法跟踪 {file_path}: {e}")
            return False
    
    def _calculate_file_hash(self, file_path: Path) -> str:
        """计算文件内容哈希"""
        hash_obj = hashlib.sha256()
        with open(file_path, 'rb') as f:
            for chunk in iter(lambda: f.read(4096), b""):
                hash_obj.update(chunk)
        return hash_obj.hexdigest()
    
    def _generate_version_id(self, file_path: Path, content_hash: str) -> str:
        """生成版本ID"""
        timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
        file_name = file_path.stem
        return f"{file_name}_{timestamp}_{content_hash[:8]}"
    
    def _generate_commit_message(self, previous_version: Optional[DocumentVersion], 
                               new_size: int) -> str:
        """生成提交信息"""
        if not previous_version:
            return "初始版本"
        
        size_diff = new_size - previous_version.content_length
        if size_diff > 0:
            return f"更新文档 (+{size_diff} 字节)"
        elif size_diff < 0:
            return f"更新文档 ({size_diff} 字节)"
        else:
            return "文档更新"
    
    def _get_latest_version(self, document_path: str) -> Optional[DocumentVersion]:
        """获取文档的最新版本"""
        with sqlite3.connect(self.version_db) as conn:
            cursor = conn.execute('''
                SELECT version_id, document_path, content_hash, content_length,
                       author, timestamp, commit_message, parent_version
                FROM document_versions
                WHERE document_path = ?
                ORDER BY timestamp DESC
                LIMIT 1
            ''', (document_path,))
            
            row = cursor.fetchone()
            if row:
                return DocumentVersion(
                    version_id=row[0],
                    document_path=row[1],
                    content_hash=row[2],
                    content_length=row[3],
                    author=row[4],
                    timestamp=datetime.fromisoformat(row[5]),
                    commit_message=row[6],
                    parent_version=row[7]
                )
        return None
    
    def _save_version(self, version: DocumentVersion):
        """保存版本信息"""
        with sqlite3.connect(self.version_db) as conn:
            conn.execute('''
                INSERT INTO document_versions 
                (version_id, document_path, content_hash, content_length, 
                 author, timestamp, commit_message, parent_version)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            ''', (
                version.version_id,
                version.document_path,
                version.content_hash,
                version.content_length,
                version.author,
                version.timestamp.isoformat(),
                version.commit_message,
                version.parent_version
            ))
    
    def get_document_history(self, document_path: str) -> List[DocumentVersion]:
        """获取文档历史版本"""
        versions = []
        
        with sqlite3.connect(self.version_db) as conn:
            cursor = conn.execute('''
                SELECT version_id, document_path, content_hash, content_length,
                       author, timestamp, commit_message, parent_version
                FROM document_versions
                WHERE document_path = ?
                ORDER BY timestamp DESC
            ''', (document_path,))
            
            for row in cursor.fetchall():
                versions.append(DocumentVersion(
                    version_id=row[0],
                    document_path=row[1],
                    content_hash=row[2],
                    content_length=row[3],
                    author=row[4],
                    timestamp=datetime.fromisoformat(row[5]),
                    commit_message=row[6],
                    parent_version=row[7]
                ))
        
        return versions
    
    def compare_versions(self, document_path: str, version1: str, version2: str) -> str:
        """比较两个版本的差异"""
        # 这里简化实现，实际应该从存储中获取版本内容
        return f"版本 {version1} 和 {version2} 的差异比较功能待实现"
    
    def restore_version(self, document_path: str, version_id: str) -> bool:
        """恢复到指定版本"""
        # 这里是简化实现，实际需要从备份存储中恢复内容
        print(f"⚠️  版本恢复功能待实现: {document_path} -> {version_id}")
        return False
    
    def generate_history_report(self) -> Dict:
        """生成历史报告"""
        report = {
            "generated_at": datetime.now().isoformat(),
            "summary": {
                "total_documents": 0,
                "total_versions": 0,
                "recent_changes": []
            },
            "documents": {}
        }
        
        with sqlite3.connect(self.version_db) as conn:
            # 统计总数
            cursor = conn.execute('SELECT COUNT(DISTINCT document_path) FROM document_versions')
            report["summary"]["total_documents"] = cursor.fetchone()[0]
            
            cursor = conn.execute('SELECT COUNT(*) FROM document_versions')
            report["summary"]["total_versions"] = cursor.fetchone()[0]
            
            # 获取最近变更
            cursor = conn.execute('''
                SELECT document_path, author, timestamp, commit_message
                FROM document_versions
                ORDER BY timestamp DESC
                LIMIT 10
            ''')
            
            for row in cursor.fetchall():
                report["summary"]["recent_changes"].append({
                    "document": row[0],
                    "author": row[1],
                    "timestamp": row[2],
                    "message": row[3]
                })
            
            # 按文档分组统计
            cursor = conn.execute('''
                SELECT document_path, COUNT(*) as version_count,
                       MIN(timestamp) as first_version,
                       MAX(timestamp) as last_version
                FROM document_versions
                GROUP BY document_path
                ORDER BY version_count DESC
            ''')
            
            for row in cursor.fetchall():
                report["documents"][row[0]] = {
                    "versions": row[1],
                    "first_version": row[2],
                    "last_version": row[3]
                }
        
        return report

def main():
    """主函数"""
    import argparse
    
    parser = argparse.ArgumentParser(description='文档版本控制系统')
    parser.add_argument('action', choices=['track', 'history', 'compare', 'restore', 'report'],
                       help='执行的操作')
    parser.add_argument('--document', '-d', help='文档路径')
    parser.add_argument('--force', '-f', action='store_true', help='强制跟踪所有文档')
    parser.add_argument('--author', '-a', default='system', help='作者名称')
    
    args = parser.parse_args()
    
    vc = DocumentVersionControl(author=args.author)
    
    if args.action == 'track':
        count = vc.track_changes(force=args.force)
        print(f"📊 跟踪了 {count} 个文档变更")
        
    elif args.action == 'history':
        if not args.document:
            print("错误: 需要指定文档路径")
            return
            
        history = vc.get_document_history(args.document)
        print(f"\n{args.document} 的版本历史:")
        for version in history:
            print(f"  {version.timestamp.strftime('%Y-%m-%d %H:%M')} - "
                  f"{version.version_id} - {version.commit_message}")
                  
    elif args.action == 'report':
        report = vc.generate_history_report()
        report_file = Path("output/reports/version_control_report.json")
        report_file.parent.mkdir(parents=True, exist_ok=True)
        
        with open(report_file, 'w', encoding='utf-8') as f:
            json.dump(report, f, indent=2, ensure_ascii=False)
        
        print(f"✅ 报告已生成: {report_file}")
        print(f"📊 总计文档: {report['summary']['total_documents']}")
        print(f"📊 总版本数: {report['summary']['total_versions']}")

if __name__ == "__main__":
    main()