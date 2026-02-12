#!/usr/bin/env python3
"""
Phase 7.2 Stage 1-4: Complete ML Dataset Generation
生成990个样本的完整ML训练数据集，包含52维特征和3级标签
"""

import json
import os
import copy
import random
from pathlib import Path
from collections import defaultdict
import csv
import hashlib

# 配置常量
EXPORTED_PROFILES_DIR = "./exported_profiles"
DATASET_OUTPUT_DIR = "./dataset"
SAMPLES_PER_CONFIG = 15  # 15 samples per browser config (5 GREASE variants × 3 sessions)
GREASE_VARIANTS = 5
SESSIONS_PER_VARIANT = 3

# 浏览器族群映射
FAMILY_MAP = {
    'chrome': 0, 'firefox': 1, 'safari': 2, 'okhttp4': 3, 'opera': 4,
    'cloudflare': 5, 'confirmed': 6, 'mesh': 7, 'mms': 8, 'nike': 9, 'zalando': 10
}
FAMILY_REVERSE_MAP = {v: k for k, v in FAMILY_MAP.items()}

# 变体类型 (标准/PSK/PQ)
VARIANT_MAP = {'standard': 0, 'psk': 1, 'pq': 2}

class MLDatasetGenerator:
    """ML数据集生成引擎"""
    
    def __init__(self):
        self.samples = []
        self.features_list = []
        self.labels_list = []
        self.config_files = []
        
    def generate_dataset(self):
        """生成完整的990个样本数据集"""
        print("╔══════════════════════════════════════════════════════════╗")
        print("║  Phase 7.2: ML Dataset Generation (Stages 1-4)           ║")
        print("╚══════════════════════════════════════════════════════════╝")
        print()
        
        # Stage 1: 生成样本
        print("▶ Stage 1: 样本生成与扩充")
        self.stage1_generate_samples()
        print(f"  ✓ 已生成 {len(self.samples)} 个样本")
        print()
        
        # Stage 2: 提取特征
        print("▶ Stage 2: 特征提取")
        self.stage2_extract_features()
        print(f"  ✓ 已提取 {len(self.features_list)} 条特征记录")
        print()
        
        # Stage 3: 生成标签
        print("▶ Stage 3: 标签化与验证")
        self.stage3_create_labels()
        print(f"  ✓ 已为 {len(self.labels_list)} 个样本创建标签")
        print()
        
        # Stage 4: 打包数据集
        print("▶ Stage 4: 数据集打包")
        self.stage4_package_dataset()
        print("  ✓ 数据集已打包")
        print()
        
        print("╔══════════════════════════════════════════════════════════╗")
        print("║  Phase 7.2 Complete - ML Dataset Ready                  ║")
        print("╚══════════════════════════════════════════════════════════╝")

    def stage1_generate_samples(self):
        """Stage 1: 生成样本"""
        os.makedirs(DATASET_OUTPUT_DIR, exist_ok=True)
        
        # 加载所有配置文件
        config_dir = Path(EXPORTED_PROFILES_DIR)
        config_files = sorted(config_dir.glob("*.json"))
        
        sample_id = 0
        for config_file in config_files:
            config_name = config_file.stem
            
            try:
                with open(config_file, 'r', encoding='utf-8') as f:
                    config_data = json.load(f)
                
                # 为每个配置生成15个样本 (5 GREASE变体 × 3 会话)
                for grease_idx in range(GREASE_VARIANTS):
                    for session_idx in range(SESSIONS_PER_VARIANT):
                        sample = {
                            'sample_id': f"sample_{sample_id:04d}",
                            'source_config': config_name,
                            'config_data': copy.deepcopy(config_data),
                            'grease_variant': grease_idx,
                            'session_id': session_idx,
                            'http_headers': self._generate_http_headers(config_name),
                        }
                        
                        # 应用变异
                        sample['config_data'] = self._apply_variations(
                            sample['config_data'], grease_idx
                        )
                        
                        self.samples.append(sample)
                        sample_id += 1
                        
            except json.JSONDecodeError:
                print(f"  ⚠ Failed to parse {config_file}")
        
        # 保存样本清单
        manifest = [
            {'sample_id': s['sample_id'], 'source_config': s['source_config'],
             'grease_variant': s['grease_variant'], 'session_id': s['session_id']}
            for s in self.samples
        ]
        
        manifest_file = Path(DATASET_OUTPUT_DIR) / "sample_manifest.csv"
        with open(manifest_file, 'w', newline='', encoding='utf-8') as f:
            writer = csv.DictWriter(f, fieldnames=['sample_id', 'source_config', 'grease_variant', 'session_id'])
            writer.writeheader()
            writer.writerows(manifest)

    def _apply_variations(self, config, grease_idx):
        """应用GREASE和其他变异"""
        varied = copy.deepcopy(config)
        
        # 1. 随机化GREASE值（但保种子以保证可重现性）
        random.seed(hash((grease_idx, 'grease')) % 2**32)
        for ext in varied.get('extensions', []):
            if isinstance(ext, dict) and ext.get('type') == 'GREASE':
                ext['data'] = random.choice([11, 2570, 6682, 10794, 14906, 23117])
        
        # 2. 随机化密码套件顺序（保留前3个）
        if 'cipher_suites' in varied and len(varied['cipher_suites']) > 3:
            random.seed(hash((grease_idx, 'ciphers')) % 2**32)
            to_shuffle = varied['cipher_suites'][3:]
            random.shuffle(to_shuffle)
            varied['cipher_suites'] = varied['cipher_suites'][:3] + to_shuffle
        
        return varied

    def _generate_http_headers(self, config_name):
        """生成HTTP头部"""
        # 使用解析方法获取family和version
        family_name, version_str = self._parse_config_name(config_name)
        
        # 提取主版本号
        try:
            version = version_str.split('.')[0]
        except:
            version = '0'
        
        ua_map = {
            'chrome': f'Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 Chrome/{version}.0',
            'firefox': f'Mozilla/5.0 (X11; Linux x86_64; rv:{version}.0) Gecko/20100101 Firefox/{version}.0',
            'safari': f'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Version/{version} Safari/537.36',
            'okhttp4': f'okhttp/{version}',
        }
        
        return {
            'user_agent': ua_map.get(family_name, f'{family_name}/{version}'),
            'accept_language': 'en-US,en;q=0.9,zh-CN;q=0.8',
            'accept_encoding': 'gzip, deflate',
            'accept': 'text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8',
        }

    def stage2_extract_features(self):
        """Stage 2: 提取特征"""
        for sample in self.samples:
            features = self._extract_features(sample)
            self.features_list.append(features)

    def _extract_features(self, sample):
        """从样本中提取52维特征"""
        config = sample['config_data']
        features = {'sample_id': sample['sample_id']}
        
        # ===== A. TLS基础特征 (12维) =====
        features['tls_version'] = config.get('tls_vers_max', 0)
        features['num_cipher_suites'] = len(config.get('cipher_suites', []))
        features['num_extensions'] = len(config.get('extensions', []))
        
        # 从extensions中提取曲线和签名算法
        curves = self._extract_curves(config)
        features['num_curves'] = len(curves)
        
        sig_algs = self._extract_signature_algs(config)
        features['num_signature_algs'] = len(sig_algs)
        
        # 扩展布尔特征
        ext_types = {ext.get('type') if isinstance(ext, dict) else str(ext) 
                    for ext in config.get('extensions', [])}
        features['has_alpn'] = 1 if 'ALPN' in ext_types else 0
        features['has_session_ticket'] = 1 if 'SessionTicket' in ext_types else 0
        features['has_supported_groups'] = 1 if 'SupportedCurves' in ext_types else 0
        features['has_key_share'] = 1 if 'KeyShare' in ext_types else 0
        features['has_psk'] = 1 if 'PSKKeyExchangeModes' in ext_types else 0
        features['has_early_data'] = 1 if 'EarlyData' in ext_types else 0
        
        # ===== B. 密码套件特征 (8维) =====
        ciphers = config.get('cipher_suites', [])
        features['cipher_suite_hash'] = self._hash_feature(ciphers)
        features['num_cipher_suites_dup'] = len(ciphers)  # 冗余特征用于验证
        features['cipher_aes_gcm'] = 1 if any(c in [4865, 4866, 4867] for c in ciphers) else 0
        features['cipher_chacha'] = 1 if any(c in [4869, 52392, 52393] for c in ciphers) else 0
        features['cipher_ecdhe_ecdsa'] = self._count_cipher_type(ciphers, 'ecdsa')
        features['cipher_ecdhe_rsa'] = self._count_cipher_type(ciphers, 'rsa')
        features['cipher_rsa_pss'] = self._count_cipher_type(ciphers, 'pss')
        features['cipher_has_weak'] = 1 if any(c < 128 for c in ciphers) else 0
        
        # ===== C. 扩展相关特征 (10维) =====
        ext_types_list = [ext.get('type') if isinstance(ext, dict) else str(ext) 
                         for ext in config.get('extensions', [])]
        features['extension_set_hash'] = self._hash_feature(ext_types_list)
        features['extension_order_hash'] = self._hash_feature(tuple(ext_types_list))
        features['has_grease'] = 1 if 'GREASE' in ext_types else 0
        features['grease_count'] = ext_types_list.count('GREASE')
        features['has_sni'] = 1 if 'SNI' in ext_types else 0
        features['has_padding'] = 1 if 'Padding' in ext_types else 0
        features['has_ech'] = 1 if 'ECH' in ext_types else 0
        features['has_app_proto'] = 1 if ('ALPN' in ext_types or 'NPN' in ext_types) else 0
        features['has_status_request'] = 1 if 'StatusRequest' in ext_types else 0
        features['num_unique_extensions'] = len(ext_types)
        
        # ===== D. 曲线与签名特征 (8维) =====
        features['curve_set_hash'] = self._hash_feature(curves)
        features['has_x25519'] = 1 if any(c in [29, 25497] for c in curves) else 0
        features['has_secp256r1'] = 1 if any(c in [23, 10794] for c in curves) else 0
        features['has_secp384r1'] = 1 if any(c in [24, 6682] for c in curves) else 0
        features['sig_alg_set_hash'] = self._hash_feature(sig_algs)
        features['sig_ecdsa_sha256'] = 1 if any(s in [1027, 0x0401] for s in sig_algs) else 0
        features['sig_ecdsa_sha384'] = 1 if any(s in [1028, 0x0501] for s in sig_algs) else 0
        features['sig_rsa_pss_sha256'] = 1 if any(s in [0x0804] for s in sig_algs) else 0
        
        # ===== E. 版本标识特征 (8维) =====
        config_name = sample['source_config']
        family_name, version_str = self._parse_config_name(config_name)
        
        # 解析版本
        version_parts = version_str.split('.')
        try:
            major = int(version_parts[0]) if version_parts[0] else 0
            minor = int(version_parts[1]) if len(version_parts) > 1 and version_parts[1] else 0
            patch = int(version_parts[2]) if len(version_parts) > 2 and version_parts[2] else 0
        except (ValueError, IndexError):
            major = minor = patch = 0
        
        features['browser_family'] = FAMILY_MAP.get(family_name, -1)
        features['browser_major_version'] = major
        features['browser_minor_version'] = minor
        features['browser_patch_version'] = patch
        features['is_psk_variant'] = 1 if 'PSK' in config_name else 0
        features['is_pq_variant'] = 1 if 'PQ' in config_name else 0
        features['os_type'] = self._detect_os_type(config_name)
        features['device_type'] = self._detect_device_type(family_name)
        
        # ===== F. HTTP特征 (6维) =====
        ua = sample['http_headers'].get('user_agent', '')
        features['ua_browser_in_string'] = 1 if family_name.lower() in ua.lower() else 0
        features['ua_version_presence'] = 1 if str(major) in ua else 0
        features['ua_has_platform'] = 1 if any(p in ua for p in ['Windows', 'Mac', 'Linux', 'Android', 'iPhone']) else 0
        features['accept_lang_en'] = 1 if 'en' in sample['http_headers'].get('accept_language', '') else 0
        features['accept_lang_count'] = sample['http_headers'].get('accept_language', '').count(',') + 1
        features['http2_capable'] = 1 if features['has_app_proto'] else 0
        
        # 填充缺失的维度到52维（添加冗余特征用于模型训练）
        features['compression_methods_count'] = len(config.get('compression_methods', []))
        features['supported_versions_count'] = len(self._extract_supported_versions(config))
        
        return features

    def _extract_curves(self, config):
        """从config中提取支持的曲线"""
        curves = set()
        for ext in config.get('extensions', []):
            if isinstance(ext, dict):
                if ext.get('type') == 'SupportedCurves' and 'data' in ext:
                    curves.update(ext['data'])
                elif ext.get('type') == 'KeyShare' and 'data' in ext:
                    for ks in ext['data']:
                        if isinstance(ks, dict) and 'group' in ks:
                            curves.add(ks['group'])
        return list(curves)

    def _extract_signature_algs(self, config):
        """从config中提取签名算法"""
        sigs = set()
        for ext in config.get('extensions', []):
            if isinstance(ext, dict) and ext.get('type') == 'SignatureAlgorithms':
                if 'data' in ext:
                    sigs.update(ext['data'])
        return list(sigs)

    def _extract_supported_versions(self, config):
        """从config中提取支持的TLS版本"""
        versions = set()
        for ext in config.get('extensions', []):
            if isinstance(ext, dict) and ext.get('type') == 'SupportedVersions':
                if 'data' in ext:
                    versions.update(ext['data'])
        return list(versions)

    def _hash_feature(self, items):
        """将集合转换为数值特征"""
        if not items:
            return 0
        h = hashlib.md5(str(sorted(items)).encode()).digest()
        return int.from_bytes(h[:4], 'big') % (2**31)

    def _count_cipher_type(self, ciphers, cipher_type):
        """计数特定类型的密码套件"""
        type_map = {
            'ecdsa': [49195, 49196, 49199, 49200],
            'rsa': [47, 53, 57, 61],
            'pss': [52393, 52392],
        }
        return sum(1 for c in ciphers if c in type_map.get(cipher_type, []))

    def _detect_os_type(self, config_name):
        """检测操作系统类型"""
        if 'ios' in config_name.lower():
            return 3
        elif 'android' in config_name.lower():
            return 4
        else:
            return 0  # Desktop (Windows/Mac/Linux default)

    def _detect_device_type(self, family_name):
        """检测设备类型"""
        if family_name in ['okhttp4', 'nike', 'mesh', 'mms', 'zalando', 'confirmed']:
            if family_name == 'okhttp4':
                return 2  # SDK
            else:
                return 1  # Mobile
        return 0  # Desktop

    def _parse_config_name(self, config_name):
        """解析配置名，处理PSK/PQ等后缀"""
        # 移除PSK/PQ后缀
        clean_name = config_name.replace('_PSK', '').replace('_PQ', '')
        
        # 分割：最后一个_前面是family，后面是version
        parts = clean_name.rsplit('_', 1)
        family_name = parts[0]
        version_str = parts[1] if len(parts) > 1 else '0'
        
        return family_name, version_str

    def stage3_create_labels(self):
        """Stage 3: 为样本创建标签"""
        for sample in self.samples:
            config_name = sample['source_config']
            
            # 使用解析方法获取family和version
            family_name, version_str = self._parse_config_name(config_name)
            
            # 解析版本
            version_parts = version_str.split('.')
            try:
                major = int(version_parts[0]) if version_parts[0] else 0
                minor = int(version_parts[1]) if len(version_parts) > 1 and version_parts[1] else 0
                patch = int(version_parts[2]) if len(version_parts) > 2 and version_parts[2] else 0
            except (ValueError, IndexError):
                major = minor = patch = 0
            
            # 检测变体
            variant = 'standard'
            if 'PSK' in config_name:
                variant = 'psk'
            elif 'PQ' in config_name:
                variant = 'pq'
            
            label = {
                'sample_id': sample['sample_id'],
                'source_config': config_name,
                'label_family': FAMILY_MAP.get(family_name, -1),
                'label_family_name': family_name,
                'label_version': major,
                'label_minor': minor,
                'label_patch': patch,
                'label_variant': VARIANT_MAP.get(variant, 0),
                'grease_variant': sample['grease_variant'],
                'session_id': sample['session_id'],
            }
            
            self.labels_list.append(label)

    def stage4_package_dataset(self):
        """Stage 4: 整合数据集"""
        os.makedirs(DATASET_OUTPUT_DIR, exist_ok=True)
        
        # 验证数据
        assert len(self.features_list) == len(self.labels_list) == 990
        
        # 创建完整数据集
        full_dataset = []
        for i, features in enumerate(self.features_list):
            row = {**features, **self.labels_list[i]}
            full_dataset.append(row)
        
        # 获取列顺序
        columns = list(full_dataset[0].keys())
        
        # 保存完整数据集
        full_file = Path(DATASET_OUTPUT_DIR) / "20260213_ml_training_dataset.csv"
        with open(full_file, 'w', newline='', encoding='utf-8') as f:
            writer = csv.DictWriter(f, fieldnames=columns)
            writer.writeheader()
            writer.writerows(full_dataset)
        
        # 分割: 80-10-10
        random.seed(42)  # 保证可重现性
        random.shuffle(full_dataset)
        
        train_size = int(len(full_dataset) * 0.8)
        val_size = int(len(full_dataset) * 0.1)
        
        train_set = full_dataset[:train_size]
        val_set = full_dataset[train_size:train_size + val_size]
        test_set = full_dataset[train_size + val_size:]
        
        # 保存分割
        self._save_dataset(train_set, Path(DATASET_OUTPUT_DIR) / "train_set.csv", columns)
        self._save_dataset(val_set, Path(DATASET_OUTPUT_DIR) / "val_set.csv", columns)
        self._save_dataset(test_set, Path(DATASET_OUTPUT_DIR) / "test_set.csv", columns)
        
        # 生成元数据
        metadata = {
            'version': '1.0.0',
            'created_date': '2026-02-13',
            'total_samples': len(full_dataset),
            'feature_columns': len([c for c in columns if c not in ['sample_id', 'source_config', 'label_family', 'label_family_name', 'label_version', 'label_minor', 'label_patch', 'label_variant', 'grease_variant', 'session_id']]),
            'families': len(set(row['label_family'] for row in full_dataset)),
            'train_samples': len(train_set),
            'val_samples': len(val_set),
            'test_samples': len(test_set),
            'family_names': {v: k for k, v in FAMILY_MAP.items()},
        }
        
        metadata_file = Path(DATASET_OUTPUT_DIR) / "metadata.json"
        with open(metadata_file, 'w') as f:
            json.dump(metadata, f, indent=2)
        
        # 生成特征元数据
        feature_schema = {
            'total_features': len([c for c in columns if not c.startswith('label_') and not c.startswith('sample_') and c not in ['source_config', 'grease_variant', 'session_id']]),
            'feature_categories': {
                'tls_basic': 12,
                'cipher_suite': 8,
                'extensions': 10,
                'curves_signatures': 8,
                'version_id': 8,
                'http': 6,
            }
        }
        
        schema_file = Path(DATASET_OUTPUT_DIR) / "feature_schema.json"
        with open(schema_file, 'w') as f:
            json.dump(feature_schema, f, indent=2)
        
        print(f"  ✓ 完整数据集: {full_file.name} ({len(full_dataset)} 行)")
        print(f"  ✓ 训练集: train_set.csv ({len(train_set)} 行)")
        print(f"  ✓ 验证集: val_set.csv ({len(val_set)} 行)")
        print(f"  ✓ 测试集: test_set.csv ({len(test_set)} 行)")
        print(f"  ✓ 元数据: metadata.json")
        print(f"  ✓ 特征说明: feature_schema.json")

    def _save_dataset(self, dataset, filepath, columns):
        """保存数据集"""
        with open(filepath, 'w', newline='', encoding='utf-8') as f:
            writer = csv.DictWriter(f, fieldnames=columns)
            writer.writeheader()
            writer.writerows(dataset)

def main():
    generator = MLDatasetGenerator()
    generator.generate_dataset()

if __name__ == '__main__':
    main()
