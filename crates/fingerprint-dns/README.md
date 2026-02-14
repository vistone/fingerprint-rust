# fingerprint-dns

DNS 预解析和特征提取模块，通过 DNS 请求行为进行网络特征分析。

## 功能特性

- ✅ DNS 查询模式分析
- ✅ DNS 服务器配置检测
- ✅ 网络拓扑特征提取
- ✅ DNS 缓存行为分析
- 🔧 可选的 DNS over HTTPS (DoH) 支持

## 快速开始

```rust
use fingerprint_dns::DnsFingerprint;

let dns_fp = DnsFingerprint::extract()?;
println!("DNS servers: {:?}", dns_fp.servers);
```

## API 概览

| 类型 | 说明 |
|-----|------|
| `DnsFingerprint` | DNS 指纹容器 |
| `DnsServer` | DNS 服务器信息 |
| `DnsQueryPattern` | 查询模式 |

## 项目结构

```
src/
├── lib.rs          # 模块入口
├── fingerprint.rs  # 指纹提取
├── servers.rs      # 服务器检测
└── queries.rs      # 查询分析
```

## 许可证

MIT 许可证。详见：[LICENSE](../../LICENSE)

---

**最后更新：** 2026年2月14日
