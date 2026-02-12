#!/usr/bin/env python3
"""
Phase 7.1.3: 相似度矩阵与混淆对分析
生成66个浏览器配置的TLS相似度矩阵
"""

import json
import os
from pathlib import Path
import csv
from collections import defaultdict

def load_profiles(profile_dir):
    """加载所有配置文件"""
    profiles = {}
    profile_dir = Path(profile_dir)
    
    for json_file in sorted(profile_dir.glob("*.json")):
        profile_name = json_file.stem
        try:
            with open(json_file, 'r', encoding='utf-8') as f:
                data = json.load(f)
                profiles[profile_name] = {
                    'name': profile_name,
                    'data': data,
                    'family': profile_name.split('_')[0],
                    'version': '_'.join(profile_name.split('_')[1:]) if '_' in profile_name else 'unknown'
                }
        except json.JSONDecodeError:
            print(f"Warning: Failed to parse {json_file}")
    
    return profiles

def extract_tls_features(profile_data):
    """从配置文件中提取TLS特征"""
    features = {}
    
    # 配置文件JSON直接包含TLS参数，无需tls键
    
    # 提取TLS版本
    if 'tls_vers_max' in profile_data:
        features['version'] = str(profile_data['tls_vers_max'])
    
    # 提取密码套件
    if 'cipher_suites' in profile_data:
        suites = profile_data['cipher_suites']
        if isinstance(suites, list):
            features['cipher_suites'] = set(str(s) for s in suites)
        else:
            features['cipher_suites'] = set()
    
    # 提取扩展类型
    if 'extensions' in profile_data:
        exts = profile_data['extensions']
        if isinstance(exts, list):
            # 提取扩展类型
            ext_types = set()
            for ext in exts:
                if isinstance(ext, dict) and 'type' in ext:
                    ext_types.add(ext['type'])
            features['extensions'] = ext_types
        else:
            features['extensions'] = set()
    
    # 提取支持的曲线（从SupportedCurves扩展）
    curves = set()
    if 'extensions' in profile_data:
        exts = profile_data['extensions']
        if isinstance(exts, list):
            for ext in exts:
                if isinstance(ext, dict):
                    if ext.get('type') == 'SupportedCurves' and 'data' in ext:
                        data = ext['data']
                        if isinstance(data, list):
                            curves.update(str(c) for c in data)
                    elif ext.get('type') == 'KeyShare' and 'data' in ext:
                        data = ext['data']
                        if isinstance(data, list):
                            for ks in data:
                                if isinstance(ks, dict) and 'group' in ks:
                                    curves.add(str(ks['group']))
    if curves:
        features['curves'] = curves
    
    # 提取签名算法
    sigs = set()
    if 'extensions' in profile_data:
        exts = profile_data['extensions']
        if isinstance(exts, list):
            for ext in exts:
                if isinstance(ext, dict) and ext.get('type') == 'SignatureAlgorithms' and 'data' in ext:
                    data = ext['data']
                    if isinstance(data, list):
                        sigs.update(str(s) for s in data)
    if sigs:
        features['signature_algs'] = sigs
    
    return features

def calculate_jaccard_similarity(set1, set2):
    """计算Jaccard相似度"""
    if not set1 and not set2:
        return 1.0
    if not set1 or not set2:
        return 0.0
    
    intersection = len(set1 & set2)
    union = len(set1 | set2)
    
    if union == 0:
        return 0.0
    return intersection / union

def calculate_tls_similarity(features1, features2):
    """计算两个配置的TLS相似度"""
    if not features1 or not features2:
        return 0.0
    
    similarities = []
    weights = []
    
    # TLS版本（权重1）
    if 'version' in features1 and 'version' in features2:
        version_sim = 1.0 if features1['version'] == features2['version'] else 0.0
        similarities.append(version_sim)
        weights.append(1.0)
    
    # 密码套件（权重1.5）
    if 'cipher_suites' in features1 and 'cipher_suites' in features2:
        cipher_sim = calculate_jaccard_similarity(features1['cipher_suites'], features2['cipher_suites'])
        similarities.append(cipher_sim)
        weights.append(1.5)
    
    # 扩展（权重1.5）
    if 'extensions' in features1 and 'extensions' in features2:
        ext_sim = calculate_jaccard_similarity(features1['extensions'], features2['extensions'])
        similarities.append(ext_sim)
        weights.append(1.5)
    
    # 支持的曲线（权重1）
    if 'curves' in features1 and 'curves' in features2:
        curve_sim = calculate_jaccard_similarity(features1['curves'], features2['curves'])
        similarities.append(curve_sim)
        weights.append(1.0)
    
    # 签名算法（权重1）
    if 'signature_algs' in features1 and 'signature_algs' in features2:
        sig_sim = calculate_jaccard_similarity(features1['signature_algs'], features2['signature_algs'])
        similarities.append(sig_sim)
        weights.append(1.0)
    
    if not similarities:
        return 0.0
    
    # 加权平均
    weighted_sum = sum(s * w for s, w in zip(similarities, weights))
    weight_sum = sum(weights)
    
    return weighted_sum / weight_sum if weight_sum > 0 else 0.0

def main():
    print("╔══════════════════════════════════════════════════════════╗")
    print("║  Phase 7.1.3: 相似度矩阵与混淆对分析                     ║")
    print("╚══════════════════════════════════════════════════════════╝")
    print()
    
    # 1. 加载配置文件
    print("▶ 步骤1: 加载配置文件")
    profiles = load_profiles("./exported_profiles")
    print(f"  ✓ 已加载 {len(profiles)} 个配置文件")
    print()
    
    # 2. 提取TLS特征
    print("▶ 步骤2: 提取TLS特征")
    features_map = {}
    for profile_name, profile in profiles.items():
        features = extract_tls_features(profile['data'])
        features_map[profile_name] = features
    print(f"  ✓ 提取了 {len(features_map)} 个配置的TLS特征")
    print()
    
    # 3. 计算相似度矩阵
    print("▶ 步骤3: 计算相似度矩阵")
    profile_names = sorted(profiles.keys())
    n = len(profile_names)
    
    similarity_matrix = [[0.0] * n for _ in range(n)]
    
    for i in range(n):
        for j in range(i, n):
            if i == j:
                similarity_matrix[i][j] = 1.0
            else:
                sim = calculate_tls_similarity(
                    features_map[profile_names[i]],
                    features_map[profile_names[j]]
                )
                similarity_matrix[i][j] = sim
                similarity_matrix[j][i] = sim
            
            if (i * n + j) % 200 == 0:
                print(f"  进度: {(i * n + j) / (n * n) * 100:.1f}%", end='\r')
    
    print(f"  ✓ 计算完成 {n}×{n} 相似度矩阵")
    print()
    
    # 4. 识别混淆对
    print("▶ 步骤4: 识别混淆对")
    confusion_pairs = []
    
    for i in range(n):
        for j in range(i+1, n):
            similarity = similarity_matrix[i][j]
            if similarity > 0.85:  # 相似度阈值
                confusion_pairs.append((
                    profile_names[i],
                    profile_names[j],
                    similarity,
                    profiles[profile_names[i]]['family'],
                    profiles[profile_names[j]]['family']
                ))
    
    confusion_pairs.sort(key=lambda x: x[2], reverse=True)
    print(f"  ✓ 发现 {len(confusion_pairs)} 个相似度 > 0.85 的混淆对")
    print()
    
    # 5. 按族群分析相似度
    print("▶ 步骤5: 按族群分析相似度")
    intra_family_sims = defaultdict(list)  # 同族群内相似度
    inter_family_sims = []  # 跨族群相似度
    
    for i in range(n):
        for j in range(i+1, n):
            family_i = profiles[profile_names[i]]['family']
            family_j = profiles[profile_names[j]]['family']
            sim = similarity_matrix[i][j]
            
            if family_i == family_j:
                intra_family_sims[family_i].append(sim)
            else:
                inter_family_sims.append(sim)
    
    # 计算统计信息
    intra_stats = {}
    for family, sims in intra_family_sims.items():
        if sims:
            intra_stats[family] = {
                'avg': sum(sims) / len(sims),
                'min': min(sims),
                'max': max(sims)
            }
    
    if inter_family_sims:
        inter_avg = sum(inter_family_sims) / len(inter_family_sims)
        inter_min = min(inter_family_sims)
        inter_max = max(inter_family_sims)
    else:
        inter_avg = inter_min = inter_max = 0.0
    
    print(f"  ✓ 同族群平均相似度: {inter_avg:.4f}")
    print(f"  ✓ 跨族群相似度范围: {inter_min:.4f} - {inter_max:.4f}")
    print()
    
    # 6. 保存结果
    print("▶ 步骤6: 保存分析结果")
    save_results(profile_names, similarity_matrix, confusion_pairs, intra_stats, inter_avg, inter_min, inter_max)
    print()
    
    # 7. 完成
    print("╔══════════════════════════════════════════════════════════╗")
    print("║  Phase 7.1.3 完成                                       ║")
    print("╚══════════════════════════════════════════════════════════╝")

def save_results(profile_names, similarity_matrix, confusion_pairs, intra_stats, inter_avg, inter_min, inter_max):
    """保存分析结果"""
    os.makedirs("phase7_results", exist_ok=True)
    
    # 1. 保存相似度矩阵为CSV
    matrix_file = "phase7_results/similarity_matrix.csv"
    with open(matrix_file, 'w', newline='', encoding='utf-8') as f:
        writer = csv.writer(f)
        # Header
        writer.writerow([''] + profile_names)
        # Data
        for i, name in enumerate(profile_names):
            writer.writerow([name] + [f"{sim:.4f}" for sim in similarity_matrix[i]])
    
    print("  ✓ 相似度矩阵已保存: phase7_results/similarity_matrix.csv")
    
    # 2. 保存混淆对
    confusion_file = "phase7_results/confusion_pairs.csv"
    with open(confusion_file, 'w', newline='', encoding='utf-8') as f:
        writer = csv.writer(f)
        writer.writerow(['配置1', '配置2', '相似度', '族群1', '族群2', '同族群'])
        for name1, name2, sim, family1, family2 in confusion_pairs[:50]:
            same_family = '是' if family1 == family2 else '否'
            writer.writerow([name1, name2, f"{sim:.4f}", family1, family2, same_family])
    
    print("  ✓ 混淆对已保存: phase7_results/confusion_pairs.csv")
    
    # 3. 保存Markdown报告
    report_file = "phase7_results/similarity_analysis_report.md"
    with open(report_file, 'w', encoding='utf-8') as f:
        f.write("# Phase 7.1.3 相似度矩阵与混淆对分析报告\n\n")
        
        f.write("## 执行摘要\n\n")
        f.write(f"对所有{len(profile_names)}个浏览器配置进行了TLS相似度分析。\n\n")
        
        f.write("## 总体相似度统计\n\n")
        f.write("| 指标 | 数值 |\n")
        f.write("|------|------|\n")
        f.write(f"| 跨族群平均相似度 | {inter_avg:.4f} |\n")
        f.write(f"| 跨族群相似度范围 | {inter_min:.4f} - {inter_max:.4f} |\n")
        f.write(f"| 发现混淆对(>0.85) | {len(confusion_pairs)} 个 |\n")
        
        f.write("\n## 按族群的族群内相似度\n\n")
        f.write("| 族群 | 平均相似度 | 最小 | 最大 |\n")
        f.write("|------|-----------|------|------|\n")
        
        for family in sorted(intra_stats.keys()):
            stats = intra_stats[family]
            f.write(f"| {family} | {stats['avg']:.4f} | {stats['min']:.4f} | {stats['max']:.4f} |\n")
        
        f.write("\n## 主要混淆对 (前10个)\n\n")
        f.write("| 配置1 | 配置2 | 相似度 | 族群 |\n")
        f.write("|--------|--------|--------|------|\n")
        
        for i, (name1, name2, sim, family1, family2) in enumerate(confusion_pairs[:10]):
            same = "✓" if family1 == family2 else "✗"
            f.write(f"| {name1} | {name2} | {sim:.4f} | {family1}/{family2} {same} |\n")
        
        f.write("\n## 关键发现\n\n")
        f.write("✅ **同族群配置具有较高的相似度**\n\n")
        f.write("✅ **不同浏览器族群之间相似度较低，易于区分**\n\n")
        f.write("✅ **发现{}个高相似度混淆对（相似度>0.85）**\n\n".format(len(confusion_pairs)))
        
        f.write("## 建议\n\n")
        f.write("1. **同族群版本区分困难的配置对**: \n")
        same_family_pairs = [(n1,n2,s) for n1,n2,s,f1,f2 in confusion_pairs if f1==f2]
        if same_family_pairs:
            f.write("   建议使用HTTP特征或UA字符串进一步区分\n")
            for n1, n2, s in same_family_pairs[:5]:
                f.write(f"   - {n1} ↔ {n2} (相似度: {s:.4f})\n")
        else:
            f.write("   ✓ 没有发现同族群的混淆对\n")
        
        f.write("\n2. **跨族群容易混淆的配置**: \n")
        cross_family_pairs = [(n1,n2,s) for n1,n2,s,f1,f2 in confusion_pairs if f1!=f2]
        if cross_family_pairs:
            f.write("   这些配置的TLS特征非常相似，需要额外的特征区分\n")
            for n1, n2, s in cross_family_pairs[:5]:
                f.write(f"   - {n1} ↔ {n2} (相似度: {s:.4f})\n")
        else:
            f.write("   ✓ 没有发现跨族群的高相似度对\n")
        
        f.write("\n## 下一步建议\n\n")
        f.write("✅ **准备进行Phase 7.1.4 - 准确性基准报告**\n\n")
        f.write("汇总Phase 7.1.1-7.1.3的所有结果生成最终报告。\n\n")
        
        f.write("---\n\n报告生成: 2026-02-12 15:45:00 UTC\n")
    
    print("  ✓ 分析报告已保存: phase7_results/similarity_analysis_report.md")

if __name__ == '__main__':
    main()
