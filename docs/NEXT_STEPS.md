# 🎯 下一步行动计划

## 📊 当前状态

✅ **已完成**：
- 66 个浏览器指纹配置库
- HTTP 层面完整支持（User-Agent、Headers）
- 基础 HTTP 客户端实现
- 完整的文档和测试

⚠️ **核心问题**：
- TLS 指纹仍然是 rustls 固定的，不是我们自定义的

## 🔥 立即可做的任务（1-3 天）

### 1. 改进 HTTP 响应解析器 ⭐⭐⭐⭐⭐

**问题**：当前响应解析器不支持 chunked encoding

**任务**：
```rust
// src/http_client/response.rs
impl HttpResponse {
    /// 解析 chunked encoding
    fn parse_chunked_body(reader: &mut impl Read) -> Result<Vec<u8>>;
    
    /// 解压 gzip/deflate/br
    fn decompress_body(&self) -> Result<Vec<u8>>;
}
```

**预计时间**：1-2 天

### 2. 完善 HTTP 客户端测试 ⭐⭐⭐⭐

**任务**：
- 修复 Google Earth API 测试
- 添加更多真实网站测试
- 测试各种 HTTP 响应格式

**预计时间**：1 天

### 3. 深度集成 netconnpool ⭐⭐⭐⭐

**当前问题**：HTTP 客户端和 netconnpool 还没有真正集成

**任务**：
```rust
// src/http_client/pool.rs (新建)
pub struct PooledHttpClient {
    pool: Arc<Pool>,
    config: HttpClientConfig,
}

impl PooledHttpClient {
    /// 使用连接池发送请求
    pub fn send_with_pool(&self, request: &HttpRequest) -> Result<HttpResponse> {
        let conn = self.pool.Get()?;
        // 复用连接
        // 自动归还连接
    }
}
```

**预计时间**：2-3 天

### 4. 添加配置导出功能 ⭐⭐⭐

**目的**：让其他语言（Go、Python）可以使用我们的配置

**任务**：
```rust
// src/export.rs (新建)
pub fn export_config_json(profile_name: &str) -> Result<String> {
    let profile = mapped_tls_clients().get(profile_name)?;
    let spec = profile.get_client_hello_spec()?;
    
    let export = ExportConfig {
        cipher_suites: spec.cipher_suites,
        extensions: spec.extensions,
        // ... 其他配置
    };
    
    serde_json::to_string_pretty(&export)
}
```

**预计时间**：1 天

## 🎯 中期任务（1-2 周）

### 5. HTTP/2 实现 ⭐⭐⭐⭐

使用 `h2` crate 实现 HTTP/2 支持

```rust
// src/http_client/http2.rs
use h2::client;

pub fn send_http2_request(...) -> Result<HttpResponse> {
    // 使用 h2 crate
    // 应用 HTTP/2 Settings
}
```

**预计时间**：1-2 周

### 6. 创建 Go uTLS 集成示例 ⭐⭐⭐⭐⭐

**方案 A**：通过 JSON 配置文件
```bash
examples/go-utls/
├── export_config.rs   # Rust: 导出配置
└── main.go           # Go: 读取配置，使用 uTLS
```

**方案 B**：通过 FFI
```rust
// 使用 cgo 调用 Go uTLS
```

**预计时间**：1 周

## 🏗️ 长期任务（1-6 个月）

### 7. 自定义 TLS 实现（最核心） ⭐⭐⭐⭐⭐

**选项 A：基于 OpenSSL**
```rust
use openssl::ssl::{SslConnector, SslMethod};

// 自定义 ClientHello
// 应用 ClientHelloSpec
```

**选项 B：从零实现**
```rust
// 完整实现 TLS 1.2/1.3
// 工作量巨大
```

**选项 C：FFI + Go uTLS**
```rust
// 通过 FFI 调用 Go 的 uTLS
```

**预计时间**：
- 方案 A: 1-2 个月
- 方案 B: 3-6 个月
- 方案 C: 2-4 周

### 8. HTTP/3 / QUIC 支持 ⭐⭐⭐

使用 `quinn` 或 `quiche` crate

**预计时间**：2-3 个月

## 📝 具体行动清单

### 今天/本周可以做的

- [ ] 修复 HTTP 响应解析器的 chunked encoding 支持
- [ ] 添加 gzip/deflate 解压支持
- [ ] 修复 Google Earth API 测试
- [ ] 完善错误处理和日志

### 下周可以做的

- [ ] 深度集成 netconnpool（连接复用）
- [ ] 添加配置导出功能（JSON）
- [ ] 创建 Go uTLS 集成示例
- [ ] 编写集成文档

### 这个月可以做的

- [ ] 实现 HTTP/2 支持
- [ ] 性能优化和压力测试
- [ ] 编写完整的用户指南
- [ ] 准备发布到 crates.io

## 🎯 最优先的3个任务

如果时间有限，专注于这3个：

### 1️⃣ 修复 HTTP 响应解析 ⚠️ 紧急

**为什么重要**：当前测试失败就是因为这个

**如何做**：
```rust
// 1. 检测 Transfer-Encoding: chunked
// 2. 实现 chunked 解析
// 3. 实现压缩解压
```

### 2️⃣ 集成 netconnpool ⭐ 重要

**为什么重要**：这是您最初的目标 - netconnpool + fingerprint

**如何做**：
```rust
// 1. 在 HttpClient 中集成 Pool
// 2. 实现连接复用
// 3. 编写测试
```

### 3️⃣ 创建 Go uTLS 示例 🔥 核心

**为什么重要**：这是解决 TLS 指纹问题的现实方案

**如何做**：
```bash
# 1. 在 Rust 中导出配置
# 2. 在 Go 中使用 uTLS
# 3. 编写完整示例
```

## 💻 代码示例

### 改进响应解析（优先级最高）

```rust
// src/http_client/response.rs
impl HttpResponse {
    pub fn parse(raw_response: &[u8]) -> Result<Self, String> {
        // ... 解析 headers ...
        
        // 检查 Transfer-Encoding
        if let Some(te) = headers.get("Transfer-Encoding") {
            if te.contains("chunked") {
                body = Self::parse_chunked(body_reader)?;
            }
        }
        
        // 检查 Content-Encoding
        if let Some(ce) = headers.get("Content-Encoding") {
            body = Self::decompress(body, ce)?;
        }
        
        Ok(Self { status_code, headers, body, .. })
    }
    
    fn parse_chunked(reader: &[u8]) -> Result<Vec<u8>, String> {
        let mut result = Vec::new();
        let mut pos = 0;
        
        loop {
            // 读取 chunk size
            let size_line_end = reader[pos..]
                .windows(2)
                .position(|w| w == b"\r\n")
                .ok_or("Invalid chunked encoding")?;
            
            let size_str = std::str::from_utf8(&reader[pos..pos + size_line_end])
                .map_err(|_| "Invalid chunk size")?;
            let size = usize::from_str_radix(size_str.trim(), 16)
                .map_err(|_| "Invalid chunk size")?;
            
            if size == 0 {
                break; // 最后一个 chunk
            }
            
            pos += size_line_end + 2; // 跳过 \r\n
            result.extend_from_slice(&reader[pos..pos + size]);
            pos += size + 2; // 跳过 chunk data 和 \r\n
        }
        
        Ok(result)
    }
}
```

### netconnpool 集成

```rust
// src/http_client/pooled.rs
use netconnpool::{Pool, Config, DefaultConfig};

pub struct PooledHttpClient {
    pool: Arc<Pool>,
    config: HttpClientConfig,
}

impl PooledHttpClient {
    pub fn new(config: HttpClientConfig, max_connections: usize) -> Result<Self> {
        let mut pool_config = DefaultConfig();
        pool_config.MaxConnections = max_connections;
        
        let pool = Pool::NewPool(pool_config)?;
        
        Ok(Self {
            pool: Arc::new(pool),
            config,
        })
    }
    
    pub fn get(&self, url: &str) -> Result<HttpResponse> {
        // 从池中获取连接
        let conn = self.pool.Get()?;
        
        // 使用连接发送请求
        let response = self.send_with_conn(conn, url)?;
        
        // 归还连接到池
        self.pool.Put(conn)?;
        
        Ok(response)
    }
}
```

## 📚 参考资源

### Chunked Encoding

- [RFC 7230 - Chunked Transfer Coding](https://datatracker.ietf.org/doc/html/rfc7230#section-4.1)
- [MDN - Transfer-Encoding](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Transfer-Encoding)

### HTTP/2

- [h2 crate](https://github.com/hyperium/h2)
- [RFC 7540 - HTTP/2](https://datatracker.ietf.org/doc/html/rfc7540)

### Go uTLS 集成

- [uTLS GitHub](https://github.com/refraction-networking/utls)
- [CGO 文档](https://pkg.go.dev/cmd/cgo)

## 🎯 建议的工作流程

### 第1天：修复响应解析
```bash
# 1. 实现 chunked encoding 解析
# 2. 测试
# 3. 修复 Google Earth API 测试
```

### 第2-3天：netconnpool 集成
```bash
# 1. 创建 PooledHttpClient
# 2. 实现连接复用
# 3. 编写测试
```

### 第4-5天：Go uTLS 示例
```bash
# 1. 创建配置导出功能
# 2. 编写 Go 示例
# 3. 测试端到端流程
```

### 第6-7天：文档和测试
```bash
# 1. 完善文档
# 2. 添加更多测试
# 3. 准备发布
```

## 🏆 成功标准

### 短期目标（本周）
- ✅ HTTP 响应解析正确处理 chunked encoding
- ✅ Google Earth API 测试通过
- ✅ 基础的 netconnpool 集成

### 中期目标（本月）
- ✅ 完整的 netconnpool 集成
- ✅ Go uTLS 集成示例
- ✅ HTTP/2 基础支持

### 长期目标（3-6个月）
- ✅ 自定义 TLS 实现
- ✅ HTTP/3 支持
- ✅ 发布到 crates.io

---

**下一步行动**：优先修复 HTTP 响应解析器！🚀
