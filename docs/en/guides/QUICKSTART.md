# Quick Start Guide

**Chinese** | [中文](/docs/zh/guides/QUICKSTART.md)

---

## 🎯 5-Minute Quick Start

This guide will help you start using fingerprint-rust in 5 minutes.

### Step 1: Add Dependency

Add this to your `Cargo.toml`:

```toml
[dependencies]
fingerprint = "2.1"
tokio = { version = "1", features = ["full"] }
```

### Step 2: Get Random Fingerprint

```rust
use fingerprint::{get_random_fingerprint, BrowserType};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Method 1: Get completely random fingerprint
    let result = get_random_fingerprint()?;
    println!("Profile: {}", result.profile_id);
    println!("User-Agent: {}", result.user_agent);
    println!("Accept-Language: {}", result.headers.accept_language);
    
    Ok(())
}
```

### Step 3: Get Browser-Specific Fingerprint

```rust
use fingerprint::get_random_fingerprint_by_browser;
use fingerprint::types::BrowserType;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get Chrome fingerprint
    let chrome_fp = get_random_fingerprint_by_browser(BrowserType::Chrome)?;
    println!("Chrome User-Agent: {}", chrome_fp.user_agent);
    
    // Get Firefox fingerprint
    let firefox_fp = get_random_fingerprint_by_browser(BrowserType::Firefox)?;
    println!("Firefox User-Agent: {}", firefox_fp.user_agent);
    
    Ok(())
}
```

### Step 4: Make HTTP Requests

```rust
use fingerprint::{get_random_fingerprint, HttpClient, HttpClientConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let fp = get_random_fingerprint()?;
    
    // Create HTTP client with fingerprint
    let config = HttpClientConfig::default()
        .with_user_agent(fp.user_agent.clone())
        .with_timeout(10);
    
    let client = HttpClient::new(config)?;
    
    // Send request
    let response = client.get("https://httpbin.org/user-agent").await?;
    println!("Status: {}", response.status);
    println!("Body: {}", String::from_utf8_lossy(&response.body));
    
    Ok(())
}
```

### Common Use Cases

#### Web Scraping

```rust
use fingerprint::{get_random_fingerprint, HttpClient, HttpClientConfig};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    for i in 0..5 {
        // Use different fingerprint each time
        let fp = get_random_fingerprint()?;
        
        let config = HttpClientConfig::default()
            .with_user_agent(fp.user_agent)
            .with_timeout(15);
        
        let client = HttpClient::new(config)?;
        let response = client.get("https://example.com").await?;
        
        println!("Request {}: Status {}", i + 1, response.status);
        
        // Avoid detection
        tokio::time::sleep(Duration::from_secs(2)).await;
    }
    
    Ok(())
}
```

#### API Protection Detection

```rust
use fingerprint::{get_random_fingerprint, PassiveAnalyzer};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let fp = get_random_fingerprint()?;
    
    // Analyze if fingerprint will be detected
    let analyzer = PassiveAnalyzer::new();
    let analysis = analyzer.analyze_headers(&fp.headers)?;
    
    println!("Consistency Score: {}", analysis.consistency_score);
    println!("Anomaly Score: {}", analysis.anomaly_score);
    
    if analysis.anomaly_score > 0.8 {
        println!("⚠️  Warning: This fingerprint may be detected");
    } else {
        println!("✅ Fingerprint looks normal");
    }
    
    Ok(())
}
```

#### Machine Learning Classification

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
    println!("Anomaly Score: {}", result.anomaly_score);
    println!("Classification: {:?}", result.classification);
    
    Ok(())
}
```

### Next Steps

- [Detailed API Documentation](/docs/en/reference/API.md)
- [Developer Guide](/docs/en/developer-guides/README.md)
- [Architecture Design](/docs/en/ARCHITECTURE.md)
- [FAQ](/docs/en/FAQ.md)

### Important Notes

1. **Legal Usage**: Ensure your usage complies with all applicable laws and terms of service
2. **Privacy**: Do not use for malicious purposes or privacy violations
3. **Detection**: Some services may detect fingerprint spoofing
4. **Rate Limiting**: Set appropriate request intervals to avoid IP bans

### Getting Help

- 📖 [Full Documentation](https://github.com/vistone/fingerprint-rust/tree/main/docs)
- 🐛 [Report Issues](https://github.com/vistone/fingerprint-rust/issues)
- 💬 [Discussions](https://github.com/vistone/fingerprint-rust/discussions)
- 📧 [Contact Contributors](https://github.com/vistone/fingerprint-rust#contributors)
