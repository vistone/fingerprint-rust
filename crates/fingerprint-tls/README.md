# fingerprint-tls

TLS 协议特征提取模块，通过分析 TLS 握手参数进行浏览器和系统识别。

## 功能特性

- ✅ TLS 版本检测
- ✅ 密码套件优先级分析
- ✅ 椭圆曲线参数识别
- ✅ 签名算法特征
- ✅ TLS Extensions 分析
- ✅ JA4 指纹生成
- 🔧 可选的高级分析

## 快速开始

```rust
use fingerprint_tls::TlsFingerprint;

let tls_fp = TlsFingerprint::extract()?;
println!("TLS version: {:?}", tls_fp.version);
println!("JA4: {}", tls_fp.ja4_hash);
```

## API 概览

| 类型 | 说明 |
|-----|------|
| `TlsFingerprint` | TLS 指纹容器 |
| `CipherSuite` | 密码套件 |
| `TlsExtension` | TLS 扩展 |
| `Ja4Fingerprint` | JA4 指纹 |

## 项目结构

```
src/
├── lib.rs          # 模块入口
├── fingerprint.rs  # 指纹提取
├── ja4.rs          # JA4 生成
├── ciphers.rs      # 密码套件分析
└── extensions.rs   # 扩展分析
```

## JA4 指纹

JA4 是一个新的 TLS 指纹化格式，格式为：

```
JA4(TLSVersion,Ciphers,Extensions,EllipticCurves,SignatureAlgorithms)
```

## 许可证

MIT 许可证。详见：[LICENSE](../../LICENSE)

---

**最后更新：** 2026年2月14日
