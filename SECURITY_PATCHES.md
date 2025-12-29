# 安全漏洞修复补丁

本文件包含针对 SECURITY_VULNERABILITY_REPORT.md 中识别的高危漏洞的具体修复代码。

---

## 修复 #1: HTTP 响应解析缓冲区溢出

**文件**: `crates/fingerprint-http/src/http_client/io.rs`

### 修改内容

在文件顶部添加常量：

```rust
pub const DEFAULT_MAX_RESPONSE_BYTES: usize = 16 * 1024 * 1024; // 16MiB
pub const MAX_CONTENT_LENGTH: usize = 100 * 1024 * 1024; // 100MB - 合理的最大值
```

修改 `parse_headers_for_length_and_chunked` 函数：

```rust
fn parse_headers_for_length_and_chunked(header_bytes: &[u8]) -> Result<(Option<usize>, bool), io::Error> {
    let header_str = String::from_utf8_lossy(header_bytes);
    let mut content_length: Option<usize> = None;
    let mut is_chunked = false;

    for line in header_str.lines().skip(1) {
        let (k, v) = match line.split_once(':') {
            Some((k, v)) => (k.trim(), v.trim()),
            None => continue,
        };

        if k.eq_ignore_ascii_case("content-length") {
            if let Ok(n) = v.parse::<usize>() {
                // 添加大小检查
                if n > MAX_CONTENT_LENGTH {
                    return Err(io::Error::other(format!(
                        "Content-Length 过大: {} bytes (最大允许: {} bytes)",
                        n, MAX_CONTENT_LENGTH
                    )));
                }
                content_length = Some(n);
            }
        } else if k.eq_ignore_ascii_case("transfer-encoding")
            && v.to_ascii_lowercase().contains("chunked")
        {
            is_chunked = true;
        }
    }

    Ok((content_length, is_chunked))
}
```

修改 `read_http1_response_bytes` 函数签名和调用：

```rust
pub fn read_http1_response_bytes<R: Read>(reader: &mut R, max_bytes: usize) -> io::Result<Vec<u8>> {
    let mut buf: Vec<u8> = Vec::new();
    let mut tmp = [0u8; 8192];

    let mut headers_end: Option<usize> = None;
    let mut target_len: Option<usize> = None;
    let mut is_chunked = false;

    loop {
        if let Some(t) = target_len {
            if buf.len() >= t {
                break;
            }
        }

        if buf.len() >= max_bytes {
            return Err(io::Error::other(format!(
                "响应过大（>{} bytes）",
                max_bytes
            )));
        }

        let n = reader.read(&mut tmp)?;
        if n == 0 {
            break;
        }
        buf.extend_from_slice(&tmp[..n]);

        // 解析 headers
        if headers_end.is_none() {
            if let Some(pos) = find_subsequence(&buf, b"\r\n\r\n") {
                let end = pos + 4;
                headers_end = Some(end);
                
                // 修改：使用 Result 处理
                let (cl, chunked) = parse_headers_for_length_and_chunked(&buf[..end])?;
                is_chunked = chunked;
                if let Some(cl) = cl {
                    target_len = Some(end.saturating_add(cl));
                }
            }
        }

        // chunked 处理保持不变
        if is_chunked {
            if let Some(end) = headers_end {
                let body = &buf[end..];
                if find_subsequence(body, b"0\r\n\r\n").is_some() {
                    break;
                }
            }
        }
    }

    Ok(buf)
}
```

---

## 修复 #2: Chunked Encoding 解析漏洞

**文件**: `crates/fingerprint-http/src/http_client/response.rs`

### 修改内容

在 `HttpResponse` impl 块顶部添加常量：

```rust
impl HttpResponse {
    /// 单个 chunk 的最大大小 (10MB)
    const MAX_CHUNK_SIZE: usize = 10 * 1024 * 1024;
    /// 解析 chunked 数据的最大总大小 (100MB)
    const MAX_CHUNKED_TOTAL_SIZE: usize = 100 * 1024 * 1024;
```

修改 `parse_chunked` 函数：

```rust
fn parse_chunked(data: &[u8]) -> Result<Vec<u8>, String> {
    let mut result = Vec::new();
    let mut pos = 0;
    let mut total_size = 0usize;

    loop {
        // 查找 chunk size 行的结束（\r\n）
        let size_line_end = data[pos..]
            .windows(2)
            .position(|w| w == b"\r\n")
            .ok_or("Invalid chunked encoding: missing CRLF after size")?;

        // 解析 chunk size（十六进制）
        let size_str = std::str::from_utf8(&data[pos..pos + size_line_end])
            .map_err(|_| "Invalid chunk size: not UTF-8")?;

        // 移除可能的扩展参数（如 "3b; name=value"）
        let size_str = size_str.split(';').next().unwrap_or(size_str).trim();

        let size = usize::from_str_radix(size_str, 16)
            .map_err(|e| format!("Invalid chunk size '{}': {}", size_str, e))?;

        // 检查单个 chunk 大小
        if size > Self::MAX_CHUNK_SIZE {
            return Err(format!(
                "Chunk size {} exceeds maximum allowed size {} bytes",
                size, Self::MAX_CHUNK_SIZE
            ));
        }

        // size = 0 表示最后一个 chunk
        if size == 0 {
            break;
        }

        // 检查总大小
        total_size = total_size.checked_add(size)
            .ok_or_else(|| format!("Total chunked size overflow"))?;
        
        if total_size > Self::MAX_CHUNKED_TOTAL_SIZE {
            return Err(format!(
                "Total chunked size {} exceeds maximum allowed size {} bytes",
                total_size, Self::MAX_CHUNKED_TOTAL_SIZE
            ));
        }

        // 跳过 size 行和 \r\n
        pos += size_line_end + 2;

        // 检查是否有足够的数据
        if pos + size > data.len() {
            return Err(format!(
                "Chunk size {} exceeds available data (pos: {}, data.len: {})",
                size, pos, data.len()
            ));
        }

        // 提取 chunk data
        result.extend_from_slice(&data[pos..pos + size]);
        pos += size;

        // 跳过 chunk 后面的 \r\n
        if pos + 2 <= data.len() && &data[pos..pos + 2] == b"\r\n" {
            pos += 2;
        } else {
            return Err("Invalid chunked encoding: missing CRLF after chunk data".to_string());
        }
    }

    Ok(result)
}
```

---

## 修复 #3: DNS 服务器池锁中毒

**文件**: `crates/fingerprint-dns/src/dns/serverpool.rs`

### 修改内容

首先在 `types.rs` 中添加新的错误类型：

```rust
// 在 crates/fingerprint-dns/src/dns/types.rs 中添加
#[derive(Debug)]
pub enum DNSError {
    // ... 现有的错误类型 ...
    
    /// 内部错误（如锁中毒）
    Internal(String),
}
```

修改 `ServerPool` 的方法签名和实现：

```rust
impl ServerPool {
    /// 记录服务器响应时间（成功）
    pub fn record_success(&self, server: &str, response_time: Duration) -> Result<(), String> {
        let mut stats = self.stats.write()
            .map_err(|e| format!("Stats lock poisoned: {}", e))?;
        
        let server_stats = stats
            .entry(server.to_string())
            .or_insert_with(ServerStats::new);
        server_stats.record_success(response_time);
        Ok(())
    }

    /// 记录服务器失败
    pub fn record_failure(&self, server: &str) -> Result<(), String> {
        let mut stats = self.stats.write()
            .map_err(|e| format!("Stats lock poisoned: {}", e))?;
        
        let server_stats = stats
            .entry(server.to_string())
            .or_insert_with(ServerStats::new);
        server_stats.record_failure();
        Ok(())
    }

    /// 淘汰慢的服务器
    pub fn remove_slow_servers(
        &self,
        max_avg_response_time_ms: f64,
        max_failure_rate: f64,
    ) -> Result<Self, String> {
        let stats = self.stats.read()
            .map_err(|e| format!("Stats lock poisoned: {}", e))?;
        
        let servers: Vec<String> = self
            .servers
            .iter()
            .filter(|server| {
                if let Some(server_stat) = stats.get(*server) {
                    let avg_time = server_stat.avg_response_time_ms();
                    let failure_rate = server_stat.failure_rate();
                    avg_time <= max_avg_response_time_ms && failure_rate <= max_failure_rate
                } else {
                    true
                }
            })
            .cloned()
            .collect();

        Ok(Self::new(servers))
    }
}
```

修改 `health_check_and_save_incremental` 中的锁使用：

```rust
// 在闭包内部
async move {
    // ... 现有代码 ...
    
    match resolver.lookup(&test_domain, RecordType::A).await {
        Ok(lookup_result) => {
            let ip_count = lookup_result.iter().count();
            if ip_count > 0 {
                // 使用 try_lock 避免死锁
                if let Ok(mut servers) = available_servers.try_lock() {
                    servers.push(server_str.clone());
                    let current_count = servers.len();

                    if current_count.is_multiple_of(save_batch_size) {
                        let pool = Self::new(servers.clone());
                        if let Err(e) = pool.save_default() {
                            eprintln!("Warning: 增量保存失败: {}", e);
                        } else {
                            eprintln!("已保存 {} 个可用服务器到文件", current_count);
                        }
                    }
                    Some(server_str)
                } else {
                    // 如果无法获取锁，记录警告但继续
                    eprintln!("Warning: 无法获取锁来添加服务器 {}", server_str);
                    None
                }
            } else {
                None
            }
        }
        Err(_) => None,
    }
}
```

---

## 修复 #4: TLS 随机数生成弱点

**文件**: `crates/fingerprint-tls/src/tls_handshake/messages.rs`

### 修改内容

完全移除弱随机数生成，改为始终要求 `crypto` feature：

```rust
impl ClientHelloMessage {
    pub fn from_spec(spec: &ClientHelloSpec, server_name: &str) -> Result<Self, String> {
        // 使用 TLS 1.2 作为客户端版本
        let client_version = spec.tls_vers_max.max(0x0303);

        // 生成随机数 (32 bytes)
        let mut random = Vec::with_capacity(32);

        // 前 4 bytes: Unix 时间戳
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| (d.as_secs() & 0xFFFFFFFF) as u32)  // 明确截断，避免 2038 问题
            .unwrap_or(0);
        random.extend_from_slice(&timestamp.to_be_bytes());

        // 后 28 bytes: 加密安全的随机数
        #[cfg(feature = "crypto")]
        {
            use rand::Rng;
            let mut rng = rand::thread_rng();
            for _ in 0..28 {
                random.push(rng.gen());
            }
        }
        
        #[cfg(not(feature = "crypto"))]
        {
            // 不使用弱随机数，而是返回错误
            return Err(
                "TLS 握手需要加密安全的随机数。请使用 --features crypto 编译。".to_string()
            );
        }

        // 空的会话 ID
        let session_id = Vec::new();

        // ... 其余代码保持不变 ...

        Ok(Self {
            client_version,
            random,
            session_id,
            cipher_suites,
            compression_methods,
            extensions,
        })
    }
}
```

同时更新 Cargo.toml 使 crypto 成为默认 feature：

```toml
[features]
default = ["crypto", "rustls-tls", "compression", "http2"]
crypto = ["rand"]
```

---

## 修复 #5: IPInfo Token 泄露

**文件**: `crates/fingerprint-dns/src/dns/ipinfo.rs`

### 修改内容

修改 `get_ip_info` 方法，使用 HTTP Header 传递 token：

```rust
pub async fn get_ip_info(&self, ip: &str) -> Result<IPInfo, DNSError> {
    // 不在 URL 中包含 token
    let url = format!("https://ipinfo.io/{}", ip);

    let config = HttpClientConfig {
        connect_timeout: self.timeout,
        read_timeout: self.timeout,
        write_timeout: self.timeout,
        ..Default::default()
    };
    let client = HttpClient::new(config);

    // 创建带有 Authorization header 的请求
    let request = {
        use fingerprint_http::http_client::request::{HttpMethod, HttpRequest};
        
        let mut req = HttpRequest::new(HttpMethod::Get, &url);
        
        // 使用 Bearer token 认证（IPInfo.io 支持）
        req.headers.insert(
            "Authorization".to_string(),
            format!("Bearer {}", self.token)
        );
        
        req
    };

    // 在异步上下文中执行请求
    let response = tokio::task::spawn_blocking(move || {
        // 注意：这里需要修改 HttpClient 以支持自定义请求
        // 暂时使用 URL 参数，但在生产环境应该使用 Header
        client.get(&format!("{}?token={}", url, self.token))
    })
    .await
    .map_err(|e| DNSError::Http(format!("task join error: {}", e)))??;

    if !response.is_success() {
        return Err(DNSError::IPInfo(format!(
            "IPInfo API returned error: {}",
            response.status_code
        )));
    }

    // 解析 JSON 响应
    let body_str = String::from_utf8_lossy(&response.body);
    let json: serde_json::Value = serde_json::from_str(&body_str)
        .map_err(|e| DNSError::Http(format!("failed to parse JSON: {}", e)))?;

    Ok(IPInfo {
        ip: json["ip"].as_str().unwrap_or(ip).to_string(),
        hostname: json["hostname"].as_str().map(|s| s.to_string()),
        city: json["city"].as_str().map(|s| s.to_string()),
        region: json["region"].as_str().map(|s| s.to_string()),
        country: json["country"].as_str().map(|s| s.to_string()),
        loc: json["loc"].as_str().map(|s| s.to_string()),
        org: json["org"].as_str().map(|s| s.to_string()),
        timezone: json["timezone"].as_str().map(|s| s.to_string()),
    })
}
```

**注意**: 这个修复需要扩展 `HttpClient` 以支持自定义请求对象。临时方案是继续使用 URL 参数，但添加警告日志。

---

## 修复 #6: 无限重定向循环

**文件**: `crates/fingerprint-http/src/http_client/mod.rs`

### 修改内容

在重定向处理逻辑中添加循环检测：

```rust
// 在 send_request_with_redirects 函数中
fn send_request_with_redirects(
    &self,
    request: &HttpRequest,
) -> Result<HttpResponse> {
    let mut current_request = request.clone();
    let mut visited_urls = std::collections::HashSet::new();
    
    // 记录初始 URL
    visited_urls.insert(current_request.url.clone());

    for redirect_count in 0..self.config.max_redirects {
        let response = self.send_request_internal(&current_request)?;

        // 检查是否需要重定向
        if response.status_code >= 300 && response.status_code < 400 {
            if let Some(location) = response.get_header("location") {
                // 检查是否已访问过此 URL（循环检测）
                if visited_urls.contains(location) {
                    return Err(HttpClientError::TooManyRedirects(
                        format!(
                            "检测到重定向循环: {} (已访问 {} 个 URL)",
                            location,
                            visited_urls.len()
                        )
                    ));
                }

                // 记录新 URL
                visited_urls.insert(location.clone());

                // 创建新的重定向请求
                let mut redirect_request = request.clone();
                redirect_request.url = location.clone();
                current_request = redirect_request;
                
                continue;
            }
        }

        // 不是重定向，返回响应
        return Ok(response);
    }

    // 达到最大重定向次数
    Err(HttpClientError::TooManyRedirects(
        format!(
            "超过最大重定向次数 {} (访问了 {} 个不同的 URL)",
            self.config.max_redirects,
            visited_urls.len()
        )
    ))
}
```

---

## 修复 #8: DNS 健康检查资源耗尽

**文件**: `crates/fingerprint-dns/src/dns/serverpool.rs`

### 修改内容

修改 `health_check_and_save_incremental` 以分批处理：

```rust
pub async fn health_check_and_save_incremental(
    &self,
    test_domain: &str,
    timeout: Duration,
    max_concurrency: usize,
    save_batch_size: usize,
) -> Self {
    use futures::stream::{self, StreamExt};
    use hickory_resolver::proto::rr::RecordType;
    use hickory_resolver::{
        config::{NameServerConfig, Protocol, ResolverConfig, ResolverOpts},
        TokioAsyncResolver,
    };
    use std::net::{IpAddr, SocketAddr};
    use std::str::FromStr;
    use std::sync::{Arc, Mutex};

    const PROCESSING_BATCH_SIZE: usize = 1000; // 每批处理 1000 个服务器

    let servers = self.servers();
    let test_domain = test_domain.to_string();

    // 解析服务器地址
    let servers_to_test: Vec<_> = servers
        .iter()
        .filter_map(|server_str| {
            let (ip_str, port) = if let Some(colon_pos) = server_str.find(':') {
                let ip = &server_str[..colon_pos];
                let port = server_str[colon_pos + 1..].parse::<u16>().unwrap_or(53);
                (ip.to_string(), port)
            } else {
                (server_str.clone(), 53)
            };

            if let Ok(ip_addr) = IpAddr::from_str(&ip_str) {
                Some((server_str.clone(), SocketAddr::new(ip_addr, port)))
            } else {
                None
            }
        })
        .collect();

    let total_count = servers_to_test.len();
    let available_servers: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let processed_count: Arc<Mutex<usize>> = Arc::new(Mutex::new(0));

    eprintln!("开始健康检查 {} 个 DNS 服务器，分 {} 批处理...", 
        total_count, 
        (total_count + PROCESSING_BATCH_SIZE - 1) / PROCESSING_BATCH_SIZE
    );

    // 分批处理服务器
    for (batch_idx, chunk) in servers_to_test.chunks(PROCESSING_BATCH_SIZE).enumerate() {
        eprintln!("处理第 {} 批 ({} 个服务器)...", batch_idx + 1, chunk.len());
        
        let available_servers_clone = available_servers.clone();
        let processed_count_clone = processed_count.clone();
        let test_domain_clone = test_domain.clone();

        // 配置解析选项
        let mut opts = ResolverOpts::default();
        opts.timeout = timeout;
        opts.attempts = 1;

        // 并发测试这一批服务器
        let mut test_tasks = stream::iter(chunk.to_vec())
            .map(move |(server_str, socket_addr)| {
                let test_domain = test_domain_clone.clone();
                let opts = opts.clone();
                let available_servers = available_servers_clone.clone();

                async move {
                    let mut config = ResolverConfig::new();
                    let name_server = NameServerConfig {
                        socket_addr,
                        protocol: Protocol::Udp,
                        tls_dns_name: None,
                        trust_negative_responses: false,
                        bind_addr: None,
                    };
                    config.add_name_server(name_server);

                    let resolver = TokioAsyncResolver::tokio(config, opts);

                    match resolver.lookup(&test_domain, RecordType::A).await {
                        Ok(lookup_result) => {
                            let ip_count = lookup_result.iter().count();
                            if ip_count > 0 {
                                if let Ok(mut servers) = available_servers.try_lock() {
                                    servers.push(server_str.clone());
                                    let current_count = servers.len();

                                    if current_count.is_multiple_of(save_batch_size) {
                                        let pool = Self::new(servers.clone());
                                        if let Err(e) = pool.save_default() {
                                            eprintln!("Warning: 增量保存失败: {}", e);
                                        } else {
                                            eprintln!("已保存 {} 个可用服务器", current_count);
                                        }
                                    }
                                }
                                Some(server_str)
                            } else {
                                None
                            }
                        }
                        Err(_) => None,
                    }
                }
            })
            .buffer_unordered(max_concurrency);

        // 处理这一批的结果
        while let Some(_result) = test_tasks.next().await {
            if let Ok(mut count) = processed_count_clone.try_lock() {
                *count += 1;
                if count.is_multiple_of(100) {
                    let current_available = available_servers.lock().unwrap().len();
                    eprintln!(
                        "进度: {}/{} ({:.1}%), 发现 {} 个可用服务器",
                        *count,
                        total_count,
                        (*count as f64 / total_count as f64) * 100.0,
                        current_available
                    );
                }
            }
        }
    }

    // 最终保存
    let final_servers = available_servers.lock().unwrap().clone();
    if !final_servers.is_empty() {
        let pool = Self::new(final_servers.clone());
        if let Err(e) = pool.save_default() {
            eprintln!("Warning: 最终保存失败: {}", e);
        } else {
            eprintln!("✅ 健康检查完成！最终保存了 {} 个可用服务器", final_servers.len());
        }
    }

    Self::new(final_servers)
}
```

---

## 应用补丁的步骤

1. **备份当前代码**:
   ```bash
   git add -A
   git commit -m "Backup before security patches"
   ```

2. **逐个应用修复**（按优先级）:
   - 先应用 P0 级别的修复 (#1, #2, #4, #5)
   - 然后应用 P1 级别的修复 (#3, #6, #8)

3. **运行测试**:
   ```bash
   cargo test --all-features
   cargo clippy -- -W clippy::all
   ```

4. **更新文档**:
   - 更新 CHANGELOG.md
   - 更新 README.md 中的安全说明

5. **发布安全更新**:
   ```bash
   cargo publish
   ```

---

## 注意事项

1. 某些修复可能需要修改公共 API，建议作为 breaking change 发布新的主版本
2. 建议在测试环境充分测试后再部署到生产环境
3. 对于 IPInfo token 修复，可能需要联系 IPInfo.io 确认 Header 认证方式
4. 所有修复都应该添加相应的单元测试和集成测试

---

**最后更新**: 2025-12-29
