//! 性能基准测试
//! 验证HTTP/1.1、HTTP/2、HTTP/3的响应时间性能指标

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use fingerprint::{HttpClient, HttpClientConfig};
use std::time::Instant;

fn benchmark_http_protocols(c: &mut Criterion) {
    let mut group = c.benchmark_group("http_protocol_performance");
    group.sample_size(50); // 增加采样次数以获得更准确的结果

    // 测试目标URL（使用可靠的服务）
    let test_url = "https://httpbin.org/get";

    // HTTP/1.1测试
    group.bench_function("http1_response_time", |b| {
        b.iter(|| {
            let config = HttpClientConfig {
                user_agent: "Mozilla/5.0 (X11; Linux x86_64) Chrome/133.0.0.0".to_string(),
                prefer_http3: false,
                prefer_http2: false,
                ..Default::default()
            };
            
            let client = HttpClient::new(config).expect("Failed to create HTTP client");
            let start = Instant::now();
            let result = black_box(client.get(test_url));
            let duration = start.elapsed();
            
            // 验证响应时间和成功率
            assert!(result.is_ok(), "HTTP/1.1 request should succeed");
            assert!(duration.as_millis() < 1000, "HTTP/1.1 response should be under 1 second");
            
            duration
        })
    });

    // HTTP/2测试
    group.bench_function("http2_response_time", |b| {
        b.iter(|| {
            let config = HttpClientConfig {
                user_agent: "Mozilla/5.0 (X11; Linux x86_64) Chrome/133.0.0.0".to_string(),
                prefer_http3: false,
                prefer_http2: true,
                ..Default::default()
            };
            
            let client = HttpClient::new(config).expect("Failed to create HTTP/2 client");
            let start = Instant::now();
            let result = black_box(client.get(test_url));
            let duration = start.elapsed();
            
            assert!(result.is_ok(), "HTTP/2 request should succeed");
            assert!(duration.as_millis() < 1000, "HTTP/2 response should be under 1 second");
            
            duration
        })
    });

    // HTTP/3测试
    group.bench_function("http3_response_time", |b| {
        b.iter(|| {
            let config = HttpClientConfig {
                user_agent: "Mozilla/5.0 (X11; Linux x86_64) Chrome/133.0.0.0".to_string(),
                prefer_http3: true,
                prefer_http2: true,
                ..Default::default()
            };
            
            let client = HttpClient::new(config).expect("Failed to create HTTP/3 client");
            let start = Instant::now();
            let result = black_box(client.get(test_url));
            let duration = start.elapsed();
            
            // HTTP/3可能不可用，但不应该失败
            if let Ok(response) = result {
                assert!(duration.as_millis() < 1000, "HTTP/3 response should be under 1 second");
            }
            
            duration
        })
    });

    group.finish();
}

fn benchmark_fingerprint_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("fingerprint_generation");
    
    // 测试随机指纹生成性能
    group.bench_function("random_fingerprint_generation", |b| {
        b.iter(|| {
            let start = Instant::now();
            let result = black_box(fingerprint::get_random_fingerprint());
            let duration = start.elapsed();
            
            assert!(result.is_ok(), "Random fingerprint generation should succeed");
            duration
        })
    });

    // 测试特定浏览器指纹生成
    group.bench_function("chrome_133_fingerprint_generation", |b| {
        b.iter(|| {
            let start = Instant::now();
            let profile = black_box(fingerprint::chrome_133());
            let duration = start.elapsed();
            
            assert!(!profile.get_client_hello_str().is_empty(), "Profile should be valid");
            duration
        })
    });

    group.finish();
}

fn benchmark_ja4_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("ja4_generation");
    
    // 测试JA4指纹生成性能
    group.bench_function("ja4_fingerprint_generation", |b| {
        b.iter(|| {
            use fingerprint_core::ja4::JA4;
            
            let start = Instant::now();
            // 创建简单的TLS数据进行测试
            let tls_data = vec![0x16, 0x03, 0x03, 0x00, 0x4a]; // 简化的ClientHello头部
            let ja4 = black_box(JA4::from_client_hello(&tls_data));
            let duration = start.elapsed();
            
            // JA4生成应该快速完成
            assert!(duration.as_micros() < 1000, "JA4 generation should be under 1ms");
            duration
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_http_protocols,
    benchmark_fingerprint_generation,
    benchmark_ja4_generation
);
criterion_main!(benches);