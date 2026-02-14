# fingerprint-canvas

Canvas 指纹识别模块，通过分析 HTML5 Canvas API 特征进行浏览器和设备识别。

## 功能特性

- ✅ Canvas 绘制特征提取
- ✅ WebGL 轮廓渲染指纹
- ✅ 文本渲染差异分析
- ✅ 图像数据哈希
- 🔧 可选的扩展纹理支持

## 快速开始

### 添加到 Cargo.toml

```toml
[dependencies]
fingerprint-canvas = { path = "../fingerprint-canvas" }
```

### 基本用法

```rust
use fingerprint_canvas::CanvasFingerprint;

let canvas_fp = CanvasFingerprint::extract()?;
println!("Canvas fingerprint: {}", canvas_fp.hash);
```

## API 概览

| 类型 | 说明 |
|-----|------|
| `CanvasFingerprint` | Canvas 指纹容器 |
| `CanvasData` | 原始 Canvas 数据 |
| `TextRenderingFeatures` | 文本渲染特征 |

## 项目结构

```
src/
├── lib.rs           # 模块入口
├── fingerprint.rs   # 指纹提取
├── rendering.rs     # 渲染特性
└── hash.rs          # 哈希计算
```

## 许可证

MIT 许可证。详见：[LICENSE](../../LICENSE)

---

**最后更新：** 2026年2月14日
