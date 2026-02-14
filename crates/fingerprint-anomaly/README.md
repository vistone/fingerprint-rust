# fingerprint-anomaly

异常检测模块，用于识别和分析指纹数据中的异常模式，提供风险评估和威胁检测。

## 功能特性

- ✅ 实时异常检测算法
- ✅ 多维度特征异常分析
- ✅ 风险等级评估
- ✅ 异常模式识别
- 🔧 可选的机器学习模型支持
- 🔧 可选的时间序列异常检测

## 快速开始

### 添加到 Cargo.toml

```toml
[dependencies]
fingerprint-anomaly = { path = "../fingerprint-anomaly" }
```

### 基本用法

```rust
use fingerprint_anomaly::{AnomalyDetector, Fingerprint};

let detector = AnomalyDetector::new();
let report = detector.analyze(&fingerprint)?;

if report.is_anomalous {
    println!("Anomaly detected! Risk Level: {:?}", report.risk_level);
}
```

## API 概览

### 主要类型

| 类型 | 说明 |
|-----|------|
| `AnomalyDetector` | 异常检测器 |
| `AnomalyReport` | 检测报告 |
| `RiskLevel` | 风险等级 |
| `AnomalyPattern` | 异常模式 |

### 主要方法

| 方法 | 说明 |
|-----|------|
| `analyze(fingerprint)` | 执行异常检测 |
| `get_risk_level(fingerprint)` | 计算风险等级 |
| `detect_patterns(data)` | 检测异常模式 |
| `is_anomalous(fingerprint)` | 判断是否异常 |

## 项目结构

```
src/
├── lib.rs           # 模块入口
├── detector.rs      # 异常检测器实现
├── patterns.rs      # 模式定义
├── features.rs      # 特征提取
└── rules.rs         # 检测规则
```

## 使用示例

```rust
use fingerprint_anomaly::AnomalyDetector;

let detector = AnomalyDetector::new();

// 第一次见到的指纹
let fp1 = get_fingerprint_1();
let report1 = detector.analyze(&fp1)?;
println!("First fingerprint anomaly: {:?}", report1);

// 同一源的指纹
let fp2 = get_fingerprint_2();
let report2 = detector.analyze(&fp2)?;
println!("Risk level changed: {:?}", report2.risk_level);
```

## 依赖关系

| 依赖 | 用途 |
|-----|------|
| `fingerprint-core` | 基础类型 |
| `num-traits` | 数值特征 |
| `ndarray` | 数组操作 |

## 检测规则

- **统计异常**：偏离正常分布
- **行为异常**：不符合已知模式
- **时间异常**：时序数据异常
- **特性异常**：单个特性超出范围

## 许可证

MIT 许可证。详见：[LICENSE](../../LICENSE)

## 相关文档

- [异常检测设计](../../docs/ANOMALY_DETECTION_DESIGN.md)
- [风险模型](../../docs/RISK_MODEL.md)

---

**最后更新：** 2026年2月14日
