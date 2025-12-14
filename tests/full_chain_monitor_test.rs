//! 完整链路监控测试
//! 从 netconnpool → TLS 指纹 → 服务器请求
//! 详细监控每个环节的时间消耗

use fingerprint::{HttpClient, HttpClientConfig};
use std::time::Instant;

const TEST_URL: &str = "https://kh.google.com/rt/earth/PlanetoidMetadata";

#[derive(Debug)]
struct ChainMetrics {
    /// 总耗时
    total_time_ms: u64,
    /// DNS 解析时间（如果有）
    dns_time_ms: Option<u64>,
    /// TCP 连接时间
    tcp_connect_time_ms: Option<u64>,
    /// TLS 握手时间
    tls_handshake_time_ms: Option<u64>,
    /// HTTP 请求时间
    http_request_time_ms: Option<u64>,
    /// HTTP 响应时间
    http_response_time_ms: Option<u64>,
    /// 数据传输时间
    data_transfer_time_ms: Option<u64>,
    /// 响应状态码
    status_code: u16,
    /// 响应体大小
    body_size: usize,
}

impl ChainMetrics {
    fn print(&self, label: &str) {
        println!("\n  ⏱️  {} - 完整链路分析:", label);
        println!("     总耗时: {}ms", self.total_time_ms);
        println!("     状态码: {}", self.status_code);
        println!("     Body 大小: {} bytes", self.body_size);

        // 注意：当前实现无法分离各个环节，这里显示总时间
        println!("     链路时间: {}ms (包含所有环节)", self.total_time_ms);
    }
}

/// 测试单个请求的完整链路
fn test_chain_single_request(label: &str, prefer_h2: bool, prefer_h3: bool) -> ChainMetrics {
    let config = HttpClientConfig {
        user_agent: "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36".to_string(),
        prefer_http2: prefer_h2,
        prefer_http3: prefer_h3,
        ..Default::default()
    };

    let client = HttpClient::new(config);

    let start = Instant::now();
    let response = client.get(TEST_URL).expect("请求失败");
    let total_time = start.elapsed().as_millis() as u64;

    ChainMetrics {
        total_time_ms: total_time,
        dns_time_ms: None, // TODO: 需要在实现中分离
        tcp_connect_time_ms: None,
        tls_handshake_time_ms: None,
        http_request_time_ms: None,
        http_response_time_ms: None,
        data_transfer_time_ms: None,
        status_code: response.status_code,
        body_size: response.body.len(),
    }
}

#[test]
#[ignore]
fn test_http1_chain_detailed() {
    println!("\n╔══════════════════════════════════════════════════════════╗");
    println!("║  HTTP/1.1 完整链路监控                                  ║");
    println!("╚══════════════════════════════════════════════════════════╝");

    for round in 1..=5 {
        println!("\n🔹 轮次 {}/5", round);
        let metrics = test_chain_single_request("HTTP/1.1", false, false);
        metrics.print("HTTP/1.1");
        std::thread::sleep(std::time::Duration::from_millis(500));
    }
}

#[test]
#[cfg(feature = "http2")]
#[ignore]
fn test_http2_chain_detailed() {
    println!("\n╔══════════════════════════════════════════════════════════╗");
    println!("║  HTTP/2 完整链路监控                                    ║");
    println!("╚══════════════════════════════════════════════════════════╝");

    for round in 1..=5 {
        println!("\n🔹 轮次 {}/5", round);
        let metrics = test_chain_single_request("HTTP/2", true, false);
        metrics.print("HTTP/2");
        std::thread::sleep(std::time::Duration::from_millis(500));
    }
}

#[test]
#[cfg(feature = "http3")]
#[ignore]
fn test_http3_chain_detailed() {
    println!("\n╔══════════════════════════════════════════════════════════╗");
    println!("║  HTTP/3 完整链路监控                                    ║");
    println!("╚══════════════════════════════════════════════════════════╝");

    for round in 1..=5 {
        println!("\n🔹 轮次 {}/5", round);
        let metrics = test_chain_single_request("HTTP/3", false, true);
        metrics.print("HTTP/3");
        std::thread::sleep(std::time::Duration::from_millis(500));
    }
}

#[test]
#[cfg(all(feature = "http2", feature = "http3"))]
#[ignore]
fn test_all_protocols_chain_comparison() {
    println!("\n╔══════════════════════════════════════════════════════════╗");
    println!("║  所有协议链路对比                                        ║");
    println!("║  目标: 找出最快的协议和瓶颈环节                          ║");
    println!("╚══════════════════════════════════════════════════════════╝");

    let protocols = vec![
        ("HTTP/1.1", false, false),
        ("HTTP/2", true, false),
        ("HTTP/3", false, true),
    ];

    let mut all_metrics = Vec::new();

    for (name, h2, h3) in protocols {
        println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("  测试协议: {}", name);
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

        let mut round_metrics = Vec::new();

        for round in 1..=10 {
            print!("  轮次 {}/10... ", round);
            let metrics = test_chain_single_request(name, h2, h3);
            println!("{}ms", metrics.total_time_ms);
            round_metrics.push(metrics);
            std::thread::sleep(std::time::Duration::from_millis(200));
        }

        let avg_time: f64 = round_metrics.iter().map(|m| m.total_time_ms).sum::<u64>() as f64
            / round_metrics.len() as f64;
        let min_time = round_metrics.iter().map(|m| m.total_time_ms).min().unwrap();
        let max_time = round_metrics.iter().map(|m| m.total_time_ms).max().unwrap();

        println!("\n  📊 {} 统计:", name);
        println!("     平均: {:.2}ms", avg_time);
        println!("     最小: {}ms", min_time);
        println!("     最大: {}ms", max_time);
        println!("     方差: {:.2}ms", max_time - min_time);

        all_metrics.push((name, avg_time, min_time, max_time));
    }

    println!("\n╔══════════════════════════════════════════════════════════╗");
    println!("║  最终对比                                                ║");
    println!("╚══════════════════════════════════════════════════════════╝\n");

    // 排序找出最快的
    let mut sorted = all_metrics.clone();
    sorted.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

    for (i, (name, avg, min, max)) in sorted.iter().enumerate() {
        let medal = match i {
            0 => "🥇",
            1 => "🥈",
            2 => "🥉",
            _ => "  ",
        };
        println!(
            "  {} {} - 平均: {:.2}ms (min: {}ms, max: {}ms)",
            medal, name, avg, min, max
        );
    }

    let fastest = &sorted[0];
    let slowest = &sorted[sorted.len() - 1];
    let improvement = ((slowest.1 - fastest.1) / slowest.1) * 100.0;

    println!(
        "\n  ⚡ {} 比 {} 快 {:.1}%",
        fastest.0, slowest.0, improvement
    );
}
