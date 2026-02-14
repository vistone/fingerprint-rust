# Phase 7.2 执行计划 - 数据集构建与特征工程

## 目标和范围

**主要目标**: 构建990个样本的高质量ML训练数据集，包含50+特征维度

**数据集规模**:
- 浏览器配置数: 66个
- 样本总数: 990个 (15样本/配置)
- 特征维度: 50+
- 标签层级: 3级 (family → version → patch)

---

## 分阶段执行计划

### Stage 1: 数据采样与扩充 (2小时)

**目标**: 从66个已有配置生成990个变异样本

**步骤**:
1. **基础采样** (1小时)
   - 为每个配置生成5个GREASE变体
   - 对每个变体创建3个会话
   - 总计: 66 × 5 × 3 = 990个样本

2. **变异策略** (1小时)
   - GREASE值随机化 (GREASE扩展中的随机值)
   - 密码套件顺序随机打乱
   - 扩展顺序随机重排
   - 伪造HTTP header变体

**输出**:
- `dataset/raw_samples/` (990个JSON样本)
- `dataset/sample_manifest.csv` (样本索引)

---

### Stage 2: 特征提取 (3小时)

**目标**: 从990个样本中提取50+维度特征

**特征分类**:

#### A. TLS基础特征 (12维)
- tls_version (TLS 1.2 / 1.3)
- num_cipher_suites (密码套件数量)
- num_extensions (扩展数量)
- num_curves (支持曲线数)
- num_signature_algs (签名算法数)
- has_alpn (是否支持ALPN)
- has_session_ticket (是否支持SessionTicket)
- has_supported_groups (是否支持SupportedGroups)
- has_key_share (是否支持KeyShare)
- has_psk (是否支持PSK)
- has_early_data (是否支持EarlyData)
- max_fragment_length

#### B. 密码套件特征 (8维)
- cipher_suite_hash (密码套件集合哈希)
- top_cipher_1 (最常见密码套件类型)
- top_cipher_2 (第二常见)
- has_aes_gcm (是否包含AES-GCM)
- has_chacha (是否包含ChaCha20)
- has_ecdhe_ecdsa (是否ECDHE-ECDSA)
- has_ecdhe_rsa (是否ECDHE-RSA)
- has_rsa_pss (是否RSA-PSS)

#### C. 扩展相关特征 (10维)
- extension_set_hash (扩展集合哈希)
- extension_order_hash (扩展顺序哈希)
- has_grease (是否包含GREASE)
- grease_count (GREASE扩展个数)
- grease_positions (GREASE位置编码)
- supported_versions_hash (支持版本集合哈希)
- has_sni (是否SNI)
- has_padding (是否Padding扩展)
- has_ech (是否ECH)
- has_app_layer_proto_nego (是否ALPN/NPN)

#### D. 曲线与签名特征 (8维)
- curve_set_hash (曲线集合哈希)
- has_x25519 (是否X25519)
- has_secp256r1 (是否P-256)
- has_secp384r1 (是否P-384)
- sig_alg_set_hash (签名算法集合哈希)
- sig_alg_ecdsa_sha256 (是否ECDSA-SHA256)
- sig_alg_ecdsa_sha384 (是否ECDSA-SHA384)
- sig_alg_rsa_pss_sha256 (是否RSA-PSS-SHA256)

#### E. 版本标识特征 (8维)
- browser_family (浏览器族群ID: 0-10)
- browser_major_version (主版本号)
- browser_minor_version (次版本号)
- browser_patch_version (补丁版本号)
- is_psk_variant (是否PSK变体)
- is_pq_variant (是否PQ变体)
- os_type (操作系统: 0=Windows, 1=Mac, 2=Linux, 3=iOS, 4=Android)
- device_type (设备类型: 0=Desktop, 1=Mobile, 2=SDK)

#### F. HTTP特征 (6维)
- ua_browser_type (UA字符串中的浏览器类型)
- ua_os_type (UA字符串中的OS)
- ua_version_presence (UA中是否包含版本号)
- http2_pseudo_header_order (HTTP/2伪头顺序)
- http2_regular_header_order (常规头部顺序哈希)
- accept_language_count (Accept-Language数量)

**输出**:
- `dataset/features.csv` (990行 × 52列)
- `dataset/feature_metadata.json` (特征详细说明)

---

### Stage 3: 标签化与数据验证 (2小时)

**目标**: 添加准确的标签，验证数据质量

**标签结构**:

| 列名 | 类型 | 范围 | 说明 |
|------|------|------|------|
| label_family | categorical | 0-10 | chrome, firefox, safari等 |
| label_version | categorical | 0-255 | 主版本号 |
| label_patch | categorical | 0-255 | 补丁版本号 |
| label_variant | categorical | 0-2 | 0=standard, 1=PSK, 2=PQ |
| sample_id | string | - | 样本唯一ID |
| source_config | string | - | 源配置文件 |
| grease_variant | int | 0-5 | GREASE变体序号 |
| session_id | int | 0-2 | 会话序号 |

**数据验证**:
- ✓ 标签完整性检查 (无缺失)
- ✓ 分布均匀性检查 (各族群均衡)
- ✓ 特征异常检测
- ✓ 重复样本检测

**输出**:
- `dataset/labels.csv` (样本标签)
- `dataset/validation_report.md` (质量检查报告)

---

### Stage 4: 数据集整合与打包 (1小时)

**目标**: 生成最终的ML训练数据集

**文件结构**:
```
dataset/
├── 20260213_ml_training_dataset.csv (完整990行)
├── train_set.csv (791行, 80%)
├── val_set.csv (99行, 10%)
├── test_set.csv (99行, 10%)
├── metadata.json (数据集元数据)
├── feature_schema.json (特征定义)
└── README.md (使用说明)
```

**数据分割策略**:
- 训练集 (80%, 792样本): 用于模型训练
- 验证集 (10%, 99样本): 用于超参数调优
- 测试集 (10%, 99样本): 用于最终评估

**元数据包含**:
- 特征列表和类型
- 标签编码映射
- 统计汇总 (均值/方差/分布)
- 数据质量指标

---

## 详细实现步骤

### Step 1: 创建样本生成脚本

**文件**: `scripts/generate_ml_dataset.py`

```python
# 伪代码
import json
import os
from pathlib import Path
import pandas as pd
import numpy as np

def generate_samples():
    """生成990个变异样本"""
    samples = []
    
    for config_file in sorted(Path("exported_profiles").glob("*.json")):
        config = load_json(config_file)
        
        # 为每个配置生成5个GREASE变体
        for grease_idx in range(5):
            # 为每个变体生成3个会话
            for session_idx in range(3):
                sample = {
                    'source_config': config_file.stem,
                    'grease_variant': grease_idx,
                    'session_id': session_idx,
                    'tls_config': apply_variations(config, grease_idx),
                    'http_headers': generate_http_headers(config),
                }
                samples.append(sample)
    
    return samples

def apply_variations(config, grease_idx):
    """应用GREASE和其他变异"""
    varied = copy.deepcopy(config)
    
    # 1. 随机化GREASE值
    for ext in varied['extensions']:
        if ext['type'] == 'GREASE':
            ext['data'] = random_grease_value(grease_idx)
    
    # 2. 随机化密码套件顺序 (保留前3个)
    np.random.shuffle(varied['cipher_suites'][3:])
    
    # 3. 随机化扩展顺序 (保留关键顺序)
    preserve_order = ['SNI', 'ExtendedMasterSecret', 'SupportedCurves']
    randomize_extensions(varied['extensions'], preserve_order)
    
    return varied

# 返回990个样本的JSON列表
```

---

### Step 2: 特征提取引擎

**文件**: `scripts/extract_features.py`

```python
def extract_features_from_sample(sample):
    """从单个样本提取52维特征"""
    
    features = {}
    
    # TLS特征
    features['tls_version'] = extract_tls_version(sample)
    features['num_cipher_suites'] = len(sample['cipher_suites'])
    features['num_extensions'] = len(sample['extensions'])
    # ... 更多特征
    
    # Hash特征 (集合作为特征)
    features['cipher_suite_hash'] = hash_feature(sample['cipher_suites'])
    features['extension_set_hash'] = hash_feature(extract_ext_types(sample))
    features['curve_set_hash'] = hash_feature(extract_curves(sample))
    # ... 更多hash特征
    
    # 版本特征
    browser_family, version = parse_config_name(sample['source_config'])
    features['browser_family'] = FAMILY_MAP[browser_family]
    features['browser_major_version'] = version.split('.')[0]
    features['browser_minor_version'] = version.split('.')[1] if '.' in version else 0
    
    return features

def hash_feature(items):
    """将集合转换为数值特征"""
    return hash(frozenset(items)) % (2**31)
```

---

### Step 3: 标签生成与验证

**文件**: `scripts/label_dataset.py`

```python
def create_labels(samples, config_mapping):
    """为990个样本创建标签"""
    
    labels = []
    
    for idx, sample in enumerate(samples):
        config_name = sample['source_config']
        
        # 从配置名解析标签
        # E.g.: "chrome_103" → family=chrome, version=103
        parts = config_name.rsplit('_', 1)
        family = parts[0]
        version = parts[1] if len(parts) > 1 else "0"
        
        label = {
            'sample_id': f"sample_{idx:04d}",
            'source_config': config_name,
            'label_family': FAMILY_MAP[family],
            'label_version': int(version.split('.')[0]),
            'label_patch': int(version.split('.')[1]) if '.' in version else 0,
            'label_variant': detect_variant(config_name),  # PSK, PQ等
            'grease_variant': sample['grease_variant'],
            'session_id': sample['session_id'],
        }
        
        labels.append(label)
    
    return labels

def validate_dataset(features_df, labels_df):
    """验证数据质量"""
    
    assert len(features_df) == len(labels_df) == 990
    assert features_df.isnull().sum().sum() == 0  # 无缺失值
    
    # 检查标签分布
    family_counts = labels_df['label_family'].value_counts()
    print(f"Family distribution:\n{family_counts}")
    
    # 统计汇总
    stats = {
        'total_samples': 990,
        'feature_columns': 52,
        'unique_families': labels_df['label_family'].nunique(),
        'unique_versions': labels_df['label_version'].nunique(),
    }
    
    return stats
```

---

### Step 4: 数据集打包

**文件**: `scripts/package_dataset.py`

```python
def create_final_dataset():
    """整合所有部分生成最终数据集"""
    
    # 1. 加载特征和标签
    features_df = pd.read_csv('dataset/features.csv')
    labels_df = pd.read_csv('dataset/labels.csv')
    
    # 2. 合并
    dataset = pd.concat([features_df, labels_df], axis=1)
    
    # 3. 数据分割: 80-10-10
    # 保证每个浏览器族群在三个集合中都有代表
    
    train, val, test = stratified_split(dataset, train=0.8, val=0.1, test=0.1)
    
    # 4. 保存
    train.to_csv('dataset/train_set.csv', index=False)
    val.to_csv('dataset/val_set.csv', index=False)
    test.to_csv('dataset/test_set.csv', index=False)
    dataset.to_csv('dataset/20260213_ml_training_dataset.csv', index=False)
    
    # 5. 元数据
    metadata = {
        'version': '1.0.0',
        'created_date': '2026-02-13',
        'total_samples': 990,
        'features': 52,
        'families': 11,
        'train_samples': len(train),
        'val_samples': len(val),
        'test_samples': len(test),
    }
    
    with open('dataset/metadata.json', 'w') as f:
        json.dump(metadata, f, indent=2)
```

---

## 质量检查清单

- [ ] 样本总数: 990 (66 × 15)
- [ ] 每个浏览器配置有15个样本
- [ ] 特征完整: 52维
- [ ] 标签准确: 100% (3级标签)
- [ ] 无缺失值
- [ ] 标签分布均匀 (每个族群150样本)
- [ ] 特征统计合理 (无异常值)
- [ ] 训练/验证/测试分割: 80%/10%/10%
- [ ] 元数据完整
- [ ] 可重现 (记录所有随机种子)

---

## 交付物清单

**代码**:
- [ ] scripts/generate_ml_dataset.py (990个样本生成)
- [ ] scripts/extract_features.py (52维特征提取)
- [ ] scripts/label_dataset.py (标签化和验证)
- [ ] scripts/package_dataset.py (数据集整合)

**数据**:
- [ ] dataset/20260213_ml_training_dataset.csv (990行 × 60列)
- [ ] dataset/train_set.csv (792行)
- [ ] dataset/val_set.csv (99行)
- [ ] dataset/test_set.csv (99行)
- [ ] dataset/metadata.json (数据集元数据)
- [ ] dataset/feature_schema.json (特征定义)

**文档**:
- [ ] dataset/README.md (数据集使用说明)
- [ ] PHASE_7_2_EXECUTION_REPORT.md (执行总结)

---

## 时间表

| 阶段 | 工作内容 | 预计时间 | 状态 |
|------|---------|---------|------|
| Stage 1 | 样本生成与扩充 | 2小时 | ⏳ 准备 |
| Stage 2 | 特征提取 | 3小时 | ⏳ 准备 |
| Stage 3 | 标签化与验证 | 2小时 | ⏳ 准备 |
| Stage 4 | 数据集打包 | 1小时 | ⏳ 准备 |
| **总计** | **数据集构建** | **8小时** | ⏳ 计划中 |

---

## 下一阶段入口

**Phase 7.3: ML分类器开发** (16小时)

**输入**:
- 990个样本的完整数据集
- 52维特征
- 3级标签 (family, version, variant)

**目标**:
- 实现浏览器族群分类器 (>99% 准确率)
- 实现版本分类器 (>95% 准确率)
- 实现变体分类器 (>90% 准确率)

---

**报告生成时间**: 2026-02-12 17:45:00 UTC  
**Phase**: 7.2 Planning
**状态**: 📋 Ready to Execute
