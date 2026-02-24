# 快速开始指南

**[English](../en/guides/QUICKSTART.md)** | [中文](#中文)

---

## 中文

### 🎯 5分钟快速入门

该指南将帮助你在5分钟内开始使用fingerprint-rust库。

#### 第一步：添加依赖

将以下内容添加到`Cargo.toml`：

```toml
[dependencies]
fingerprint = "2.1"
tokio = { version = "1", features = ["full"] }
```

#### 第二步：获取随机指纹

```rust
use fingerprint::{get_random_fingerprint, BrowserType};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 方法1: 获取完全随机的指纹
    let result = get_random_fingerprint()?;
    println!("浏览器: {}", result.profile_id);
    println!("User-Agent: {}", result.user_agent);
    println!("语言: {}", result.headers.accept_language);
    
    Ok(())
}
```

#### 第三步：获取特定浏览器指纹

```rust
use fingerprint::get_random_fingerprint_by_browser;
use fingerprint::types::BrowserType;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 获取Chrome浏览器指纹
    let chrome_fp = get_random_fingerprint_by_browser(BrowserType::Chrome)?;
    println!("Chrome User-Agent: {}", chrome_fp.user_agent);
    
    // 获取Firefox指纹
    let firefox_fp = get_random_fingerprint_by_browser(BrowserType::Firefox)?;
    println!("Firefox User-Agent: {}", firefox_fp.user_agent);
    
    Ok(())
}
```

#### 第四步：访问HTTP请求

```rust
use fingerprint::{get_random_fingerprint, HttpClient, HttpClientConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let fp = get_random_fingerprint()?;
    
    // 创建HTTP客户端
    let config = HttpClientConfig::default()
        .with_user_agent(fp.user_agent.clone())
        .with_timeout(10);
    
    let client = HttpClient::new(config)?;
    
    // 发送请求
    let response = client.get("https://httpbin.org/user-agent").await?;
    println!("状态码: {}", response.status);
    println!("响应体: {}", String::from_utf8_lossy(&response.body));
    
    Ok(())
}
```

### 📚 常见场景

#### 场景1：Web爬虫

```rust
use fingerprint::{get_random_fingerprint, HttpClient, HttpClientConfig};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    for i in 0..5 {
        // 每次爬取使用不同的指纹
        let fp = get_random_fingerprint()?;
        
        let config = HttpClientConfig::default()
            .with_user_agent(fp.user_agent)
            .with_timeout(15);
        
        let client = HttpClient::new(config)?;
        let response = client.get("https://example.com").await?;
        
        println!("请求 {}: 状态码 {}", i + 1, response.status);
        
        // 避免被检测到
        tokio::time::sleep(Duration::from_secs(2)).await;
    }
    
    Ok(())
}
```

#### 场景2：API防护检测

```rust
use fingerprint::{get_random_fingerprint, PassiveAnalyzer};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let fp = get_random_fingerprint()?;
    
    // 分析指纹是否会被检测
    let analyzer = PassiveAnalyzer::new();
    let analysis = analyzer.analyze_headers(&fp.headers)?;
    
    println!("指纹一致性: {}", analysis.consistency_score);
    println!("异常分数: {}", analysis.anomaly_score);
    
    if analysis.anomaly_score > 0.8 {
        println!("⚠️  警告: 该指纹可能会被检测");
    } else {
        println!("✅ 指纹看起来正常");
    }
    
    Ok(())
}
```

#### 场景3：机器学习分类

```rust
use fingerprint::{get_random_fingerprint, fingerprint_ml::AdvancedAnomalyDetector};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let fp = get_random_fingerprint()?;
    
    let detector = AdvancedAnomalyDetector::new();
    let vector = fingerprint_ml::FingerprintVector::new(
        vec![0.1, 0.2, 0.3, 0.4, 0.5],
        Some(fp.profile_id.clone()),
        0.95,
    );
    
    let result = detector.detect_anomalies(&vector);
    println!("异常分数: {}", result.anomaly_score);
    println!("分类: {:?}", result.classification);
    
    Ok(())
}
```

### 🔗 下一步

- [详细API文档](/docs/en/reference/API.md)
- [开发者指南](/docs/en/developer-guides/README.md)
- [架构设计](/docs/ARCHITECTURE.md)
- [常见问题](/docs/FAQ.md)

### ⚠️ 重要注意事项

1. **合法使用**: 确保你的使用符合所有适用的法律和服务条款
2. **尊重隐私**: 不要用于恶意用途或侵犯隐私
3. **标准转换**: 一些服务可能会检测指纹欺骗行为
4. **速率限制**: 合理设置请求间隔，避免被IP封禁

### 🤝 获取帮助

- 📖 [完整文档](https://github.com/vistone/fingerprint-rust/tree/main/docs)
- 🐛 [报告问题](https://github.com/vistone/fingerprint-rust/issues)
- 💬 [讨论问题](https://github.com/vistone/fingerprint-rust/discussions)
- 📧 [联系贡献者](https://github.com/vistone/fingerprint-rust#contributors)
