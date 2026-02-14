# fingerprint-fonts

字体检测模块，通过分析系统中安装的字体进行浏览器和设备识别。

## 功能特性

- ✅ 系统字体枚举
- ✅ 字体渲染差异检测
- ✅ 字宽度测量
- ✅ 字体安装配置分析
- 🔧 可选的高级字体测量

## 快速开始

```rust
use fingerprint_fonts::FontFingerprint;

let fonts_fp = FontFingerprint::extract()?;
println!("Installed fonts: {:?}", fonts_fp.fonts);
```

## API 概览

| 类型 | 说明 |
|-----|------|
| `FontFingerprint` | 字体指纹容器 |
| `Font` | 单个字体信息 |
| `FontMetrics` | 字体度量 |

## 项目结构

```
src/
├── lib.rs          # 模块入口
├── fingerprint.rs  # 指纹提取
├── detection.rs    # 字体检测
└── metrics.rs      # 字体度量
```

## 许可证

MIT 许可证。详见：[LICENSE](../../LICENSE)

---

**最后更新：** 2026年2月14日
