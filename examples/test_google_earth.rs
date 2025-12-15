//! Google Earth API 全面测试可执行程序
//!
//! 测试地址: https://kh.google.com/rt/earth/PlanetoidMetadata
//! 测试所有 66 个浏览器指纹，使用 HTTP/1.1, HTTP/2, HTTP/3
//!
//! 运行方式:
//! ```bash
//! cargo run --example test_google_earth --features rustls-tls,http2,http3 -- --help
//! cargo run --example test_google_earth --features rustls-tls,http2,http3 -- http1
//! cargo run --example test_google_earth --features rustls-tls,http2,http3 -- http2
//! cargo run --example test_google_earth --features rustls-tls,http2,http3 -- http3
//! cargo run --example test_google_earth --features rustls-tls,http2,http3 -- all
//! ```

use fingerprint::{
    get_user_agent_by_profile_name, mapped_tls_clients, HttpClient, HttpClientConfig,
};
use std::collections::HashMap;
use std::env;
use std::time::{Duration, Instant};

const TEST_URL: &str = "https://kh.google.com/rt/earth/PlanetoidMetadata";

#[derive(Debug, Clone)]
struct TestResult {
    profile_name: String,
    protocol: String,
    success: bool,
    #[allow(dead_code)]
    status_code: Option<u16>,
    #[allow(dead_code)]
    response_size: usize,
    duration_ms: u64,
    error: Option<String>,
}

impl TestResult {
    fn success(
        profile_name: String,
        protocol: String,
        status_code: u16,
        response_size: usize,
        duration_ms: u64,
    ) -> Self {
        Self {
            profile_name,
            protocol,
            success: true,
            status_code: Some(status_code),
            response_size,
            duration_ms,
            error: None,
        }
    }

    fn failure(profile_name: String, protocol: String, error: String, duration_ms: u64) -> Self {
        Self {
            profile_name,
            protocol,
            success: false,
            status_code: None,
            response_size: 0,
            duration_ms,
            error: Some(error),
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 || args[1] == "--help" || args[1] == "-h" {
        print_usage();
        return;
    }

    let protocol = &args[1];

    match protocol.as_str() {
        "http1" => run_http1_test(),
        "http2" => {
            #[cfg(feature = "http2")]
            run_http2_test();
            #[cfg(not(feature = "http2"))]
            eprintln!("错误: HTTP/2 支持未启用，请使用 --features http2 编译");
        }
        "http3" => {
            #[cfg(feature = "http3")]
            run_http3_test();
            #[cfg(not(feature = "http3"))]
            eprintln!("错误: HTTP/3 支持未启用，请使用 --features http3 编译");
        }
        "all" => run_all_protocols_test(),
        _ => {
            eprintln!("错误: 未知的协议 '{}'", protocol);
            print_usage();
            std::process::exit(1);
        }
    }
}

fn print_usage() {
    println!("Google Earth API 全面测试工具");
    println!();
    println!("用法:");
    println!(
        "  cargo run --example test_google_earth --features rustls-tls,http2,http3 -- <protocol>"
    );
    println!();
    println!("协议选项:");
    println!("  http1  - 测试所有指纹使用 HTTP/1.1");
    println!("  http2  - 测试所有指纹使用 HTTP/2 (需要 --features http2)");
    println!("  http3  - 测试所有指纹使用 HTTP/3 (需要 --features http3)");
    println!("  all    - 测试所有协议 (66个指纹 × 3个协议 = 198个测试)");
    println!();
    println!("示例:");
    println!("  cargo run --example test_google_earth --features rustls-tls,http2,http3 -- http1");
    println!("  cargo run --example test_google_earth --features rustls-tls,http2,http3 -- all");
}

fn run_http1_test() {
    println!("\n╔══════════════════════════════════════════════════════════╗");
    println!("║  Google Earth API 全面测试 - HTTP/1.1                    ║");
    println!("║  地址: {}  ║", TEST_URL);
    println!("╚══════════════════════════════════════════════════════════╝\n");

    let profiles = mapped_tls_clients();
    let total = profiles.len();
    let mut results = Vec::new();
    let start = Instant::now();

    println!("🔍 测试所有 {} 个浏览器指纹 (HTTP/1.1)...\n", total);

    for (i, (name, profile)) in profiles.iter().enumerate() {
        print!("  [{:2}/{:2}] {:35} ... ", i + 1, total, name);
        std::io::Write::flush(&mut std::io::stdout()).unwrap();

        let test_start = Instant::now();
        let result = test_single_profile_http1(name, profile);
        let duration = test_start.elapsed().as_millis() as u64;

        match result {
            Ok((status, size)) => {
                println!("✅ {} ({}ms)", status, duration);
                results.push(TestResult::success(
                    name.clone(),
                    "HTTP/1.1".to_string(),
                    status,
                    size,
                    duration,
                ));
            }
            Err(e) => {
                println!("❌ {} ({}ms)", e, duration);
                results.push(TestResult::failure(
                    name.clone(),
                    "HTTP/1.1".to_string(),
                    e,
                    duration,
                ));
            }
        }

        std::thread::sleep(Duration::from_millis(100));
    }

    print_summary("HTTP/1.1", &results, start.elapsed());
}

#[cfg(feature = "http2")]
fn run_http2_test() {
    println!("\n╔══════════════════════════════════════════════════════════╗");
    println!("║  Google Earth API 全面测试 - HTTP/2                      ║");
    println!("║  地址: {}  ║", TEST_URL);
    println!("╚══════════════════════════════════════════════════════════╝\n");

    let profiles = mapped_tls_clients();
    let total = profiles.len();
    let mut results = Vec::new();
    let start = Instant::now();

    println!("🔍 测试所有 {} 个浏览器指纹 (HTTP/2)...\n", total);

    for (i, (name, profile)) in profiles.iter().enumerate() {
        print!("  [{:2}/{:2}] {:35} ... ", i + 1, total, name);
        std::io::Write::flush(&mut std::io::stdout()).unwrap();

        let test_start = Instant::now();
        let result = test_single_profile_http2(name, profile);
        let duration = test_start.elapsed().as_millis() as u64;

        match result {
            Ok((status, size)) => {
                println!("✅ {} ({}ms)", status, duration);
                results.push(TestResult::success(
                    name.clone(),
                    "HTTP/2".to_string(),
                    status,
                    size,
                    duration,
                ));
            }
            Err(e) => {
                println!("❌ {} ({}ms)", e, duration);
                results.push(TestResult::failure(
                    name.clone(),
                    "HTTP/2".to_string(),
                    e,
                    duration,
                ));
            }
        }

        std::thread::sleep(Duration::from_millis(100));
    }

    print_summary("HTTP/2", &results, start.elapsed());
}

#[cfg(feature = "http3")]
fn run_http3_test() {
    println!("\n╔══════════════════════════════════════════════════════════╗");
    println!("║  Google Earth API 全面测试 - HTTP/3                      ║");
    println!("║  地址: {}  ║", TEST_URL);
    println!("╚══════════════════════════════════════════════════════════╝\n");

    let profiles = mapped_tls_clients();
    let total = profiles.len();
    let mut results = Vec::new();
    let start = Instant::now();

    println!("🔍 测试所有 {} 个浏览器指纹 (HTTP/3)...\n", total);

    for (i, (name, profile)) in profiles.iter().enumerate() {
        print!("  [{:2}/{:2}] {:35} ... ", i + 1, total, name);
        std::io::Write::flush(&mut std::io::stdout()).unwrap();

        let test_start = Instant::now();
        let result = test_single_profile_http3(name, profile);
        let duration = test_start.elapsed().as_millis() as u64;

        match result {
            Ok((status, size)) => {
                println!("✅ {} ({}ms)", status, duration);
                results.push(TestResult::success(
                    name.clone(),
                    "HTTP/3".to_string(),
                    status,
                    size,
                    duration,
                ));
            }
            Err(e) => {
                println!("❌ {} ({}ms)", e, duration);
                results.push(TestResult::failure(
                    name.clone(),
                    "HTTP/3".to_string(),
                    e,
                    duration,
                ));
            }
        }

        std::thread::sleep(Duration::from_millis(200));
    }

    print_summary("HTTP/3", &results, start.elapsed());
}

fn run_all_protocols_test() {
    println!("\n╔══════════════════════════════════════════════════════════╗");
    println!("║  Google Earth API 全面测试 - 所有协议                    ║");
    println!("║  地址: {}  ║", TEST_URL);
    println!("║  测试: 所有 66 个浏览器指纹 × HTTP/1.1/HTTP/2/HTTP/3     ║");
    println!("╚══════════════════════════════════════════════════════════╝\n");

    let profiles = mapped_tls_clients();
    let total = profiles.len();
    let mut all_results = Vec::new();
    let start = Instant::now();

    println!("🔍 测试所有 {} 个浏览器指纹 × 所有协议...\n", total);

    for (i, (name, profile)) in profiles.iter().enumerate() {
        println!("\n[{:2}/{:2}] 测试指纹: {}", i + 1, total, name);
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

        // HTTP/1.1
        print!("  HTTP/1.1 ... ");
        std::io::Write::flush(&mut std::io::stdout()).unwrap();
        let h1_start = Instant::now();
        match test_single_profile_http1(name, profile) {
            Ok((status, size)) => {
                let duration = h1_start.elapsed().as_millis() as u64;
                println!("✅ {} ({}ms)", status, duration);
                all_results.push(TestResult::success(
                    name.clone(),
                    "HTTP/1.1".to_string(),
                    status,
                    size,
                    duration,
                ));
            }
            Err(e) => {
                let duration = h1_start.elapsed().as_millis() as u64;
                println!("❌ {} ({}ms)", e, duration);
                all_results.push(TestResult::failure(
                    name.clone(),
                    "HTTP/1.1".to_string(),
                    e,
                    duration,
                ));
            }
        }
        std::thread::sleep(Duration::from_millis(200));

        // HTTP/2
        #[cfg(feature = "http2")]
        {
            print!("  HTTP/2   ... ");
            std::io::Write::flush(&mut std::io::stdout()).unwrap();
            let h2_start = Instant::now();
            match test_single_profile_http2(name, profile) {
                Ok((status, size)) => {
                    let duration = h2_start.elapsed().as_millis() as u64;
                    println!("✅ {} ({}ms)", status, duration);
                    all_results.push(TestResult::success(
                        name.clone(),
                        "HTTP/2".to_string(),
                        status,
                        size,
                        duration,
                    ));
                }
                Err(e) => {
                    let duration = h2_start.elapsed().as_millis() as u64;
                    println!("❌ {} ({}ms)", e, duration);
                    all_results.push(TestResult::failure(
                        name.clone(),
                        "HTTP/2".to_string(),
                        e,
                        duration,
                    ));
                }
            }
            std::thread::sleep(Duration::from_millis(200));
        }

        // HTTP/3
        #[cfg(feature = "http3")]
        {
            print!("  HTTP/3   ... ");
            std::io::Write::flush(&mut std::io::stdout()).unwrap();
            let h3_start = Instant::now();
            match test_single_profile_http3(name, profile) {
                Ok((status, size)) => {
                    let duration = h3_start.elapsed().as_millis() as u64;
                    println!("✅ {} ({}ms)", status, duration);
                    all_results.push(TestResult::success(
                        name.clone(),
                        "HTTP/3".to_string(),
                        status,
                        size,
                        duration,
                    ));
                }
                Err(e) => {
                    let duration = h3_start.elapsed().as_millis() as u64;
                    println!("❌ {} ({}ms)", e, duration);
                    all_results.push(TestResult::failure(
                        name.clone(),
                        "HTTP/3".to_string(),
                        e,
                        duration,
                    ));
                }
            }
            std::thread::sleep(Duration::from_millis(300));
        }

        std::thread::sleep(Duration::from_millis(500));
    }

    print_comprehensive_summary(&all_results, start.elapsed());
}

fn test_single_profile_http1(
    profile_name: &str,
    profile: &fingerprint::ClientProfile,
) -> Result<(u16, usize), String> {
    let ua = get_user_agent_by_profile_name(profile_name)
        .map_err(|e| format!("获取 User-Agent 失败: {}", e))?;

    let config = HttpClientConfig {
        user_agent: ua,
        profile: Some(profile.clone()),
        prefer_http2: false,
        prefer_http3: false,
        connect_timeout: Duration::from_secs(10),
        read_timeout: Duration::from_secs(10),
        ..Default::default()
    };

    let client = HttpClient::new(config);
    let response = client
        .get(TEST_URL)
        .map_err(|e| format!("请求失败: {}", e))?;

    if !response.http_version.contains("HTTP/1.1") {
        return Err(format!(
            "协议不匹配: 期望 HTTP/1.1，实际 {}",
            response.http_version
        ));
    }

    if response.status_code != 200 {
        return Err(format!("状态码错误: {}", response.status_code));
    }

    Ok((response.status_code, response.body.len()))
}

#[cfg(feature = "http2")]
fn test_single_profile_http2(
    profile_name: &str,
    profile: &fingerprint::ClientProfile,
) -> Result<(u16, usize), String> {
    let ua = get_user_agent_by_profile_name(profile_name)
        .map_err(|e| format!("获取 User-Agent 失败: {}", e))?;

    let config = HttpClientConfig {
        user_agent: ua,
        profile: Some(profile.clone()),
        prefer_http2: true,
        prefer_http3: false,
        connect_timeout: Duration::from_secs(10),
        read_timeout: Duration::from_secs(10),
        ..Default::default()
    };

    let client = HttpClient::new(config);
    let response = client
        .get(TEST_URL)
        .map_err(|e| format!("请求失败: {}", e))?;

    if !response.http_version.contains("HTTP/2") {
        return Err(format!(
            "协议不匹配: 期望 HTTP/2，实际 {}",
            response.http_version
        ));
    }

    if response.status_code != 200 {
        return Err(format!("状态码错误: {}", response.status_code));
    }

    Ok((response.status_code, response.body.len()))
}

#[cfg(feature = "http3")]
fn test_single_profile_http3(
    profile_name: &str,
    profile: &fingerprint::ClientProfile,
) -> Result<(u16, usize), String> {
    let ua = get_user_agent_by_profile_name(profile_name)
        .map_err(|e| format!("获取 User-Agent 失败: {}", e))?;

    let config = HttpClientConfig {
        user_agent: ua,
        profile: Some(profile.clone()),
        prefer_http2: false,
        prefer_http3: true,
        connect_timeout: Duration::from_secs(15),
        read_timeout: Duration::from_secs(15),
        ..Default::default()
    };

    let client = HttpClient::new(config);
    let response = client
        .get(TEST_URL)
        .map_err(|e| format!("请求失败: {}", e))?;

    if !response.http_version.contains("HTTP/3") {
        return Err(format!(
            "协议不匹配: 期望 HTTP/3，实际 {}",
            response.http_version
        ));
    }

    if response.status_code != 200 {
        return Err(format!("状态码错误: {}", response.status_code));
    }

    Ok((response.status_code, response.body.len()))
}

fn print_summary(protocol: &str, results: &[TestResult], total_duration: Duration) {
    let success: Vec<_> = results.iter().filter(|r| r.success).collect();
    let failed: Vec<_> = results.iter().filter(|r| !r.success).collect();

    println!("\n╔══════════════════════════════════════════════════════════╗");
    println!("║  {} 测试结果汇总                        ║", protocol);
    println!("╚══════════════════════════════════════════════════════════╝\n");

    println!("  总测试数: {}", results.len());
    println!("  成功: {} ✅", success.len());
    println!("  失败: {} ❌", failed.len());
    println!(
        "  成功率: {:.1}%",
        (success.len() as f64 / results.len() as f64) * 100.0
    );
    println!("  总耗时: {:.2}s", total_duration.as_secs_f64());

    if !success.is_empty() {
        let avg_time: f64 =
            success.iter().map(|r| r.duration_ms as f64).sum::<f64>() / success.len() as f64;
        let min_time = success.iter().map(|r| r.duration_ms).min().unwrap_or(0);
        let max_time = success.iter().map(|r| r.duration_ms).max().unwrap_or(0);
        println!("\n  响应时间统计:");
        println!("    平均: {:.0}ms", avg_time);
        println!("    最小: {}ms", min_time);
        println!("    最大: {}ms", max_time);
    }

    if !failed.is_empty() {
        println!("\n  ❌ 失败的指纹:");
        for result in failed.iter().take(10) {
            println!(
                "    - {}: {}",
                result.profile_name,
                result.error.as_ref().unwrap_or(&"未知错误".to_string())
            );
        }
        if failed.len() > 10 {
            println!("    ... 还有 {} 个失败", failed.len() - 10);
        }
    }
}

fn print_comprehensive_summary(results: &[TestResult], total_duration: Duration) {
    let success: Vec<_> = results.iter().filter(|r| r.success).collect();
    let failed: Vec<_> = results.iter().filter(|r| !r.success).collect();

    // 按协议分组
    let mut by_protocol: HashMap<String, Vec<&TestResult>> = HashMap::new();
    for result in results {
        by_protocol
            .entry(result.protocol.clone())
            .or_insert_with(Vec::new)
            .push(result);
    }

    println!("\n╔══════════════════════════════════════════════════════════╗");
    println!("║  全面测试结果汇总                                        ║");
    println!("╚══════════════════════════════════════════════════════════╝\n");

    println!("  总测试数: {}", results.len());
    println!("  成功: {} ✅", success.len());
    println!("  失败: {} ❌", failed.len());
    println!(
        "  成功率: {:.1}%",
        (success.len() as f64 / results.len() as f64) * 100.0
    );
    println!("  总耗时: {:.2}s", total_duration.as_secs_f64());

    println!("\n  按协议统计:");
    for (protocol, protocol_results) in &by_protocol {
        let protocol_success = protocol_results.iter().filter(|r| r.success).count();
        println!(
            "    {}: {}/{} ({:.1}%)",
            protocol,
            protocol_success,
            protocol_results.len(),
            (protocol_success as f64 / protocol_results.len() as f64) * 100.0
        );
    }

    // 按浏览器类型分组统计
    let mut by_browser: HashMap<String, (usize, usize)> = HashMap::new();
    for result in results {
        let browser_type = result.profile_name.split('_').next().unwrap_or("unknown");
        let entry = by_browser.entry(browser_type.to_string()).or_insert((0, 0));
        if result.success {
            entry.0 += 1;
        }
        entry.1 += 1;
    }

    println!("\n  按浏览器类型统计:");
    for (browser, (success_count, total_count)) in &by_browser {
        println!(
            "    {}: {}/{} ({:.1}%)",
            browser,
            success_count,
            total_count,
            (*success_count as f64 / *total_count as f64) * 100.0
        );
    }

    if !failed.is_empty() {
        println!("\n  ❌ 失败的测试 (前 20 个):");
        for result in failed.iter().take(20) {
            println!(
                "    - {} [{}]: {}",
                result.profile_name,
                result.protocol,
                result.error.as_ref().unwrap_or(&"未知错误".to_string())
            );
        }
        if failed.len() > 20 {
            println!("    ... 还有 {} 个失败", failed.len() - 20);
        }
    }
}
