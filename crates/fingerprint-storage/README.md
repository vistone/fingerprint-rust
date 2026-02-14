# fingerprint-storage

存储指纹识别模块，通过分析浏览器的本地存储特性进行识别。

## 功能特性

- ✅ LocalStorage 特征分析
- ✅ SessionStorage 容量检测
- ✅ IndexedDB 行为分析
- ✅ Cookie 策略检测
- ✅ 存储配额估计
- 🔧 可选的隐私模式检测

## 快速开始

```rust
use fingerprint_storage::StorageFingerprint;

let storage_fp = StorageFingerprint::extract()?;
println!("Storage quota: {} bytes", storage_fp.quota);
```

## API 概览

| 类型 | 说明 |
|-----|------|
| `StorageFingerprint` | 存储指纹容器 |
| `StorageQuota` | 存储配额 |
| `StorageFeatures` | 存储特征 |

## 项目结构

```
src/
├── lib.rs          # 模块入口
├── fingerprint.rs  # 指纹提取
├── quota.rs        # 配额检测
└── analysis.rs     # 特征分析
```

## 许可证

MIT 许可证。详见：[LICENSE](../../LICENSE)

---

**最后更新：** 2026年2月14日
