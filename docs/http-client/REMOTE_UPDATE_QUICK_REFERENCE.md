# Fingerprint-Rust 远程更新代码 - 快速参考手册

## 快速开始

### 最简单的 GET 请求
```rust
let client = HttpClient::new(HttpClientConfig::default());
let response = client.get("https://api.example.com/data")?;
println!("状态码: {}", response.status_code);
```

### 最简单的 POST 请求
```rust
let client = HttpClient::new(HttpClientConfig::default());
let response = client.post("https://api.example.com/data", b"request body")?;
```

---

## 关键类型速查

### HttpClient
```rust
// 创建方式
HttpClient::new(config)                          // 基础客户端
HttpClient::with_pool(config, pool_config)       // 带连接池的客户端
HttpClient::with_profile(profile, headers, ua)   // 带浏览器指纹的客户端

// 核心方法
client.get(url)                                  // GET 请求
client.post(url, body)                          // POST 请求
client.send_request(&request)                   // 发送自定义请求
client.pool_stats()                             // 获取连接池统计
client.cleanup_idle_connections()               // 清理空闲连接
```

### HttpClientConfig
```rust
let mut config = HttpClientConfig::default();

config.user_agent = "Mozilla/5.0 ...".to_string();
config.headers = HTTPHeaders::default();
config.profile = Some(chrome_133());
config.connect_timeout = Duration::from_secs(10);
config.read_timeout = Duration::from_secs(10);
config.write_timeout = Duration::from_secs(10);
config.max_redirects = 10;
config.verify_tls = true;
config.prefer_http2 = true;
config.prefer_http3 = false;
config.cookie_store = Some(Arc::new(CookieStore::new()));

let client = HttpClient::new(config);
```

### HttpRequest
```rust
let mut request = HttpRequest::new(HttpMethod::Get, "https://example.com");
request = request
    .with_header("Authorization", "Bearer token")
    .with_header("Content-Type", "application/json")
    .with_body(body_bytes.to_vec());

let response = client.send_request(&request)?;
```

### HttpResponse
```rust
println!("状态码: {}", response.status_code);
println!("头部: {:?}", response.headers);
println!("体: {:?}", response.body);

if let Some(content_type) = response.headers.get("content-type") {
    println!("Content-Type: {}", content_type);
}
```

---

## 浏览器指纹速查表

### 使用预定义指纹
```rust
use fingerprint::*;

let profile = chrome_133();         // Chrome 133
let profile = chrome_131();         // Chrome 131
let profile = firefox_133();        // Firefox 133
let profile = safari_16_0();        // Safari 16.0
let profile = opera_91();           // Opera 91

// 创建带指纹的客户端
let client = HttpClient::with_profile(
    profile,
    HTTPHeaders::default(),
    "User-Agent-String".to_string()
);
```

### 随机指纹（推荐用于反爬虫）
```rust
let random_fp = get_random_fingerprint()?;
let random_fp = get_random_fingerprint_by_browser(BrowserType::Chrome)?;
let random_fp = get_random_fingerprint_with_os(OperatingSystem::Windows10)?;
```

### 可用的浏览器指纹列表
- Chrome: 103, 104, 105, ..., 133 (多个版本)
- Firefox: 102, 104, 105, ..., 135 (多个版本)
- Safari: 15.6.1, 16.0 等
- Opera: 89, 90, 91
- Edge 和其他现代浏览器

---

## 常见任务

### 任务 1: 简单的 API 调用
```rust
let client = HttpClient::new(HttpClientConfig::default());
let response = client.get("https://api.example.com/data")?;

match response.status_code {
    200 => println!("成功"),
    404 => println!("未找到"),
    _ => println!("其他错误"),
}
```

### 任务 2: 发送 JSON 数据
```rust
let json = r#"{"key": "value", "number": 42}"#;
let response = client.post("https://api.example.com/submit", json.as_bytes())?;
```

### 任务 3: 处理认证
```rust
let mut request = HttpRequest::new(HttpMethod::Get, "https://api.example.com/protected");
request = request.with_header("Authorization", "Bearer YOUR_TOKEN");
let response = client.send_request(&request)?;
```

### 任务 4: 批量请求（带连接池）
```rust
let client = HttpClient::with_pool(
    HttpClientConfig::default(),
    PoolManagerConfig::default()
);

for i in 0..100 {
    let response = client.get(&format!("https://api.example.com/item/{}", i))?;
}
```

### 任务 5: 下载文件
```rust
let response = client.get("https://example.com/file.pdf")?;
std::fs::write("file.pdf", &response.body)?;
```

### 任务 6: 处理 Cookie
```rust
let cookie_store = Arc::new(CookieStore::new());
let mut config = HttpClientConfig::default();
config.cookie_store = Some(cookie_store);
let client = HttpClient::new(config);

// 登录
client.post("https://api.example.com/login", login_data)?;
// 自动包含 Cookie
client.get("https://api.example.com/protected")?;
```

### 任务 7: 设置超时
```rust
let mut config = HttpClientConfig::default();
config.connect_timeout = Duration::from_secs(5);
config.read_timeout = Duration::from_secs(10);
let client = HttpClient::new(config);
```

### 任务 8: 自定义 User-Agent
```rust
let mut config = HttpClientConfig::default();
config.user_agent = "MyApp/1.0".to_string();
let client = HttpClient::new(config);
```

### 任务 9: 禁用证书验证（仅测试）
```rust
let mut config = HttpClientConfig::default();
config.verify_tls = false;  // ⚠️ 不安全！仅用于测试
let client = HttpClient::new(config);
```

### 任务 10: HTTP/2 配置
```rust
let mut config = HttpClientConfig::default();
config.prefer_http2 = true;   // 优先 HTTP/2
config.prefer_http3 = false;  // 不用 HTTP/3
let client = HttpClient::new(config);
```

---

## 错误处理

### 错误类型
```rust
use fingerprint::HttpClientError;

match client.get(url) {
    Ok(resp) => { /* 处理成功 */ }
    Err(HttpClientError::Timeout) => println!("超时"),
    Err(HttpClientError::TlsError(e)) => println!("TLS 错误: {}", e),
    Err(HttpClientError::ConnectionFailed(e)) => println!("连接失败: {}", e),
    Err(HttpClientError::InvalidUrl(e)) => println!("URL 无效: {}", e),
    Err(HttpClientError::InvalidResponse(e)) => println!("响应无效: {}", e),
    Err(e) => println!("其他错误: {}", e),
}
```

### HTTP 状态码处理
```rust
match response.status_code {
    200..=299 => println!("成功"),
    300..=399 => println!("重定向"),
    400..=499 => println!("客户端错误"),
    500..=599 => println!("服务器错误"),
    _ => println!("未知状态码"),
}
```

---

## 性能优化

### ✓ 推荐做法
```rust
// 1. 重用客户端（不要每次都创建新的）
let client = HttpClient::new(config);
for url in urls {
    client.get(url)?;
}

// 2. 使用连接池进行批量请求
let client = HttpClient::with_pool(config, pool_config);

// 3. 共享 Cookie 存储
let cookie_store = Arc::new(CookieStore::new());
let mut config = HttpClientConfig::default();
config.cookie_store = Some(cookie_store.clone());

// 4. 设置合理的超时
config.connect_timeout = Duration::from_secs(10);
config.read_timeout = Duration::from_secs(15);
```

### ✗ 避免的做法
```rust
// 不好：每次都创建新客户端
for url in urls {
    let client = HttpClient::new(config.clone());
    client.get(url)?;
}

// 不好：无限重定向
let mut config = HttpClientConfig::default();
config.max_redirects = 1000;

// 不好：超长超时
config.connect_timeout = Duration::from_secs(300);
```

---

## 高级功能

### 自定义请求头
```rust
let mut request = HttpRequest::new(HttpMethod::Get, url);
request = request
    .with_header("Accept", "application/json")
    .with_header("Accept-Language", "en-US,en;q=0.9")
    .with_header("User-Agent", "MyApp/1.0")
    .with_header("Authorization", "Bearer token")
    .with_header("X-Custom-Header", "custom-value");
```

### 带浏览器指纹和自定义头部
```rust
let profile = chrome_133();
let mut headers = HTTPHeaders::default();
headers.insert("X-Custom-Header", "value");

let client = HttpClient::with_profile(profile, headers, user_agent);
```

### 重定向限制
```rust
let mut config = HttpClientConfig::default();
config.max_redirects = 3;  // 最多 3 次重定向
let client = HttpClient::new(config);
```

### 获取响应头信息
```rust
let response = client.get(url)?;

// 获取特定头部
if let Some(content_type) = response.headers.get("content-type") {
    println!("内容类型: {}", content_type);
}

// 遍历所有头部
for (key, value) in &response.headers {
    println!("{}: {}", key, value);
}
```

---

## 常见问题解答 (FAQ)

### Q1: 如何处理大文件下载？
```rust
let response = client.get("https://example.com/large-file.zip")?;
if response.status_code == 200 {
    std::fs::write("large-file.zip", &response.body)?;
}
```

### Q2: 如何实现重试机制？
```rust
let mut retries = 0;
loop {
    match client.get(url) {
        Ok(resp) => break,
        Err(e) if retries < 3 => {
            retries += 1;
            eprintln!("失败，重试 {}...", retries);
            std::thread::sleep(Duration::from_secs(1));
        }
        Err(e) => return Err(e.into()),
    }
}
```

### Q3: 如何同时进行多个请求？
```rust
// 使用 Rayon 或 tokio 进行并发请求
use rayon::prelude::*;

let urls: Vec<_> = (0..100).map(|i| format!("https://api.example.com/item/{}", i)).collect();
urls.par_iter().for_each(|url| {
    match client.get(url) {
        Ok(resp) => println!("成功: {}", url),
        Err(e) => eprintln!("失败: {}", e),
    }
});
```

### Q4: 如何模拟特定浏览器的行为？
```rust
let profile = chrome_133();
let ua = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36...";
let client = HttpClient::with_profile(profile, HTTPHeaders::default(), ua.to_string());
// 现在的请求会使用 Chrome 133 的 TLS 指纹
```

### Q5: 如何处理 API 速率限制？
```rust
if response.status_code == 429 {
    // 获取重试等待时间
    let wait_time = response
        .headers
        .get("retry-after")
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(60);
    
    std::thread::sleep(Duration::from_secs(wait_time));
    // 重试
}
```

---

## 编译特性 (Features)

在 `Cargo.toml` 中启用：

```toml
[dependencies]
fingerprint = { version = "1.0", features = ["connection-pool", "http2", "http3"] }
```

可用特性：
- `connection-pool` - 连接池支持
- `http2` - HTTP/2 协议支持
- `http3` - HTTP/3 协议支持
- `rustls-tls` - TLS 支持
- `export` - 配置导出功能
- `dns` - DNS 功能

---

## 快速配置模板

### 模板 1: 高性能 API 客户端
```rust
let pool_config = PoolManagerConfig::default();
let mut config = HttpClientConfig::default();
config.prefer_http2 = true;
config.connect_timeout = Duration::from_secs(10);
config.max_redirects = 5;

let client = HttpClient::with_pool(config, pool_config);
```

### 模板 2: 浏览器模拟客户端
```rust
let profile = get_random_fingerprint()?;
let client = HttpClient::with_profile(
    profile,
    HTTPHeaders::default(),
    "Mozilla/5.0...".to_string()
);
```

### 模板 3: 保守的安全客户端
```rust
let mut config = HttpClientConfig::default();
config.verify_tls = true;
config.max_redirects = 3;
config.connect_timeout = Duration::from_secs(5);

let client = HttpClient::new(config);
```

### 模板 4: 带 Session 的客户端
```rust
let cookie_store = Arc::new(CookieStore::new());
let mut config = HttpClientConfig::default();
config.cookie_store = Some(cookie_store);

let client = HttpClient::new(config);
// 所有 Cookie 会自动管理
```

---

## 相关文件位置

- **HTTP 客户端** - `src/http_client/mod.rs`
- **请求定义** - `src/http_client/request.rs`
- **响应定义** - `src/http_client/response.rs`
- **Cookie 管理** - `src/http_client/cookie.rs`
- **连接池** - `src/http_client/pool.rs`
- **TLS 实现** - `src/http_client/tls.rs`
- **HTTP/2 实现** - `src/http_client/http2.rs`
- **HTTP/3 实现** - `src/http_client/http3.rs`

---

## 更多信息

- **完整指南** - 参见 `REMOTE_UPDATE_CODE_GUIDE.md`
- **代码示例** - 参见 `REMOTE_UPDATE_EXAMPLES.rs`
- **API 文档** - `docs/API.md`
- **项目仓库** - https://github.com/vistone/fingerprint-rust

---

**最后更新**: 2026-02-11
**版本**: 1.0.0

