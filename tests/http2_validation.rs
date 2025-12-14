//! HTTP/2 真实网络验证测试
//!
//! 测试所有 66 个浏览器指纹在 HTTP/2 下的表现

use fingerprint::{
    get_user_agent_by_profile_name, mapped_tls_clients, HttpClient, HttpClientConfig,
};

#[test]
#[ignore]
fn test_http2_google_api() {
    // 测试 HTTP/2 连接到 Google API
    let profile = mapped_tls_clients()
        .get("chrome_133")
        .expect("无法获取 Chrome 133 profile");

    let user_agent =
        get_user_agent_by_profile_name("chrome_133").unwrap_or_else(|_| "Mozilla/5.0".to_string());

    let mut config = HttpClientConfig::default();
    config.user_agent = user_agent;
    config.prefer_http2 = true;
    config.prefer_http3 = false;

    let client = HttpClient::new(config);

    let result = client.get("https://www.google.com/");
    assert!(result.is_ok(), "HTTP/2 请求失败: {:?}", result.err());

    let response = result.unwrap();
    println!("HTTP 版本: {}", response.http_version);
    println!("状态码: {}", response.status_code);
    println!("响应时间: {} ms", response.response_time_ms);

    assert_eq!(response.http_version, "HTTP/2");
    assert!(response.is_success());
}

#[test]
#[ignore]
fn test_all_browsers_http2() {
    println!("\n========== 开始测试所有浏览器的 HTTP/2 支持 ==========\n");

    let profiles = mapped_tls_clients();
    let total = profiles.len();
    let mut success_count = 0;
    let mut failed_profiles = Vec::new();

    for (profile_name, profile) in profiles.iter() {
        print!("测试 {} ... ", profile_name);

        let user_agent = get_user_agent_by_profile_name(profile_name)
            .unwrap_or_else(|_| "Mozilla/5.0".to_string());

        let mut config = HttpClientConfig::default();
        config.user_agent = user_agent;
        config.prefer_http2 = true;
        config.prefer_http3 = false;

        let client = HttpClient::new(config);

        // 测试多个端点
        let test_urls = vec!["https://www.google.com/", "https://www.cloudflare.com/"];

        let mut profile_success = false;
        for url in &test_urls {
            if let Ok(response) = client.get(url) {
                if response.http_version == "HTTP/2" && response.is_success() {
                    profile_success = true;
                    break;
                }
            }
        }

        if profile_success {
            println!("✓ 成功");
            success_count += 1;
        } else {
            println!("✗ 失败");
            failed_profiles.push(profile_name.clone());
        }
    }

    println!(
        "\n========== HTTP/2 测试完成 ==========\n\
         总数: {}\n\
         成功: {}\n\
         失败: {}\n\
         成功率: {:.2}%\n",
        total,
        success_count,
        total - success_count,
        (success_count as f64 / total as f64) * 100.0
    );

    if !failed_profiles.is_empty() {
        println!("失败的浏览器指纹:");
        for profile in &failed_profiles {
            println!("  - {}", profile);
        }
    }

    // 要求至少 90% 成功率
    assert!(
        (success_count as f64 / total as f64) >= 0.9,
        "HTTP/2 成功率低于 90%"
    );
}

#[test]
#[ignore]
fn test_http2_vs_http1() {
    println!("\n========== HTTP/2 vs HTTP/1.1 性能对比 ==========\n");

    let profile_name = "chrome_133";
    let user_agent =
        get_user_agent_by_profile_name(profile_name).unwrap_or_else(|_| "Mozilla/5.0".to_string());

    // HTTP/1.1 测试
    let mut config_h1 = HttpClientConfig::default();
    config_h1.user_agent = user_agent.clone();
    config_h1.prefer_http2 = false;
    let client_h1 = HttpClient::new(config_h1);

    let start_h1 = std::time::Instant::now();
    let response_h1 = client_h1
        .get("https://www.google.com/")
        .expect("HTTP/1.1 请求失败");
    let time_h1 = start_h1.elapsed().as_millis();

    // HTTP/2 测试
    let mut config_h2 = HttpClientConfig::default();
    config_h2.user_agent = user_agent;
    config_h2.prefer_http2 = true;
    let client_h2 = HttpClient::new(config_h2);

    let start_h2 = std::time::Instant::now();
    let response_h2 = client_h2
        .get("https://www.google.com/")
        .expect("HTTP/2 请求失败");
    let time_h2 = start_h2.elapsed().as_millis();

    println!("HTTP/1.1 版本: {}", response_h1.http_version);
    println!("HTTP/1.1 响应时间: {} ms", time_h1);
    println!("HTTP/1.1 状态码: {}", response_h1.status_code);
    println!();
    println!("HTTP/2 版本: {}", response_h2.http_version);
    println!("HTTP/2 响应时间: {} ms", time_h2);
    println!("HTTP/2 状态码: {}", response_h2.status_code);
    println!();

    if time_h2 < time_h1 {
        println!(
            "HTTP/2 比 HTTP/1.1 快 {} ms ({:.1}%)",
            time_h1 - time_h2,
            ((time_h1 - time_h2) as f64 / time_h1 as f64) * 100.0
        );
    } else {
        println!("HTTP/1.1 比 HTTP/2 快 {} ms", time_h2 - time_h1);
    }

    assert_eq!(response_h2.http_version, "HTTP/2");
    assert!(response_h2.is_success());
}
