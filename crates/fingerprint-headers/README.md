# fingerprint-headers

HTTP 头部处理和分析模块，用于提取和分析 HTTP 请求头中的浏览器特征。

## 功能特性

- ✅ HTTP 头部规范化
- ✅ User-Agent 解析
- ✅ Accept-Language 分析
- ✅ 头部顺序特征
- ✅ TLS 握手参数提取
- 🔧 可选的高级头部分析

## 快速开始

```rust
use fingerprint_headers::HeaderFingerprint;

let headers_fp = HeaderFingerprint::from_headers(http_headers)?;
println!("User-Agent: {}", headers_fp.user_agent);
```

## API 概览

| 类型 | 说明 |
|-----|------|
| `HeaderFingerprint` | 头部指纹容器 |
| `ParsedHeaders` | 解析的头部 |
| `UserAgent` | User-Agent 信息 |

## 项目结构

```
src/
├── lib.rs          # 模块入口
├── fingerprint.rs  # 指纹提取
├── parser.rs       # 头部解析
└── normalization.rs # 标准化
```

## 许可证

MIT 许可证。详见：[LICENSE](../../LICENSE)

---

**最后更新：** 2026年2月14日
