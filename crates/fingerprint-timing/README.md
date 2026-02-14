# fingerprint-timing

时间特征分析模块，通过分析浏览器的计时行为进行识别和分析。

## 功能特性

- ✅ 高精度计时特征提取
- ✅ JavaScript 执行时间分析
- ✅ 系统时钟分辨率检测
- ✅ 性能 API 特征
- ✅ Meltdown/Spectre 缓解检测
- 🔧 可选的定时旁道分析

## 快速开始

```rust
use fingerprint_timing::TimingFingerprint;

let timing_fp = TimingFingerprint::extract()?;
println!("Timing entropy: {:.2}", timing_fp.entropy);
```

## API 概览

| 类型 | 说明 |
|-----|------|
| `TimingFingerprint` | 计时指纹 |
| `TimingFeatures` | 计时特征 |
| `Entropy` | 熵值计算 |

## 项目结构

```
src/
├── lib.rs          # 模块入口
├── fingerprint.rs  # 指纹提取
├── features.rs     # 特征计算
└── entropy.rs      # 熵分析
```

## 许可证

MIT 许可证。详见：[LICENSE](../../LICENSE)

---

**最后更新：** 2026年2月14日
