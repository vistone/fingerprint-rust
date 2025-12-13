# fingerprint-rust

<div align="center">

[![docs](https://docs.rs/fingerprint/badge.svg)](https://docs.rs/fingerprint)
[![crates.io](https://img.shields.io/crates/v/fingerprint.svg)](https://crates.io/crates/fingerprint)
[![Downloads](https://img.shields.io/crates/d/fingerprint.svg)](https://crates.io/crates/fingerprint)
[![License](https://img.shields.io/badge/license-BSD_3--Clause-blue.svg)](https://opensource.org/licenses/BSD-3-Clause)
[![CI](https://github.com/vistone/fingerprint/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/vistone/fingerprint/actions)
[![Pure Rust](https://img.shields.io/badge/pure-Rust-brightgreen.svg)](https://www.rust-lang.org/)

</div>

一个独立的浏览器 TLS 指纹库，从 [golang 版本](https://github.com/vistone/fingerprint) 迁移而来。

## 特性

- ✅ **真实浏览器指纹**：66 个真实浏览器指纹（Chrome、Firefox、Safari、Opera）
- ✅ **真实 TLS 配置**：完整的 TLS Client Hello Spec（密码套件、椭圆曲线、扩展等）
- ✅ **JA4 指纹生成**：完整的 JA4 TLS 客户端指纹生成（sorted 和 unsorted 版本）
- ✅ **指纹比较**：支持指纹相似度比较和最佳匹配查找
- ✅ **GREASE 处理**：完整的 GREASE 值过滤和处理
- ✅ **HTTP/2 配置**：完整的 HTTP/2 Settings、Pseudo Header Order、Header Priority
- ✅ **移动端支持**：iOS、Android 移动端指纹
- ✅ **User-Agent 匹配**：自动生成匹配的 User-Agent
- ✅ **标准 HTTP Headers**：完整的标准 HTTP 请求头
- ✅ **全球语言支持**：30+ 种语言的 Accept-Language
- ✅ **操作系统随机化**：随机选择操作系统
- ✅ **高性能**：零分配的关键操作，并发安全
- ✅ **Rust 标准**：严格遵循 Rust 语言标准和最佳实践
- ✅ **完整实现**：对应 Go 版本的所有功能，包括真实的 TLS 指纹配置

## 安装

在 `Cargo.toml` 中添加：

```toml
[dependencies]
fingerprint = { path = "." }
```

或者从 crates.io（如果发布）：

```toml
[dependencies]
fingerprint = "1.0.0"
```

## 快速开始

### 最简单的方式（推荐）⭐

```rust
use fingerprint::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 一行代码，获取指纹和完整的 HTTP Headers
    let result = get_random_fingerprint()?;
    
    // result.profile - TLS 指纹配置
    // result.headers - 完整的 HTTP Headers（包括 User-Agent、Accept-Language）
    // result.hello_client_id - Client Hello ID
    
    println!("User-Agent: {}", result.user_agent);
    println!("Profile: {}", result.hello_client_id);
    
    // 使用 Headers
    let headers_map = result.headers.to_map();
    for (key, value) in headers_map.iter() {
        println!("{}: {}", key, value);
    }
    
    Ok(())
}
```

### 指定浏览器类型

```rust
use fingerprint::*;

// 随机获取 Chrome 指纹
let result = get_random_fingerprint_by_browser("chrome")?;

// 指定浏览器和操作系统
let result = get_random_fingerprint_by_browser_with_os(
    "firefox",
    Some(OperatingSystem::Windows10),
)?;
```

### 自定义 Headers

```rust
use fingerprint::*;

let mut result = get_random_fingerprint()?;

// 设置自定义 header
result.headers.set("Cookie", "session_id=abc123");
result.headers.set("Authorization", "Bearer token");

// 批量设置
result.headers.set_headers(&[
    ("Cookie", "session_id=abc123"),
    ("X-API-Key", "your-api-key"),
]);

// 自动合并，直接使用
let headers = result.headers.to_map();
```

## 支持的指纹

### 浏览器指纹（66 个）

**Chrome 系列** (19 个)
- Chrome 103, 104, 105, 106, 107, 108, 109, 110, 111, 112
- Chrome 116_PSK, 116_PSK_PQ, 117, 120, 124
- Chrome 130_PSK, 131, 131_PSK, 133, 133_PSK

**Firefox 系列** (12 个)
- Firefox 102, 104, 105, 106, 108, 110, 117, 120, 123, 132, 133, 135

**Safari 系列** (9 个)
- Safari 15.6.1, 16.0, iPad 15.6
- Safari iOS 15.5, 15.6, 16.0, 17.0, 18.0, 18.5

**Opera 系列** (3 个)
- Opera 89, 90, 91

**移动端和自定义** (23 个)
- Zalando (2), Nike (2), MMS (3), Mesh (4), Confirmed (3)
- OkHttp4 Android (7), Cloudflare (1)

## API 参考

### 核心函数

```rust
// 随机指纹（推荐）
pub fn get_random_fingerprint() -> Result<FingerprintResult, String>
pub fn get_random_fingerprint_with_os(os: Option<OperatingSystem>) -> Result<FingerprintResult, String>
pub fn get_random_fingerprint_by_browser(browser_type: &str) -> Result<FingerprintResult, Box<dyn Error>>
pub fn get_random_fingerprint_by_browser_with_os(
    browser_type: &str,
    os: Option<OperatingSystem>,
) -> Result<FingerprintResult, Box<dyn Error>>

// TLS 指纹配置
pub fn extract_signature(spec: &ClientHelloSpec) -> ClientHelloSignature
pub fn compare_specs(spec1: &ClientHelloSpec, spec2: &ClientHelloSpec) -> FingerprintMatch
pub fn compare_signatures(sig1: &ClientHelloSignature, sig2: &ClientHelloSignature) -> FingerprintMatch
pub fn find_best_match(signature: &ClientHelloSignature, specs: &[ClientHelloSpec]) -> Option<usize>

// JA4 指纹生成
pub fn generate_ja4(signature: &Ja4Signature) -> Ja4Payload
pub fn generate_ja4_original(signature: &Ja4Signature) -> Ja4Payload

// GREASE 处理
pub fn is_grease_value(value: u16) -> bool
pub fn filter_grease_values(values: &[u16]) -> Vec<u16>
pub fn remove_grease_values(values: &[u16]) -> Vec<u16>

// User-Agent
pub fn get_user_agent_by_profile_name(profile_name: &str) -> Result<String, String>
pub fn get_user_agent_by_profile_name_with_os(
    profile_name: &str,
    os: OperatingSystem,
) -> Result<String, String>
pub fn random_os() -> OperatingSystem
pub fn random_language() -> String

// Headers
pub fn generate_headers(
    browser_type: BrowserType,
    user_agent: &str,
    is_mobile: bool,
) -> HTTPHeaders
```

### 数据结构

```rust
pub struct FingerprintResult {
    pub profile: ClientProfile,      // TLS 指纹配置（包含真实的 TLS Client Hello Spec）
    pub user_agent: String,          // 对应的 User-Agent
    pub hello_client_id: String,      // Client Hello ID
    pub headers: HTTPHeaders,        // 标准 HTTP 请求头
}

// 获取真实的 TLS Client Hello Spec
let client_hello_spec = profile.get_client_hello_spec()?;
// client_hello_spec 包含：
// - cipher_suites: 密码套件列表
// - elliptic_curves: 椭圆曲线列表
// - extensions: TLS 扩展列表
// - alpn_protocols: ALPN 协议列表
// - signature_algorithms: 签名算法列表
// 等等...

// 获取 HTTP/2 Settings
let settings = profile.get_settings();
let pseudo_header_order = profile.get_pseudo_header_order();
let header_priority = profile.get_header_priority();
```

pub struct HTTPHeaders {
    pub accept: String,
    pub accept_language: String,
    pub accept_encoding: String,
    pub user_agent: String,
    pub sec_fetch_site: String,
    pub sec_fetch_mode: String,
    pub sec_fetch_user: String,
    pub sec_fetch_dest: String,
    pub sec_ch_ua: String,
    pub sec_ch_ua_mobile: String,
    pub sec_ch_ua_platform: String,
    pub upgrade_insecure_requests: String,
    pub custom: HashMap<String, String>,  // 自定义 headers
}
```

### 操作系统

```rust
pub enum OperatingSystem {
    Windows10, Windows11,           // Windows
    MacOS13, MacOS14, MacOS15,     // macOS
    Linux, LinuxUbuntu, LinuxDebian, // Linux
}
```

### 浏览器类型

```rust
pub enum BrowserType {
    Chrome, Firefox, Safari, Opera, Edge,
}
```

## 项目结构

```
/workspace/
├── src/              # 源代码
│   ├── lib.rs        # 库入口
│   ├── types.rs      # 类型定义
│   ├── utils.rs      # 工具函数
│   ├── headers.rs    # HTTP Headers
│   ├── useragent.rs  # User-Agent 生成
│   ├── random.rs     # 随机指纹
│   └── profiles.rs   # 指纹配置
├── tests/            # 集成测试
├── examples/         # 示例代码
├── docs/             # 文档
├── bin/              # 编译输出（自动生成）
└── README.md
```

## 示例

查看 `examples/` 目录获取更多示例：
- `examples/basic.rs` - 基础使用
- `examples/useragent.rs` - User-Agent 生成
- `examples/headers.rs` - Headers 使用
- `examples/tls_config.rs` - **TLS 指纹配置使用**（展示真实的 TLS Client Hello Spec）

运行示例：

```bash
cargo run --example basic
cargo run --example useragent
cargo run --example headers
cargo run --example tls_config  # 查看真实的 TLS 配置
```

### TLS 配置示例

```rust
use fingerprint::*;

// 获取指纹配置
let profile = mapped_tls_clients().get("chrome_133").unwrap();

// 获取真实的 TLS Client Hello Spec
let client_hello_spec = profile.get_client_hello_spec()?;
println!("密码套件: {:?}", client_hello_spec.cipher_suites);
println!("扩展数量: {}", client_hello_spec.extensions.len());

// 提取签名并生成 JA4 指纹
let signature = extract_signature(&client_hello_spec);
let ja4_signature = Ja4Signature {
    version: signature.version,
    cipher_suites: signature.cipher_suites,
    extensions: signature.extensions,
    signature_algorithms: signature.signature_algorithms,
    sni: signature.sni,
    alpn: signature.alpn,
};
let ja4 = ja4_signature.generate_ja4();
println!("JA4: {}", ja4.full.value());
println!("JA4 Raw: {}", ja4.raw.value());

// 获取 HTTP/2 配置
let settings = profile.get_settings();
let pseudo_header_order = profile.get_pseudo_header_order();
println!("Pseudo Header Order: {:?}", pseudo_header_order);
```

### JA4 指纹生成示例

```rust
use fingerprint::{ClientHelloSpec, extract_signature, Ja4Signature};

// 从 ClientHelloSpec 提取签名
let spec = ClientHelloSpec::chrome_133();
let signature = extract_signature(&spec);

// 创建 JA4 签名
let ja4_sig = Ja4Signature {
    version: signature.version,
    cipher_suites: signature.cipher_suites,
    extensions: signature.extensions,
    signature_algorithms: signature.signature_algorithms,
    sni: signature.sni,
    alpn: signature.alpn,
};

// 生成 JA4 指纹（排序版本）
let ja4 = ja4_sig.generate_ja4();
println!("JA4: {}", ja4.full.value());
println!("JA4 Raw: {}", ja4.raw.value());

// 生成 JA4 指纹（原始顺序版本）
let ja4_original = ja4_sig.generate_ja4_original();
println!("JA4 Original: {}", ja4_original.full.value());
```

### 指纹比较示例

```rust
use fingerprint::{ClientHelloSpec, compare_specs, find_best_match, extract_signature};

// 比较两个指纹
let spec1 = ClientHelloSpec::chrome_133();
let spec2 = ClientHelloSpec::chrome_103();
let match_result = compare_specs(&spec1, &spec2);
match match_result {
    FingerprintMatch::Exact => println!("完全匹配"),
    FingerprintMatch::Similar => println!("相似匹配"),
    FingerprintMatch::None => println!("不匹配"),
}

// 查找最佳匹配
let signature = extract_signature(&spec1);
let candidates = vec![
    ClientHelloSpec::chrome_103(),
    ClientHelloSpec::chrome_133(),
    ClientHelloSpec::firefox_133(),
];
if let Some(index) = find_best_match(&signature, &candidates) {
    println!("最佳匹配索引: {}", index);
}
```

## 测试

```bash
# 运行所有测试
cargo test

# 运行集成测试
cargo test --test integration_test

# 运行示例
cargo run --example basic
```

## 依赖

- `rand = "0.8"` - 随机数生成
- `once_cell = "1.19"` - 线程安全的单例
- `sha2 = "0.10"` - SHA256 哈希（用于 JA4 指纹生成）
- `thiserror = "2.0"` - 错误处理（可选）

## 许可证

BSD 3-Clause License。原始代码来自 [vistone/fingerprint](https://github.com/vistone/fingerprint)。

## 版本历史

查看 [RELEASE_NOTES.md](docs/RELEASE_NOTES.md) 了解详细的版本历史。

## 相关项目

- [fingerprint (Go)](https://github.com/vistone/fingerprint) - Go 版本的指纹库
- [refraction-networking/utls](https://github.com/refraction-networking/utls) - Go TLS 指纹库参考
- [biandratti/huginn-net](https://github.com/biandratti/huginn-net) - Rust 网络指纹库参考
