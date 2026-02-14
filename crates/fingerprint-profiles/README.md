# fingerprint-profiles

浏览器配置文件管理模块，维护和管理各种浏览器的指纹特征配置。

## 功能特性

- ✅ 200+ 浏览器配置
- ✅ 版本特定指纹数据
- ✅ 平台和设备配置
- ✅ 动态配置更新
- ✅ 配置验证和冲突检测
- 🔧 可选的自定义配置支持

## 快速开始

```rust
use fingerprint_profiles::BrowserProfiles;

let profiles = BrowserProfiles::load_default()?;
let chrome_fp = profiles.get("Chrome", "120.0")?;
println!("Chrome 120 ID: {}", chrome_fp.id);
```

## API 概览

| 类型 | 说明 |
|-----|------|
| `BrowserProfiles` | 配置管理器 |
| `BrowserProfile` | 单个浏览器配置 |
| `ProfileVersion` | 版本信息 |

## 项目结构

```
src/
├── lib.rs          # 模块入口
├── profiles.rs     # 配置管理
├── loader.rs       # 配置加载
└── database.rs     # 配置数据库
```

## 配置文件

配置文件位于 `data/profiles/`：

```
data/profiles/
├── chrome/
├── firefox/
├── safari/
├── edge/
└── ...
```

## 许可证

MIT 许可证。详见：[LICENSE](../../LICENSE)

---

**最后更新：** 2026年2月14日
