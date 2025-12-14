//! 使用现有 HTTP 客户端测试 Google Earth API
//! 验证 API 是否可访问，然后逐步替换为我们的自定义 TLS

use fingerprint::{
    get_user_agent_by_profile_name, mapped_tls_clients, HttpClient, HttpClientConfig,
};

const TEST_URL: &str = "https://kh.google.com/rt/earth/PlanetoidMetadata";

#[test]
#[ignore] // 需要网络连接
fn test_google_earth_api_basic_http_client() {
    println!("\n╔══════════════════════════════════════════════════════════╗");
    println!("║   测试: 使用 HTTP 客户端访问 Google Earth API            ║");
    println!("╚══════════════════════════════════════════════════════════╝\n");

    // 使用 Chrome 133 配置
    let user_agent = get_user_agent_by_profile_name("chrome_133").expect("无法生成 User-Agent");

    let mut config = HttpClientConfig::default();
    config.user_agent = user_agent;
    config.prefer_http2 = true;

    let client = HttpClient::new(config);

    println!("🌐 访问: {}", TEST_URL);
    println!("📋 配置: Chrome 133 User-Agent");
    println!("📋 协议: 优先 HTTP/2\n");

    match client.get(TEST_URL) {
        Ok(response) => {
            println!("✅ 请求成功！");
            println!("  - HTTP 版本: {}", response.http_version);
            println!("  - 状态码: {}", response.status_code);
            println!("  - Headers 数量: {}", response.headers.len());

            if let Ok(body) = response.body_as_string() {
                let preview = if body.len() > 200 {
                    format!("{}...", &body[..200])
                } else {
                    body.clone()
                };
                println!("  - Body 大小: {} bytes", body.len());
                println!("  - Body 预览:\n{}", preview);
            }

            println!("\n✅ Google Earth API 可以正常访问！");
            println!("   现在的问题是: 我们需要完整的 TLS 握手实现，");
            println!("   而不仅仅是发送 ClientHello。");
        }
        Err(e) => {
            println!("❌ 请求失败: {}", e);
            println!("  提示: 可能需要 VPN 或网络配置");
        }
    }
}

#[test]
#[ignore] // 需要网络连接
fn test_all_browsers_with_http_client() {
    println!("\n╔══════════════════════════════════════════════════════════╗");
    println!("║   测试所有 66 个浏览器访问 Google Earth API              ║");
    println!("╚══════════════════════════════════════════════════════════╝\n");

    let profiles = mapped_tls_clients();
    let total = profiles.len();
    let mut success = 0;
    let mut failed = Vec::new();

    for (i, (name, _)) in profiles.iter().enumerate() {
        print!("  [{:2}/{:2}] {:25} ... ", i + 1, total, name);

        match test_single_browser_http_client(name) {
            Ok(status) => {
                println!("✅ ({})", status);
                success += 1;
            }
            Err(e) => {
                println!("❌ ({})", e);
                failed.push((name.clone(), e));
            }
        }
    }

    println!("\n╔══════════════════════════════════════════════════════════╗");
    println!("║                     测试结果汇总                         ║");
    println!("╚══════════════════════════════════════════════════════════╝\n");
    println!("  总计: {}", total);
    println!("  成功: {} ✅", success);
    println!("  失败: {} ❌", failed.len());
    println!("  成功率: {:.1}%", (success as f64 / total as f64) * 100.0);

    if !failed.is_empty() {
        println!("\n❌ 失败的浏览器 (前10个):");
        for (name, err) in failed.iter().take(10) {
            println!("  - {}: {}", name, err);
        }
    }

    // 要求至少 80% 成功率
    assert!((success as f64 / total as f64) >= 0.8, "成功率低于 80%");
}

fn test_single_browser_http_client(browser_name: &str) -> Result<String, String> {
    let user_agent =
        get_user_agent_by_profile_name(browser_name).map_err(|e| format!("生成 UA 失败: {}", e))?;

    let config = HttpClientConfig {
        user_agent,
        prefer_http2: true,
        read_timeout: std::time::Duration::from_secs(10),
        ..Default::default()
    };

    let client = HttpClient::new(config);

    match client.get(TEST_URL) {
        Ok(response) => Ok(format!(
            "{} {}",
            response.http_version, response.status_code
        )),
        Err(e) => Err(format!("{}", e)),
    }
}
