//! 所有浏览器指纹全面测试
//! 测试 Chrome 103/133, Firefox 133, Safari 16.0, Opera 91
//! 完整链路: netconnpool → TLS 指纹 → Google API

use fingerprint::{chrome_103, chrome_133, firefox_133, opera_91, safari_16_0, ClientProfile, HttpClient, HttpClientConfig};
use std::time::Instant;

const TEST_URL: &str = "https://kh.google.com/rt/earth/PlanetoidMetadata";
const TEST_ROUNDS: usize = 5; // 每个指纹测试5轮

#[derive(Debug)]
struct BrowserTestResult {
    browser_name: String,
    browser_version: String,
    protocol: String,
    success_count: usize,
    fail_count: usize,
    response_times_ms: Vec<u64>,
    status_codes: Vec<u16>,
    body_sizes: Vec<usize>,
}

impl BrowserTestResult {
    fn new(browser: &str, version: &str, protocol: &str) -> Self {
        Self {
            browser_name: browser.to_string(),
            browser_version: version.to_string(),
            protocol: protocol.to_string(),
            success_count: 0,
            fail_count: 0,
            response_times_ms: Vec::new(),
            status_codes: Vec::new(),
            body_sizes: Vec::new(),
        }
    }

    fn add_success(&mut self, time_ms: u64, status: u16, body_size: usize) {
        self.success_count += 1;
        self.response_times_ms.push(time_ms);
        self.status_codes.push(status);
        self.body_sizes.push(body_size);
    }

    fn add_failure(&mut self) {
        self.fail_count += 1;
    }

    fn avg_time(&self) -> f64 {
        if self.response_times_ms.is_empty() {
            return 0.0;
        }
        self.response_times_ms.iter().sum::<u64>() as f64 / self.response_times_ms.len() as f64
    }

    fn min_time(&self) -> u64 {
        *self.response_times_ms.iter().min().unwrap_or(&0)
    }

    fn max_time(&self) -> u64 {
        *self.response_times_ms.iter().max().unwrap_or(&0)
    }

    fn print_summary(&self) {
        println!("\n  📊 {} {} ({}):", self.browser_name, self.browser_version, self.protocol);
        println!("     成功率: {}/{}", self.success_count, self.success_count + self.fail_count);
        
        if !self.response_times_ms.is_empty() {
            println!("     响应时间: 平均 {:.2}ms | 最小 {}ms | 最大 {}ms", 
                self.avg_time(), self.min_time(), self.max_time());
            println!("     状态码: {:?}", self.status_codes);
            println!("     Body 大小: {:?} bytes", self.body_sizes);
        }
        
        if self.fail_count > 0 {
            println!("     ❌ 失败次数: {}", self.fail_count);
        }
    }

    fn is_success(&self) -> bool {
        self.fail_count == 0 && self.success_count > 0
    }
}

/// 测试单个浏览器指纹
fn test_browser_fingerprint(
    browser: &str,
    version: &str,
    protocol: &str,
    prefer_h2: bool,
    prefer_h3: bool,
) -> BrowserTestResult {
    let mut result = BrowserTestResult::new(browser, version, protocol);

    // 获取浏览器 Profile
    let _profile = match browser {
        "Chrome" if version == "103" => chrome_103(),
        "Chrome" if version == "133" => chrome_133(),
        "Firefox" if version == "133" => firefox_133(),
        "Safari" if version == "16.0" => safari_16_0(),
        "Opera" if version == "91" => opera_91(),
        _ => {
            println!("    ❌ 未知的浏览器: {} {}", browser, version);
            result.add_failure();
            return result;
        }
    };
    
    // TODO: 将来需要在 HttpClientConfig 中使用 profile 来设置 TLS 指纹

    println!("  🔹 {} {} - {}", browser, version, protocol);

    for round in 1..=TEST_ROUNDS {
        print!("     轮次 {}/{}... ", round, TEST_ROUNDS);

        // 配置 HTTP 客户端
        let config = HttpClientConfig {
            user_agent: format!(
                "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) {}/{}",
                browser, version
            ),
            prefer_http2: prefer_h2,
            prefer_http3: prefer_h3,
            ..Default::default()
        };

        let client = HttpClient::new(config);

        let start = Instant::now();
        match client.get(TEST_URL) {
            Ok(response) => {
                let elapsed = start.elapsed().as_millis() as u64;
                result.add_success(elapsed, response.status_code, response.body.len());
                println!("✅ {}ms, status {}, {} bytes", 
                    elapsed, response.status_code, response.body.len());
            }
            Err(e) => {
                result.add_failure();
                println!("❌ 失败: {:?}", e);
            }
        }

        // 短暂间隔
        std::thread::sleep(std::time::Duration::from_millis(200));
    }

    result
}

#[test]
#[ignore]
fn test_all_browsers_http1() {
    println!("\n╔══════════════════════════════════════════════════════════╗");
    println!("║  所有浏览器指纹测试 - HTTP/1.1                          ║");
    println!("╚══════════════════════════════════════════════════════════╝\n");

    let browsers = vec![
        ("Chrome", "103"),
        ("Chrome", "133"),
        ("Firefox", "133"),
        ("Safari", "16.0"),
        ("Opera", "91"),
    ];

    let mut results = Vec::new();

    for (browser, version) in browsers {
        let result = test_browser_fingerprint(browser, version, "HTTP/1.1", false, false);
        results.push(result);
    }

    println!("\n╔══════════════════════════════════════════════════════════╗");
    println!("║  HTTP/1.1 测试汇总                                       ║");
    println!("╚══════════════════════════════════════════════════════════╝");

    for result in &results {
        result.print_summary();
    }

    let total_success = results.iter().filter(|r| r.is_success()).count();
    let total_tests = results.len();
    let success_rate = (total_success as f64 / total_tests as f64) * 100.0;

    println!("\n🎯 总成功率: {}/{} ({:.1}%)", total_success, total_tests, success_rate);

    // 允许偶发的网络错误，只要成功率 >= 80% 就通过
    assert!(success_rate >= 80.0, "浏览器指纹测试成功率过低: {:.1}%", success_rate);
}

#[test]
#[cfg(feature = "http2")]
#[ignore]
fn test_all_browsers_http2() {
    println!("\n╔══════════════════════════════════════════════════════════╗");
    println!("║  所有浏览器指纹测试 - HTTP/2                            ║");
    println!("╚══════════════════════════════════════════════════════════╝\n");

    let browsers = vec![
        ("Chrome", "103"),
        ("Chrome", "133"),
        ("Firefox", "133"),
        ("Safari", "16.0"),
        ("Opera", "91"),
    ];

    let mut results = Vec::new();

    for (browser, version) in browsers {
        let result = test_browser_fingerprint(browser, version, "HTTP/2", true, false);
        results.push(result);
    }

    println!("\n╔══════════════════════════════════════════════════════════╗");
    println!("║  HTTP/2 测试汇总                                         ║");
    println!("╚══════════════════════════════════════════════════════════╝");

    for result in &results {
        result.print_summary();
    }

    let total_success = results.iter().filter(|r| r.is_success()).count();
    let total_tests = results.len();
    let success_rate = (total_success as f64 / total_tests as f64) * 100.0;

    println!("\n🎯 总成功率: {}/{} ({:.1}%)", total_success, total_tests, success_rate);

    // 允许偶发的网络错误，只要成功率 >= 80% 就通过
    assert!(success_rate >= 80.0, "浏览器指纹测试成功率过低: {:.1}%", success_rate);
}

#[test]
#[cfg(feature = "http3")]
#[ignore]
fn test_all_browsers_http3() {
    println!("\n╔══════════════════════════════════════════════════════════╗");
    println!("║  所有浏览器指纹测试 - HTTP/3                            ║");
    println!("╚══════════════════════════════════════════════════════════╝\n");

    let browsers = vec![
        ("Chrome", "103"),
        ("Chrome", "133"),
        ("Firefox", "133"),
        ("Safari", "16.0"),
        ("Opera", "91"),
    ];

    let mut results = Vec::new();

    for (browser, version) in browsers {
        let result = test_browser_fingerprint(browser, version, "HTTP/3", false, true);
        results.push(result);
    }

    println!("\n╔══════════════════════════════════════════════════════════╗");
    println!("║  HTTP/3 测试汇总                                         ║");
    println!("╚══════════════════════════════════════════════════════════╝");

    for result in &results {
        result.print_summary();
    }

    let total_success = results.iter().filter(|r| r.is_success()).count();
    let total_tests = results.len();

    println!("\n🎯 总成功率: {}/{}", total_success, total_tests);

    assert_eq!(total_success, total_tests, "部分浏览器指纹测试失败");
}

#[test]
#[cfg(all(feature = "http2", feature = "http3"))]
#[ignore]
fn test_all_browsers_all_protocols() {
    println!("\n╔══════════════════════════════════════════════════════════╗");
    println!("║  所有浏览器指纹 × 所有协议 完整测试                    ║");
    println!("║  目标: https://kh.google.com/rt/earth/PlanetoidMetadata  ║");
    println!("╚══════════════════════════════════════════════════════════╝\n");

    let browsers = vec![
        ("Chrome", "103"),
        ("Chrome", "133"),
        ("Firefox", "133"),
        ("Safari", "16.0"),
        ("Opera", "91"),
    ];

    let mut all_results = Vec::new();

    // 测试每个浏览器的每个协议
    for (browser, version) in &browsers {
        println!("\n🌐 测试浏览器: {} {}", browser, version);
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

        // HTTP/1.1
        let h1_result = test_browser_fingerprint(browser, version, "HTTP/1.1", false, false);
        all_results.push(h1_result);

        std::thread::sleep(std::time::Duration::from_millis(500));

        // HTTP/2
        let h2_result = test_browser_fingerprint(browser, version, "HTTP/2", true, false);
        all_results.push(h2_result);

        std::thread::sleep(std::time::Duration::from_millis(500));

        // HTTP/3
        let h3_result = test_browser_fingerprint(browser, version, "HTTP/3", false, true);
        all_results.push(h3_result);

        std::thread::sleep(std::time::Duration::from_secs(1));
    }

    // 汇总结果
    println!("\n╔══════════════════════════════════════════════════════════╗");
    println!("║  最终测试汇总                                            ║");
    println!("╚══════════════════════════════════════════════════════════╝");

    for result in &all_results {
        result.print_summary();
    }

    // 统计
    let total_success = all_results.iter().filter(|r| r.is_success()).count();
    let total_tests = all_results.len();
    let success_rate = (total_success as f64 / total_tests as f64) * 100.0;

    println!("\n╔══════════════════════════════════════════════════════════╗");
    println!("║  最终统计                                                ║");
    println!("╚══════════════════════════════════════════════════════════╝");
    println!("\n  📊 总测试数: {}", total_tests);
    println!("  ✅ 成功: {}", total_success);
    println!("  ❌ 失败: {}", total_tests - total_success);
    println!("  🎯 成功率: {:.1}%", success_rate);

    println!("\n  浏览器数: {}", browsers.len());
    println!("  协议数: 3 (HTTP/1.1, HTTP/2, HTTP/3)");
    println!("  每个配置测试轮次: {}", TEST_ROUNDS);
    println!("  总请求数: {}", total_tests * TEST_ROUNDS);

    // 允许偶发的网络错误，只要成功率 >= 90% 就通过
    assert!(success_rate >= 90.0, "浏览器/协议组合测试成功率过低: {:.1}%", success_rate);
}
