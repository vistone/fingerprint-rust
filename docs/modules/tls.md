# TLS模块文档

**版本**: v1.0  
**最后更新**: 2026-02-13  
**模块**: fingerprint-tls

---

## 🎯 模块概述

fingerprint-tls 是指纹识别系统的核心TLS处理模块，负责TLS配置、扩展和握手处理，提供完整的TLS 1.3兼容实现。

## 📦 主要组件

### 1. TLS配置 (tls_config)
处理TLS客户端配置，包括：
- 密码套件选择和配置
- 协议版本管理（TLS 1.2/1.3）
- 证书验证设置
- 会话恢复配置

### 2. TLS扩展 (tls_extensions)
管理TLS扩展字段，支持：
- Server Name Indication (SNI)
- Application-Layer Protocol Negotiation (ALPN)
- Extended Master Secret
- 各种自定义扩展字段
- GREASE值处理

### 3. TLS握手 (tls_handshake)
实现TLS握手过程，包括：
- ClientHello消息构造
- 服务器响应处理
- 密钥交换协商
- 握手完成验证
- 真实密钥对生成（X25519, P-256, P-384）

## 🔧 核心功能

### TLS指纹生成
```rust
use fingerprint_tls::{TLSConfig, TLSHandshakeBuilder};

let config = TLSConfig::builder()
    .with_cipher_suites(vec![
        "TLS_AES_128_GCM_SHA256",
        "TLS_AES_256_GCM_SHA384",
        "TLS_CHACHA20_POLY1305_SHA256"
    ])
    .with_extensions(vec![
        "server_name",
        "extended_master_secret",
        "renegotiation_info"
    ])
    .build()?;

let handshake = TLSHandshakeBuilder::new()
    .with_config(config)
    .build_client_hello()?;
```

### 扩展字段处理
```rust
use fingerprint_tls::{TLSServerName, TLSALPN, TLSExtension};

// SNI扩展
let sni = TLSServerName::new("example.com");

// ALPN扩展
let alpn = TLSALPN::new(vec!["h2", "http/1.1"]);

// 自定义扩展
let custom_ext = TLSExtension::new(0xFF01, vec![0x01, 0x02, 0x03]);
```

### 真实密钥生成
```rust
use fingerprint_tls::KeyGenerator;

// 生成真实的ECDH密钥对
let key_gen = KeyGenerator::new();
let key_pair = key_gen.generate_x25519_keypair()?;
let public_key = key_pair.public_key();
```

## 📊 技术特性

### TLS 1.3 完整支持
- ✅ 真实Session ID生成（非空）
- ✅ ChangeCipherSpec消息处理
- ✅ 完整的密钥交换流程
- ✅ BoringSSL填充策略兼容

### 性能优化
- **零拷贝**: 关键路径上的零内存分配
- **并发安全**: 支持多线程并发使用
- **高性能**: 基于rustls实现，性能优异

### 安全特性
- **真实密钥**: 使用ring库生成X25519, P-256, P-384密钥对
- **GREASE处理**: 完整的GREASE值过滤和处理
- **扩展验证**: 严格的扩展字段验证机制

## 🔗 相关模块

- [fingerprint-core](core.md) - 核心抽象层
- [fingerprint-http](http.md) - HTTP协议支持
- [fingerprint-profiles](profiles.md) - 浏览器指纹配置

## 🧪 使用示例

### 基础TLS配置
```rust
use fingerprint_tls::TLSConfig;

let config = TLSConfig::builder()
    .min_version(TLSVersion::TLS13)
    .max_version(TLSVersion::TLS13)
    .with_default_cipher_suites()
    .enable_sni(true)
    .enable_alpn(true)
    .build()?;
```

### 高级握手构建
```rust
use fingerprint_tls::{TLSHandshakeBuilder, SignatureScheme};

let handshake = TLSHandshakeBuilder::new()
    .with_signature_schemes(vec![
        SignatureScheme::ECDSA_NISTP256_SHA256,
        SignatureScheme::ECDSA_NISTP384_SHA384,
        SignatureScheme::ED25519
    ])
    .with_supported_groups(vec![
        NamedGroup::X25519,
        NamedGroup::SECP256R1,
        NamedGroup::SECP384R1
    ])
    .enable_psk(false)
    .build_client_hello()?;
```

---
*最后更新: 2026-02-13*