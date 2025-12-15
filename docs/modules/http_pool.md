# HTTP 连接池支持文档

## 概述

`fingerprint-rust` 通过集成 `netconnpool` 库为所有 HTTP 协议提供连接池支持。

## 当前支持状态

### ✅ HTTP/1.1 连接池
- **状态**: 完全支持
- **实现**: `src/http_client/http1_pool.rs`
- **使用方式**: 通过 `HttpClient::with_pool()` 自动启用
- **特性**: `connection-pool`

### ✅ HTTP/2 连接池
- **状态**: 完全支持
- **实现**: `src/http_client/http2_pool.rs`
- **使用方式**: 通过 `HttpClient::with_pool()` 自动启用（内部使用异步运行时）
- **特性**: `connection-pool` + `http2`

### ✅ HTTP/3 连接池
- **状态**: 完全支持
- **实现**: `src/http_client/http3_pool.rs`
- **使用方式**: 通过 `HttpClient::with_pool()` 自动启用（内部使用异步运行时）
- **特性**: `connection-pool` + `http3`

## 使用示例

### HTTP/1.1 with 连接池

```rust
use fingerprint::{HttpClient, HttpClientConfig};
use fingerprint::http_client::PoolManagerConfig;

// 创建带连接池的客户端
let config = HttpClientConfig {
    user_agent: "MyApp/1.0".to_string(),
    ..Default::default()
};

let client = HttpClient::with_pool(config, PoolManagerConfig::default());

// HTTP/1.1 请求会自动使用连接池
let response = client.get("http://example.com/")?;
```

### HTTP/2 with 连接池 (异步)

```rust
use fingerprint::http_client::http2_pool;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pool_manager = Arc::new(ConnectionPoolManager::new(PoolManagerConfig::default()));
    
    let response = http2_pool::send_http2_request_with_pool(
        "httpbin.org",
        443,
        "/get",
        &request,
        &config,
        &pool_manager,
    ).await?;
    
    Ok(())
}
```

### HTTP/3 with 连接池 (异步)

```rust
use fingerprint::http_client::http3_pool;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pool_manager = Arc::new(ConnectionPoolManager::new(PoolManagerConfig::default()));
    
    let response = http3_pool::send_http3_request_with_pool(
        "cloudflare-quic.com",
        443,
        "/",
        &request,
        &config,
        &pool_manager,
    ).await?;
    
    Ok(())
}
```

## 架构说明

### 同步 vs 异步

- **HTTP/1.1**: 使用标准的 TCP `TcpStream`，完全同步
  - `netconnpool` 可以直接管理 TCP 连接
  - 连接池无缝集成到同步 API

- **HTTP/2**: 基于 `h2` crate，本质上是异步的
  - 使用 `tokio` 运行时
  - 需要异步函数才能充分利用连接池

- **HTTP/3**: 基于 `quinn` (QUIC)，完全异步
  - 使用 `tokio` 运行时
  - 需要异步函数才能充分利用连接池

### 同步 API 中的异步处理

`HttpClient::get()` 方法是同步的，但内部会自动处理异步调用：

```rust
pub fn get(&self, url: &str) -> Result<HttpResponse>
```

对于 HTTP/2 和 HTTP/3，内部会：
1. 创建临时的 `tokio` runtime（如果不存在）
2. 调用异步的连接池函数
3. 等待结果并返回

**注意**：虽然可以使用同步 API，但如果有大量并发请求，建议直接使用异步 API 以获得更好的性能。

## 未来改进

### 计划添加异步 API

```rust
impl HttpClient {
    // 异步版本
    pub async fn get_async(&self, url: &str) -> Result<HttpResponse> {
        // 可以直接调用 http2_pool 和 http3_pool
    }
    
    pub async fn post_async(&self, url: &str, body: Vec<u8>) -> Result<HttpResponse> {
        // ...
    }
}
```

### 使用建议

1. **HTTP/1.1 用户**: 直接使用 `HttpClient::with_pool()` + 同步 API ✅
2. **HTTP/2 用户**: 
   - 简单场景: 使用 `HttpClient::with_pool()` + `get()`（自动使用连接池）✅
   - 高性能场景: 直接调用 `http2_pool::send_http2_request_with_pool()`（异步）
3. **HTTP/3 用户**:
   - 简单场景: 使用 `HttpClient::with_pool()` + `get()`（自动使用连接池）✅
   - 高性能场景: 直接调用 `http3_pool::send_http3_request_with_pool()`（异步）

## 连接池配置

```rust
use fingerprint::http_client::PoolManagerConfig;

let pool_config = PoolManagerConfig {
    max_connections: 100,
    min_idle: 10,
    connect_timeout: std::time::Duration::from_secs(30),
    idle_timeout: std::time::Duration::from_secs(90),
    max_lifetime: std::time::Duration::from_secs(600),
    enable_reuse: true,
};

let client = HttpClient::with_pool(config, pool_config);
```

## 性能优势

使用连接池可以：
- ✅ 复用 TCP/TLS 连接
- ✅ 减少握手开销
- ✅ 降低延迟
- ✅ 提高吞吐量
- ✅ 更好的资源利用

## 示例

查看完整示例：
- `examples/connection_pool.rs` - HTTP/1.1 连接池
- `examples/http2_with_pool.rs` - HTTP/2 连接池
- `examples/http3_with_pool.rs` - HTTP/3 连接池
