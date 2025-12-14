//! 持续压力测试 - 不停的测试所有浏览器指纹
//! 用于长时间验证稳定性和性能

use fingerprint::{chrome_103, chrome_133, firefox_133, opera_91, safari_16_0, HttpClient, HttpClientConfig};
use std::time::{Duration, Instant};

const TEST_URL: &str = "https://kh.google.com/rt/earth/PlanetoidMetadata";

#[derive(Debug)]
struct ContinuousTestStats {
    total_requests: usize,
    success_count: usize,
    fail_count: usize,
    total_time_ms: u64,
    min_time_ms: u64,
    max_time_ms: u64,
    response_times: Vec<u64>,
}

impl ContinuousTestStats {
    fn new() -> Self {
        Self {
            total_requests: 0,
            success_count: 0,
            fail_count: 0,
            total_time_ms: 0,
            min_time_ms: u64::MAX,
            max_time_ms: 0,
            response_times: Vec::new(),
        }
    }

    fn add_success(&mut self, time_ms: u64) {
        self.total_requests += 1;
        self.success_count += 1;
        self.total_time_ms += time_ms;
        self.min_time_ms = self.min_time_ms.min(time_ms);
        self.max_time_ms = self.max_time_ms.max(time_ms);
        self.response_times.push(time_ms);
    }

    fn add_failure(&mut self) {
        self.total_requests += 1;
        self.fail_count += 1;
    }

    fn avg_time(&self) -> f64 {
        if self.success_count == 0 {
            return 0.0;
        }
        self.total_time_ms as f64 / self.success_count as f64
    }

    fn success_rate(&self) -> f64 {
        if self.total_requests == 0 {
            return 0.0;
        }
        (self.success_count as f64 / self.total_requests as f64) * 100.0
    }

    fn print_summary(&self, label: &str) {
        println!("\n  📊 {} 统计:", label);
        println!("     总请求: {}", self.total_requests);
        println!("     成功: {} | 失败: {}", self.success_count, self.fail_count);
        println!("     成功率: {:.1}%", self.success_rate());
        if self.success_count > 0 {
            println!("     平均响应: {:.2}ms", self.avg_time());
            println!("     最小: {}ms | 最大: {}ms", self.min_time_ms, self.max_time_ms);
        }
    }
}

#[test]
#[ignore]
fn test_continuous_http1_10_rounds() {
    println!("\n╔══════════════════════════════════════════════════════════╗");
    println!("║  持续测试 - HTTP/1.1 (10 轮 × 5 浏览器)                ║");
    println!("╚══════════════════════════════════════════════════════════╝\n");

    let browsers = vec![
        ("Chrome 103", chrome_103()),
        ("Chrome 133", chrome_133()),
        ("Firefox 133", firefox_133()),
        ("Safari 16.0", safari_16_0()),
        ("Opera 91", opera_91()),
    ];

    let mut stats = ContinuousTestStats::new();
    let start_time = Instant::now();

    for round in 1..=10 {
        println!("\n🔄 轮次 {}/10", round);
        
        for (name, _profile) in &browsers {
            let config = HttpClientConfig {
                user_agent: format!("Mozilla/5.0 (compatible; {}))", name),
                prefer_http2: false,
                prefer_http3: false,
                ..Default::default()
            };

            let client = HttpClient::new(config);
            print!("  {} ... ", name);

            let start = Instant::now();
            match client.get(TEST_URL) {
                Ok(response) => {
                    let elapsed = start.elapsed().as_millis() as u64;
                    stats.add_success(elapsed);
                    println!("✅ {}ms (status {})", elapsed, response.status_code);
                }
                Err(e) => {
                    stats.add_failure();
                    println!("❌ {:?}", e);
                }
            }

            // 短暂间隔
            std::thread::sleep(Duration::from_millis(100));
        }

        std::thread::sleep(Duration::from_millis(500));
    }

    let total_duration = start_time.elapsed();

    println!("\n╔══════════════════════════════════════════════════════════╗");
    println!("║  测试完成                                                ║");
    println!("╚══════════════════════════════════════════════════════════╝");

    stats.print_summary("HTTP/1.1 持续测试");
    
    println!("\n  ⏱️  总耗时: {:.2}秒", total_duration.as_secs_f64());
    println!("  📈 吞吐量: {:.2} 请求/秒", stats.total_requests as f64 / total_duration.as_secs_f64());

    // 断言成功率
    assert!(stats.success_rate() >= 90.0, "成功率过低: {:.1}%", stats.success_rate());
}

#[test]
#[cfg(all(feature = "http2", feature = "http3"))]
#[ignore]
fn test_continuous_all_protocols_marathon() {
    println!("\n╔══════════════════════════════════════════════════════════╗");
    println!("║  马拉松测试 - 所有协议 × 所有浏览器 (50 轮)            ║");
    println!("║  这将需要约 5-10 分钟                                   ║");
    println!("╚══════════════════════════════════════════════════════════╝\n");

    let browsers = vec![
        ("Chrome 103", chrome_103()),
        ("Chrome 133", chrome_133()),
        ("Firefox 133", firefox_133()),
        ("Safari 16.0", safari_16_0()),
        ("Opera 91", opera_91()),
    ];

    let protocols = vec![
        ("HTTP/1.1", false, false),
        ("HTTP/2", true, false),
        ("HTTP/3", false, true),
    ];

    let mut h1_stats = ContinuousTestStats::new();
    let mut h2_stats = ContinuousTestStats::new();
    let mut h3_stats = ContinuousTestStats::new();

    let start_time = Instant::now();
    let total_rounds = 50;

    for round in 1..=total_rounds {
        if round % 10 == 0 {
            println!("\n🔄 进度: {}/{} 轮 ({:.0}%)", round, total_rounds, (round as f64 / total_rounds as f64) * 100.0);
        } else {
            print!(".");
        }

        for (protocol_name, h2, h3) in &protocols {
            for (browser_name, _profile) in &browsers {
                let config = HttpClientConfig {
                    user_agent: format!("Mozilla/5.0 (compatible; {}))", browser_name),
                    prefer_http2: *h2,
                    prefer_http3: *h3,
                    ..Default::default()
                };

                let client = HttpClient::new(config);
                let start = Instant::now();

                let stats = match protocol_name {
                    &"HTTP/1.1" => &mut h1_stats,
                    &"HTTP/2" => &mut h2_stats,
                    &"HTTP/3" => &mut h3_stats,
                    _ => unreachable!(),
                };

                match client.get(TEST_URL) {
                    Ok(_) => {
                        let elapsed = start.elapsed().as_millis() as u64;
                        stats.add_success(elapsed);
                    }
                    Err(_) => {
                        stats.add_failure();
                    }
                }

                // 极短间隔
                std::thread::sleep(Duration::from_millis(50));
            }
        }
    }

    println!("\n");
    let total_duration = start_time.elapsed();

    println!("\n╔══════════════════════════════════════════════════════════╗");
    println!("║  马拉松测试完成！                                        ║");
    println!("╚══════════════════════════════════════════════════════════╝");

    h1_stats.print_summary("HTTP/1.1");
    h2_stats.print_summary("HTTP/2");
    h3_stats.print_summary("HTTP/3");

    let total_requests = h1_stats.total_requests + h2_stats.total_requests + h3_stats.total_requests;
    let total_success = h1_stats.success_count + h2_stats.success_count + h3_stats.success_count;
    let total_fail = h1_stats.fail_count + h2_stats.fail_count + h3_stats.fail_count;

    println!("\n╔══════════════════════════════════════════════════════════╗");
    println!("║  总体统计                                                ║");
    println!("╚══════════════════════════════════════════════════════════╝");
    println!("\n  📊 总请求数: {}", total_requests);
    println!("  ✅ 总成功: {}", total_success);
    println!("  ❌ 总失败: {}", total_fail);
    println!("  🎯 总成功率: {:.1}%", (total_success as f64 / total_requests as f64) * 100.0);
    println!("\n  ⏱️  总耗时: {:.2}分钟", total_duration.as_secs_f64() / 60.0);
    println!("  📈 吞吐量: {:.2} 请求/秒", total_requests as f64 / total_duration.as_secs_f64());

    // 断言总成功率
    let overall_success_rate = (total_success as f64 / total_requests as f64) * 100.0;
    assert!(overall_success_rate >= 90.0, "总成功率过低: {:.1}%", overall_success_rate);
}

#[test]
#[cfg(all(feature = "http2", feature = "http3"))]
#[ignore]
fn test_continuous_quick_cycle() {
    println!("\n╔══════════════════════════════════════════════════════════╗");
    println!("║  快速循环测试 - 验证稳定性 (20 轮)                     ║");
    println!("╚══════════════════════════════════════════════════════════╝\n");

    let mut stats = ContinuousTestStats::new();

    for round in 1..=20 {
        print!("轮次 {}/20... ", round);

        // 随机选择协议
        let (h2, h3) = match round % 3 {
            0 => (false, false), // H1
            1 => (true, false),  // H2
            _ => (false, true),  // H3
        };

        let protocol = if h3 {
            "HTTP/3"
        } else if h2 {
            "HTTP/2"
        } else {
            "HTTP/1.1"
        };

        let config = HttpClientConfig {
            user_agent: "Mozilla/5.0 Test".to_string(),
            prefer_http2: h2,
            prefer_http3: h3,
            ..Default::default()
        };

        let client = HttpClient::new(config);
        let start = Instant::now();

        match client.get(TEST_URL) {
            Ok(response) => {
                let elapsed = start.elapsed().as_millis() as u64;
                stats.add_success(elapsed);
                println!("✅ {} {}ms (status {})", protocol, elapsed, response.status_code);
            }
            Err(e) => {
                stats.add_failure();
                println!("❌ {} {:?}", protocol, e);
            }
        }

        std::thread::sleep(Duration::from_millis(200));
    }

    println!("\n");
    stats.print_summary("快速循环测试");

    assert!(stats.success_rate() >= 85.0, "成功率过低: {:.1}%", stats.success_rate());
}
