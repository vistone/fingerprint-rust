# fingerprint-audio

音频指纹识别模块，通过分析浏览器的 Web Audio API 特征进行设备和浏览器识别。

## 功能特性

- ✅ Web Audio API 特征提取
- ✅ 音频上下文指纹识别
- ✅ OscillatorNode 特性分析
- ✅ AnalyserNode 频谱分析
- 🔧 可选的音频处理演示

## 快速开始

### 添加到 Cargo.toml

```toml
[dependencies]
fingerprint-audio = { path = "../fingerprint-audio" }
```

### 基本用法

```rust
use fingerprint_audio::AudioFingerprint;

let audio_fp = AudioFingerprint::extract()?;
println!("Audio fingerprint: {:?}", audio_fp.id);
```

## API 概览

### 主要类型

| 类型 | 说明 |
|-----|------|
| `AudioFingerprint` | 音频指纹容器 |
| `AudioContext` | 音频上下文特征 |
| `OscillatorFeatures` | 震荡器特征 |
| `AnalyserFeatures` | 分析器特征 |

## 项目结构

```
src/
├── lib.rs           # 模块入口
├── fingerprint.rs   # 指纹提取
├── context.rs       # 上下文特征
└── oscillator.rs    # 震荡器分析
```

## 使用示例

```rust
use fingerprint_audio::AudioFingerprint;

let fp = AudioFingerprint::extract()?;
let oscillator_value = fp.oscillator_features.value;
let db_values = fp.analyser_features.get_byte_frequency_data();

println!("Oscillator value: {}", oscillator_value);
println!("DB values length: {}", db_values.len());
```

## 许可证

MIT 许可证。详见：[LICENSE](../../LICENSE)

---

**最后更新：** 2026年2月14日
