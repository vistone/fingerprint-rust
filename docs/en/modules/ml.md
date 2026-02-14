# ML模块文档

**版本**: v1.0  
**最后更新**: 2026-02-13  
**模块**: fingerprint-ml

---

## 🎯 模块概述

fingerprint-ml 是机器学习指纹匹配模块，提供高级的指纹相似度计算和分类能力，采用三级分层分类器架构实现高准确率的浏览器指纹识别。

## 📦 三级分层分类架构

### Level 1: 浏览器族识别 (11个类别)
**识别目标**: Chrome、Firefox、Safari、Opera、Edge等浏览器族
**准确率**: 99.00%
**特征输入**: JA3、JA4H、JA4T等跨层指纹特征

```rust
use fingerprint_ml::{BrowserFamilyClassifier, FingerprintInput};

let classifier = BrowserFamilyClassifier::new();
let input = FingerprintInput::from_features(&ja3, &ja4h, &ja4t);
let family_result = classifier.classify(&input)?;
println!("Browser Family: {:?} (Confidence: {:.2}%)", 
         family_result.family, family_result.confidence * 100.0);
```

### Level 2: 浏览器版本识别 (100+版本)
**识别目标**: 具体的浏览器版本号
**准确率**: 95.50%
**数据集**: 990个样本（训练792 + 验证99 + 测试99）

```rust
use fingerprint_ml::VersionClassifier;

let version_classifier = VersionClassifier::new();
let version_result = version_classifier.classify_detailed(&family_result)?;
println!("Version: {} (Accuracy: {:.2}%)", 
         version_result.version, version_result.accuracy * 100.0);
```

### Level 3: 浏览器变体识别 (Standard/PSK/PQ)
**识别目标**: 浏览器的安全变体类型
**准确率**: 92.00%
**变体类型**:
- Standard: 标准配置
- PSK: 预共享密钥支持
- PQ: 后量子密码支持

```rust
use fingerprint_ml::VariantClassifier;

let variant_classifier = VariantClassifier::new();
let variant_result = variant_classifier.detect_variant(&version_result)?;
println!("Variant: {:?} (Confidence: {:.2}%)", 
         variant_result.variant, variant_result.confidence * 100.0);
```

## 🔧 核心功能

### 相似度计算
```rust
use fingerprint_ml::{FingerprintMatcher, SimilarityMetric};

let matcher = FingerprintMatcher::new();
let similarity = matcher.calculate_similarity(
    &fingerprint1, 
    &fingerprint2, 
    SimilarityMetric::Cosine
)?;

match similarity {
    s if s > 0.95 => println!("Almost identical fingerprints"),
    s if s > 0.80 => println!("High similarity"),
    s if s > 0.60 => println!("Moderate similarity"),
    _ => println!("Low similarity or different browsers")
}
```

### 风险评分计算
```rust
use fingerprint_ml::RiskAssessor;

let assessor = RiskAssessor::new();
let risk_score = assessor.calculate_risk_score(&fingerprint_analysis)?;
println!("Risk Score: {:.2} (Scale: 0-100)", risk_score);

// 风险等级分类
match risk_score {
    r if r > 80.0 => println!("High Risk - Potential Bot"),
    r if r > 60.0 => println!("Medium Risk - Suspicious"),
    r if r > 40.0 => println!("Low Risk - Normal"),
    _ => println!("Very Low Risk - Trusted")
}
```

### 指纹聚类分析
```rust
use fingerprint_ml::{FingerprintClusterer, ClusteringAlgorithm};

let clusterer = FingerprintClusterer::builder()
    .algorithm(ClusteringAlgorithm::DBSCAN)
    .min_samples(5)
    .epsilon(0.3)
    .build()?;

let clusters = clusterer.cluster_fingerprints(&fingerprint_dataset)?;
println!("Found {} distinct fingerprint clusters", clusters.len());

for (i, cluster) in clusters.iter().enumerate() {
    println!("Cluster {}: {} fingerprints (Centroid similarity: {:.2})", 
             i, cluster.size(), cluster.centroid_similarity());
}
```

## 📊 模型训练与优化

### 训练数据管理
```rust
use fingerprint_ml::{TrainingDataManager, DatasetSplit};

let data_manager = TrainingDataManager::new();
let dataset = data_manager.load_dataset("training_data.json")?;

// 数据集分割
let splits = data_manager.split_dataset(&dataset, DatasetSplit {
    train_ratio: 0.8,
    validation_ratio: 0.1,
    test_ratio: 0.1,
})?;

println!("Training samples: {}", splits.train.len());
println!("Validation samples: {}", splits.validation.len());
println!("Test samples: {}", splits.test.len());
```

### 模型训练
```rust
use fingerprint_ml::{ModelTrainer, TrainingConfig};

let trainer = ModelTrainer::new();
let config = TrainingConfig {
    epochs: 100,
    batch_size: 32,
    learning_rate: 0.001,
    validation_frequency: 10,
};

let training_result = trainer.train_model(&splits.train, &splits.validation, config)?;
println!("Training completed in {:.2}s", training_result.duration.as_secs_f32());
println!("Final accuracy: {:.2}%", training_result.final_accuracy * 100.0);
```

### 模型评估
```rust
use fingerprint_ml::ModelEvaluator;

let evaluator = ModelEvaluator::new();
let evaluation = evaluator.evaluate_model(&trained_model, &splits.test)?;

println!("Test Accuracy: {:.2}%", evaluation.accuracy * 100.0);
println!("Precision: {:.2}%", evaluation.precision * 100.0);
println!("Recall: {:.2}%", evaluation.recall * 100.0);
println!("F1-Score: {:.2}%", evaluation.f1_score * 100.0);
```

## 🎯 高级应用场景

### 实时分类系统
```rust
use fingerprint_ml::{RealTimeClassifier, ClassificationThresholds};

let realtime_classifier = RealTimeClassifier::builder()
    .load_model("production_model.cbm")?
    .set_thresholds(ClassificationThresholds {
        family_confidence: 0.95,
        version_confidence: 0.90,
        variant_confidence: 0.85,
    })
    .enable_batching(true)
    .batch_size(100)
    .build()?;

// 实时处理网络流量
while let Some(flow) = network_capture.next_flow().await? {
    let classification = realtime_classifier.classify_flow(&flow).await?;
    
    match classification.confidence_level {
        ConfidenceLevel::High => {
            log_info!("High confidence classification: {:?}", classification.result);
        }
        ConfidenceLevel::Medium => {
            log_warn!("Medium confidence - manual review recommended");
        }
        ConfidenceLevel::Low => {
            log_alert!("Low confidence - potential unknown fingerprint");
        }
    }
}
```

### 异常检测集成
```rust
use fingerprint_ml::{AnomalyDetector, AnomalyThresholds};

let anomaly_detector = AnomalyDetector::new();
let thresholds = AnomalyThresholds {
    similarity_threshold: 0.7,
    frequency_threshold: 5,
    temporal_threshold: Duration::from_secs(60),
};

let anomalies = anomaly_detector.detect_anomalies(
    &fingerprint_stream, 
    &reference_profiles, 
    thresholds
)?;

for anomaly in anomalies {
    println!("Anomaly detected: {:?}", anomaly);
    println!("Severity: {:?}", anomaly.severity);
    println!("Recommendation: {}", anomaly.recommendation);
}
```

### 模型更新与A/B测试
```rust
use fingerprint_ml::{ModelUpdater, ABTester};

let updater = ModelUpdater::new();
let ab_tester = ABTester::builder()
    .control_model("current_model.cbm")
    .test_model("new_model.cbm")
    .traffic_split(0.5)
    .build()?;

// A/B测试新模型
let test_results = ab_tester.run_test(Duration::from_hours(24)).await?;
if test_results.test_model_performance > test_results.control_model_performance {
    updater.deploy_new_model("new_model.cbm")?;
    println!("New model deployed successfully");
}
```

## 🔧 性能优化

### 模型压缩
```rust
use fingerprint_ml::ModelOptimizer;

let optimizer = ModelOptimizer::new();
let compressed_model = optimizer.compress_model(&trained_model, CompressionLevel::High)?;
println!("Model size reduced by {:.1}%", 
         (1.0 - compressed_model.size() as f64 / trained_model.size() as f64) * 100.0);
```

### 并行处理
```rust
use fingerprint_ml::ParallelProcessor;

let processor = ParallelProcessor::new()
    .num_threads(8)
    .enable_gpu_acceleration(true);

let results = processor.process_batch_parallel(&fingerprint_batch)?;
```

## 🔗 相关模块

- [fingerprint-core](core.md) - 核心数据结构
- [fingerprint-defense](defense.md) - 防护系统集成
- [fingerprint-anomaly](anomaly.md) - 异常检测模块

---
*最后更新: 2026-02-13*