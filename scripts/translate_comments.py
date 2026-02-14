#!/usr/bin/env python3
"""
Improved Chinese comment translation script
改进的中文注释翻译脚本
"""

import os
import re
import sys
from typing import List, Tuple

# 更精确的中英翻译映射表
TRANSLATION_MAP = {
    # 模块和功能相关
    '模块': 'module',
    '功能': 'functionality',
    '实现': 'implementation',
    '支持': 'support',
    '提供': 'provide',
    '包含': 'include',
    '使用': 'use',
    '应用': 'apply',
    '配置': 'configure',
    '设置': 'set',
    '创建': 'create',
    '初始化': 'initialize',
    '生成': 'generate',
    '处理': 'process',
    '解析': 'parse',
    '验证': 'validate',
    '检查': 'check',
    '测试': 'test',
    '分析': 'analyze',
    '检测': 'detect',
    '识别': 'identify',
    '枚举': 'enumerate',
    '加载': 'load',
    '保存': 'save',
    '存储': 'store',
    '获取': 'get',
    '更新': 'update',
    '删除': 'delete',
    '查询': 'query',
    '计算': 'calculate',
    '统计': 'statistics',
    '监控': 'monitor',
    '防护': 'protection',
    '泄露': 'leak',
    '隐藏': 'hide',
    '伪造': 'forge',
    '过滤': 'filter',
    '验证': 'verify',
    '标准化': 'standardize',
    '转换': 'convert',
    '提取': 'extract',
    '渲染': 'render',
    '精度': 'precision',
    '特征': 'features',
    '参数': 'parameters',
    '数据': 'data',
    '信息': 'information',
    '结构': 'structure',
    '类型': 'type',
    '状态': 'status',
    '模式': 'mode',
    '时间': 'time',
    '地址': 'address',
    '候选': 'candidate',
    '本地': 'local',
    '远程': 'remote',
    '连接': 'connection',
    '会话': 'session',
    '缓冲': 'buffer',
    '大小': 'size',
    '数量': 'count',
    '哈希': 'hash',
    '向量': 'vector',
    '标签': 'label',
    '置信度': 'confidence',
    '评分': 'score',
    '方差': 'variance',
    '频率': 'frequency',
    '样本': 'sample',
    '通道': 'channel',
    '目标': 'target',
    '融合': 'blend',
    '振荡器': 'oscillator',
    '字体': 'font',
    '系统': 'system',
    '操作系统': 'operating system',
    '子集': 'subset',
    '列表': 'list',
    '库': 'library',
    '分析器': 'analyzer',
    '检测器': 'detector',
    '防护器': 'protector',
    '报告': 'report',
    '统计信息': 'statistics',
    '错误': 'error',
    '异常': 'exception',
    '失败': 'failure',
    '成功': 'success',
    '警告': 'warning',
    '信息': 'info',
    '调试': 'debug',
    '人类': 'human',
    '用户': 'user',
    '机器人': 'bot',
    '行为': 'behavior',
    '风险': 'risk',
    '正常': 'normal',
    '可疑': 'suspicious',
    '未知': 'unknown',
    '分类': 'classification',
    '相似度': 'similarity',
    '余弦': 'cosine',
    '随机': 'random',
    '延迟': 'delay',
    '分辨率': 'resolution',
    '防护': 'protection',
    '时序': 'timing',
    '标准化': 'normalize',
    '时间戳': 'timestamp',
    '毫秒': 'milliseconds',
    '赫兹': 'Hz',
    'FFT': 'FFT',
    'GPU': 'GPU',
    'WebGL': 'WebGL',
    'WebRTC': 'WebRTC',
    'Audio': 'Audio',
    'Context': 'Context',
    'API': 'API',
    'IP': 'IP',
    'mDNS': 'mDNS',
    'srflx': 'srflx',
    'prflx': 'prflx',
    'relay': 'relay',
    '主机': 'host',
    '新建': 'new',
    '连接中': 'connecting',
    '已连接': 'connected',
    '已完成': 'completed',
    '断开连接': 'disconnected',
    '失败': 'failed',
    '已关闭': 'closed',
    '无效': 'invalid',
    '其他': 'other',
    '常见': 'common',
    '预加载': 'preload',
    '默认': 'default',
    '独特': 'unique',
    '基于': 'based on',
    '模拟': 'simulate',
    '长度': 'length',
    '名称': 'name',
    '结尾': 'ending',
    '标准': 'standard',
    '临时': 'temporary',
    '稳定': 'stable',
    '审核': 'review',
    '待审核': 'pending review',
    '索引': 'index',
    '性能': 'performance',
    '提高': 'improve',
    '查询': 'query',
    '所有': 'all',
    '更新': 'update',
    '获取': 'get',
    '结构体': 'struct',
    '统计': 'statistics',
    '信息': 'information',
    '学习': 'learning',
    '机制': 'mechanism',
    '自动': 'automatic',
    '发现': 'discover',
    '记录': 'record',
    '威胁': 'threat',
    '狩猎': 'hunting',
    '蜜罐': 'honeypot',
    '新增': 'newly added',
    '版本': 'version',
    '函数': 'function',
    '调用': 'call',
    '注册': 'register',
    '加载': 'load',
    '具有一致': 'have consistent',
    '一致的': 'consistent',
    '确保': 'ensure',
    '都有': 'all have',
    '变体': 'variant',
    '枚举': 'enumeration',
    '识别': 'identification',
    '检测': 'detection',
    '支持': 'support',
    '子集': 'subset',
    '渲染': 'rendering',
    '特征': 'characteristics',
    '防护': 'protection',
    '泄露': 'leakage',
    '防护': 'protection',
    '指纹': 'fingerprint',
    '识别': 'recognition',
    '能力': 'capability',
    '包括': 'including',
    '参数': 'parameters',
    '提取': 'extraction',
    '样本率': 'sample rate',
    '频率': 'frequency',
    '分析': 'analysis',
    '音频': 'audio',
    '处理': 'processing',
    '精度': 'precision',
    '检测': 'detection',
    '振荡器': 'oscillator',
    '类型': 'type',
    '融合': 'blending',
    '模式': 'mode',
    '配置文件': 'profile',
    '库': 'library',
    '分析器': 'analyzer',
    '数据': 'data',
    '标准化': 'normalized',
    '频率': 'frequency',
    '检测': 'detection',
    '融合': 'blending',
    '音频': 'audio',
    '精度': 'precision',
    '配置': 'configuration',
    '库': 'library',
    '创建': 'create',
    '新的': 'new',
    '预加载': 'pre-load',
    '常见': 'common',
    'GPU': 'GPU',
    '分析': 'analyze',
    'WebGL': 'WebGL',
    '字体': 'font',
    '枚举': 'enumeration',
    '指纹': 'fingerprint',
    '识别': 'recognition',
    '系统': 'system',
    '字体': 'font',
    '列表': 'list',
    '枚举': 'enumeration',
    '字体': 'font',
    '加载': 'loading',
    '时间': 'time',
    '分析': 'analysis',
    '字体': 'font',
    '渲染': 'rendering',
    '特征': 'features',
    '识别': 'recognition',
    '子集': 'subset',
    '支持': 'support',
    '检测': 'detection',
    '字体': 'font',
    '错误': 'error',
    '类型': 'type',
    '无效': 'invalid',
    '数据': 'data',
    '枚举': 'enumeration',
    '失败': 'failure',
    '其他': 'other',
    '错误': 'error',
    '字体': 'font',
    '分析器': 'analyzer',
    '分析': 'analyze',
    '系统': 'system',
    '字体': 'fonts',
    '转换': 'convert',
    '为': 'to',
    '字符串': 'string',
    '向量': 'vector',
    '计算': 'calculate',
    '加载': 'loading',
    '时间': 'time',
    '生成': 'generate',
    '唯一': 'unique',
    '哈希': 'hash',
    '检测': 'detect',
    '子集': 'subset',
    '支持': 'support',
    '获取': 'get',
    '渲染': 'rendering',
    '特征': 'features',
    '字体': 'font',
    '系统': 'system',
    '检测器': 'detector',
    '检测': 'detect',
    '操作系统': 'operating system',
    '字体': 'fonts',
    '计算': 'calculate',
    '字体': 'font',
    '加载': 'loading',
    '时间': 'time',
    '基于': 'based on',
    '字体': 'font',
    '名称': 'name',
    '长度': 'length',
    '和': 'and',
    '特征': 'features',
    '的': 'of',
    '模拟': 'simulated',
    '时间': 'time',
    '生成': 'generate',
    '字体': 'font',
    '哈希': 'hash',
    '检测': 'detect',
    '支持': 'supported',
    '的': 'of',
    '子集': 'subsets',
    '基于': 'based on',
    '字体': 'font',
    '名称': 'name',
    '检测': 'detect',
    '子集': 'subset',
    '默认': 'default',
    '子集': 'subset',
    '获取': 'get',
    '渲染': 'rendering',
    '特征': 'features',
    'WebRTC': 'WebRTC',
    '泄露': 'leakage',
    '防护': 'protection',
    '模块': 'module',
    '提供': 'provide',
    'WebRTC': 'WebRTC',
    'IP': 'IP',
    '泄露': 'leakage',
    '防护': 'protection',
    '和': 'and',
    '指纹': 'fingerprint',
    '识别': 'recognition',
    '能力': 'capabilities',
    'WebRTC': 'WebRTC',
    '指纹': 'fingerprint',
    '信息': 'information',
    '本地': 'local',
    'IP': 'IP',
    '候选': 'candidate',
    '地址': 'addresses',
    '远程': 'remote',
    'IP': 'IP',
    '地址': 'address',
    '连接': 'connection',
    '状态': 'status',
    'mDNS': 'mDNS',
    '候选': 'candidate',
    '隐藏': 'hiding',
    '候选': 'candidate',
    '过滤': 'filtering',
    '统计': 'statistics',
    '连接': 'connection',
    '状态': 'status',
    '新建': 'new',
    '连接中': 'connecting',
    '已连接': 'connected',
    '已完成': 'completed',
    '断开连接': 'disconnected',
    '失败': 'failed',
    '已关闭': 'closed',
    '候选': 'candidate',
    '统计信息': 'statistics',
    '主机': 'host',
    '候选数': 'candidates',
    'srflx': 'srflx',
    '候选数': 'candidates',
    'prflx': 'prflx',
    '候选数': 'candidates',
    'relay': 'relay',
    '候选数': 'candidates',
    'WebRTC': 'WebRTC',
    '错误': 'error',
    '类型': 'type',
    '无效': 'invalid',
    'IP': 'IP',
    '分析': 'analysis',
    '失败': 'failure',
    '其他': 'other',
    '错误': 'errors',
    'WebRTC': 'WebRTC',
    '分析器': 'analyzer',
    '分析': 'analyze',
    'WebRTC': 'WebRTC',
    '候选': 'candidates',
    '提取': 'extract',
    'IP': 'IP',
    '地址': 'addresses',
    '验证': 'validate',
    'IP': 'IP',
    '地址': 'address',
    'WebRTC': 'WebRTC',
    '防护器': 'protector',
    '隐藏': 'hide',
    'mDNS': 'mDNS',
    '候选': 'candidates',
    '伪造': 'forge',
    'IP': 'IP',
    '地址': 'addresses',
    '检测': 'detect',
    'WebRTC': 'WebRTC',
    '泄露': 'leaks',
    'WebRTC': 'WebRTC',
    '泄露': 'leak',
    '报告': 'report',
    'mDNS': 'mDNS',
    '地址': 'addresses',
    '以': 'ending with',
    'local': '.local',
    '结尾': '',
    'RFC': '(RFC',
    '6762)': '6762)',
}

def find_chinese_comments(file_path: str) -> List[Tuple[int, str]]:
    """查找文件中的中文注释"""
    chinese_comments = []
    
    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            for line_num, line in enumerate(f, 1):
                # 查找注释中的中文字符
                if '//' in line:
                    comment_part = line.split('//', 1)[1]
                    if re.search(r'[\u4e00-\u9fff]', comment_part):
                        chinese_comments.append((line_num, line.rstrip()))
    except Exception as e:
        print(f"Error reading {file_path}: {e}")
    
    return chinese_comments

def translate_comment(comment: str) -> str:
    """翻译单行注释"""
    # 提取注释部分
    if '//' in comment:
        prefix, text = comment.split('//', 1)
        text = text.strip()
        
        # 使用翻译映射表进行替换
        translated = text
        # 按照词汇长度降序排列，优先匹配长词汇
        sorted_items = sorted(TRANSLATION_MAP.items(), key=lambda x: len(x[0]), reverse=True)
        for chinese, english in sorted_items:
            translated = translated.replace(chinese, english)
        
        # 处理一些特殊情况
        translated = re.sub(r'\s+', ' ', translated)  # 合并多余空格
        translated = translated.strip()
        
        return f"{prefix}// {translated}"
    
    return comment

def process_file(file_path: str) -> bool:
    """处理单个文件的中文注释翻译"""
    chinese_comments = find_chinese_comments(file_path)
    
    if not chinese_comments:
        return False
    
    print(f"\nProcessing {file_path}:")
    print(f"Found {len(chinese_comments)} Chinese comments")
    
    # 读取原文件
    with open(file_path, 'r', encoding='utf-8') as f:
        lines = f.readlines()
    
    # 翻译注释
    modified = False
    for line_num, original_line in chinese_comments:
        translated_line = translate_comment(original_line)
        if translated_line != original_line:
            lines[line_num - 1] = translated_line + '\n'
            print(f"  Line {line_num}: {original_line.strip()}")
            print(f"           -> {translated_line.strip()}")
            modified = True
    
    # 写回文件
    if modified:
        with open(file_path, 'w', encoding='utf-8') as f:
            f.writelines(lines)
        print(f"  ✓ Translated {len([c for c in chinese_comments if translate_comment(c[1]) != c[1]])} comments")
        return True
    
    return False

def main():
    """主函数"""
    if len(sys.argv) > 1:
        target_path = sys.argv[1]
    else:
        target_path = "crates"
    
    print(f"Searching for Chinese comments in {target_path}...")
    
    total_files = 0
    total_comments = 0
    modified_files = 0
    
    # 遍历所有Rust文件
    for root, dirs, files in os.walk(target_path):
        for file in files:
            if file.endswith('.rs'):
                file_path = os.path.join(root, file)
                total_files += 1
                
                chinese_comments = find_chinese_comments(file_path)
                if chinese_comments:
                    total_comments += len(chinese_comments)
                    if process_file(file_path):
                        modified_files += 1
    
    print(f"\nSummary:")
    print(f"  Total files scanned: {total_files}")
    print(f"  Files with Chinese comments: {len([f for f in os.listdir(target_path) if f.endswith('.rs') and find_chinese_comments(os.path.join(target_path, f))])}")
    print(f"  Total Chinese comments found: {total_comments}")
    print(f"  Files modified: {modified_files}")

if __name__ == "__main__":
    main()