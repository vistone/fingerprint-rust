# fingerprint

fingerprint-rust 项目的主 crate，整合所有功能模块，提供统一的指纹识别和防护 API。

## 功能特性

- ✅ 浏览器行为指纹识别
- ✅ 多维度指纹特征提取
- ✅ 被动识别和主动防护
- ✅ 异常检测和威胁分析
- ✅ 高性能指纹计算引擎
- 🔧 可选的 Redis 缓存支持
- 🔧 可选的机器学习推断

## 快速开始

### 添加到 Cargo.toml

```toml
[dependencies]
fingerprint = { path = "." }
```

### 基本用法

```rust
use fingerprint::{FingerprintEngine, BrowserInfo};

#[tokio::main]
async fn main() -> Result<()> {
    let engine = FingerprintEngine::new();
    
    // 创建浏览器信息
    let info = BrowserInfo {
        user_agent: "Mozilla/5.0...".to_string(),
        headers: vec![/* HTTP headers */],
        canvas: Some(canvas_data),
    };
    
    // 生成指纹
    let fingerprint = engine.generate(&info).await?;
    println!("Fingerprint ID: {}", fingerprint.id);
    
    // 检测异常
    if engine.is_suspicious(&fingerprint)? {
        println!("Suspicious fingerprint detected!");
    }
    
    Ok(())
}
```

## API 概览

### 主要类型

| 类型 | 说明 |
|-----|------|
| `FingerprintEngine` | 主引擎，协调所有识别模块 |
| `BrowserInfo` | 浏览器信息聚合体 |
| `Fingerprint` | 生成的完整指纹 |
| `RiskLevel` | 风险等级评估 |
| `AnomalyReport` | 异常检测报告 |

### 主要方法

| 方法 | 说明 |
|-----|------|
| `generate(info)` | 生成指纹 |
| `is_suspicious(fp)` | 检测可疑性 |
| `analyze_anomalies(fp)` | 分析异常 |
| `get_risk_level(fp)` | 获取风险等级 |
| `match_fingerprints(fp1, fp2)` | 对比指纹 |

## 使用示例

### 示例 1：完整的指纹流程

```rust
use fingerprint::{FingerprintEngine, BrowserInfo};

#[tokio::main]
async fn main() -> Result<()> {
    let engine = FingerprintEngine::new();
    
    // 收集浏览器信息（可来自 HTTP 请求）
    let info = BrowserInfo::from_request(&request)?;
    
    // 生成指纹
    let fp = engine.generate(&info).await?;
    
    // 检查风险
    let risk = engine.get_risk_level(&fp)?;
    println!("Risk Level: {:?}", risk);
    
    // 保存指纹
    save_fingerprint(&fp)?;
    
    Ok(())
}
```

### 示例 2：与网关集成

```rust
use fingerprint::FingerprintEngine;

async fn handle_request(req: Request) -> Response {
    let engine = FingerprintEngine::new();
    
    // 识别请求来源
    match engine.identify(&req).await {
        Ok(fp) => {
            // 检查指纹是否在黑名单
            if is_blacklisted(&fp.id) {
                return Response::forbidden();
            }
            Response::ok()
        }
        Err(e) => Response::error(e),
    }
}
```

### 示例 3：批量处理

```rust
use fingerprint::FingerprintEngine;

async fn batch_process(requests: Vec<Request>) -> Result<Vec<Fingerprint>> {
    let engine = FingerprintEngine::new();
    let mut results = Vec::new();
    
    for req in requests {
        let info = BrowserInfo::from_request(&req)?;
        let fp = engine.generate(&info).await?;
        results.push(fp);
    }
    
    Ok(results)
}
```

## 项目结构

```
src/
├── lib.rs              # 主入口，重新导出公开 API
├── engine.rs           # 指纹引擎实现
├── collector.rs        # 特征收集器
├── analyzer.rs         # 分析模块
├── cache.rs            # 缓存管理
└── integration/
    ├── tls.rs          # TLS 特征集成
    ├── http.rs         # HTTP 特征集成
    └── defense.rs      # 防护机制集成
```

## 模块依赖关系

```
fingerprint (Main)
├── fingerprint-core        (Base types)
├── fingerprint-tls         (TLS detection)
├── fingerprint-http        (HTTP detection)
├── fingerprint-canvas      (Canvas fingerprinting)
├── fingerprint-webgl       (WebGL fingerprinting)
├── fingerprint-audio       (Audio fingerprinting)
├── fingerprint-fonts       (Font detection)
├── fingerprint-storage     (Storage fingerprinting)
├── fingerprint-hardware    (Hardware detection)
├── fingerprint-timing      (Timing analysis)
├── fingerprint-webrtc      (WebRTC detection)
├── fingerprint-headers     (Header analysis)
├── fingerprint-dns         (DNS features)
├── fingerprint-defense     (Anti-detection)
├── fingerprint-anomaly     (Anomaly detection)
├── fingerprint-ml          (ML inference)
├── fingerprint-profiles    (Browser profiles)
├── fingerprint-gateway     (API gateway)
└── fingerprint-api-noise   (Noise generation)
```

## 可选特性

```toml
[features]
default = ["full"]
full = ["tls", "http", "ml", "cache"]
tls = ["fingerprint-tls"]
http = ["fingerprint-http"]
ml = ["fingerprint-ml"]
cache = ["fingerprint-core/redis"]
lightweight = ["fingerprint-core"]
```

## 性能特性

- **吞吐量**：> 10,000 fingerprints/second
- **延迟**：平均 < 50ms per fingerprint
- **准确度**：> 99% 识别率
- **缓存命中**：> 95%（启用缓存时）

## 网关集成

本 crate 已通过 `fingerprint-gateway` 集成到 API 网关中。详见：

- [Gateway Documentation](../fingerprint-gateway/README.md)
- [API 文档](../../docs/API.md)

## 部署建议

1. **生产环境**
   ```toml
   [dependencies]
   fingerprint = { path = ".", features = ["full"] }
   ```

2. **轻量级部署**
   ```toml
   [dependencies]
   fingerprint = { path = ".", features = ["lightweight"] }
   ```

3. **边界节点**
   ```toml
   [dependencies]
   fingerprint = { path = ".", features = ["cache", "http"] }
   ```

## 故障排查

### 常见问题

| 问题 | 解决方案 |
|-----|--------|
| 指纹生成缓慢 | 启用 `cache` 特性或使用 Redis |
| 内存占用过高 | 关闭 ML 特性或减少缓存大小 |
| 识别准确度低 | 更新浏览器配置文件或调整权重 |

## 贡献指南

欢迎提交 Issue 和 Pull Request！

详见：[CONTRIBUTING.md](../../CONTRIBUTING.md)

## 许可证

本项目采用 MIT 许可证。详见：[LICENSE](../../LICENSE)

## 相关文档

- [API 文档](../../docs/API.md)
- [架构设计](../../docs/ARCHITECTURE.md)
- [项目治理规范](../../PROJECT_GOVERNANCE.md)
- [性能优化](../../docs/PERFORMANCE_OPTIMIZATION.md)

---

**最后更新：** 2026年2月14日
