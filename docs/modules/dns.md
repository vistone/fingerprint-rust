# DNS 预解析模块文档

## 概述

`dns` 模块提供 DNS 预解析服务，定期解析域名列表，并集成 IPInfo.io 获取 IP 地理信息。该模块设计为自动维护，无需人工干预。

## 模块位置

`src/dns/`

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

## 使用示例

### 基础使用

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
    let service = Service::from_config(config)?;
    
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
use fingerprint::dns::{Service, load_config};

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
