# DNS 预解析模块文档

**版本 (Version)**: v1.0  
**最后更新 (Last Updated)**: 2026-02-13  
**文档类型 (Document Type)**: 技术文档

---



## 概述

`dns` 模块提供 DNS 预解析服务，定期解析域名列表，并集成 IPInfo.io 获取 IP 地理信息。该模块设计为自动维护，无需人工干预。

**v2.1 新增功能**：
- ✅ **DNS 缓存 (DNSCache)**：内存缓存功能，减少重复解析，提高性能
- ✅ **HTTP 客户端集成**：通过 `DNSHelper` 无缝集成到 HTTP 客户端 (HTTP Client)
- ✅ **智能 IP 选择**：基于地理位置信息实现智能 IP 路由
- ✅ **缓存管理**：自动过期清理和手动失效控制

## 模块位置

**Crate**: `fingerprint-dns`  
**代码路径**: `crates/fingerprint-dns/src/dns/`  
**注意**: 需要启用 `dns` feature

## 核心组件

### Service

DNS 服务主接口，提供 `start()` 和 `stop()` 方法。

```rust
pub struct Service {
    config: Arc<DNSConfig>,
    resolver: Arc<RwLock<DNSResolver>>,
    ipinfo_client: Arc<IPInfoClient>,
    running: Arc<RwLock<bool>>,
}

impl Service {
    /// 启动服务（在后台线程运行，不阻塞主线程）
    pub async fn start(&self) -> Result<(), DNSError>;
    
    /// 停止服务
    pub async fn stop(&self) -> Result<(), DNSError>;
    
    /// 检查服务是否运行
    pub async fn is_running(&self) -> bool;
}
```

### DNSResolver

DNS 解析器，支持高并发查询。

```rust
pub struct DNSResolver {
    timeout: Duration,
    server_pool: Arc<ServerPool>,
}

impl DNSResolver {
    /// 解析单个域名
    pub async fn resolve(&self, domain: &str) -> Result<DNSResult, DNSError>;
    
    /// 并发解析多个域名
    pub async fn resolve_many(&self, domains: Vec<String>, max_concurrency: usize) -> Vec<(String, Result<DNSResult, DNSError>)>;
}
```

### ServerPool

DNS 服务器池管理，支持动态优化。

```rust
pub struct ServerPool {
    servers: Arc<Vec<String>>,
    #[cfg(feature = "hickory-resolver")]
    stats: Arc<RwLock<HashMap<String, ServerStats>>>,
}

impl ServerPool {
    /// 加载默认服务器池
    pub fn load_default() -> Self;
    
    /// 健康检查并增量保存
    pub async fn health_check_and_save_incremental(
        &self,
        test_domain: &str,
        timeout: Duration,
        max_concurrency: usize,
        save_batch_size: usize,
    ) -> Self;
    
    /// 移除慢的服务器
    pub fn remove_slow_servers(&self, max_avg_response_time_ms: f64, max_failure_rate: f64) -> Self;
}
```

### ServerCollector

DNS 服务器收集器，从多个源收集服务器。

```rust
pub struct ServerCollector;

impl ServerCollector {
    /// 从 public-dns.info 获取公共 DNS 服务器列表
    pub async fn collect_public_dns(timeout: Option<Duration>) -> Result<ServerPool, DNSError>;
    
    /// 收集所有可用的 DNS 服务器
    pub async fn collect_all(timeout: Option<Duration>) -> ServerPool;
}
```

### IPInfoClient

IP 地理信息客户端，集成 IPInfo.io API。

```rust
pub struct IPInfoClient {
    token: String,
    timeout: Duration,
}

impl IPInfoClient {
    /// 获取单个 IP 的详细信息
    pub async fn get_ip_info(&self, ip: &str) -> Result<IPInfo, DNSError>;
    
    /// 并发获取多个 IP 的详细信息
    pub async fn get_ip_infos(&self, ips: Vec<String>, max_concurrency: usize) -> Vec<IPInfo>;
}
```

### DNSCache (v2.1 新增)

DNS 缓存模块，提供内存缓存功能。

```rust
pub struct DNSCache {
    cache: Arc<RwLock<HashMap<String, CacheEntry>>>,
    default_ttl: Duration,
}

impl DNSCache {
    /// 创建新的 DNS 缓存
    pub fn new(default_ttl: Duration) -> Self;
    
    /// 从缓存获取域名的 IP 信息
    pub fn get(&self, domain: &str) -> Option<DomainIPs>;
    
    /// 将域名的 IP 信息存入缓存
    pub fn put(&self, domain: &str, ips: DomainIPs);
    
    /// 使缓存失效（删除）
    pub fn invalidate(&self, domain: &str);
    
    /// 清理所有过期的缓存条目
    pub fn cleanup_expired(&self) -> usize;
    
    /// 获取缓存统计信息
    pub fn stats(&self) -> (usize, usize);
}
```

### DNSHelper (HTTP 客户端集成)

DNS 辅助器，提供简化的 DNS 缓存功能，专为 HTTP 客户端设计。

```rust
pub struct DNSHelper {
    cache: Arc<RwLock<HashMap<String, Vec<IpAddr>>>>,
    ttl: Duration,
}

impl DNSHelper {
    /// 创建新的 DNS 辅助器
    pub fn new(ttl: Duration) -> Self;
    
    /// 解析域名到 IP 地址（带缓存）
    pub fn resolve(&self, host: &str, port: u16) -> std::io::Result<Vec<SocketAddr>>;
    
    /// 预热缓存（预先解析一组域名）
    pub fn warmup(&self, domains: &[&str]);
    
    /// 清除缓存
    pub fn clear_cache(&self);
    
    /// 获取缓存统计信息
    pub fn stats(&self) -> (usize, usize);
}
```

## HTTP 客户端集成

### 集成方式 1: 使用 DNSHelper（推荐）

`DNSHelper` 提供简单的 DNS 缓存功能，可以直接集成到 `HttpClientConfig`：

```rust
use fingerprint::{HttpClient, HttpClientConfig, DNSHelper, chrome_133};
use std::sync::Arc;
use std::time::Duration;

// 1. 创建 DNS 辅助器
let dns_helper = Arc::new(DNSHelper::new(Duration::from_secs(300)));

// 2. 可选：预热缓存
dns_helper.warmup(&["www.google.com", "www.github.com"]);

// 3. 配置 HTTP 客户端 (HTTP Client)
let config = HttpClientConfig {
    user_agent: "Mozilla/5.0 ...".to_string(),
    prefer_http2: true,
    profile: Some(chrome_133()),
    dns_helper: Some(dns_helper.clone()),  // 集成 DNS 缓存
    ..Default::default()
};

// 4. 创建客户端并使用
let client = HttpClient::new(config);
let response = client.get("https://www.google.com/")?;
```

### 集成方式 2: 使用完整 DNS 模块

使用 `DNSCache` 和 `DNSResolver` 实现更高级的功能：

```rust
use fingerprint::{DNSCache, DNSResolver, HttpClient, HttpClientConfig};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. 创建 DNS 缓存
    let dns_cache = DNSCache::new(Duration::from_secs(300));
    
    // 2. 创建 DNS 解析器
    let resolver = DNSResolver::new(Duration::from_secs(4));
    
    // 3. 预解析域名
    let domains = vec!["www.google.com", "www.github.com"];
    for domain in &domains {
        let result = resolver.resolve(domain).await?;
        dns_cache.put(domain, result.ips);
    }
    
    // 4. 创建 HTTP 客户端 (HTTP Client)（DNS 已缓存）
    let client = HttpClient::new(HttpClientConfig::default());
    
    // 5. 发送请求（受益于预解析的 DNS）
    let response = client.get("https://www.google.com/")?;
    
    Ok(())
}
```

### 集成方式 3: DNS 服务自动维护

使用 DNS 服务自动维护域名 IP 列表：

```rust
use fingerprint::{DNSService, DNSConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. 配置 DNS 服务
    let config = DNSConfig::new(
        "your-ipinfo-token",
        &["google.com", "github.com"],
    );
    
    // 2. 创建并启动服务
    let service = DNSService::new(config)?;
    service.start().await?;
    
    // 3. 服务会自动维护域名 IP，保存到 dns_output 目录
    // 4. HTTP 客户端可以从文件读取最新 IP 信息
    
    Ok(())
}
```

## 配置

### DNSConfig

```rust
pub struct DNSConfig {
    pub ipinfo_token: String,
    pub domain_list: Vec<String>,
    pub domain_ips_dir: String,
    pub interval: String,              // 如 "2m", "30s", "1h"
    pub max_concurrency: usize,
    pub dns_timeout: String,
    pub http_timeout: String,
    pub max_ip_fetch_conc: usize,
}

impl DNSConfig {
    /// 创建新的 DNS 配置（便利方法，可以直接使用字符串字面量）
    pub fn new<S: AsRef<str>>(ipinfo_token: &str, domain_list: &[S]) -> Self;
}
```

### 配置文件格式

支持 JSON、YAML、TOML 三种格式：

**JSON 示例** (`config.json`):
```json
{
  "ipinfoToken": "your-token",
  "domainList": ["google.com", "github.com"],
  "domainIpsDir": "./dns_output",
  "interval": "2m",
  "maxConcurrency": 1000,
  "dnsTimeout": "4s",
  "httpTimeout": "20s",
  "maxIpFetchConc": 50
}
```

**YAML 示例** (`config.yaml`):
```yaml
ipinfoToken: "your-token"
domainList:
  - "google.com"
  - "github.com"
domainIpsDir: "./dns_output"
interval: "2m"
maxConcurrency: 1000
dnsTimeout: "4s"
httpTimeout: "20s"
maxIpFetchConc: 50
```

## 使用示例 (Usage Examples)

### 基础使用 (Basic Usage)

```rust
use fingerprint::dns::{Service, DNSConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建配置（使用便利方法）
    let config = DNSConfig::new(
        "your-ipinfo-token",
        &["google.com", "github.com"],  // 可以直接使用 &str
    );
    
    // 自定义其他配置
    config.domain_ips_dir = "./dns_output".to_string();
    config.interval = "2m".to_string();
    
    // 创建服务
    let service = Service::new(config)?;
    
    // 启动服务（后台运行，不阻塞）
    service.start().await?;
    
    // 主线程可以继续做其他事情
    tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
    
    // 停止服务
    service.stop().await?;
    
    Ok(())
}
```

### 从配置文件加载

```rust
use fingerprint::dns::Service;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 从配置文件加载
    let service = Service::from_config_file("config/dns_config.json")?;
    
    // 启动服务
    service.start().await?;
    
    // ... 其他代码
    
    Ok(())
}
```

## 特性

### 1. 自动维护

- 自动收集和验证 DNS 服务器
- 自动淘汰慢的服务器
- 自动保存可用服务器到 `dnsservernames.json`

### 2. 智能间隔调整

- 发现新 IP 时：使用基础间隔（高频检测）
- 未发现新 IP 时：指数退避（最多 10 倍基础间隔）

### 3. IP 去重

- DNS 解析结果自动去重
- 与本地存储的 IP 池去重后，只查询新 IP 的详细信息

### 4. 高并发

- DNS 查询：支持 1000+ 并发
- IPInfo 查询：可配置并发数（默认 50）

### 5. 多格式支持

- 配置格式：JSON、YAML、TOML
- 输出格式：JSON、YAML、TOML

## 输出格式

解析结果保存在 `domain_ips_dir` 目录下，每个域名一个文件：

**`google.com.json`**:
```json
{
  "domain": "google.com",
  "ipv4": [
    {
      "ip": "142.250.191.14",
      "country": "US",
      "region": "California",
      "city": "Mountain View",
      "org": "Google LLC",
      "timezone": "America/Los_Angeles"
    }
  ],
  "ipv6": [
    {
      "ip": "2607:f8b0:4004:c1b::65",
      "country": "US",
      "region": "California",
      "city": "Mountain View",
      "org": "Google LLC",
      "timezone": "America/Los_Angeles"
    }
  ],
  "last_updated": "2024-01-01T12:00:00Z"
}
```

## 相关文档

- [使用指南](../guides/USAGE_GUIDE.md#dns-预解析服务)
- [README.md](../../README.md#dns-预解析服务)
