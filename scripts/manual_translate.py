#!/usr/bin/env python3
"""
Manual Chinese to English translation script
手工中英翻译脚本
"""

import os
import re

# 手工翻译映射表 - 更自然的表达
MANUAL_TRANSLATIONS = {
    # fingerprint-ml 模块
    "//! 机器学习指纹匹配模块": "//! Machine learning fingerprint matching module",
    "//! 提供高级的指纹相似度计算和分类能力": "//! Provides advanced fingerprint similarity calculation and classification capabilities",
    "/// 指纹向量": "/// Fingerprint vector",
    "/// 特征向量": "/// Feature vector", 
    "/// 标签": "/// Label",
    "/// 置信度": "/// Confidence",
    "/// ML 指纹匹配器": "/// ML fingerprint matcher",
    "/// 创建新的匹配器": "/// Create new matcher",
    "/// 添加参考指纹": "/// Add reference fingerprint",
    "/// 查找最相似的指纹": "/// Find most similar fingerprint",
    "/// 计算余弦相似度": "/// Calculate cosine similarity",
    "/// 获取所有匹配项": "/// Get all matches",
    "/// 行为分类器": "/// Behavior classifier",
    "/// 分类行为": "/// Classify behavior",
    "/// 计算风险评分": "/// Calculate risk score",
    "/// 计算方差": "/// Calculate variance",
    "/// 行为分类": "/// Behavior classification",
    "/// 人类用户": "/// Human user",
    "/// 正常行为": "/// Normal behavior",
    "/// 可疑行为": "/// Suspicious behavior",
    "/// 机器人": "/// Bot",
    "/// 未知": "/// Unknown",
    
    # fingerprint-defense 模块
    "//! - ✅ **学习机制** (`learner`): 自动发现并记录未知指纹": "//! - ✅ **Learning mechanism** (`learner`): Automatically discover and record unknown fingerprints",
    "//! - **威胁狩猎** (`hunting`): 蜜罐和行为分析": "//! - **Threat hunting** (`hunting`): Honeypot and behavior analysis",
    
    # database.rs
    "//! 提供指纹持久化存储和查询功能": "//! Provides fingerprint persistent storage and query functionality",
    "/// 存储指纹对象": "/// Store fingerprint object",
    "// 添加候选指纹表用于存储待审核的稳定指纹": "// Add candidate fingerprint table for storing stable fingerprints pending review",
    "// 创建索引提高查询性能": "// Create indexes to improve query performance",
    "/// 存储候选指纹（用于待审核的稳定未知指纹）": "/// Store candidate fingerprint (for stable unknown fingerprints pending review)",
    "/// 获取待审核的候选指纹列表": "/// Get list of candidate fingerprints pending review",
    "/// 更新候选指纹状态": "/// Update candidate fingerprint status",
    "/// 获取候选指纹统计信息": "/// Get candidate fingerprint statistics",
    "/// 候选指纹结构体": "/// Candidate fingerprint struct",
    "/// 候选指纹统计信息": "/// Candidate fingerprint statistics",
    
    # learner.rs
    "//! 指纹学习模块": "//! Fingerprint learning module",
    "//! 自动发现和记录稳定但未知的指纹模式": "//! Automatically discover and record stable but unknown fingerprint patterns",
    "/// 指纹观察器": "/// Fingerprint observer",
    "/// 观察网络流量并收集指纹": "/// Observe network traffic and collect fingerprints",
    "/// 指纹稳定性评估器": "/// Fingerprint stability evaluator",
    "/// 评估指纹的稳定性和可信度": "/// Evaluate fingerprint stability and credibility",
    "/// 时序防护器": "/// Timing protector",
    "/// 添加随机延迟": "/// Add random delay",
    "/// 隐藏时间分辨率": "/// Hide time resolution",
    "/// 检测时间异常": "/// Detect time anomalies",
    "/// 标准化时间戳": "/// Normalize timestamps",
    
    # 测试文件
    "// 测试新增的Chrome版本函数是否存在且能正常调用": "// Test if newly added Chrome version functions exist and can be called normally",
    "// 测试新增的Firefox版本函数": "// Test newly added Firefox version functions",
    "// 测试新增的Safari版本函数": "// Test newly added Safari version functions",
    "// 测试新增的Opera版本函数": "// Test newly added Opera version functions",
    "// 验证Chrome版本注册": "// Verify Chrome version registration",
    "// 验证Firefox版本注册": "// Verify Firefox version registration",
    "// 验证Safari版本注册": "// Verify Safari version registration",
    "// 验证Opera版本注册": "// Verify Opera version registration",
    "// 测试加载新增的版本": "// Test loading newly added versions",
    "// 验证所有配置文件具有一致的结构": "// Verify all configuration files have consistent structure",
    "// 确保都有HTTP/2设置": "// Ensure all have HTTP/2 settings",
    "// 测试移动端变体": "// Test mobile variants",
    
    # audio模块
    "//! Audio Context 指纹识别模块": "//! Audio Context fingerprint recognition module",
    "//! 提供 Web Audio API 指纹识别能力，包括：": "//! Provides Web Audio API fingerprint recognition capabilities, including:",
    "//! - Audio Context 参数提取": "//! - Audio Context parameter extraction",
    "//! - 样本率识别": "//! - Sample rate identification",
    "//! - 频率分析": "//! - Frequency analysis",
    "//! - 音频处理精度检测": "//! - Audio processing precision detection",
    "/// Audio Context 指纹": "/// Audio Context fingerprint",
    "/// 样本率 (Hz)": "/// Sample rate (Hz)",
    "/// 通道数": "/// Number of channels",
    "/// 目标通道数": "/// Target number of channels",
    "/// FFT 大小": "/// FFT size",
    "/// 频率分析数据": "/// Frequency analysis data",
    "/// 音频处理精度": "/// Audio processing precision",
    "/// 振荡器类型": "/// Oscillator type",
    "/// 融合模式": "/// Blending mode",
    "/// Audio 指纹错误类型": "/// Audio fingerprint error types",
    "/// 无效数据": "/// Invalid data",
    "/// 分析失败": "/// Analysis failed",
    "/// 其他错误": "/// Other errors",
    "/// Audio Context 分析器": "/// Audio Context analyzer",
    "/// 创建新的分析器": "/// Create new analyzer",
    "/// 分析 Audio Context 数据": "/// Analyze Audio Context data",
    "// 标准化频率数据": "// Normalize frequency data",
    "/// 标准化频率数据": "/// Normalize frequency data",
    "/// 检测振荡器类型": "/// Detect oscillator type",
    "/// 检测融合模式": "/// Detect blending mode",
    "/// 检测音频精度": "/// Detect audio precision",
    "/// Audio 配置文件库": "/// Audio profile library",
    "/// 创建新的库": "/// Create new library",
    "// 常见配置": "// Common configurations",
    "/// 获取配置数": "/// Get configuration count",
    
    # webgl模块
    "//! WebGL 指纹识别模块": "//! WebGL fingerprint recognition module",
    "//! 提供 GPU 和 WebGL 指纹识别能力": "//! Provides GPU and WebGL fingerprint recognition capabilities",
    "/// WebGL 指纹信息": "/// WebGL fingerprint information",
    "/// WebGL 分析器": "/// WebGL analyzer",
    "/// 创建新分析器": "/// Create new analyzer",
    "// 预加载常见 GPU 配置": "// Pre-load common GPU configurations",
    "/// 分析 WebGL 数据": "/// Analyze WebGL data",
    
    # fonts模块
    "//! 字体枚举和指纹识别模块": "//! Font enumeration and fingerprint recognition module",
    "//! 提供字体识别能力，包括：": "//! Provides font recognition capabilities, including:",
    "//! - 系统字体列表枚举": "//! - System font list enumeration",
    "//! - 字体加载时间分析": "//! - Font loading time analysis",
    "//! - 字体渲染特征识别": "//! - Font rendering feature recognition",
    "//! - 子集支持检测": "//! - Subset support detection",
    "/// 字体指纹": "/// Font fingerprint",
    "/// 检测到的系统字体列表": "/// Detected system font list",
    "/// 字体加载时间 (ms)": "/// Font loading time (ms)",
    "/// 独特字体指纹哈希": "/// Unique font fingerprint hash",
    "/// 字体数量": "/// Font count",
    "/// 支持的子集": "/// Supported subsets",
    "/// 渲染特征": "/// Rendering features",
    "/// 字体错误类型": "/// Font error types",
    "/// 无效数据": "/// Invalid data",
    "/// 枚举失败": "/// Enumeration failed",
    "/// 其他错误": "/// Other errors",
    "/// 字体分析器": "/// Font analyzer",
    "/// 分析系统字体": "/// Analyze system fonts",
    "// 转换为字符串向量": "// Convert to string vector",
    "// 计算加载时间": "// Calculate loading time",
    "// 生成唯一哈希": "// Generate unique hash",
    "// 检测子集支持": "// Detect subset support",
    "// 获取渲染特征": "// Get rendering features",
    "/// 计算字体加载时间": "/// Calculate font loading time",
    "// 基于字体名称长度和特征的模拟时间": "// Simulated time based on font name length and characteristics",
    "/// 生成字体哈希": "/// Generate font hash",
    "/// 检测支持的子集": "/// Detect supported subsets",
    "// 基于字体名称检测子集": "// Detect subset based on font name",
    "// 默认子集": "// Default subset",
    "/// 获取渲染特征": "/// Get rendering features",
    "/// 字体系统检测器": "/// Font system detector",
    "/// 检测操作系统字体": "/// Detect operating system fonts",
    
    # webrtc模块
    "//! WebRTC 泄露防护模块": "//! WebRTC leak protection module",
    "//! 提供 WebRTC IP 泄露防护和指纹识别能力": "//! Provides WebRTC IP leak protection and fingerprint recognition capabilities",
    "/// WebRTC 指纹": "/// WebRTC fingerprint",
    "/// 本地 IP 候选地址": "/// Local IP candidate addresses",
    "/// 远程 IP 地址": "/// Remote IP address",
    "/// 连接状态": "/// Connection status",
    "/// mDNS 候选隐藏": "/// mDNS candidate hiding",
    "/// 候选过滤统计": "/// Candidate filtering statistics",
    "/// 新建": "/// New",
    "/// 连接中": "/// Connecting",
    "/// 已连接": "/// Connected",
    "/// 已完成": "/// Completed",
    "/// 断开连接": "/// Disconnected",
    "/// 失败": "/// Failed",
    "/// 已关闭": "/// Closed",
    "/// 候选统计信息": "/// Candidate statistics",
    "/// 主机候选数": "/// Host candidate count",
    "/// srflx 候选数": "/// srflx candidate count",
    "/// prflx 候选数": "/// prflx candidate count",
    "/// relay 候选数": "/// relay candidate count",
    "/// WebRTC 错误类型": "/// WebRTC error types",
    "/// 无效 IP": "/// Invalid IP",
    "/// 分析失败": "/// Analysis failed",
    "/// 其他错误": "/// Other errors",
    "/// WebRTC 分析器": "/// WebRTC analyzer",
    "/// 分析 WebRTC 候选": "/// Analyze WebRTC candidates",
    "/// 提取 IP 地址": "/// Extract IP addresses",
    "/// 验证 IP 地址": "/// Validate IP address",
    "/// WebRTC 防护器": "/// WebRTC protector",
    "/// 隐藏 mDNS 候选": "/// Hide mDNS candidates",
    "/// 伪造 IP 地址": "/// Forge IP addresses",
    "/// 检测 WebRTC 泄露": "/// Detect WebRTC leaks",
    "/// WebRTC 泄露报告": "/// WebRTC leak report",
    "// mDNS地址以.local结尾（RFC 6762)": "// mDNS addresses ending with .local (RFC 6762)",
    "// 真正的mDNS格式": "// Real mDNS format",
    "// 只有第一个非mDNS候选保留": "// Only the first non-mDNS candidate is retained",
}

def translate_file(file_path: str) -> bool:
    """翻译文件中的中文注释"""
    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            content = f.read()
        
        original_content = content
        modified = False
        
        # 应用手动翻译映射
        for chinese, english in MANUAL_TRANSLATIONS.items():
            if chinese in content:
                content = content.replace(chinese, english)
                modified = True
                print(f"  Translated: {chinese[:50]}... -> {english[:50]}...")
        
        if modified:
            with open(file_path, 'w', encoding='utf-8') as f:
                f.write(content)
            print(f"✓ Modified {file_path}")
            return True
            
    except Exception as e:
        print(f"Error processing {file_path}: {e}")
    
    return False

def main():
    """主函数"""
    target_dirs = [
        "crates/fingerprint-ml",
        "crates/fingerprint-defense", 
        "crates/fingerprint-audio",
        "crates/fingerprint-webgl",
        "crates/fingerprint-fonts",
        "crates/fingerprint-webrtc",
        "crates/fingerprint-profiles/tests"
    ]
    
    modified_files = 0
    
    print("Starting manual translation...")
    
    for target_dir in target_dirs:
        if os.path.exists(target_dir):
            print(f"\nProcessing {target_dir}:")
            for root, dirs, files in os.walk(target_dir):
                for file in files:
                    if file.endswith('.rs'):
                        file_path = os.path.join(root, file)
                        if translate_file(file_path):
                            modified_files += 1
    
    print(f"\nTranslation complete!")
    print(f"Modified files: {modified_files}")

if __name__ == "__main__":
    main()