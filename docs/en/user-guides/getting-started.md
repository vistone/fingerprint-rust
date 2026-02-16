# 快速开始指南

欢迎使用 fingerprint-rust！本指南将帮助您快速上手项目。

## 🚀 安装

### 系统要求
- Rust 1.92.0 或更高版本
- Cargo 包管理器
- Linux/macOS/Windows (推荐Linux)

### 安装步骤
```bash
# 克隆项目
git clone https://github.com/vistone/fingerprint-rust.git
cd fingerprint-rust

# 构建项目
cargo build --workspace --release

# 运行测试
cargo test --workspace
```

## 🎯 第一个指纹生成

```rust
use fingerprint::{get_random_fingerprint, mapped_tls_clients};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 方法1：获取随机指纹和 HTTP 头
    let result = get_random_fingerprint()?;
    println!("Profile: {}", result.profile_id);
    println!("User-Agent: {}", result.user_agent);
    println!("Browser Type: {:?}", result.browser_type);
    
    // 方法2：直接使用浏览器配置
    let profiles = mapped_tls_clients();
    let chrome = profiles.get("chrome_133").unwrap();
    let spec = chrome.get_client_hello_spec()?;
    println!("Cipher suites: {}", spec.cipher_suites.len());
    
    Ok(())
}
```

## 📚 下一步

- [指纹使用指南](fingerprint-guide.md) - 详细了解各种浏览器指纹
- [API调用指南](api-usage.md) - 学习如何使用REST API
- [性能优化](performance-optimization.md) - 优化您的应用性能

---
*更多详细信息请参阅 [完整文档](../INDEX.md)*