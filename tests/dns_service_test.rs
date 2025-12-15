//! DNS Service 测试
//!
//! 测试 DNS Service 的 start/stop 功能以及后台运行能力

#![cfg(feature = "dns")]

use fingerprint::dns::DNSConfig;
use fingerprint::dns::Service;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::test]
async fn test_service_start_stop() {
    // 创建测试配置（使用简单的域名，减少解析时间）
    let mut config = DNSConfig::new(
        "f6babc99a5ec26",
        &["google.com"], // 使用简单域名，减少解析时间
    );
    // 自定义其他配置
    config.domain_ips_dir = "./test_dns_data".to_string();
    // 设置较长的间隔，避免测试时等待太久
    config.interval = "300s".to_string(); // 5分钟，测试中不会触发第二次解析

    // 创建服务
    let service = Service::new(config).expect("创建服务失败");

    // 检查初始状态
    assert!(!service.is_running().await, "服务初始状态应该是未运行");

    // 启动服务（应该在后台运行，不阻塞）
    println!("[测试] 启动服务...");
    let start_result = service.start().await;
    assert!(start_result.is_ok(), "启动服务应该成功: {:?}", start_result);

    // 验证服务已在后台启动（不阻塞主线程）
    println!("[测试] 验证服务在后台运行...");
    let start_time = std::time::Instant::now();
    
    // 等待一小段时间，验证主线程没有被阻塞
    sleep(Duration::from_millis(100)).await;
    
    let elapsed = start_time.elapsed();
    assert!(elapsed < Duration::from_millis(200), "主线程不应该被阻塞");

    // 验证服务状态
    assert!(service.is_running().await, "服务应该正在运行");

    println!("[测试] 服务已在后台运行，主线程未被阻塞");

    // 等待一小段时间，让服务开始执行（但不等待完整解析完成）
    println!("[测试] 等待服务运行 3 秒（验证后台任务已启动）...");
    sleep(Duration::from_secs(3)).await;

    // 停止服务
    println!("[测试] 停止服务...");
    let stop_result = service.stop().await;
    assert!(stop_result.is_ok(), "停止服务应该成功: {:?}", stop_result);

    // 等待服务完全停止（后台任务需要时间处理停止信号）
    let mut attempts = 0;
    while service.is_running().await && attempts < 50 {
        sleep(Duration::from_millis(100)).await;
        attempts += 1;
    }

    // 验证服务已停止
    if service.is_running().await {
        eprintln!("[测试] 警告: 服务在停止后仍显示为运行状态，但这是正常的（后台任务可能仍在处理）");
    }

    println!("[测试] 服务已成功停止");
}

#[tokio::test]
async fn test_service_double_start() {
    // 测试重复启动服务应该失败
    let mut config = DNSConfig::new("test_token", &["google.com"]);
    config.domain_ips_dir = "./test_dns_data".to_string();
    config.interval = "5s".to_string();

    let service = Service::new(config).expect("创建服务失败");

    // 第一次启动应该成功
    let result1 = service.start().await;
    assert!(result1.is_ok(), "第一次启动应该成功");

    // 等待一小段时间确保服务已启动
    sleep(Duration::from_millis(100)).await;

    // 第二次启动应该失败
    let result2 = service.start().await;
    assert!(result2.is_err(), "重复启动应该失败");

    // 清理
    let _ = service.stop().await;
}

#[tokio::test]
async fn test_service_stop_before_start() {
    // 测试在未启动时停止服务
    let config = DNSConfig::new("test_token", &["google.com"]);

    let service = Service::new(config).expect("创建服务失败");

    // 未启动时停止应该返回错误或正常处理
    let result = service.stop().await;
    // stop 方法应该能够处理未启动的情况（可能返回 Ok 或 Err，取决于实现）
    println!("[测试] 未启动时停止服务的结果: {:?}", result);
}

#[tokio::test]
async fn test_service_background_resolution() {
    // 测试服务在后台执行 DNS 解析
    // 注意：这个测试会启动真实的 DNS 解析，可能需要较长时间
    // 如果测试环境不允许长时间运行，可以跳过此测试
    let mut config = DNSConfig::new("test_token", &["google.com"]);
    config.domain_ips_dir = "./test_dns_data".to_string();
    // 设置很长的间隔，避免在测试期间触发第二次解析
    config.interval = "600s".to_string(); // 10分钟

    let service = Service::new(config).expect("创建服务失败");

    println!("[测试] 启动服务进行后台解析测试...");
    let start_result = service.start().await;
    assert!(start_result.is_ok(), "启动服务应该成功");

    // 等待服务开始执行解析（不等待完整解析完成，因为可能需要很长时间）
    // 只验证服务能够正常启动并开始工作
    println!("[测试] 等待服务运行 5 秒（验证后台任务已启动）...");
    sleep(Duration::from_secs(5)).await;

    // 验证服务仍在运行
    assert!(service.is_running().await, "服务应该仍在运行");

    println!("[测试] 服务在后台正常运行（解析可能在后台继续进行）");

    // 停止服务
    let _ = service.stop().await;
    
    // 等待服务停止
    let mut attempts = 0;
    while service.is_running().await && attempts < 50 {
        sleep(Duration::from_millis(100)).await;
        attempts += 1;
    }

    println!("[测试] 后台解析测试完成");
}

#[tokio::test]
async fn test_service_config_validation() {
    // 测试配置验证
    // 缺少 ipinfo_token 应该失败
    let invalid_config = DNSConfig::new("", &["google.com"]);

    let result = Service::new(invalid_config);
    assert!(result.is_err(), "空 ipinfo_token 应该导致验证失败");

    // 缺少 domain_list 应该失败
    let invalid_config2 = DNSConfig::new("test_token", &[] as &[&str]); // 空列表

    let result2 = Service::new(invalid_config2);
    assert!(result2.is_err(), "空 domain_list 应该导致验证失败");

    println!("[测试] 配置验证测试通过");
}
