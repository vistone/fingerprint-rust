# Profiles模块文档

**版本 (Version)**: v1.0  
**最后更新 (Last Updated)**: 2026-02-13  
**模块**: fingerprint-profiles

---

## 🎯 模块概述

fingerprint-profiles 是浏览器指纹配置模块，提供97+个真实浏览器版本的完整指纹配置，支持Chrome、Firefox、Safari、Opera、Edge等主流浏览器及其移动端变体。

## 📦 浏览器版本支持

### Chrome系列 (36个版本)
**桌面版**:
- Chrome 103-138 (36个版本)
- 包含特殊变体：PSK、0-RTT、PQ等

**移动端**:
- Chrome Mobile 120, 134
- Chrome iOS 120-138

### Firefox系列 (17个版本)
**桌面版**:
- Firefox 102-138 (17个版本)

**移动端**:
- Firefox Mobile 120, 135
- Firefox iOS 120-138

### Safari系列 (16个版本)
**桌面版**:
- Safari 15.0, 15.7, 16.0, 17.0, 18.0, 18.2

**移动端**:
- Safari iOS 16.0, 17.0, 18.0, 18.1, 18.2, 18.3

### Edge系列 (18个版本)
- Edge 120-137 (18个版本)
- 包含Chromium内核版本

### Opera系列 (4个版本)
- Opera 91-94 (4个版本)

### 其他浏览器
- Brave浏览器支持
- Vivaldi浏览器支持
- 特殊应用客户端配置

## 🔧 核心功能

### 指纹配置管理
```rust
use fingerprint_profiles::{BrowserProfile, ProfileManager};

// 获取特定浏览器配置
let chrome_profile = BrowserProfile::chrome_120();
let firefox_profile = BrowserProfile::firefox_120();

// 随机选择浏览器配置
let random_profile = ProfileManager::random_profile()?;

// 按条件筛选配置
let modern_chrome = ProfileManager::filter_by_criteria(|p| {
    p.browser_family() == "Chrome" && p.version_major() >= 120
})?;
```

### 版本适配系统
```rust
use fingerprint_profiles::VersionAdapter;

let adapter = VersionAdapter::new();
let user_agent = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 Chrome/120.0.0.0";
let profile = adapter.adapt_from_useragent(user_agent)?;
println!("Adapted profile: {:?}", profile.name());
```

### 配置池管理
```rust
use fingerprint_profiles::ProfilePool;

let pool = ProfilePool::builder()
    .add_profile(BrowserProfile::chrome_120())
    .add_profile(BrowserProfile::firefox_120())
    .add_profile(BrowserProfile::safari_17())
    .rotation_strategy(RotationStrategy::RoundRobin)
    .build()?;

// 轮询使用不同配置
for i in 0..10 {
    let profile = pool.next_profile()?;
    println!("Request {}: Using {}", i, profile.name());
}
```

## 📊 配置结构

### 完整指纹配置示例
```rust
pub struct BrowserFingerprint {
    pub browser: &'static str,      // 浏览器名称
    pub version: &'static str,      // 版本号
    pub user_agent: &'static str,   // User-Agent字符串
    pub tls_config: TLSConfig,      // TLS配置
    pub http_headers: HttpHeaders,  // HTTP头部配置
    pub features: FeatureFlags,     // 支持的特性标志
    pub metadata: ProfileMetadata,  // 元数据信息
}

pub struct TLSConfig {
    pub ja3: &'static str,          // JA3指纹
    pub cipher_suites: Vec<&'static str>,  // 密码套件
    pub extensions: Vec<&'static str>,     // 扩展字段
    pub signature_algorithms: Vec<&'static str>, // 签名算法
    pub supported_groups: Vec<&'static str>,     // 支持的群组
}
```

### 特性标志系统
```rust
bitflags::bitflags! {
    pub struct FeatureFlags: u32 {
        const HTTP2 = 1 << 0;           // 支持HTTP/2
        const HTTP3 = 1 << 1;           // 支持HTTP/3
        const TLS13 = 1 << 2;           // 支持TLS 1.3
        const ECH = 1 << 3;             // 支持ECH
        const PSK = 1 << 4;             // 支持PSK
        const PQ = 1 << 5;              // 支持后量子密码
        const MOBILE = 1 << 6;          // 移动端配置
    }
}
```

## 🎯 使用场景

### 网络爬虫
```rust
use fingerprint_profiles::CrawlerProfile;

let crawler = CrawlerProfile::builder()
    .desktop_browsers(vec!["Chrome", "Firefox", "Safari"])
    .mobile_ratio(0.3)  // 30%移动端流量
    .build()?;

for url in targets {
    let profile = crawler.next_profile()?;
    send_request_with_profile(url, profile).await?;
}
```

### 负载测试
```rust
use fingerprint_profiles::LoadTestProfile;

let load_tester = LoadTestProfile::builder()
    .concurrent_users(1000)
    .browser_distribution([
        ("Chrome", 0.6),
        ("Firefox", 0.25),
        ("Safari", 0.15)
    ])
    .build()?;

// 生成多样化的测试流量
let test_profiles = load_tester.generate_test_set(10000)?;
```

### 安全测试
```rust
use fingerprint_profiles::SecurityTestProfile;

let security_tester = SecurityTestProfile::builder()
    .include_obsolete_versions(true)    // 包含过时版本
    .enable_anomaly_detection(true)     // 启用异常检测
    .build()?;

let suspicious_profiles = security_tester.detect_anomalies()?;
```

## 🔧 高级功能

### 动态配置生成
```rust
use fingerprint_profiles::DynamicProfileGenerator;

let generator = DynamicProfileGenerator::new();
let custom_profile = generator.create_profile(ProfileTemplate {
    browser_family: "Chrome",
    version_range: (120, 125),
    platform: "Windows",
    features: FeatureFlags::HTTP2 | FeatureFlags::TLS13,
})?;
```

### 配置验证
```rust
use fingerprint_profiles::ProfileValidator;

let validator = ProfileValidator::new();
let is_valid = validator.validate_profile(&profile)?;
let compatibility = validator.check_compatibility(&profile, &target_server)?;
```

## 📈 性能优化

### 配置缓存
```rust
use fingerprint_profiles::ProfileCache;

let cache = ProfileCache::builder()
    .max_size(1000)
    .ttl(Duration::from_hours(1))
    .build()?;

// 缓存热点配置
cache.store("chrome_120", chrome_120_profile);
let cached_profile = cache.get("chrome_120")?;
```

### 批量操作
```rust
use fingerprint_profiles::BatchProcessor;

let processor = BatchProcessor::new();
let profiles_batch = processor.load_profiles_batch(&profile_names)?;
let results = processor.validate_batch(profiles_batch)?;
```

## 🔗 相关模块

- [fingerprint-core](core.md) - 核心抽象层
- [fingerprint-tls](tls.md) - TLS配置支持
- [fingerprint-headers](headers.md) - HTTP头部生成
- [fingerprint-ml](ml.md) - 机器学习分类

---
*最后更新 (Last Updated): 2026-02-13*