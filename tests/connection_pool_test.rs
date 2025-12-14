//! 连接池功能测试
//!
//! 验证 netconnpool 集成和连接复用

#[cfg(feature = "connection-pool")]
use fingerprint::{
    get_user_agent_by_profile_name, HttpClient, HttpClientConfig, PoolManagerConfig,
};

#[test]
#[cfg(feature = "connection-pool")]
#[ignore] // 需要网络
fn test_connection_pool_basic() {
    println!("\n========== 连接池基础测试 ==========\n");

    let user_agent = get_user_agent_by_profile_name("chrome_133")
        .unwrap_or_else(|_| "Mozilla/5.0".to_string());

    // 创建带连接池的客户端
    let mut config = HttpClientConfig::default();
    config.user_agent = user_agent;

    let pool_config = PoolManagerConfig {
        max_connections: 10,
        min_idle: 2,
        ..Default::default()
    };

    let client = HttpClient::with_pool(config, pool_config);

    println!("1. 发送第一个请求...");
    match client.get("http://example.com/") {
        Ok(response) => {
            println!("  ✅ 状态码: {}", response.status_code);
            println!("  响应时间: {} ms", response.response_time_ms);
        }
        Err(e) => {
            println!("  ❌ 错误: {:?}", e);
        }
    }

    // 检查连接池统计
    if let Some(stats) = client.pool_stats() {
        println!("\n📊 连接池统计（第一次请求后）:");
        for stat in stats {
            stat.print();
        }
    }

    println!("\n2. 发送第二个请求（应该复用连接）...");
    match client.get("http://example.com/") {
        Ok(response) => {
            println!("  ✅ 状态码: {}", response.status_code);
            println!("  响应时间: {} ms", response.response_time_ms);
        }
        Err(e) => {
            println!("  ❌ 错误: {:?}", e);
        }
    }

    // 再次检查统计
    if let Some(stats) = client.pool_stats() {
        println!("\n📊 连接池统计（第二次请求后）:");
        for stat in stats {
            stat.print();
            // 验证连接复用
            assert!(stat.total_requests >= 2, "应该至少有 2 次请求");
        }
    }
}

#[test]
#[cfg(feature = "connection-pool")]
#[ignore] // 需要网络
fn test_connection_pool_multiple_hosts() {
    println!("\n========== 多主机连接池测试 ==========\n");

    let user_agent = get_user_agent_by_profile_name("chrome_133")
        .unwrap_or_else(|_| "Mozilla/5.0".to_string());

    let mut config = HttpClientConfig::default();
    config.user_agent = user_agent;

    let client = HttpClient::with_pool(config, PoolManagerConfig::default());

    let urls = vec![
        "http://example.com/",
        "http://httpbin.org/get",
        "http://example.com/", // 重复，应该复用连接
    ];

    for (i, url) in urls.iter().enumerate() {
        println!("{}. 请求: {}", i + 1, url);
        match client.get(url) {
            Ok(response) => {
                println!("  ✅ 状态码: {}", response.status_code);
            }
            Err(e) => {
                println!("  ⚠️ 错误: {:?}", e);
            }
        }
    }

    // 显示所有连接池的统计
    if let Some(stats) = client.pool_stats() {
        println!("\n📊 所有连接池统计:");
        println!("  总端点数: {}", stats.len());
        for stat in stats {
            stat.print();
        }
    }
}

#[test]
#[cfg(feature = "connection-pool")]
#[ignore] // 需要网络
fn test_connection_pool_performance() {
    println!("\n========== 连接池性能对比测试 ==========\n");

    use std::time::Instant;

    let user_agent = get_user_agent_by_profile_name("chrome_133")
        .unwrap_or_else(|_| "Mozilla/5.0".to_string());

    // 无连接池客户端
    let mut config1 = HttpClientConfig::default();
    config1.user_agent = user_agent.clone();
    let client_no_pool = HttpClient::new(config1);

    // 有连接池客户端
    let mut config2 = HttpClientConfig::default();
    config2.user_agent = user_agent;
    let client_with_pool = HttpClient::with_pool(config2, PoolManagerConfig::default());

    let test_count = 5;
    let url = "http://example.com/";

    // 测试无连接池
    println!("1. 无连接池测试 ({} 次请求):", test_count);
    let start = Instant::now();
    let mut no_pool_success = 0;
    for i in 0..test_count {
        if client_no_pool.get(url).is_ok() {
            no_pool_success += 1;
        }
        if (i + 1) % 2 == 0 {
            print!(".");
            std::io::Write::flush(&mut std::io::stdout()).unwrap();
        }
    }
    let no_pool_time = start.elapsed();
    println!("\n  ✅ 成功: {}/{}", no_pool_success, test_count);
    println!("  ⏱️ 总耗时: {:?}", no_pool_time);
    println!("  📊 平均: {:?}/请求", no_pool_time / test_count);

    // 测试有连接池
    println!("\n2. 有连接池测试 ({} 次请求):", test_count);
    let start = Instant::now();
    let mut with_pool_success = 0;
    for i in 0..test_count {
        if client_with_pool.get(url).is_ok() {
            with_pool_success += 1;
        }
        if (i + 1) % 2 == 0 {
            print!(".");
            std::io::Write::flush(&mut std::io::stdout()).unwrap();
        }
    }
    let with_pool_time = start.elapsed();
    println!("\n  ✅ 成功: {}/{}", with_pool_success, test_count);
    println!("  ⏱️ 总耗时: {:?}", with_pool_time);
    println!("  📊 平均: {:?}/请求", with_pool_time / test_count);

    // 对比
    println!("\n📈 性能对比:");
    if with_pool_time < no_pool_time {
        let improvement = (no_pool_time.as_millis() - with_pool_time.as_millis()) as f64
            / no_pool_time.as_millis() as f64
            * 100.0;
        println!("  🚀 连接池快 {:.1}%", improvement);
    }

    // 显示连接池统计
    if let Some(stats) = client_with_pool.pool_stats() {
        println!("\n📊 连接池统计:");
        for stat in stats {
            stat.print();
        }
    }
}

#[test]
#[cfg(not(feature = "connection-pool"))]
fn test_connection_pool_not_enabled() {
    // 如果未启用 connection-pool 功能，这个测试会通过
    println!("⚠️ connection-pool 功能未启用");
    println!("使用 --features connection-pool 编译以启用连接池功能");
}
