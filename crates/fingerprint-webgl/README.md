# fingerprint-webgl

WebGL 指纹识别模块，通过分析 WebGL API 特性进行 GPU 和驱动程序识别。

## 功能特性

- ✅ WebGL 扩展列表提取
- ✅ GPU 供应商识别
- ✅ 渲染器信息分析
- ✅ GLSL 编译特征
- ✅ 纹理格式支持检测
- 🔧 可选的高级 GPU 分析

## 快速开始

```rust
use fingerprint_webgl::WebGlFingerprint;

let webgl_fp = WebGlFingerprint::extract()?;
println!("GPU: {}", webgl_fp.renderer);
println!("Extensions: {:?}", webgl_fp.extensions);
```

## API 概览

| 类型 | 说明 |
|-----|------|
| `WebGlFingerprint` | WebGL 指纹容器 |
| `GpuInfo` | GPU 信息 |
| `WebGlExtension` | WebGL 扩展 |

## 项目结构

```
src/
├── lib.rs          # 模块入口
├── fingerprint.rs  # 指纹提取
├── gpu.rs          # GPU 识别
└── extensions.rs   # 扩展分析
```

## 许可证

MIT 许可证。详见：[LICENSE](../../LICENSE)

---

**最后更新：** 2026年2月14日
