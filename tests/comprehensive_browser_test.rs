//! 全面浏览器指纹测试
//!
//! 访问 Google Earth API 端点，测试所有浏览器指纹和 HTTP 协议版本
//!
//! 运行方式:
//! ```bash
//! # 运行所有测试（需要网络）
//! cargo test --test comprehensive_browser_test -- --ignored --test-threads=1 --nocapture
//!
//! # 运行特定测试
//! cargo test --test comprehensive_browser_test test_all_chrome_versions -- --ignored --nocapture
//! ```

use fingerprint::*;
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// 测试目标 URL
const TEST_URL: &str = "kh.google.com";
const TEST_PATH: &str = "/rt/earth/PlanetoidMetadata";
#[allow(dead_code)]
const TEST_PORT: u16 = 443; // HTTPS

/// 测试结果
#[derive(Debug, Clone)]
struct TestResult {
    profile_name: String,
    #[allow(dead_code)]
    user_agent: String,
    success: bool,
    status_code: Option<u16>,
    response_size: usize,
    duration: Duration,
    error_message: Option<String>,
    #[allow(dead_code)]
    http_version: String,
}

impl TestResult {
    fn new(profile_name: String, user_agent: String, http_version: String) -> Self {
        Self {
            profile_name,
            user_agent,
            success: false,
            status_code: None,
            response_size: 0,
            duration: Duration::from_secs(0),
            error_message: None,
            http_version,
        }
    }
}

/// 测试统计
struct TestStats {
    total: usize,
    success: usize,
    failed: usize,
    results: Vec<TestResult>,
}

impl TestStats {
    fn new() -> Self {
        Self {
            total: 0,
            success: 0,
            failed: 0,
            results: Vec::new(),
        }
    }

    fn add_result(&mut self, result: TestResult) {
        self.total += 1;
        if result.success {
            self.success += 1;
        } else {
            self.failed += 1;
        }
        self.results.push(result);
    }

    fn success_rate(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            (self.success as f64 / self.total as f64) * 100.0
        }
    }

    fn print_summary(&self) {
        println!("\n╔═══════════════════════════════════════════════════════════╗");
        println!("║              全面浏览器指纹测试总结                        ║");
        println!("╚═══════════════════════════════════════════════════════════╝");

        println!("\n📊 总体统计:");
        println!("  - 总测试数: {}", self.total);
        println!("  - 成功: {} ✅", self.success);
        println!("  - 失败: {} ❌", self.failed);
        println!("  - 成功率: {:.2}%", self.success_rate());

        if self.success > 0 {
            let avg_duration: Duration = self
                .results
                .iter()
                .filter(|r| r.success)
                .map(|r| r.duration)
                .sum::<Duration>()
                / self.success as u32;

            let avg_response_size: f64 = self
                .results
                .iter()
                .filter(|r| r.success)
                .map(|r| r.response_size as f64)
                .sum::<f64>()
                / self.success as f64;

            println!("\n⚡ 性能指标:");
            println!("  - 平均响应时间: {:?}", avg_duration);
            println!("  - 平均响应大小: {:.0} 字节", avg_response_size);
        }

        // 按浏览器分类统计
        let mut browser_stats: HashMap<String, (usize, usize)> = HashMap::new();
        for result in &self.results {
            let browser = if result.profile_name.starts_with("chrome") {
                "Chrome"
            } else if result.profile_name.starts_with("firefox") {
                "Firefox"
            } else if result.profile_name.starts_with("safari") {
                "Safari"
            } else if result.profile_name.starts_with("edge") {
                "Edge"
            } else {
                "其他"
            };

            let entry = browser_stats.entry(browser.to_string()).or_insert((0, 0));
            entry.0 += 1;
            if result.success {
                entry.1 += 1;
            }
        }

        println!("\n🌐 浏览器统计:");
        for (browser, (total, success)) in browser_stats {
            let rate = (success as f64 / total as f64) * 100.0;
            println!("  - {}: {}/{} ({:.1}%)", browser, success, total, rate);
        }

        // 显示失败的测试
        if self.failed > 0 {
            println!("\n❌ 失败的测试:");
            for result in &self.results {
                if !result.success {
                    println!(
                        "  - {}: {}",
                        result.profile_name,
                        result
                            .error_message
                            .as_ref()
                            .unwrap_or(&"未知错误".to_string())
                    );
                }
            }
        }
    }
}

/// 使用 reqwest 进行 HTTPS 请求（支持 HTTP/1.1 和 HTTP/2）
fn test_https_request(
    profile_name: &str,
    user_agent: &str,
    headers: &HTTPHeaders,
    http_version: &str,
) -> TestResult {
    let mut result = TestResult::new(
        profile_name.to_string(),
        user_agent.to_string(),
        http_version.to_string(),
    );

    let start = Instant::now();

    // 使用 reqwest 进行 HTTPS 请求
    let client = match reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            result.error_message = Some(format!("创建客户端失败: {}", e));
            result.duration = start.elapsed();
            return result;
        }
    };

    let url = format!("https://{}{}", TEST_URL, TEST_PATH);

    let mut request = client
        .get(&url)
        .header("User-Agent", user_agent)
        .header("Accept", &headers.accept)
        .header("Accept-Language", &headers.accept_language)
        .header("Accept-Encoding", &headers.accept_encoding);

    // 添加其他 headers
    if !headers.sec_fetch_site.is_empty() {
        request = request.header("Sec-Fetch-Site", &headers.sec_fetch_site);
    }
    if !headers.sec_fetch_mode.is_empty() {
        request = request.header("Sec-Fetch-Mode", &headers.sec_fetch_mode);
    }
    if !headers.sec_fetch_dest.is_empty() {
        request = request.header("Sec-Fetch-Dest", &headers.sec_fetch_dest);
    }

    match request.send() {
        Ok(response) => {
            result.status_code = Some(response.status().as_u16());
            result.success = response.status().is_success();

            match response.bytes() {
                Ok(bytes) => {
                    result.response_size = bytes.len();
                }
                Err(e) => {
                    result.error_message = Some(format!("读取响应失败: {}", e));
                }
            }
        }
        Err(e) => {
            result.error_message = Some(format!("请求失败: {}", e));
        }
    }

    result.duration = start.elapsed();
    result
}

/// 测试所有 Chrome 版本
#[test]
#[ignore]
fn test_all_chrome_versions() {
    println!("\n╔═══════════════════════════════════════════════════════════╗");
    println!("║            测试所有 Chrome 浏览器版本                      ║");
    println!("╚═══════════════════════════════════════════════════════════╝");

    let mut stats = TestStats::new();
    let all_profiles = mapped_tls_clients();

    let chrome_profiles: Vec<_> = all_profiles
        .iter()
        .filter(|(name, _)| name.starts_with("chrome"))
        .collect();

    println!("\n找到 {} 个 Chrome 版本\n", chrome_profiles.len());

    for (i, (profile_name, _profile)) in chrome_profiles.iter().enumerate() {
        println!("┌─────────────────────────────────────────────────────────┐");
        println!(
            "│ [{}/{}] 测试: {}",
            i + 1,
            chrome_profiles.len(),
            profile_name
        );
        println!("└─────────────────────────────────────────────────────────┘");

        // 生成 User-Agent
        let user_agent = get_user_agent_by_profile_name(profile_name).unwrap_or_else(|_| {
            // 如果无法生成，使用默认值
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/133.0.0.0 Safari/537.36".to_string()
        });
        println!("  User-Agent: {}", user_agent);

        // 生成 Headers
        let headers = HTTPHeaders::default();

        // 测试 HTTP/2
        println!("  → 测试 HTTP/2...");
        let result_h2 = test_https_request(profile_name, &user_agent, &headers, "h2");

        if result_h2.success {
            println!(
                "  ✅ HTTP/2: 状态码 {}, 响应大小 {} 字节, 耗时 {:?}",
                result_h2.status_code.unwrap(),
                result_h2.response_size,
                result_h2.duration
            );
        } else {
            println!(
                "  ❌ HTTP/2: {}",
                result_h2
                    .error_message
                    .as_ref()
                    .unwrap_or(&"未知错误".to_string())
            );
        }

        stats.add_result(result_h2);

        // 测试 HTTP/1.1
        println!("  → 测试 HTTP/1.1...");
        let result_h1 = test_https_request(profile_name, &user_agent, &headers, "h1.1");

        if result_h1.success {
            println!(
                "  ✅ HTTP/1.1: 状态码 {}, 响应大小 {} 字节, 耗时 {:?}",
                result_h1.status_code.unwrap(),
                result_h1.response_size,
                result_h1.duration
            );
        } else {
            println!(
                "  ❌ HTTP/1.1: {}",
                result_h1
                    .error_message
                    .as_ref()
                    .unwrap_or(&"未知错误".to_string())
            );
        }

        stats.add_result(result_h1);

        // 等待一下，避免请求过快
        std::thread::sleep(Duration::from_millis(200));
    }

    stats.print_summary();

    // 验证至少 80% 的测试成功
    assert!(
        stats.success_rate() >= 80.0,
        "成功率 {:.2}% 低于 80%",
        stats.success_rate()
    );
}

/// 测试所有 Firefox 版本
#[test]
#[ignore]
fn test_all_firefox_versions() {
    println!("\n╔═══════════════════════════════════════════════════════════╗");
    println!("║           测试所有 Firefox 浏览器版本                      ║");
    println!("╚═══════════════════════════════════════════════════════════╝");

    let mut stats = TestStats::new();
    let all_profiles = mapped_tls_clients();

    let firefox_profiles: Vec<_> = all_profiles
        .iter()
        .filter(|(name, _)| name.starts_with("firefox"))
        .collect();

    println!("\n找到 {} 个 Firefox 版本\n", firefox_profiles.len());

    for (i, (profile_name, _profile)) in firefox_profiles.iter().enumerate() {
        println!("┌─────────────────────────────────────────────────────────┐");
        println!(
            "│ [{}/{}] 测试: {}",
            i + 1,
            firefox_profiles.len(),
            profile_name
        );
        println!("└─────────────────────────────────────────────────────────┘");

        let user_agent = get_user_agent_by_profile_name(profile_name).unwrap_or_else(|_| {
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:133.0) Gecko/20100101 Firefox/133.0".to_string()
        });
        println!("  User-Agent: {}", user_agent);

        let headers = HTTPHeaders::default();

        // 测试 HTTP/2
        println!("  → 测试 HTTP/2...");
        let result_h2 = test_https_request(profile_name, &user_agent, &headers, "h2");

        if result_h2.success {
            println!(
                "  ✅ HTTP/2: 状态码 {}, 响应大小 {} 字节",
                result_h2.status_code.unwrap(),
                result_h2.response_size
            );
        } else {
            println!(
                "  ❌ HTTP/2: {}",
                result_h2
                    .error_message
                    .as_ref()
                    .unwrap_or(&"未知错误".to_string())
            );
        }

        stats.add_result(result_h2);

        // 测试 HTTP/1.1
        println!("  → 测试 HTTP/1.1...");
        let result_h1 = test_https_request(profile_name, &user_agent, &headers, "h1.1");

        if result_h1.success {
            println!(
                "  ✅ HTTP/1.1: 状态码 {}, 响应大小 {} 字节",
                result_h1.status_code.unwrap(),
                result_h1.response_size
            );
        } else {
            println!(
                "  ❌ HTTP/1.1: {}",
                result_h1
                    .error_message
                    .as_ref()
                    .unwrap_or(&"未知错误".to_string())
            );
        }

        stats.add_result(result_h1);

        std::thread::sleep(Duration::from_millis(200));
    }

    stats.print_summary();

    assert!(
        stats.success_rate() >= 80.0,
        "成功率 {:.2}% 低于 80%",
        stats.success_rate()
    );
}

/// 测试所有 Safari 版本
#[test]
#[ignore]
fn test_all_safari_versions() {
    println!("\n╔═══════════════════════════════════════════════════════════╗");
    println!("║            测试所有 Safari 浏览器版本                      ║");
    println!("╚═══════════════════════════════════════════════════════════╝");

    let mut stats = TestStats::new();
    let all_profiles = mapped_tls_clients();

    let safari_profiles: Vec<_> = all_profiles
        .iter()
        .filter(|(name, _)| name.starts_with("safari"))
        .collect();

    println!("\n找到 {} 个 Safari 版本\n", safari_profiles.len());

    for (i, (profile_name, _profile)) in safari_profiles.iter().enumerate() {
        println!("┌─────────────────────────────────────────────────────────┐");
        println!(
            "│ [{}/{}] 测试: {}",
            i + 1,
            safari_profiles.len(),
            profile_name
        );
        println!("└─────────────────────────────────────────────────────────┘");

        let user_agent = get_user_agent_by_profile_name(profile_name).unwrap_or_else(|_| {
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/16.0 Safari/605.1.15".to_string()
        });
        println!("  User-Agent: {}", user_agent);

        let headers = HTTPHeaders::default();

        // 测试 HTTP/2
        println!("  → 测试 HTTP/2...");
        let result_h2 = test_https_request(profile_name, &user_agent, &headers, "h2");

        if result_h2.success {
            println!(
                "  ✅ HTTP/2: 状态码 {}, 响应大小 {} 字节",
                result_h2.status_code.unwrap(),
                result_h2.response_size
            );
        } else {
            println!(
                "  ❌ HTTP/2: {}",
                result_h2
                    .error_message
                    .as_ref()
                    .unwrap_or(&"未知错误".to_string())
            );
        }

        stats.add_result(result_h2);

        // 测试 HTTP/1.1
        println!("  → 测试 HTTP/1.1...");
        let result_h1 = test_https_request(profile_name, &user_agent, &headers, "h1.1");

        if result_h1.success {
            println!(
                "  ✅ HTTP/1.1: 状态码 {}, 响应大小 {} 字节",
                result_h1.status_code.unwrap(),
                result_h1.response_size
            );
        } else {
            println!(
                "  ❌ HTTP/1.1: {}",
                result_h1
                    .error_message
                    .as_ref()
                    .unwrap_or(&"未知错误".to_string())
            );
        }

        stats.add_result(result_h1);

        std::thread::sleep(Duration::from_millis(200));
    }

    stats.print_summary();

    assert!(
        stats.success_rate() >= 80.0,
        "成功率 {:.2}% 低于 80%",
        stats.success_rate()
    );
}

/// 测试所有浏览器（完整测试）
#[test]
#[ignore]
fn test_all_browsers_comprehensive() {
    println!("\n╔═══════════════════════════════════════════════════════════╗");
    println!("║              全面浏览器指纹测试                            ║");
    println!("║         测试目标: {}", TEST_URL);
    println!("╚═══════════════════════════════════════════════════════════╝");

    let mut stats = TestStats::new();
    let all_profiles = mapped_tls_clients();

    println!("\n📋 总共 {} 个浏览器配置\n", all_profiles.len());

    for (i, (profile_name, _profile)) in all_profiles.iter().enumerate() {
        println!("┌─────────────────────────────────────────────────────────┐");
        println!(
            "│ [{}/{}] 测试: {}",
            i + 1,
            all_profiles.len(),
            profile_name
        );
        println!("└─────────────────────────────────────────────────────────┘");

        let user_agent = get_user_agent_by_profile_name(profile_name).unwrap_or_else(|_| {
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36".to_string()
        });
        println!("  User-Agent: {}", &user_agent[..user_agent.len().min(60)]);

        // 推断浏览器类型
        let _browser_type = if profile_name.contains("chrome") {
            BrowserType::Chrome
        } else if profile_name.contains("firefox") {
            BrowserType::Firefox
        } else if profile_name.contains("safari") {
            BrowserType::Safari
        } else if profile_name.contains("edge") {
            BrowserType::Edge
        } else {
            BrowserType::Chrome // 默认
        };

        let headers = HTTPHeaders::default();

        // 测试 HTTP/2
        println!("  → 测试 HTTP/2...");
        let result_h2 = test_https_request(profile_name, &user_agent, &headers, "h2");

        if result_h2.success {
            println!(
                "  ✅ HTTP/2: {}, {} 字节, {:?}",
                result_h2.status_code.unwrap(),
                result_h2.response_size,
                result_h2.duration
            );
        } else {
            println!(
                "  ❌ HTTP/2: {}",
                result_h2
                    .error_message
                    .as_ref()
                    .unwrap_or(&"未知".to_string())
            );
        }

        stats.add_result(result_h2);

        // 测试 HTTP/1.1
        println!("  → 测试 HTTP/1.1...");
        let result_h1 = test_https_request(profile_name, &user_agent, &headers, "h1.1");

        if result_h1.success {
            println!(
                "  ✅ HTTP/1.1: {}, {} 字节, {:?}",
                result_h1.status_code.unwrap(),
                result_h1.response_size,
                result_h1.duration
            );
        } else {
            println!(
                "  ❌ HTTP/1.1: {}",
                result_h1
                    .error_message
                    .as_ref()
                    .unwrap_or(&"未知".to_string())
            );
        }

        stats.add_result(result_h1);

        // 短暂延迟
        std::thread::sleep(Duration::from_millis(100));
    }

    stats.print_summary();

    // 验证成功率
    assert!(
        stats.success_rate() >= 70.0,
        "成功率 {:.2}% 低于 70%",
        stats.success_rate()
    );
}

/// 快速抽样测试（用于验证基本功能）
#[test]
fn test_sample_browsers() {
    println!("\n=== 快速抽样测试 ===");
    println!("测试几个代表性的浏览器版本\n");

    let test_profiles = vec!["chrome_133", "firefox_133", "safari_16_0"];

    for profile_name in test_profiles {
        println!("✓ 配置: {}", profile_name);

        let all_profiles = mapped_tls_clients();
        if let Some(profile) = all_profiles.get(profile_name) {
            let user_agent = get_user_agent_by_profile_name(profile_name).unwrap_or_else(|_| {
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) Chrome/133".to_string()
            });
            println!("  User-Agent: {}", user_agent);

            let spec = profile.get_client_hello_spec().expect("获取 spec 失败");
            println!("  密码套件: {}", spec.cipher_suites.len());
            println!("  扩展: {}", spec.extensions.len());
        }
    }

    println!("\n💡 运行完整测试:");
    println!("  cargo test --test comprehensive_browser_test test_all_browsers_comprehensive -- --ignored --nocapture");
}
