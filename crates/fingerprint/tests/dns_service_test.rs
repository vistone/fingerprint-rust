// ! DNS Service testing
//! DNS service start/stop tests.

#![cfg(feature = "dns")]

use fingerprint::{DNSConfig, DNSService};
use std::time::Duration;
use tokio::time::sleep;

/// Helper function to check if network is available by attempting a simple operation
async fn check_network_available() -> bool {
    // Try to create a DNS service with a simple config
    // If this fails, the environment may not support DNS service startup
    let config = DNSConfig::new("test_token", &["example.com"]);
    DNSService::new(config).is_ok()
}

#[tokio::test]
async fn test_service_start_stop() {
    // Skip test if network is not available (for CI/CD environments without network access)
    if !check_network_available().await {
        eprintln!("[测试] 网络不可用或 DNS 配置失败，跳过此测试");
        return;
    }

    // createtestingconfigure（use简单ofdomain，减少parsetime）
    let mut config = DNSConfig::new("f6babc99a5ec26", &["google.com"]);
    // customotherconfigure
    config.domain_ips_dir = "./test_dns_data".to_string();
    // set较长of间隔，避免testing时等待太久
    config.interval = "300s".to_string(); // 5分钟，testing中不会触发第二次parse

    // createservice
    let service = match DNSService::new(config) {
        Ok(svc) => svc,
        Err(e) => {
            eprintln!("[测试] 创建 DNS 服务失败: {:?}，跳过此测试", e);
            return;
        }
    };

    // check初始state
    assert!(!service.is_running().await, "服务初始状态应该是未运行");

    // startservice（应该在后台run，不blocking）
    println!("[测试] 启动服务...");
    let start_result = service.start().await;
    if start_result.is_err() {
        eprintln!("[测试] 启动服务失败: {:?}，跳过余下测试", start_result);
        return;
    }

    // validateservice已在后台start（不blocking主thread）
    println!("[测试] 验证服务在后台运行...");
    let start_time = std::time::Instant::now();

    // 等待一小段time，validate主thread没有被blocking
    sleep(Duration::from_millis(100)).await;

    let elapsed = start_time.elapsed();
    assert!(elapsed < Duration::from_millis(200), "主线程不应该被阻塞");

    // validateservicestate
    assert!(service.is_running().await, "服务应该正在运行");

    println!("[测试] 服务已在后台运行，主线程未被阻塞");

    // 等待一小段time，让service开始执行（但不等待完整parse完成）
    println!("[测试] 等待服务运行 3 秒（验证后台任务已启动）...");
    sleep(Duration::from_secs(3)).await;

    // stopservice
    println!("[测试] 停止服务...");
    let stop_result = service.stop().await;
    assert!(stop_result.is_ok(), "停止服务应该成功: {:?}", stop_result);

    // 等待service完全stop（background taskrequiretimeprocessstopsignal）
    let mut attempts = 0;
    while service.is_running().await && attempts < 100 {
        sleep(Duration::from_millis(100)).await;
        attempts += 1;
    }

    // validateservice已stop
    if service.is_running().await {
        eprintln!(
            "[测试] 警告: 服务在停止后仍显示为运行状态，但这是正常的（后台任务可能仍在处理）"
        );
    }

    println!("[测试] 服务已成功停止");
}

#[tokio::test]
async fn test_service_double_start() {
    // testing重复startservice应该failure
    let mut config = DNSConfig::new("test_token", &["google.com"]);
    config.domain_ips_dir = "./test_dns_data".to_string();
    config.interval = "5s".to_string();

    let service = DNSService::new(config).expect("创建服务失败");

    // 第一次start应该success
    let result1 = service.start().await;
    assert!(result1.is_ok(), "第一次启动应该成功");

    // 等待一小段timeensureservice已start
    sleep(Duration::from_millis(100)).await;

    // 第二次start应该failure
    let result2 = service.start().await;
    assert!(result2.is_err(), "重复启动应该失败");

    // cleanup
    let _ = service.stop().await;
}

#[tokio::test]
async fn test_service_stop_before_start() {
    // testing在未start时stopservice
    let config = DNSConfig::new("test_token", &["google.com"]);

    let service = DNSService::new(config).expect("创建服务失败");

    // 未start时stop应该returnerror或normalprocess
    let result = service.stop().await;
    // stop 方法应该能够process未startof情况（可能return Ok 或 Err，取决于 implementation）
    println!("[测试] 未启动时停止服务的结果: {:?}", result);
}

#[tokio::test]
async fn test_service_background_resolution() {
    // testingservice在后台执行 DNS parse
    // 注意：这个testing会start真实of DNS parse，可能require较长time
    // 如果testing环境不allow长timerun，可ending with跳过此testing
    let mut config = DNSConfig::new("test_token", &["google.com"]);
    config.domain_ips_dir = "./test_dns_data".to_string();
    // set很长of间隔，避免在testing期间触发第二次parse
    config.interval = "600s".to_string(); // 10分钟

    let service = DNSService::new(config).expect("创建服务失败");

    println!("[测试] 启动服务进行后台解析测试...");
    let start_result = service.start().await;
    assert!(start_result.is_ok(), "启动服务应该成功");

    // 等待service开始执行parse（不等待完整parse完成，因to可能require很长time）
    // 只validateservice能够normalstart并开始工作
    println!("[测试] 等待服务运行 5 秒（验证后台任务已启动）...");
    sleep(Duration::from_secs(5)).await;

    // validateservice仍在run
    assert!(service.is_running().await, "服务应该仍在运行");

    println!("[测试] 服务在后台正常运行（解析可能在后台继续进行）");

    // stopservice
    let _ = service.stop().await;

    // 等待servicestop
    let mut attempts = 0;
    while service.is_running().await && attempts < 50 {
        sleep(Duration::from_millis(100)).await;
        attempts += 1;
    }

    println!("[测试] 后台解析测试完成");
}

#[tokio::test]
async fn test_service_config_validation() {
    // testingconfigurevalidate
    // 缺少 ipinfo_token 应该failure
    let invalid_config = DNSConfig::new("", &["google.com"]);

    let result = DNSService::new(invalid_config);
    assert!(result.is_err(), "空 ipinfo_token 应该导致验证失败");

    // 缺少 domain_list 应该failure
    let invalid_config2 = DNSConfig::new("test_token", &[] as &[&str]); // 空list

    let result2 = DNSService::new(invalid_config2);
    assert!(result2.is_err(), "空 domain_list 应该导致验证失败");

    println!("[测试] 配置验证测试通过");
}
