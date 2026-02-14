# fingerprint-api-noise

**版本**: v1.0  
**最后更新**: 2026-02-13  
**文档类型**: 技术文档

---



浏览器 API 噪声注入模块，用于对抗基于 JavaScript 的指纹识别。

## 概述

`fingerprint-api-noise` 模块提供浏览器 API 噪声注入功能，模拟现代浏览器（Safari、Firefox、Brave）的反指纹技术。

## 功能特性

- ✅ **Canvas 指纹噪声**：为 Canvas 数据添加微小的、可重现的噪声
- ✅ **WebGL 参数噪声**：为 WebGL 参数添加细微变化
- ✅ **AudioContext 噪声**：为音频样本添加不可察觉的噪声
- ✅ **字体枚举噪声**：随机化字体列表顺序和数量
- ✅ **屏幕信息噪声**：为屏幕分辨率添加微小变化
- ✅ **Navigator API 噪声**：为硬件信息添加噪声

## 使用场景

- 对抗基于 Canvas 的指纹识别
- 对抗基于 WebGL 的指纹识别
- 对抗基于 AudioContext 的指纹识别
- 增强隐私保护

## 快速开始

在你的 `Cargo.toml` 中添加：

```toml
[dependencies]
fingerprint-api-noise = { path = "crates/fingerprint-api-noise" }
```

### 基本用法

```rust
use fingerprint_api_noise::{ApiNoiseInjector, NoiseConfig};

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
    let canvas_data = vec![255u8; 1000];
    let noisy_canvas = injector.canvas().add_noise(&canvas_data);
    let fingerprint = injector.canvas().fingerprint_hash(&canvas_data);
    
    println!("Canvas 指纹: {}", fingerprint);
    
    // 字体枚举
    let fonts = injector.fonts().get_fonts_with_noise(12345);
    println!("检测到的字体: {:?}", fonts);
}
```

### 使用默认配置

```rust
use fingerprint_api_noise::ApiNoiseInjector;

let injector = ApiNoiseInjector::with_defaults();
let canvas_data = get_canvas_data();
let noisy_data = injector.canvas().add_noise(&canvas_data);
```

## API 文档

### NoiseConfig

配置 API 噪声注入行为。

```rust
pub struct NoiseConfig {
    pub seed: u64,                      // 噪声种子（用于可重现性）
    pub canvas_noise_level: f64,        // Canvas 噪声等级 (0.0 - 1.0)
    pub enable_webgl_noise: bool,       // 启用 WebGL 噪声
    pub enable_audio_noise: bool,       // 启用 Audio 噪声
    pub enable_font_noise: bool,        // 启用字体噪声
}
```

### ApiNoiseInjector

统一的 API 噪声注入器。

```rust
impl ApiNoiseInjector {
    pub fn new(config: NoiseConfig) -> Self;
    pub fn with_defaults() -> Self;
    pub fn canvas(&self) -> &CanvasNoiseInjector;
    pub fn webgl(&self) -> &WebGLNoiseInjector;
    pub fn audio(&self) -> &AudioNoiseInjector;
    pub fn fonts(&self) -> &FontNoiseInjector;
}
```

### CanvasNoiseInjector

Canvas 指纹噪声注入。

```rust
impl CanvasNoiseInjector {
    pub fn new(seed: u64, noise_level: f64) -> Self;
    pub fn add_noise(&self, data: &[u8]) -> Vec<u8>;
    pub fn fingerprint_hash(&self, canvas_data: &[u8]) -> String;
}
```

## 示例

运行示例程序：

```bash
cargo run --example api_noise_demo
```

## 测试

运行测试：

```bash
# 运行单元测试
cargo test -p fingerprint-api-noise

# 运行集成测试
cargo test -p fingerprint-api-noise --test integration_tests
```

## 技术细节

### 噪声特性

- **可重现性**：相同的种子产生相同的噪声
- **微小变化**：噪声足够小，不影响视觉效果
- **真实模拟**：参考真实浏览器的噪声模式

### 性能

- Canvas 噪声注入：< 1ms（1000 字节）
- Audio 噪声注入：< 0.5ms（100 样本）
- 字体枚举：< 0.1ms

## 许可证

BSD-3-Clause

## 贡献

欢迎提交 Issue 和 Pull Request！
