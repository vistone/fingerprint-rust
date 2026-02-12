#!/usr/bin/env python3
"""
Phase 7.3: ML Classifier Development
构建3级分层浏览器指纹分类器
- Level 1: 浏览器族群分类 (11类, 目标>99%)
- Level 2: 版本分类 (族群内平均8-20类, 目标>95%)
- Level 3: 变体分类 (3类, 目标>90%)
"""

import os
import json
import numpy as np
import pandas as pd
from pathlib import Path
import pickle
from typing import Tuple, Dict, List
import warnings
warnings.filterwarnings('ignore')

# 机器学习库
from sklearn.preprocessing import StandardScaler, LabelEncoder
from sklearn.ensemble import RandomForestClassifier, GradientBoostingClassifier
from sklearn.metrics import (accuracy_score, precision_score, recall_score, f1_score,
                             confusion_matrix, classification_report, roc_auc_score)
from sklearn.model_selection import cross_val_score, StratifiedKFold
try:
    import xgboost as xgb
    HAS_XGBOOST = True
except ImportError:
    HAS_XGBOOST = False
    print("⚠ XGBoost not available, using RandomForest instead")

from sklearn.pipeline import Pipeline

class BrowserFingerprintClassifier:
    """3级分层浏览器指纹分类器"""
    
    def __init__(self, dataset_dir="./dataset", model_dir="./models"):
        self.dataset_dir = Path(dataset_dir)
        self.model_dir = Path(model_dir)
        os.makedirs(self.model_dir, exist_ok=True)
        
        self.train_df = None
        self.val_df = None
        self.test_df = None
        self.metadata = None
        
        # 模型容器
        self.scaler = None
        self.family_model = None
        self.version_models = {}  # 每个族群一个版本分类器
        self.variant_models = {}  # 每个族群一个变体分类器
        
        # 编码器
        self.family_encoders = {}
        self.version_encoders = {}
        
        # 性能指标
        self.results = {}
        
    def load_data(self):
        """加载训练/验证/测试数据集"""
        print("▶ 加载数据集...")
        
        self.train_df = pd.read_csv(self.dataset_dir / "train_set.csv")
        self.val_df = pd.read_csv(self.dataset_dir / "val_set.csv")
        self.test_df = pd.read_csv(self.dataset_dir / "test_set.csv")
        
        with open(self.dataset_dir / "metadata.json", 'r') as f:
            self.metadata = json.load(f)
        
        print(f"  ✓ 训练集: {len(self.train_df)} 样本")
        print(f"  ✓ 验证集: {len(self.val_df)} 样本")
        print(f"  ✓ 测试集: {len(self.test_df)} 样本")
        print(f"  ✓ 特征维度: {len([c for c in self.train_df.columns if not c.startswith('label_') and not c.startswith('sample_') and c not in ['source_config', 'grease_variant', 'session_id']])}")

    def prepare_features(self):
        """准备特征矩阵"""
        print("\n▶ 准备特征矩阵...")
        
        # 识别特征列（排除标签和元数据）
        exclude_cols = {'sample_id', 'source_config', 'grease_variant', 'session_id',
                       'label_family', 'label_family_name', 'label_version', 
                       'label_minor', 'label_patch', 'label_variant'}
        
        self.feature_cols = [c for c in self.train_df.columns if c not in exclude_cols]
        
        print(f"  ✓ 特征列: {len(self.feature_cols)}")
        
        # 提取特征和标签
        X_train = self.train_df[self.feature_cols].values
        y_family_train = self.train_df['label_family'].values
        y_version_train = self.train_df['label_version'].values
        y_variant_train = self.train_df['label_variant'].values
        
        X_val = self.val_df[self.feature_cols].values
        y_family_val = self.val_df['label_family'].values
        y_version_val = self.val_df['label_version'].values
        y_variant_val = self.val_df['label_variant'].values
        
        X_test = self.test_df[self.feature_cols].values
        y_family_test = self.test_df['label_family'].values
        y_version_test = self.test_df['label_version'].values
        y_variant_test = self.test_df['label_variant'].values
        
        # 标准化特征
        self.scaler = StandardScaler()
        X_train = self.scaler.fit_transform(X_train)
        X_val = self.scaler.transform(X_val)
        X_test = self.scaler.transform(X_test)
        
        return (X_train, y_family_train, y_version_train, y_variant_train,
                X_val, y_family_val, y_version_val, y_variant_val,
                X_test, y_family_test, y_version_test, y_variant_test)

    def train_family_classifier(self, X_train, y_family_train, X_val, y_family_val):
        """训练Level 1: 族群分类器"""
        print("\n▶ 训练 Level 1: 浏览器族群分类器")
        
        # 使用RandomForest (XGBoost API版本可能不兼容，使用稳定的RandomForest)
        self.family_model = RandomForestClassifier(
            n_estimators=200,
            max_depth=8,
            random_state=42,
            n_jobs=-1,
            verbose=0
        )
        self.family_model.fit(X_train, y_family_train)
        
        # 评估
        y_pred_train = self.family_model.predict(X_train)
        y_pred_val = self.family_model.predict(X_val)
        
        acc_train = accuracy_score(y_family_train, y_pred_train)
        acc_val = accuracy_score(y_family_val, y_pred_val)
        
        print(f"  ✓ 训练准确率: {acc_train:.4f}")
        print(f"  ✓ 验证准确率: {acc_val:.4f} (目标: >99%)")
        
        # 特征重要性
        importances = self.family_model.feature_importances_
        top_features = np.argsort(importances)[-10:][::-1]
        print(f"  ✓ 前10重要特征:")
        for idx in top_features:
            print(f"    - {self.feature_cols[idx]}: {importances[idx]:.4f}")
        
        self.results['family_classifier'] = {
            'train_acc': acc_train,
            'val_acc': acc_val,
            'top_features': [self.feature_cols[i] for i in top_features]
        }
        
        return acc_val >= 0.99

    def train_version_classifiers(self, X_train, y_family_train, y_version_train,
                                  X_val, y_family_val, y_version_val):
        """训练Level 2: 版本分类器（每个族群一个）"""
        print("\n▶ 训练 Level 2: 浏览器版本分类器")
        
        unique_families = sorted(set(y_family_train))
        results = {}
        
        for family_id in unique_families:
            # 筛选该族群的样本
            mask_train = y_family_train == family_id
            mask_val = y_family_val == family_id
            
            if mask_train.sum() < 5 or mask_val.sum() < 2:  # 样本不足
                continue
            
            X_train_fam = X_train[mask_train]
            y_version_train_fam = y_version_train[mask_train]
            X_val_fam = X_val[mask_val]
            y_version_val_fam = y_version_val[mask_val]
            
            # 建立版本编码器
            unique_versions = sorted(set(y_version_train_fam))
            if len(unique_versions) <= 1:  # 只有一个版本，跳过
                continue
            
            le = LabelEncoder()
            le.fit(unique_versions)
            self.version_encoders[family_id] = le
            
            y_version_train_fam_encoded = le.transform(y_version_train_fam)
            y_version_val_fam_encoded = le.transform(y_version_val_fam)
            
            # 训练分类器（使用RandomForest）
            model = RandomForestClassifier(
                n_estimators=150,
                max_depth=6,
                random_state=42,
                n_jobs=-1,
                verbose=0
            )
            model.fit(X_train_fam, y_version_train_fam_encoded)
            
            self.version_models[family_id] = model
            
            # 评估
            y_pred_train = model.predict(X_train_fam)
            y_pred_val = model.predict(X_val_fam)
            
            acc_train = accuracy_score(y_version_train_fam_encoded, y_pred_train)
            acc_val = accuracy_score(y_version_val_fam_encoded, y_pred_val)
            
            family_name = self.metadata['family_names'].get(str(family_id), f'family_{family_id}')
            print(f"  ✓ {family_name}: 训练{acc_train:.4f}, 验证{acc_val:.4f} (样本数: {len(unique_versions)})")
            
            results[family_id] = {'train_acc': acc_train, 'val_acc': acc_val, 'n_versions': len(unique_versions)}
        
        self.results['version_classifiers'] = results

    def train_variant_classifiers(self, X_train, y_family_train, y_variant_train,
                                  X_val, y_family_val, y_variant_val):
        """训练Level 3: 变体分类器（每个族群一个）"""
        print("\n▶ 训练 Level 3: 浏览器变体分类器")
        
        unique_families = sorted(set(y_family_train))
        results = {}
        
        for family_id in unique_families:
            # 筛选该族群的样本
            mask_train = y_family_train == family_id
            mask_val = y_family_val == family_id
            
            if mask_train.sum() < 5 or mask_val.sum() < 2:
                continue
            
            X_train_fam = X_train[mask_train]
            y_variant_train_fam = y_variant_train[mask_train]
            X_val_fam = X_val[mask_val]
            y_variant_val_fam = y_variant_val[mask_val]
            
            # 检查是否有足够的变体多样性
            unique_variants = sorted(set(y_variant_train_fam))
            if len(unique_variants) <= 1:  # 只有一种变体，跳过
                continue
            
            # 训练分类器（使用RandomForest）
            model = RandomForestClassifier(
                n_estimators=100,
                max_depth=5,
                random_state=42,
                n_jobs=-1,
                verbose=0
            )
            model.fit(X_train_fam, y_variant_train_fam)
            
            self.variant_models[family_id] = model
            
            # 评估
            y_pred_train = model.predict(X_train_fam)
            y_pred_val = model.predict(X_val_fam)
            
            acc_train = accuracy_score(y_variant_train_fam, y_pred_train)
            acc_val = accuracy_score(y_variant_val_fam, y_pred_val)
            
            family_name = self.metadata['family_names'].get(str(family_id), f'family_{family_id}')
            print(f"  ✓ {family_name}: 训练{acc_train:.4f}, 验证{acc_val:.4f} (变体数: {len(unique_variants)})")
            
            results[family_id] = {'train_acc': acc_train, 'val_acc': acc_val, 'n_variants': len(unique_variants)}
        
        self.results['variant_classifiers'] = results

    def evaluate_on_test_set(self, X_test, y_family_test, y_version_test, y_variant_test):
        """在测试集上进行完整评估"""
        print("\n╔══════════════════════════════════════════════════════════╗")
        print("║  Level 1: 浏览器族群分类器 - 测试集评估                 ║")
        print("╚══════════════════════════════════════════════════════════╝")
        
        y_pred_family = self.family_model.predict(X_test)
        
        accuracy = accuracy_score(y_family_test, y_pred_family)
        precision = precision_score(y_family_test, y_pred_family, average='weighted', zero_division=0)
        recall = recall_score(y_family_test, y_pred_family, average='weighted', zero_division=0)
        f1 = f1_score(y_family_test, y_pred_family, average='weighted', zero_division=0)
        
        print(f"\n总体性能:")
        print(f"  准确率 (Accuracy):  {accuracy:.4f} (目标: >99%)")
        print(f"  精确率 (Precision): {precision:.4f}")
        print(f"  召回率 (Recall):    {recall:.4f}")
        print(f"  F1-Score:          {f1:.4f}")
        
        # 按族群统计
        print(f"\n按浏览器族群的准确率:")
        report = classification_report(y_family_test, y_pred_family, output_dict=True)
        for label_idx, metrics in report.items():
            if label_idx not in ['accuracy', 'macro avg', 'weighted avg']:
                family_name = self.metadata['family_names'].get(str(int(float(label_idx))), f'family_{label_idx}')
                family_id = int(float(label_idx))
                support = int(metrics['support'])
                if support > 0:
                    print(f"  {family_name:<15} {metrics['precision']:.4f} (支持度: {support})")
        
        self.results['test_set_family'] = {
            'accuracy': accuracy,
            'precision': precision,
            'recall': recall,
            'f1': f1
        }
        
        print("\n╔══════════════════════════════════════════════════════════╗")
        print("║  完整3级分层评估 - 测试集                               ║")
        print("╚══════════════════════════════════════════════════════════╝")
        
        # 计算完整管道准确率（3级都正确）
        correct_count = 0
        family_correct = 0
        version_correct = 0
        variant_correct = 0
        
        for i in range(len(X_test)):
            family_pred = y_pred_family[i]
            family_true = y_family_test[i]
            
            # Level 1: 族群
            if family_pred == family_true:
                family_correct += 1
            
            # Level 2: 版本
            if family_pred in self.version_models:
                X_sample = X_test[i:i+1]
                version_pred_encoded = self.version_models[family_pred].predict(X_sample)[0]
                if family_pred in self.version_encoders:
                    version_pred = self.version_encoders[family_pred].inverse_transform([version_pred_encoded])[0]
                    if version_pred == y_version_test[i]:
                        version_correct += 1
            
            # Level 3: 变体
            if family_pred in self.variant_models:
                X_sample = X_test[i:i+1]
                variant_pred = self.variant_models[family_pred].predict(X_sample)[0]
                if variant_pred == y_variant_test[i]:
                    variant_correct += 1
            
            # 完整匹配
            if family_pred == family_true:
                if family_pred in self.version_models:
                    X_sample = X_test[i:i+1]
                    version_pred_encoded = self.version_models[family_pred].predict(X_sample)[0]
                    if family_pred in self.version_encoders:
                        version_pred = self.version_encoders[family_pred].inverse_transform([version_pred_encoded])[0]
                        if version_pred == y_version_test[i]:
                            if family_pred not in self.variant_models or self.variant_models[family_pred].predict(X_sample)[0] == y_variant_test[i]:
                                correct_count += 1
        
        print(f"\n3级分层准确率:")
        print(f"  Level 1 (族群): {family_correct}/{len(X_test)} = {family_correct/len(X_test):.4f}")
        print(f"  Level 2 (版本): {version_correct}/{len(X_test)} = {version_correct/len(X_test):.4f}")
        print(f"  Level 3 (变体): {variant_correct}/{len(X_test)} = {variant_correct/len(X_test):.4f}")
        print(f"  完整匹配:       {correct_count}/{len(X_test)} = {correct_count/len(X_test):.4f}")
        
        self.results['test_set_hierarchical'] = {
            'family_accuracy': family_correct / len(X_test),
            'version_accuracy': version_correct / len(X_test),
            'variant_accuracy': variant_correct / len(X_test),
            'complete_accuracy': correct_count / len(X_test)
        }

    def save_models(self):
        """保存所有模型"""
        print("\n▶ 保存模型...")
        
        # 保存族群分类器
        with open(self.model_dir / "family_model.pkl", 'wb') as f:
            pickle.dump(self.family_model, f)
        
        # 保存版本分类器
        with open(self.model_dir / "version_models.pkl", 'wb') as f:
            pickle.dump(self.version_models, f)
        
        # 保存变体分类器
        with open(self.model_dir / "variant_models.pkl", 'wb') as f:
            pickle.dump(self.variant_models, f)
        
        # 保存编码器
        with open(self.model_dir / "version_encoders.pkl", 'wb') as f:
            pickle.dump(self.version_encoders, f)
        
        # 保存标准化器
        with open(self.model_dir / "scaler.pkl", 'wb') as f:
            pickle.dump(self.scaler, f)
        
        # 保存特征列信息
        with open(self.model_dir / "feature_info.json", 'w') as f:
            json.dump({
                'feature_columns': self.feature_cols,
                'n_features': len(self.feature_cols)
            }, f, indent=2)
        
        print(f"  ✓ 模型已保存到: {self.model_dir}")

    def generate_report(self):
        """生成执行报告"""
        report = "# Phase 7.3 ML分类器训练完成报告\n\n"
        
        report += "## 执行摘要\n\n"
        report += "Phase 7.3 成功构建了3级分层浏览器指纹分类器。\n\n"
        
        report += "## 性能结果\n\n"
        report += "### Level 1: 浏览器族群分类\n"
        fam_results = self.results.get('test_set_family', {})
        report += f"- 准确率: {fam_results.get('accuracy', 0):.4f} (目标: >99%)\n"
        report += f"- 精确率: {fam_results.get('precision', 0):.4f}\n"
        report += f"- 召回率: {fam_results.get('recall', 0):.4f}\n"
        report += f"- F1-Score: {fam_results.get('f1', 0):.4f}\n\n"
        
        report += "### Level 2-3: 版本与变体分类\n"
        report += f"- 版本分类准确率: {self.results.get('test_set_hierarchical', {}).get('version_accuracy', 0):.4f}\n"
        report += f"- 变体分类准确率: {self.results.get('test_set_hierarchical', {}).get('variant_accuracy', 0):.4f}\n"
        report += f"- 完整3级匹配: {self.results.get('test_set_hierarchical', {}).get('complete_accuracy', 0):.4f}\n\n"
        
        report += "## 模型复杂度\n\n"
        report += f"- 族群分类器: 1个\n"
        report += f"- 版本分类器: {len(self.version_models)}个\n"
        report += f"- 变体分类器: {len(self.variant_models)}个\n"
        report += f"- 总特征维度: {len(self.feature_cols)}\n\n"
        
        report += "## 下一步建议\n\n"
        report += "- Phase 7.4: REST API开发 (预计12小时)\n"
        report += "- 部署生产环境\n"
        report += "- 性能监控和持续改进\n"
        
        with open("phase7_results/PHASE_7_3_CLASSIFIER_REPORT.md", 'w') as f:
            f.write(report)
        
        print(f"  ✓ 报告已保存: phase7_results/PHASE_7_3_CLASSIFIER_REPORT.md")

    def run_pipeline(self):
        """执行完整的模型训练管道"""
        print("╔══════════════════════════════════════════════════════════╗")
        print("║  Phase 7.3: 3级分层浏览器指纹分类器训练                   ║")
        print("╚══════════════════════════════════════════════════════════╝")
        print()
        
        # 加载和准备数据
        self.load_data()
        (X_train, y_family_train, y_version_train, y_variant_train,
         X_val, y_family_val, y_version_val, y_variant_val,
         X_test, y_family_test, y_version_test, y_variant_test) = self.prepare_features()
        
        # 训练3级分类器
        family_acc = self.train_family_classifier(X_train, y_family_train, X_val, y_family_val)
        self.train_version_classifiers(X_train, y_family_train, y_version_train,
                                       X_val, y_family_val, y_version_val)
        self.train_variant_classifiers(X_train, y_family_train, y_variant_train,
                                       X_val, y_family_val, y_variant_val)
        
        # 测试集评估
        self.evaluate_on_test_set(X_test, y_family_test, y_version_test, y_variant_test)
        
        # 保存模型
        self.save_models()
        
        # 生成报告
        self.generate_report()
        
        print("\n╔══════════════════════════════════════════════════════════╗")
        print("║  Phase 7.3 Complete - ML分类器就绪                      ║")
        print("╚══════════════════════════════════════════════════════════╝")

if __name__ == '__main__':
    classifier = BrowserFingerprintClassifier()
    classifier.run_pipeline()
