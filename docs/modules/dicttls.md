# DictTLS 实现文档

## 概述

根据 `dicttls` 目录的实现方式，我们创建了对应的 Rust 版本的 TLS 字典模块，提供所有 TLS 相关的常量定义。

## 模块结构

### dicttls/cipher_suites.rs
提供 TLS 密码套件常量，对应 Go 版本的 `dicttls/cipher_suites.go`：
- `TLS_AES_128_GCM_SHA256` (0x1301)
- `TLS_AES_256_GCM_SHA384` (0x1302)
- `TLS_CHACHA20_POLY1305_SHA256` (0x1303)
- `TLS_ECDHE_ECDSA_WITH_AES_128_GCM_SHA256` (0xc02b)
- `TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256` (0xc02f)
- 等等...

### dicttls/signature_schemes.rs
提供 TLS 签名方案常量，对应 Go 版本的 `dicttls/signaturescheme.go`：
- `RSA_PKCS1_SHA256` (0x0401)
- `RSA_PSS_RSAE_SHA256` (0x0804)
- `ECDSA_WITH_P256_AND_SHA256` (0x0403)
- 等等...

还提供了与 Go 版本一致的别名：
- `ECDSA_WITH_P256_AND_SHA256` (对应 `tls.ECDSAWithP256AndSHA256`)
- `PSS_WITH_SHA256` (对应 `tls.PSSWithSHA256`)
- `PKCS1_WITH_SHA256` (对应 `tls.PKCS1WithSHA256`)
- 等等...

### dicttls/supported_groups.rs
提供 TLS 支持的组（椭圆曲线）常量，对应 Go 版本的 `dicttls/supported_groups.go`：
- `SECP256R1` (0x0017)
- `SECP384R1` (0x0018)
- `SECP521R1` (0x0019)
- `X25519` (0x001d)
- `X25519_MLKEM768` (0x6399) - Chrome 133 新增

别名：
- `CURVE_P256` (对应 `tls.CurveP256`)
- `CURVE_P384` (对应 `tls.CurveP384`)
- `X25519` (对应 `tls.X25519`)

### dicttls/extensions.rs
提供 TLS 扩展类型和其他常量：
- 扩展类型：`SERVER_NAME`, `STATUS_REQUEST`, `SUPPORTED_GROUPS` 等
- 压缩方法：`COMPRESSION_NONE`
- 点格式：`POINT_FORMAT_UNCOMPRESSED`
- PSK 模式：`PSK_MODE_DHE`
- TLS 版本：`VERSION_TLS12`, `VERSION_TLS13`
- 证书压缩：`CERT_COMPRESSION_BROTLI`
- 重新协商：`RENEGOTIATE_ONCE_AS_CLIENT`

## 使用方式

```rust
use fingerprint::dicttls::{
    cipher_suites::{TLS_AES_128_GCM_SHA256, GREASE_PLACEHOLDER},
    signature_schemes::{ECDSA_WITH_P256_AND_SHA256, PSS_WITH_SHA256},
    supported_groups::{X25519, CURVE_P256},
    extensions::{VERSION_TLS13, COMPRESSION_NONE},
};

// 在 ClientHelloSpec 中使用
let cipher_suites = vec![
    GREASE_PLACEHOLDER,
    TLS_AES_128_GCM_SHA256,
    // ...
];
```

## 与 Go 版本的对应关系

| Go 版本 | Rust 版本 | 说明 |
|---------|-----------|------|
| `dicttls.TLS_AES_128_GCM_SHA256` | `dicttls::cipher_suites::TLS_AES_128_GCM_SHA256` | 密码套件常量 |
| `tls.ECDSAWithP256AndSHA256` | `dicttls::signature_schemes::ECDSA_WITH_P256_AND_SHA256` | 签名方案（别名） |
| `tls.CurveP256` | `dicttls::supported_groups::CURVE_P256` | 椭圆曲线（别名） |
| `tls.VersionTLS13` | `dicttls::extensions::VERSION_TLS13` | TLS 版本 |
| `tls.CompressionNone` | `dicttls::extensions::COMPRESSION_NONE` | 压缩方法 |

## 数据来源

所有常量值来自 IANA TLS Parameters：
- https://www.iana.org/assignments/tls-parameters/tls-parameters.xhtml
- https://www.iana.org/assignments/tls-extensiontype-values/tls-extensiontype-values.xhtml

最后更新：March 2023（与 Go 版本保持一致）

## 注意事项

1. **GREASE_PLACEHOLDER**: 在 `cipher_suites` 和 `supported_groups` 中都定义了，使用时需要明确指定模块路径或使用别名。

2. **常量值**: 所有常量值都与 Go 版本和 IANA 标准完全一致。

3. **扩展顺序**: TLS 扩展的顺序非常重要，不同浏览器的扩展顺序不同，这是指纹识别的重要特征。
