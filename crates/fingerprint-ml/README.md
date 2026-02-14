# fingerprint-ml

机器学习推断模块，提供预训练的 ML 模型用于指纹分类、异常检测和风险评估。

## 功能特性

- ✅ 预训练的神经网络模型
- ✅ 指纹相似度评估
- ✅ 异常检测推断
- ✅ 风险分数计算
- 🔧 可选的模型热更新
- 🔧 可选的量化推断优化

## 快速开始

```rust
use fingerprint_ml::ModelInference;

let model = ModelInference::load_default()?;
let score = model.predict(&fingerprint)?;
println!("Risk score: {:.2}", score);
```

## API 概览

| 类型 | 说明 |
|-----|------|
| `ModelInference` | 模型推断引擎 |
| `PredictionResult` | 预测结果 |
| `FeatureVector` | 特征向量 |

## 项目结构

```
src/
├── lib.rs          # 模块入口
├── inference.rs    # 推断引擎
├── models.rs       # 模型管理
└── features.rs     # 特征向量化
```

## 模型描述

- **分类模型**：指纹类型分类（真实/模拟/代理）
- **异常检测**：检测可疑指纹
- **风险评分**：综合生成风险评分

## 许可证

MIT 许可证。详见：[LICENSE](../../LICENSE)

---

**最后更新：** 2026年2月14日
