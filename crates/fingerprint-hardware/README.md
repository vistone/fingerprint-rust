# fingerprint-hardware

硬件指纹识别模块，通过分析设备硬件特性进行设备类型和规格识别。

## 功能特性

- ✅ CPU 核心数检测
- ✅ 内存大小估计
- ✅ 屏幕分辨率和 DPI
- ✅ GPU 特性分析
- ✅ 电池续航能力检测
- 🔧 可选的硬件性能基准测试

## 快速开始

```rust
use fingerprint_hardware::HardwareFingerprint;

let hw_fp = HardwareFingerprint::extract()?;
println!("CPU cores: {}", hw_fp.cpu_cores);
println!("Screen: {}x{}", hw_fp.screen_width, hw_fp.screen_height);
```

## API 概览

| 类型 | 说明 |
|-----|------|
| `HardwareFingerprint` | 硬件指纹容器 |
| `CpuInfo` | CPU 信息 |
| `GpuInfo` | GPU 信息 |
| `ScreenInfo` | 屏幕信息 |

## 项目结构

```
src/
├── lib.rs          # 模块入口
├── fingerprint.rs  # 指纹提取
├── cpu.rs          # CPU 检测
├── gpu.rs          # GPU 检测
└── screen.rs       # 屏幕检测
```

## 许可证

MIT 许可证。详见：[LICENSE](../../LICENSE)

---

**最后更新：** 2026年2月14日
