//! Google Earth API 完整协议测试
//!
//! 测试 https://kh.google.com/rt/earth/PlanetoidMetadata
//! 验证 HTTP/1.1、HTTP/2、HTTP/3 都能正常工作

use fingerprint::{get_user_agent_by_profile_name, HttpClient, HttpClientConfig};

const TEST_URL: &str = "https://kh.google.com/rt/earth/PlanetoidMetadata";
const TEST_HOST: &str = "kh.google.com";
const TEST_PORT: u16 = 443;
const TEST_PATH: &str = "/rt/earth/PlanetoidMetadata";

/// 测试 HTTP/1.1
#[test]
#[ignore] // 需要网络连接
fn test_google_earth_http1() {
    println!("\n╔══════════════════════════════════════════════════════════╗");
    println!("║     测试 Google Earth API - HTTP/1.1                     ║");
    println!("╚══════════════════════════════════════════════════════════╝\n");

    let user_agent =
        get_user_agent_by_profile_name("chrome_133").unwrap_or_else(|_| "Mozilla/5.0".to_string());

    let config = HttpClientConfig {
        user_agent,
        prefer_http2: false,
        prefer_http3: false,
        ..Default::default()
    };

    let client = HttpClient::new(config);

    println!("📡 发送请求: {}", TEST_URL);
    println!("协议: HTTP/1.1\n");

    match client.get(TEST_URL) {
        Ok(response) => {
            println!("✅ 成功！");
            println!("  HTTP 版本: {}", response.http_version);
            println!("  状态码: {}", response.status_code);
            println!("  Body 大小: {} bytes", response.body.len());

            // 验证响应
            assert!(
                response.is_success(),
                "预期成功响应，实际状态码: {}",
                response.status_code
            );
            assert_eq!(response.status_code, 200);
            assert!(!response.body.is_empty(), "响应体不应该为空");

            // 检查内容类型
            if let Some(content_type) = response.headers.get("content-type") {
                println!("  Content-Type: {}", content_type);
            }

            println!("\n✅ HTTP/1.1 测试通过！");
        }
        Err(e) => {
            panic!("❌ HTTP/1.1 测试失败: {}", e);
        }
    }
}

/// 测试 HTTP/2
#[test]
#[cfg(feature = "http2")]
#[ignore] // 需要网络连接
fn test_google_earth_http2() {
    println!("\n╔══════════════════════════════════════════════════════════╗");
    println!("║     测试 Google Earth API - HTTP/2                       ║");
    println!("╚══════════════════════════════════════════════════════════╝\n");

    let user_agent =
        get_user_agent_by_profile_name("chrome_133").unwrap_or_else(|_| "Mozilla/5.0".to_string());

    let config = HttpClientConfig {
        user_agent,
        prefer_http2: true,
        prefer_http3: false,
        ..Default::default()
    };

    let client = HttpClient::new(config);

    println!("📡 发送请求: {}", TEST_URL);
    println!("协议: HTTP/2\n");

    match client.get(TEST_URL) {
        Ok(response) => {
            println!("✅ 成功！");
            println!("  HTTP 版本: {}", response.http_version);
            println!("  状态码: {}", response.status_code);
            println!("  Body 大小: {} bytes", response.body.len());

            // 验证响应
            assert!(
                response.is_success(),
                "预期成功响应，实际状态码: {}",
                response.status_code
            );
            assert_eq!(response.status_code, 200);
            assert!(!response.body.is_empty(), "响应体不应该为空");

            // 验证是 HTTP/2
            assert!(
                response.http_version.contains("HTTP/2") || response.http_version.contains("h2"),
                "预期 HTTP/2 响应，实际: {}",
                response.http_version
            );

            println!("\n✅ HTTP/2 测试通过！");
        }
        Err(e) => {
            panic!("❌ HTTP/2 测试失败: {}", e);
        }
    }
}

/// 测试 HTTP/3
#[test]
#[cfg(feature = "http3")]
#[ignore] // 需要网络连接
fn test_google_earth_http3() {
    println!("\n╔══════════════════════════════════════════════════════════╗");
    println!("║     测试 Google Earth API - HTTP/3                       ║");
    println!("╚══════════════════════════════════════════════════════════╝\n");

    let user_agent =
        get_user_agent_by_profile_name("chrome_133").unwrap_or_else(|_| "Mozilla/5.0".to_string());

    let config = HttpClientConfig {
        user_agent,
        prefer_http2: false,
        prefer_http3: true,
        ..Default::default()
    };

    let client = HttpClient::new(config);

    println!("📡 发送请求: {}", TEST_URL);
    println!("协议: HTTP/3 (QUIC)\n");

    match client.get(TEST_URL) {
        Ok(response) => {
            println!("✅ 成功！");
            println!("  HTTP 版本: {}", response.http_version);
            println!("  状态码: {}", response.status_code);
            println!("  Body 大小: {} bytes", response.body.len());

            // 验证响应
            assert!(
                response.is_success(),
                "预期成功响应，实际状态码: {}",
                response.status_code
            );
            assert_eq!(response.status_code, 200);
            assert!(!response.body.is_empty(), "响应体不应该为空");

            // 验证是 HTTP/3
            assert!(
                response.http_version.contains("HTTP/3")
                    || response.http_version.contains("h3")
                    || response.http_version.contains("quic"),
                "预期 HTTP/3 响应，实际: {}",
                response.http_version
            );

            println!("\n✅ HTTP/3 测试通过！");
        }
        Err(e) => {
            panic!("❌ HTTP/3 测试失败: {}", e);
        }
    }
}

/// 测试 HTTP/1.1 with 连接池
#[test]
#[cfg(feature = "connection-pool")]
#[ignore] // 需要网络连接
fn test_google_earth_http1_with_pool() {
    println!("\n╔══════════════════════════════════════════════════════════╗");
    println!("║     测试 Google Earth API - HTTP/1.1 + 连接池            ║");
    println!("╚══════════════════════════════════════════════════════════╝\n");

    let user_agent =
        get_user_agent_by_profile_name("chrome_133").unwrap_or_else(|_| "Mozilla/5.0".to_string());

    let config = HttpClientConfig {
        user_agent,
        prefer_http2: false,
        prefer_http3: false,
        ..Default::default()
    };

    let client = HttpClient::with_pool(
        config,
        fingerprint::http_client::PoolManagerConfig::default(),
    );

    println!("📡 发送 3 个连续请求测试连接复用");
    println!("URL: {}\n", TEST_URL);

    for i in 1..=3 {
        println!("请求 {}/3...", i);

        match client.get(TEST_URL) {
            Ok(response) => {
                println!(
                    "  ✅ 成功: {} {}",
                    response.http_version, response.status_code
                );
                println!("  Body: {} bytes", response.body.len());

                assert_eq!(response.status_code, 200);
                assert!(!response.body.is_empty());
            }
            Err(e) => {
                panic!("❌ 请求 {} 失败: {}", i, e);
            }
        }
    }

    // 检查连接池统计
    if let Some(stats) = client.pool_stats() {
        println!("\n📊 连接池统计:");
        for stat in stats {
            println!("  端点: {}", stat.endpoint);
            println!("  总请求: {}", stat.total_requests);
            println!("  活跃连接: {}", stat.active_connections);
            println!("  空闲连接: {}", stat.idle_connections);
        }
    }

    println!("\n✅ HTTP/1.1 + 连接池测试通过！");
}

/// 测试 HTTP/2 with 连接池（异步）
#[tokio::test]
#[cfg(all(feature = "connection-pool", feature = "http2"))]
#[ignore] // 需要网络连接
async fn test_google_earth_http2_with_pool() {
    use fingerprint::http_client::{http2_pool, pool::ConnectionPoolManager, PoolManagerConfig};
    use fingerprint::HttpRequest;
    use std::sync::Arc;

    println!("\n╔══════════════════════════════════════════════════════════╗");
    println!("║     测试 Google Earth API - HTTP/2 + 连接池              ║");
    println!("╚══════════════════════════════════════════════════════════╝\n");

    let user_agent =
        get_user_agent_by_profile_name("chrome_133").unwrap_or_else(|_| "Mozilla/5.0".to_string());

    let config = HttpClientConfig {
        user_agent,
        prefer_http2: true,
        ..Default::default()
    };

    let pool_manager = Arc::new(ConnectionPoolManager::new(PoolManagerConfig::default()));
    let request = HttpRequest::new(fingerprint::http_client::request::HttpMethod::Get, TEST_URL);

    println!("📡 发送 HTTP/2 请求（使用连接池）");
    println!("URL: {}\n", TEST_URL);

    match http2_pool::send_http2_request_with_pool(
        TEST_HOST,
        TEST_PORT,
        TEST_PATH,
        &request,
        &config,
        &pool_manager,
    )
    .await
    {
        Ok(response) => {
            println!("✅ 成功！");
            println!("  HTTP 版本: {}", response.http_version);
            println!("  状态码: {}", response.status_code);
            println!("  Body 大小: {} bytes", response.body.len());

            assert_eq!(response.status_code, 200);
            assert!(!response.body.is_empty());
            assert!(response.http_version.contains("HTTP/2"));

            println!("\n✅ HTTP/2 + 连接池测试通过！");
        }
        Err(e) => {
            panic!("❌ HTTP/2 + 连接池测试失败: {}", e);
        }
    }
}

/// 测试 HTTP/3 with 连接池（异步）
#[tokio::test]
#[cfg(all(feature = "connection-pool", feature = "http3"))]
#[ignore] // 需要网络连接
async fn test_google_earth_http3_with_pool() {
    use fingerprint::http_client::{http3_pool, pool::ConnectionPoolManager, PoolManagerConfig};
    use fingerprint::HttpRequest;
    use std::sync::Arc;

    println!("\n╔══════════════════════════════════════════════════════════╗");
    println!("║     测试 Google Earth API - HTTP/3 + 连接池              ║");
    println!("╚══════════════════════════════════════════════════════════╝\n");

    let user_agent =
        get_user_agent_by_profile_name("chrome_133").unwrap_or_else(|_| "Mozilla/5.0".to_string());

    let config = HttpClientConfig {
        user_agent,
        prefer_http3: true,
        ..Default::default()
    };

    let pool_manager = Arc::new(ConnectionPoolManager::new(PoolManagerConfig::default()));
    let request = HttpRequest::new(fingerprint::http_client::request::HttpMethod::Get, TEST_URL);

    println!("📡 发送 HTTP/3 请求（使用连接池）");
    println!("URL: {}\n", TEST_URL);

    match http3_pool::send_http3_request_with_pool(
        TEST_HOST,
        TEST_PORT,
        TEST_PATH,
        &request,
        &config,
        &pool_manager,
    )
    .await
    {
        Ok(response) => {
            println!("✅ 成功！");
            println!("  HTTP 版本: {}", response.http_version);
            println!("  状态码: {}", response.status_code);
            println!("  Body 大小: {} bytes", response.body.len());

            assert_eq!(response.status_code, 200);
            assert!(!response.body.is_empty());
            assert!(response.http_version.contains("HTTP/3"));

            println!("\n✅ HTTP/3 + 连接池测试通过！");
        }
        Err(e) => {
            panic!("❌ HTTP/3 + 连接池测试失败: {}", e);
        }
    }
}

/// 综合测试：所有协议按顺序测试
#[test]
#[ignore] // 需要网络连接
fn test_google_earth_all_protocols() {
    println!("\n╔══════════════════════════════════════════════════════════╗");
    println!("║     Google Earth API 全协议测试                          ║");
    println!("║     URL: https://kh.google.com/rt/earth/PlanetoidMetadata ║");
    println!("╚══════════════════════════════════════════════════════════╝\n");

    let user_agent =
        get_user_agent_by_profile_name("chrome_133").unwrap_or_else(|_| "Mozilla/5.0".to_string());

    let mut results = Vec::new();

    // 测试 HTTP/1.1
    println!("🔹 测试 1/3: HTTP/1.1");
    let config_h1 = HttpClientConfig {
        user_agent: user_agent.clone(),
        prefer_http2: false,
        prefer_http3: false,
        ..Default::default()
    };
    let client_h1 = HttpClient::new(config_h1);

    match client_h1.get(TEST_URL) {
        Ok(response) => {
            println!(
                "  ✅ HTTP/1.1: {} ({})",
                response.status_code, response.http_version
            );
            results.push(("HTTP/1.1", true, response.status_code));
        }
        Err(e) => {
            println!("  ❌ HTTP/1.1: {}", e);
            results.push(("HTTP/1.1", false, 0));
        }
    }

    // 测试 HTTP/2
    #[cfg(feature = "http2")]
    {
        println!("\n🔹 测试 2/3: HTTP/2");
        let config_h2 = HttpClientConfig {
            user_agent: user_agent.clone(),
            prefer_http2: true,
            prefer_http3: false,
            ..Default::default()
        };
        let client_h2 = HttpClient::new(config_h2);

        match client_h2.get(TEST_URL) {
            Ok(response) => {
                println!(
                    "  ✅ HTTP/2: {} ({})",
                    response.status_code, response.http_version
                );
                results.push(("HTTP/2", true, response.status_code));
            }
            Err(e) => {
                println!("  ❌ HTTP/2: {}", e);
                results.push(("HTTP/2", false, 0));
            }
        }
    }

    // 测试 HTTP/3
    #[cfg(feature = "http3")]
    {
        println!("\n🔹 测试 3/3: HTTP/3");
        let config_h3 = HttpClientConfig {
            user_agent,
            prefer_http2: false,
            prefer_http3: true,
            ..Default::default()
        };
        let client_h3 = HttpClient::new(config_h3);

        match client_h3.get(TEST_URL) {
            Ok(response) => {
                println!(
                    "  ✅ HTTP/3: {} ({})",
                    response.status_code, response.http_version
                );
                results.push(("HTTP/3", true, response.status_code));
            }
            Err(e) => {
                println!("  ❌ HTTP/3: {}", e);
                results.push(("HTTP/3", false, 0));
            }
        }
    }

    // 汇总结果
    println!("\n╔══════════════════════════════════════════════════════════╗");
    println!("║     测试结果汇总                                          ║");
    println!("╚══════════════════════════════════════════════════════════╝\n");

    let mut success_count = 0;
    let total_count = results.len();

    for (protocol, success, status) in &results {
        if *success {
            println!("✅ {}: 状态码 {}", protocol, status);
            success_count += 1;
        } else {
            println!("❌ {}: 失败", protocol);
        }
    }

    println!("\n📊 成功率: {}/{}", success_count, total_count);

    // 所有测试都必须通过
    assert_eq!(
        success_count, total_count,
        "部分协议测试失败！预期 {} 个通过，实际 {} 个通过",
        total_count, success_count
    );

    println!("\n✅✅✅ 所有协议测试通过！✅✅✅");
}
