/// Fingerprint-Rust 远程更新代码调用示例
/// 这个文件展示了如何使用 HTTP 客户端进行各种远程更新操作

#![allow(dead_code)]

use fingerprint::{
    chrome_133, firefox_133, HTTPHeaders, HttpClient, HttpClientConfig, HttpClientError,
    HttpRequest, HttpMethod, get_random_fingerprint,
};
use std::sync::Arc;
use std::time::Duration;

// ============================================================================
// 示例 1: 最简单的 GET 请求
// ============================================================================

fn example_simple_get() -> Result<(), Box<dyn std::error::Error>> {
    let config = HttpClientConfig::default();
    let client = HttpClient::new(config);

    // 发送简单的 GET 请求
    let response = client.get("https://api.github.com/repos/vistone/fingerprint-rust")?;

    println!("状态码: {}", response.status_code);
    println!("响应大小: {} 字节", response.body.len());

    Ok(())
}

// ============================================================================
// 示例 2: 带自定义 User-Agent 的 GET 请求
// ============================================================================

fn example_get_with_user_agent() -> Result<(), Box<dyn std::error::Error>> {
    let mut config = HttpClientConfig::default();
    config.user_agent = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/133.0.0.0 Safari/537.36".to_string();

    let client = HttpClient::new(config);
    let response = client.get("https://example.com")?;

    println!("状态码: {}", response.status_code);

    Ok(())
}

// ============================================================================
// 示例 3: POST 请求 - 发送 JSON 数据
// ============================================================================

fn example_post_json() -> Result<(), Box<dyn std::error::Error>> {
    let config = HttpClientConfig::default();
    let client = HttpClient::new(config);

    // 构建 JSON 请求体
    let json_body = r#"{"username": "testuser", "password": "secret123", "email": "test@example.com"}"#;

    // 发送 POST 请求
    let response = client.post(
        "https://api.example.com/auth/register",
        json_body.as_bytes(),
    )?;

    println!("注册状态码: {}", response.status_code);

    if response.status_code == 201 {
        println!("用户注册成功");
        if let Some(auth_token) = response.headers.get("x-auth-token") {
            println!("获得 Token: {}", auth_token);
        }
    }

    Ok(())
}

// ============================================================================
// 示例 4: 带自定义请求头的请求
// ============================================================================

fn example_custom_headers() -> Result<(), Box<dyn std::error::Error>> {
    let config = HttpClientConfig::default();
    let client = HttpClient::new(config);

    // 创建自定义请求
    let mut request = HttpRequest::new(HttpMethod::Get, "https://api.example.com/protected");

    // 添加自定义头部
    request = request
        .with_header("Authorization", "Bearer YOUR_API_TOKEN_HERE")
        .with_header("X-API-Version", "2.0")
        .with_header("X-Client-ID", "fingerprint-rust");

    // 发送请求
    let response = client.send_request(&request)?;

    println!("API 调用状态码: {}", response.status_code);

    Ok(())
}

// ============================================================================
// 示例 5: 处理重定向
// ============================================================================

fn example_redirect_handling() -> Result<(), Box<dyn std::error::Error>> {
    let mut config = HttpClientConfig::default();
    config.max_redirects = 5;  // 限制最多 5 次重定向

    let client = HttpClient::new(config);

    // 这个 URL 可能会重定向
    let response = client.get("https://example.com/old-api")?;

    println!("最终状态码: {}", response.status_code);
    println!("最终 URL 可能已经重定向");

    Ok(())
}

// ============================================================================
// 示例 6: 使用 Chrome 浏览器指纹
// ============================================================================

fn example_chrome_fingerprint() -> Result<(), Box<dyn std::error::Error>> {
    // 获取 Chrome 133 的指纹
    let profile = chrome_133();

    // 创建客户端
    let client = HttpClient::with_profile(
        profile,
        HTTPHeaders::default(),
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/133.0.0.0 Safari/537.36".to_string(),
    );

    // 使用 Chrome 指纹发送请求
    let response = client.get("https://example.com")?;

    println!("使用 Chrome 133 指纹的请求成功");
    println!("状态码: {}", response.status_code);

    Ok(())
}

// ============================================================================
// 示例 7: 使用 Firefox 浏览器指纹
// ============================================================================

fn example_firefox_fingerprint() -> Result<(), Box<dyn std::error::Error>> {
    let profile = firefox_133();

    let client = HttpClient::with_profile(
        profile,
        HTTPHeaders::default(),
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:133.0) Gecko/20100101 Firefox/133.0".to_string(),
    );

    let response = client.get("https://example.com")?;

    println!("使用 Firefox 133 指纹的请求成功");
    println!("状态码: {}", response.status_code);

    Ok(())
}

// ============================================================================
// 示例 8: 随机浏览器指纹（防反爬虫）
// ============================================================================

fn example_random_fingerprint() -> Result<(), Box<dyn std::error::Error>> {
    // 获取随机浏览器指纹
    let random_fp = get_random_fingerprint()?;

    let client = HttpClient::new(HttpClientConfig::default());

    // 每次请求使用不同的指纹，避免被检测
    for i in 0..3 {
        let response = client.get("https://api.example.com/data")?;
        println!("请求 {} - 状态码: {}", i + 1, response.status_code);
    }

    Ok(())
}

// ============================================================================
// 示例 9: 超时配置
// ============================================================================

fn example_timeout_config() -> Result<(), Box<dyn std::error::Error>> {
    let mut config = HttpClientConfig::default();

    // 设置严格的超时（快速失败）
    config.connect_timeout = Duration::from_secs(5);   // 连接超时 5 秒
    config.read_timeout = Duration::from_secs(10);     // 读取超时 10 秒
    config.write_timeout = Duration::from_secs(10);    // 写入超时 10 秒

    let client = HttpClient::new(config);

    // 如果连接超过 5 秒，会返回 Timeout 错误
    match client.get("https://very-slow-api.example.com") {
        Ok(response) => println!("成功: {}", response.status_code),
        Err(HttpClientError::Timeout) => println!("请求超时"),
        Err(e) => println!("其他错误: {}", e),
    }

    Ok(())
}

// ============================================================================
// 示例 10: 连接池 - 批量请求优化
// ============================================================================

fn example_connection_pool() -> Result<(), Box<dyn std::error::Error>> {
    use fingerprint::PoolManagerConfig;

    let config = HttpClientConfig::default();

    // 配置连接池
    let pool_config = PoolManagerConfig {
        max_idle_per_host: 10,                      // 每个主机最多 10 个空闲连接
        idle_timeout: Duration::from_secs(300),     // 空闲超时 5 分钟
        ..Default::default()
    };

    // 创建带连接池的客户端
    let client = HttpClient::with_pool(config, pool_config);

    // 批量请求（连接会被复用）
    for id in 1..=100 {
        let url = format!("https://api.example.com/items/{}", id);
        let response = client.get(&url)?;
        println!("请求 {} - 状态码: {}", id, response.status_code);
    }

    // 获取连接池统计信息
    if let Some(stats) = client.pool_stats() {
        for stat in stats {
            println!("连接池统计:");
            println!("  活跃连接: {}", stat.active_conns);
            println!("  空闲连接: {}", stat.idle_conns);
        }
    }

    // 清理空闲连接
    client.cleanup_idle_connections();

    Ok(())
}

// ============================================================================
// 示例 11: Cookie 管理
// ============================================================================

fn example_cookie_management() -> Result<(), Box<dyn std::error::Error>> {
    use fingerprint::CookieStore;

    let mut config = HttpClientConfig::default();

    // 创建 Cookie 存储
    let cookie_store = Arc::new(CookieStore::new());
    config.cookie_store = Some(cookie_store.clone());

    let client = HttpClient::new(config);

    // 第一个请求 - 登录
    println!("步骤 1: 登录");
    let login_body = r#"{"username": "user", "password": "pass"}"#;
    let response = client.post(
        "https://api.example.com/auth/login",
        login_body.as_bytes(),
    )?;

    println!("登录状态码: {}", response.status_code);
    println!("Cookie 已自动保存");

    // 第二个请求 - 获取受保护的资源（Cookie 会自动发送）
    println!("步骤 2: 获取受保护的资源");
    let response = client.get("https://api.example.com/user/profile")?;
    println!("获取资源状态码: {}", response.status_code);
    println!("(Cookie 已自动包含在请求中)");

    Ok(())
}

// ============================================================================
// 示例 12: 获取远程配置文件 (JSON)
// ============================================================================

fn example_fetch_remote_config() -> Result<(), Box<dyn std::error::Error>> {
    let config = HttpClientConfig::default();
    let client = HttpClient::new(config);

    // 从远程获取配置文件
    let response = client.get("https://config.example.com/app-config.json")?;

    if response.status_code == 200 {
        // 解析 JSON（这里示例使用字符串）
        let config_text = String::from_utf8(response.body)?;
        println!("获取到的配置:");
        println!("{}", config_text);

        // 在实际应用中，可以使用 serde_json 解析
        // let config_value: serde_json::Value = serde_json::from_str(&config_text)?;
    } else {
        println!("获取配置失败: {}", response.status_code);
    }

    Ok(())
}

// ============================================================================
// 示例 13: 下载文件
// ============================================================================

fn example_download_file() -> Result<(), Box<dyn std::error::Error>> {
    use std::fs::File;
    use std::io::Write;

    let config = HttpClientConfig::default();
    let client = HttpClient::new(config);

    // 下载文件
    let response = client.get("https://example.com/files/document.pdf")?;

    if response.status_code == 200 {
        // 保存文件
        let mut file = File::create("document.pdf")?;
        file.write_all(&response.body)?;
        println!("文件下载成功, 大小: {} 字节", response.body.len());
    } else {
        println!("下载失败: {}", response.status_code);
    }

    Ok(())
}

// ============================================================================
// 示例 14: 错误处理最佳实践
// ============================================================================

fn example_error_handling() -> Result<(), Box<dyn std::error::Error>> {
    let config = HttpClientConfig::default();
    let client = HttpClient::new(config);

    match client.get("https://api.example.com/data") {
        Ok(response) => {
            // 检查 HTTP 状态码
            match response.status_code {
                200 => {
                    println!("请求成功");
                    println!("响应大小: {} 字节", response.body.len());
                }
                404 => {
                    println!("资源不存在");
                }
                401 | 403 => {
                    println!("无权限访问");
                }
                500..=599 => {
                    println!("服务器错误");
                }
                _ => {
                    println!("其他状态码: {}", response.status_code);
                }
            }
        }
        Err(HttpClientError::Timeout) => {
            println!("请求超时，请检查网络连接");
        }
        Err(HttpClientError::TlsError(e)) => {
            println!("TLS 验证失败: {}", e);
        }
        Err(HttpClientError::ConnectionFailed(e)) => {
            println!("连接失败: {}", e);
        }
        Err(HttpClientError::InvalidUrl(e)) => {
            println!("URL 无效: {}", e);
        }
        Err(e) => {
            println!("未知错误: {}", e);
        }
    }

    Ok(())
}

// ============================================================================
// 示例 15: 定时更新（模拟定期检查更新）
// ============================================================================

fn example_periodic_update() -> Result<(), Box<dyn std::error::Error>> {
    use std::thread;

    let config = HttpClientConfig::default();
    let client = HttpClient::new(config);

    // 模拟定期检查更新
    for round in 1..=5 {
        println!("检查 round {}...", round);

        match client.get("https://api.example.com/latest-version") {
            Ok(response) => {
                if response.status_code == 200 {
                    let version = String::from_utf8_lossy(&response.body);
                    println!("最新版本: {}", version);
                }
            }
            Err(e) => {
                println!("检查失败: {}", e);
            }
        }

        // 等待 5 秒后继续
        if round < 5 {
            println!("等待 5 秒...");
            thread::sleep(Duration::from_secs(5));
        }
    }

    Ok(())
}

// ============================================================================
// 示例 16: API 速率限制处理
// ============================================================================

fn example_rate_limit_handling() -> Result<(), Box<dyn std::error::Error>> {
    use std::thread;

    let config = HttpClientConfig::default();
    let client = HttpClient::new(config);

    let mut retry_count = 0;
    let max_retries = 3;

    loop {
        match client.get("https://api.example.com/data") {
            Ok(response) => {
                match response.status_code {
                    200 => {
                        println!("成功获取数据");
                        break;
                    }
                    429 => {
                        // 速率限制
                        retry_count += 1;
                        if retry_count >= max_retries {
                            println!("超过最大重试次数");
                            break;
                        }

                        // 从响应头获取重试等待时间
                        let wait_time = response
                            .headers
                            .get("retry-after")
                            .and_then(|s| s.parse::<u64>().ok())
                            .unwrap_or(60);

                        println!("遇到速率限制，等待 {} 秒后重试...", wait_time);
                        thread::sleep(Duration::from_secs(wait_time));
                    }
                    _ => {
                        println!("其他错误: {}", response.status_code);
                        break;
                    }
                }
            }
            Err(e) => {
                println!("请求失败: {}", e);
                break;
            }
        }
    }

    Ok(())
}

// ============================================================================
// 示例 17: HTTP/2 优先级配置
// ============================================================================

fn example_http2_preference() -> Result<(), Box<dyn std::error::Error>> {
    let mut config = HttpClientConfig::default();
    config.prefer_http2 = true;    // 优先使用 HTTP/2
    config.prefer_http3 = false;   // 不使用 HTTP/3

    let client = HttpClient::new(config);

    // 请求会优先使用 HTTP/2
    let response = client.get("https://example.com")?;
    println!("请求成功（优先使用 HTTP/2）");
    println!("状态码: {}", response.status_code);

    Ok(())
}

// ============================================================================
// 示例 18: TLS 证书验证禁用（仅用于测试）
// ============================================================================

fn example_disable_tls_verify() -> Result<(), Box<dyn std::error::Error>> {
    let mut config = HttpClientConfig::default();
    config.verify_tls = false;  // 禁用 TLS 证书验证（不安全！仅用于测试）

    let client = HttpClient::new(config);

    // 即使证书无效也能连接
    let response = client.get("https://self-signed-certificate.example.com")?;
    println!("成功连接到自签名证书的服务器（仅用于测试）");
    println!("状态码: {}", response.status_code);

    Ok(())
}

// ============================================================================
// 示例 19: 完整的 API 调用流程
// ============================================================================

fn example_complete_api_flow() -> Result<(), Box<dyn std::error::Error>> {
    let mut config = HttpClientConfig::default();
    config.user_agent = "MyApp/1.0 (Fingerprint-Rust)".to_string();

    let client = HttpClient::new(config);

    println!("=== 完整的 API 调用流程 ===");

    // 步骤 1: 获取 API 凭证
    println!("\n步骤 1: 获取 API 凭证");
    let auth_body = r#"{"api_key": "your_key", "secret": "your_secret"}"#;
    let auth_response = client.post("https://api.example.com/auth/token", auth_body.as_bytes())?;

    if auth_response.status_code != 200 {
        println!("获取凭证失败");
        return Ok(());
    }
    println!("✓ 获取凭证成功");

    // 步骤 2: 使用凭证访问 API
    println!("\n步骤 2: 访问受保护的 API");
    let token = "some_token_from_auth_response";
    let mut request = HttpRequest::new(HttpMethod::Get, "https://api.example.com/user/data");
    request = request.with_header("Authorization", &format!("Bearer {}", token));

    let data_response = client.send_request(&request)?;

    if data_response.status_code == 200 {
        println!("✓ 成功获取数据");
    } else {
        println!("✗ 获取数据失败: {}", data_response.status_code);
    }

    // 步骤 3: 更新数据
    println!("\n步骤 3: 更新数据");
    let update_body = r#"{"name": "Updated Name", "age": 30}"#;
    let mut update_request =
        HttpRequest::new(HttpMethod::Post, "https://api.example.com/user/update");
    update_request = update_request
        .with_header("Authorization", &format!("Bearer {}", token))
        .with_body(update_body.as_bytes().to_vec());

    let update_response = client.send_request(&update_request)?;

    if update_response.status_code == 200 {
        println!("✓ 数据更新成功");
    } else {
        println!("✗ 数据更新失败: {}", update_response.status_code);
    }

    Ok(())
}

// ============================================================================
// 主函数 - 运行示例
// ============================================================================

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Fingerprint-Rust HTTP 客户端示例\n");

    // 取消注释下面的任意示例来运行它

    // example_simple_get()?;
    // example_get_with_user_agent()?;
    // example_post_json()?;
    // example_custom_headers()?;
    // example_redirect_handling()?;
    // example_chrome_fingerprint()?;
    // example_firefox_fingerprint()?;
    // example_random_fingerprint()?;
    // example_timeout_config()?;
    // example_connection_pool()?;
    // example_cookie_management()?;
    // example_fetch_remote_config()?;
    // example_download_file()?;
    // example_error_handling()?;
    // example_periodic_update()?;
    // example_rate_limit_handling()?;
    // example_http2_preference()?;
    // example_disable_tls_verify()?;
    // example_complete_api_flow()?;

    println!("✓ 所有示例都已定义，选择任意一个取消注释来运行");

    Ok(())
}

