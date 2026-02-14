#!/usr/bin/env python3
"""
文档更新提醒工具
定期检查文档更新状态并发送提醒
"""

import os
import json
from pathlib import Path
from datetime import datetime, timedelta
from typing import Dict, List, Tuple

class DocumentUpdateReminder:
    def __init__(self, project_root: str = "."):
        self.project_root = Path(project_root)
        self.tracking_file = self.project_root / "output" / "data" / "document_tracking.json"
        self.reminder_threshold = timedelta(days=90)  # 90天未更新提醒
        self.critical_threshold = timedelta(days=180)  # 180天未更新标记为严重

    def scan_documents(self) -> Dict[str, Dict]:
        """扫描所有文档并记录更新信息"""
        documents = {}
        
        # 查找所有Markdown文件
        for md_file in self.project_root.rglob("*.md"):
            if self.should_track_file(md_file):
                doc_info = self.get_document_info(md_file)
                documents[str(md_file.relative_to(self.project_root))] = doc_info
                
        return documents

    def should_track_file(self, file_path: Path) -> bool:
        """判断是否应该跟踪此文件"""
        # 排除临时文件和构建产物
        excluded_patterns = [
            "target/", ".git/", "vendor/", "venv/",
            "output/temp/", "output/logs/"
        ]
        
        path_str = str(file_path)
        return not any(pattern in path_str for pattern in excluded_patterns)

    def get_document_info(self, file_path: Path) -> Dict:
        """获取文档信息"""
        try:
            stat = file_path.stat()
            mtime = datetime.fromtimestamp(stat.st_mtime)
            
            # 读取文件内容检查元数据
            content = file_path.read_text(encoding='utf-8')
            last_updated = self.extract_last_updated(content)
            
            return {
                "last_modified": mtime.isoformat(),
                "last_updated_meta": last_updated,
                "size": stat.st_size,
                "needs_update": self.needs_update(mtime),
                "update_status": self.get_update_status(mtime)
            }
        except Exception as e:
            return {
                "error": str(e),
                "last_modified": "unknown",
                "needs_update": False,
                "update_status": "error"
            }

    def extract_last_updated(self, content: str) -> str:
        """从文档内容中提取最后更新日期"""
        import re
        
        # 匹配常见的更新日期格式
        patterns = [
            r'最后更新[：:]\s*(\d{4}-\d{2}-\d{2})',
            r'Last updated[：:]\s*(\d{4}-\d{2}-\d{2})',
            r'更新时间[：:]\s*(\d{4}-\d{2}-\d{2})'
        ]
        
        for pattern in patterns:
            match = re.search(pattern, content)
            if match:
                return match.group(1)
                
        return "unknown"

    def needs_update(self, modification_time: datetime) -> bool:
        """判断文档是否需要更新"""
        return datetime.now() - modification_time > self.reminder_threshold

    def get_update_status(self, modification_time: datetime) -> str:
        """获取更新状态"""
        age = datetime.now() - modification_time
        
        if age > self.critical_threshold:
            return "critical"
        elif age > self.reminder_threshold:
            return "needs_attention"
        else:
            return "up_to_date"

    def generate_reminder_report(self) -> Dict:
        """生成更新提醒报告"""
        documents = self.scan_documents()
        now = datetime.now()
        
        report = {
            "generated_at": now.isoformat(),
            "summary": {
                "total_documents": len(documents),
                "up_to_date": 0,
                "needs_attention": 0,
                "critical": 0,
                "errors": 0
            },
            "documents_by_status": {
                "up_to_date": [],
                "needs_attention": [],
                "critical": [],
                "errors": []
            }
        }
        
        # 分类文档
        for path, info in documents.items():
            status = info.get("update_status", "error")
            report["documents_by_status"][status].append({
                "path": path,
                "last_modified": info.get("last_modified", "unknown"),
                "size": info.get("size", 0)
            })
            report["summary"][status] += 1
            
        return report

    def send_reminders(self, report: Dict):
        """发送更新提醒"""
        # 创建提醒文件
        reminder_file = self.project_root / "output" / "reports" / "document_update_reminders.md"
        reminder_file.parent.mkdir(parents=True, exist_ok=True)
        
        with open(reminder_file, 'w', encoding='utf-8') as f:
            f.write("# 文档更新提醒报告\n\n")
            f.write(f"**生成时间**: {report['generated_at']}\n\n")
            
            # 摘要
            summary = report['summary']
            f.write("## 📊 更新状态摘要\n\n")
            f.write(f"- **总计文档**: {summary['total_documents']}\n")
            f.write(f"- **最新文档**: {summary['up_to_date']}\n")
            f.write(f"- **需要注意**: {summary['needs_attention']}\n")
            f.write(f"- **急需更新**: {summary['critical']}\n")
            f.write(f"- **错误文件**: {summary['errors']}\n\n")
            
            # 详细列表
            for status, docs in report['documents_by_status'].items():
                if docs and status != 'up_to_date':
                    f.write(f"## {self.format_status_title(status)}\n\n")
                    for doc in sorted(docs, key=lambda x: x['path']):
                        f.write(f"- `{doc['path']}` ")
                        f.write(f"(修改时间: {doc['last_modified'][:10]}, ")
                        f.write(f"大小: {doc['size']} bytes)\n")
                    f.write("\n")
        
        print(f"✅ 更新提醒报告已生成: {reminder_file}")

    def format_status_title(self, status: str) -> str:
        """格式化状态标题"""
        titles = {
            "needs_attention": "🟡 需要注意的文档 (90天以上未更新)",
            "critical": "🔴 急需更新的文档 (180天以上未更新)",
            "errors": "❌ 处理出错的文档"
        }
        return titles.get(status, status)

    def save_tracking_data(self, documents: Dict):
        """保存跟踪数据"""
        tracking_data = {
            "last_scan": datetime.now().isoformat(),
            "documents": documents
        }
        
        self.tracking_file.parent.mkdir(parents=True, exist_ok=True)
        with open(self.tracking_file, 'w', encoding='utf-8') as f:
            json.dump(tracking_data, f, indent=2, ensure_ascii=False)

    def load_tracking_data(self) -> Dict:
        """加载历史跟踪数据"""
        if self.tracking_file.exists():
            try:
                with open(self.tracking_file, 'r', encoding='utf-8') as f:
                    return json.load(f)
            except:
                pass
        return {}

    def compare_with_history(self, current_documents: Dict) -> Dict:
        """与历史数据比较"""
        history = self.load_tracking_data()
        changes = {
            "new_documents": [],
            "updated_documents": [],
            "deleted_documents": []
        }
        
        if "documents" in history:
            old_docs = set(history["documents"].keys())
            current_docs = set(current_documents.keys())
            
            # 新增文档
            changes["new_documents"] = list(current_docs - old_docs)
            
            # 删除文档
            changes["deleted_documents"] = list(old_docs - current_docs)
            
            # 更新文档
            for doc in current_docs.intersection(old_docs):
                old_time = history["documents"][doc].get("last_modified")
                current_time = current_documents[doc].get("last_modified")
                if old_time and current_time and old_time != current_time:
                    changes["updated_documents"].append(doc)
        
        return changes

def main():
    reminder = DocumentUpdateReminder()
    
    print("📅 开始文档更新检查...")
    
    # 扫描文档
    documents = reminder.scan_documents()
    
    # 生成报告
    report = reminder.generate_reminder_report()
    
    # 发送提醒
    reminder.send_reminders(report)
    
    # 保存跟踪数据
    reminder.save_tracking_data(documents)
    
    # 显示摘要
    summary = report['summary']
    print(f"\n📊 文档更新检查完成!")
    print(f"📁 总计文档: {summary['total_documents']}")
    print(f"✅ 最新文档: {summary['up_to_date']}")
    print(f"⚠️  需注意: {summary['needs_attention']}")
    print(f"🔴 急需更新: {summary['critical']}")
    
    if summary['critical'] > 0:
        print(f"\n🚨 发现 {summary['critical']} 个急需更新的文档!")
        print("请查看详细报告了解具体文档。")
    elif summary['needs_attention'] > 0:
        print(f"\n💡 建议关注 {summary['needs_attention']} 个可能需要更新的文档。")

if __name__ == "__main__":
    main()