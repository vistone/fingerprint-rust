//! performancetesting套件
//!
//! testingallprotocolofperformance表现，including响应time、throughput等指标
//!
//! run方式:
//! ```bash
//! # runallperformancetesting
//! cargo test --test performance_test --features rustls-tls,http2,http3 -- --ignored --nocapture
//!
//! # run特定protocoltesting
//! cargo test --test performance_test benchmark_http1 --features rustls-tls -- --ignored
//! cargo test --test performance_test benchmark_http2 --features rustls-tls,http2 -- --ignored
//! cargo test --test performance_test benchmark_http3 --features rustls-tls,http3 -- --ignored
//! ```

use fingerprint::{HttpClient, HttpClientConfig};
use std::time::Instant;

const TEST_URL: &str = "https://kh.google.com/rt/earth/PlanetoidMetadata";
const TEST_ROUNDS: usize = 10; // 每个protocoltesting10次

#[derive(Debug)]
struct PerformanceMetrics {
    protocol: String,
    total_time_ms: Vec<u64>,
    body_size_bytes: Vec<usize>,
    success_count: usize,
    fail_count: usize,
}

impl PerformanceMetrics {
    fn new(protocol: &str) -> Self {
        Self {
            protocol: protocol.to_string(),
            total_time_ms: Vec::new(),
            body_size_bytes: Vec::new(),
            success_count: 0,
            fail_count: 0,
        }
    }

    fn add_success(&mut self, total_ms: u64, body_size: usize) {
        self.total_time_ms.push(total_ms);
        self.body_size_bytes.push(body_size);
        self.success_count += 1;
    }

    fn add_failure(&mut self) {
        self.fail_count += 1;
    }

    fn avg(&self, data: &[u64]) -> f64 {
        if data.is_empty() {
            return 0.0;
        }
        data.iter().sum::<u64>() as f64 / data.len() as f64
    }

    fn min(&self, data: &[u64]) -> u64 {
        *data.iter().min().unwrap_or(&0)
    }

    fn max(&self, data: &[u64]) -> u64 {
        *data.iter().max().unwrap_or(&0)
    }

    fn median(&self, data: &[u64]) -> f64 {
        let mut sorted = data.to_vec();
        sorted.sort();
        let len = sorted.len();
        if len == 0 {
            return 0.0;
        }
        if len.is_multiple_of(2) {
            (sorted[len / 2 - 1] + sorted[len / 2]) as f64 / 2.0
        } else {
            sorted[len / 2] as f64
        }
    }

    fn print_summary(&self) {
        println!("\n╔══════════════════════════════════════════════════════════╗");
        println!("║  {} 性能报告", self.protocol);
        println!("╚══════════════════════════════════════════════════════════╝");

        println!("\n📊 测试结果:");
        println!(
            "  成功: {} / {}",
            self.success_count,
            self.success_count + self.fail_count
        );
        println!("  失败: {}", self.fail_count);

        if !self.total_time_ms.is_empty() {
            println!("\n⏱️  总响应时间 (ms):");
            println!("  平均: {:.2}", self.avg(&self.total_time_ms));
            println!("  最小: {}", self.min(&self.total_time_ms));
            println!("  最大: {}", self.max(&self.total_time_ms));
            println!("  中位: {:.2}", self.median(&self.total_time_ms));

            println!("\n📦 数据大小 (bytes):");
            let body_sizes: Vec<u64> = self.body_size_bytes.iter().map(|&x| x as u64).collect();
            println!("  平均: {:.2}", self.avg(&body_sizes));
            println!("  最小: {}", self.min(&body_sizes));
            println!("  最大: {}", self.max(&body_sizes));

            if self.success_count > 0 {
                let throughput = (self.body_size_bytes.iter().sum::<usize>() as f64 * 1000.0)
                    / (self.total_time_ms.iter().sum::<u64>() as f64);
                println!("\n🚀 吞吐量:");
                println!("  {:.2} bytes/s", throughput);
                println!("  {:.2} KB/s", throughput / 1024.0);
            }
        }
    }
}

// ============================================================================
// 1. 单protocolperformancetesting
// ============================================================================

#[test]
#[ignore] // requirenetworkconnect
fn benchmark_http1() {
    println!("\n═══════════════════════════════════════");
    println!("  HTTP/1.1 性能基准测试");
    println!("═══════════════════════════════════════\n");

    let config = HttpClientConfig {
        user_agent: "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36".to_string(),
        prefer_http2: false,
        prefer_http3: false,
        ..Default::default()
    };

    let client = HttpClient::new(config);
    let mut metrics = PerformanceMetrics::new("HTTP/1.1");

    for round in 1..=TEST_ROUNDS {
        print!("  轮次 {}/{}... ", round, TEST_ROUNDS);

        let start = Instant::now();
        match client.get(TEST_URL) {
            Ok(response) => {
                let elapsed = start.elapsed().as_millis() as u64;
                metrics.add_success(elapsed, response.body.len());
                println!("✅ {}ms, {} bytes", elapsed, response.body.len());
            }
            Err(e) => {
                metrics.add_failure();
                println!("❌ 失败: {:?}", e);
            }
        }

        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    metrics.print_summary();
}

#[test]
#[cfg(feature = "http2")]
#[ignore] // requirenetworkconnect
fn benchmark_http2() {
    println!("\n═══════════════════════════════════════");
    println!("  HTTP/2 性能基准测试");
    println!("═══════════════════════════════════════\n");

    let config = HttpClientConfig {
        user_agent: "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36".to_string(),
        prefer_http2: true,
        prefer_http3: false,
        ..Default::default()
    };

    let client = HttpClient::new(config);
    let mut metrics = PerformanceMetrics::new("HTTP/2");

    for round in 1..=TEST_ROUNDS {
        print!("  轮次 {}/{}... ", round, TEST_ROUNDS);

        let start = Instant::now();
        match client.get(TEST_URL) {
            Ok(response) => {
                let elapsed = start.elapsed().as_millis() as u64;
                metrics.add_success(elapsed, response.body.len());
                println!("✅ {}ms, {} bytes", elapsed, response.body.len());
            }
            Err(e) => {
                metrics.add_failure();
                println!("❌ 失败: {:?}", e);
            }
        }

        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    metrics.print_summary();
}

#[test]
#[cfg(feature = "http3")]
#[ignore] // requirenetworkconnect
fn benchmark_http3() {
    println!("\n═══════════════════════════════════════");
    println!("  HTTP/3 性能基准测试");
    println!("═══════════════════════════════════════\n");

    let config = HttpClientConfig {
        user_agent: "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36".to_string(),
        prefer_http2: false,
        prefer_http3: true,
        ..Default::default()
    };

    let client = HttpClient::new(config);
    let mut metrics = PerformanceMetrics::new("HTTP/3");

    for round in 1..=TEST_ROUNDS {
        print!("  轮次 {}/{}... ", round, TEST_ROUNDS);

        let start = Instant::now();
        match client.get(TEST_URL) {
            Ok(response) => {
                let elapsed = start.elapsed().as_millis() as u64;
                metrics.add_success(elapsed, response.body.len());
                println!("✅ {}ms, {} bytes", elapsed, response.body.len());
            }
            Err(e) => {
                metrics.add_failure();
                println!("❌ 失败: {:?}", e);
            }
        }

        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    metrics.print_summary();
}

// ============================================================================
// 2. 全protocolperformance对比
// ============================================================================

#[test]
#[cfg(all(feature = "http2", feature = "http3"))]
#[ignore] // requirenetworkconnect
fn benchmark_all_protocols() {
    println!("\n╔══════════════════════════════════════════════════════════╗");
    println!("║  Google Earth API 全协议性能对比                        ║");
    println!("║  URL: {}              ║", TEST_URL);
    println!("╚══════════════════════════════════════════════════════════╝");

    benchmark_http1();
    benchmark_http2();
    benchmark_http3();

    println!("\n╔══════════════════════════════════════════════════════════╗");
    println!("║  测试完成                                                ║");
    println!("╚══════════════════════════════════════════════════════════╝\n");
}
