# 快速开始指南

欢迎使用 fingerprint-rust！本指南将帮助您快速上手项目。

## 🚀 安装

### 系统要求
- Rust 1.92.0 或更高版本
- Cargo 包管理器
- Linux/macOS/Windows（推荐 Linux）

### 安装步骤
\`\`\`bash
# 克隆项目
git clone https://github.com/vistone/fingerprint-rust.git
cd fingerprint-rust

# 构建项目
cargo build --release

# 运行示例
cargo run --example basic
\`\`\`

## 🎯 第一个指纹生成

\`\`\`rust
use fingerprint::prelude::*;

fn main() -> Result<()> {
    // 创建 Chrome 131 指纹
    let profile = BrowserProfile::chrome_131()?;
    
    // 生成 TLS ClientHello
    let client_hello = profile.generate_client_hello()?;
    
    println!("Generated fingerprint: {:?}", client_hello.signature());
    Ok(())
}
\`\`\`

## 📚 下一步

- [指纹使用指南](fingerprint-guide.md) - 详细了解各种浏览器指纹
- [API 调用指南](api-usage.md) - 学习如何使用 REST API
- [性能优化](../reference/performance-optimization.md) - 优化您的应用性能

---

*更多详细信息请参阅 [完整文档](../INDEX.md)*
