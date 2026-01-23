# API 噪声注入模块

## 概述

`fingerprint-api-noise` 模块提供浏览器 API 噪声注入功能，模拟现代浏览器（Safari、Firefox、Brave）的反指纹技术。

## 背景

2026 年的反指纹技术趋势：
- Safari、Firefox、Brave 对 Canvas/WebGL/Audio API 添加噪声
- 检测系统通过 API 输出的一致性识别伪装
- 需要模拟真实浏览器的 API 噪声模式

## 使用场景

- 对抗基于 Canvas 的指纹识别
- 对抗基于 WebGL 的指纹识别
- 对抗基于 AudioContext 的指纹识别
- 对抗基于字体枚举的指纹识别
- 增强隐私保护

## 快速开始

### 基本用法

```rust
use fingerprint::api_noise::{ApiNoiseInjector, NoiseConfig};

fn main() {
    // 创建噪声注入器
    let config = NoiseConfig {
        seed: 12345,
        canvas_noise_level: 0.15,
        enable_webgl_noise: true,
        enable_audio_noise: true,
        enable_font_noise: true,
    };
    
    let injector = ApiNoiseInjector::new(config);
    
    // Canvas 噪声注入
    let canvas_data = get_canvas_data();
    let noisy_data = injector.canvas().add_noise(&canvas_data);
    
    // 生成带噪声的指纹哈希
    let fingerprint = injector.canvas().fingerprint_hash(&canvas_data);
    println!("Canvas 指纹: {}", fingerprint);
}
```

### 使用默认配置

```rust
use fingerprint::api_noise::ApiNoiseInjector;

let injector = ApiNoiseInjector::with_defaults();
let canvas_data = get_canvas_data();
let noisy_data = injector.canvas().add_noise(&canvas_data);
```

## API 参考

### NoiseConfig

噪声配置结构体，用于自定义噪声行为。

```rust
pub struct NoiseConfig {
    /// 噪声种子（用于可重现的噪声）
    pub seed: u64,
    /// Canvas 噪声等级 (0.0 - 1.0)
    pub canvas_noise_level: f64,
    /// WebGL 噪声启用
    pub enable_webgl_noise: bool,
    /// Audio 噪声启用
    pub enable_audio_noise: bool,
    /// 字体噪声启用
    pub enable_font_noise: bool,
}
```

默认值：
- `seed`: 随机生成
- `canvas_noise_level`: 0.1
- 所有噪声类型：启用

### ApiNoiseInjector

统一的 API 噪声注入器，提供对所有噪声类型的访问。

```rust
impl ApiNoiseInjector {
    /// 使用配置创建新的注入器
    pub fn new(config: NoiseConfig) -> Self;
    
    /// 使用默认配置创建注入器
    pub fn with_defaults() -> Self;
    
    /// 获取 Canvas 噪声注入器
    pub fn canvas(&self) -> &CanvasNoiseInjector;
    
    /// 获取 WebGL 噪声注入器
    pub fn webgl(&self) -> &WebGLNoiseInjector;
    
    /// 获取 Audio 噪声注入器
    pub fn audio(&self) -> &AudioNoiseInjector;
    
    /// 获取字体噪声注入器
    pub fn fonts(&self) -> &FontNoiseInjector;
}
```

### CanvasNoiseInjector

Canvas 指纹噪声注入器。

```rust
impl CanvasNoiseInjector {
    /// 创建新的 Canvas 噪声注入器
    /// 
    /// # 参数
    /// - `seed`: 随机数种子，相同种子产生相同噪声
    /// - `noise_level`: 噪声等级 (0.0-1.0)，推荐 0.1-0.2
    pub fn new(seed: u64, noise_level: f64) -> Self;
    
    /// 为 Canvas 数据添加噪声
    /// 
    /// 对 RGBA 像素数据的每个通道添加 ±1 的微小变化
    pub fn add_noise(&self, data: &[u8]) -> Vec<u8>;
    
    /// 生成带噪声的 Canvas 指纹哈希
    /// 
    /// 返回 SHA256 哈希的十六进制字符串
    pub fn fingerprint_hash(&self, canvas_data: &[u8]) -> String;
}
```

### WebGLNoiseInjector

WebGL 参数噪声注入器。

```rust
impl WebGLNoiseInjector {
    pub fn new() -> Self;
    
    /// 为 WebGL 参数添加噪声
    pub fn add_webgl_noise(&self, params: &WebGLParams) -> WebGLParams;
}

pub struct WebGLParams {
    pub renderer: String,
    pub vendor: String,
    pub aliased_line_width_range: Option<[f32; 2]>,
    pub aliased_point_size_range: Option<[f32; 2]>,
    pub max_texture_size: Option<u32>,
    pub max_viewport_dims: Option<[u32; 2]>,
}
```

### AudioNoiseInjector

AudioContext 指纹噪声注入器。

```rust
impl AudioNoiseInjector {
    pub fn new(seed: u64) -> Self;
    
    /// 为音频样本添加微小噪声
    /// 
    /// 每个样本添加 ±0.0001 范围内的随机值
    pub fn add_audio_noise(&self, samples: &[f32]) -> Vec<f32>;
    
    /// 生成带噪声的音频指纹
    pub fn audio_fingerprint(&self, samples: &[f32]) -> Vec<u8>;
}
```

### FontNoiseInjector

字体枚举噪声注入器。

```rust
impl FontNoiseInjector {
    pub fn new() -> Self;
    
    /// 生成带噪声的字体列表
    /// 
    /// 每次调用返回略有不同的字体顺序和数量
    pub fn get_fonts_with_noise(&self, seed: u64) -> Vec<String>;
}
```

## 示例

### Canvas 指纹噪声

```rust
use fingerprint::api_noise::CanvasNoiseInjector;

let injector = CanvasNoiseInjector::new(12345, 0.15);
let canvas_data = vec![255u8; 1000]; // 模拟 Canvas 数据

let noisy_data = injector.add_noise(&canvas_data);
let fingerprint = injector.fingerprint_hash(&canvas_data);

println!("指纹哈希: {}", fingerprint);
```

### 字体枚举噪声

```rust
use fingerprint::api_noise::FontNoiseInjector;

let injector = FontNoiseInjector::new();

// 第一次枚举
let fonts1 = injector.get_fonts_with_noise(111);
println!("字体列表 1: {:?}", fonts1);

// 第二次枚举（使用不同种子）
let fonts2 = injector.get_fonts_with_noise(222);
println!("字体列表 2: {:?}", fonts2);

// 使用相同种子会得到相同结果
let fonts3 = injector.get_fonts_with_noise(111);
assert_eq!(fonts1, fonts3);
```

### Audio 指纹噪声

```rust
use fingerprint::api_noise::AudioNoiseInjector;

let injector = AudioNoiseInjector::new(12345);
let audio_samples = vec![0.5f32; 100];

let noisy_samples = injector.add_audio_noise(&audio_samples);
let fingerprint = injector.audio_fingerprint(&audio_samples);

println!("音频指纹长度: {} 字节", fingerprint.len());
```

## 技术细节

### 噪声特性

1. **可重现性**
   - 相同的种子产生相同的噪声
   - 适用于需要一致性的场景

2. **微小变化**
   - Canvas: ±1 像素值变化
   - Audio: ±0.0001 幅度变化
   - WebGL: ±0.01 参数变化
   - 不影响视觉或听觉效果

3. **真实模拟**
   - 参考 Safari、Firefox、Brave 的实现
   - 模拟真实浏览器的噪声模式

### 性能特点

| 操作类型 | 数据量 | 处理时间 |
|---------|--------|----------|
| Canvas 噪声 | 1000 字节 | < 1ms |
| Audio 噪声 | 100 样本 | < 0.5ms |
| 字体枚举 | 11 个字体 | < 0.1ms |
| WebGL 参数 | 6 个参数 | < 0.01ms |

### 安全考虑

1. **种子管理**
   - 使用加密安全的随机数生成器生成种子
   - 避免使用可预测的种子值

2. **噪声范围**
   - 噪声应足够小以保持功能性
   - 噪声应足够大以改变指纹

3. **一致性**
   - 在同一会话中保持相同的噪声
   - 避免过于频繁的变化

## 集成指南

### 在主 Crate 中启用

在 `Cargo.toml` 中启用 `api-noise` 功能：

```toml
[dependencies]
fingerprint = { version = "2.1", features = ["api-noise"] }
```

### 在代码中使用

```rust
#[cfg(feature = "api-noise")]
use fingerprint::api_noise::ApiNoiseInjector;

#[cfg(feature = "api-noise")]
fn apply_noise() {
    let injector = ApiNoiseInjector::with_defaults();
    // ... 使用注入器
}
```

## 常见问题

### Q: 噪声会影响正常功能吗？

A: 不会。噪声被设计为足够小，不会影响视觉或听觉效果，只改变指纹特征。

### Q: 如何确保噪声的可重现性？

A: 使用相同的 `seed` 值会产生完全相同的噪声。保存并重用种子即可。

### Q: 噪声等级应该设置为多少？

A: 推荐：
- Canvas: 0.1 - 0.2
- 其他类型使用默认值

### Q: 性能开销是多少？

A: 非常小，通常 < 1ms。对于大多数应用场景几乎无影响。

## 相关资源

- [完整 API 文档](https://docs.rs/fingerprint-api-noise)
- [示例代码](../examples/api_noise_demo.rs)
- [测试用例](../tests/integration_tests.rs)
- [README](../README.md)

## 未来计划

- [ ] 添加更多浏览器 API 支持（Bluetooth, USB 等）
- [ ] 优化噪声算法以更好地模拟真实浏览器
- [ ] 添加自适应噪声等级
- [ ] 支持自定义噪声模式
