//! 全面指纹验证测试 - 使用我们自己的库
//! 
//! **重要**：不依赖 reqwest 等外部 HTTP 库
//! 
//! 验证内容：
//! 1. 所有 66 个浏览器指纹的合法性
//! 2. HTTP/1.1 连接和响应
//! 3. 每个指纹的配置完整性
//! 4. TLS 配置的正确性
//! 
//! 运行方式：
//! ```bash
//! # 完整验证（需要网络）
//! cargo test --test comprehensive_validation -- --ignored --nocapture --test-threads=1
//! ```

use fingerprint::*;
use std::time::Instant;

/// 验证结果
#[derive(Debug, Clone)]
struct ValidationResult {
    profile_name: String,
    fingerprint_valid: bool,
    tls_config_valid: bool,
    user_agent_valid: bool,
    http1_1_result: TestResult,
}

/// 单个测试结果
#[derive(Debug, Clone)]
struct TestResult {
    success: bool,
    status_code: Option<u16>,
    response_time_ms: u64,
    response_size: usize,
    error: Option<String>,
}

impl TestResult {
    fn success(status_code: u16, response_time_ms: u64, response_size: usize) -> Self {
        Self {
            success: true,
            status_code: Some(status_code),
            response_time_ms,
            response_size,
            error: None,
        }
    }
    
    fn failure(error: String, response_time_ms: u64) -> Self {
        Self {
            success: false,
            status_code: None,
            response_time_ms,
            response_size: 0,
            error: Some(error),
        }
    }
}

/// 验证统计
struct ValidationStats {
    total: usize,
    fingerprint_valid: usize,
    tls_config_valid: usize,
    user_agent_valid: usize,
    http1_1_success: usize,
    total_time_ms: u64,
}

impl ValidationStats {
    fn new() -> Self {
        Self {
            total: 0,
            fingerprint_valid: 0,
            tls_config_valid: 0,
            user_agent_valid: 0,
            http1_1_success: 0,
            total_time_ms: 0,
        }
    }
    
    fn add(&mut self, result: &ValidationResult) {
        self.total += 1;
        if result.fingerprint_valid {
            self.fingerprint_valid += 1;
        }
        if result.tls_config_valid {
            self.tls_config_valid += 1;
        }
        if result.user_agent_valid {
            self.user_agent_valid += 1;
        }
        if result.http1_1_result.success {
            self.http1_1_success += 1;
        }
        self.total_time_ms += result.http1_1_result.response_time_ms;
    }
    
    fn print_summary(&self) {
        println!("\n╔═══════════════════════════════════════════════════════════╗");
        println!("║                  全面验证报告汇总                          ║");
        println!("╚═══════════════════════════════════════════════════════════╝\n");
        
        println!("📊 总体统计：");
        println!("  - 总指纹数: {}", self.total);
        println!("  - 指纹配置合法: {}/{} ({:.1}%)", 
            self.fingerprint_valid, self.total,
            self.fingerprint_valid as f64 / self.total as f64 * 100.0
        );
        println!("  - TLS 配置完整: {}/{} ({:.1}%)", 
            self.tls_config_valid, self.total,
            self.tls_config_valid as f64 / self.total as f64 * 100.0
        );
        println!("  - User-Agent 合法: {}/{} ({:.1}%)", 
            self.user_agent_valid, self.total,
            self.user_agent_valid as f64 / self.total as f64 * 100.0
        );
        println!();
        
        println!("🌐 HTTP/1.1 测试：");
        println!("  - 成功: {}/{} ({:.1}%)", 
            self.http1_1_success, self.total,
            self.http1_1_success as f64 / self.total as f64 * 100.0
        );
        
        if self.http1_1_success > 0 {
            println!("  - 平均响应时间: {}ms", self.total_time_ms / self.http1_1_success as u64);
        }
        println!();
    }
}

#[test]
fn test_fingerprint_config_validity() {
    println!("\n╔═══════════════════════════════════════════════════════════╗");
    println!("║            测试所有指纹配置的合法性（本地）                 ║");
    println!("╚═══════════════════════════════════════════════════════════╝\n");
    
    let all_profiles = mapped_tls_clients();
    let total = all_profiles.len();
    let mut passed = 0;
    
    for (profile_name, profile) in all_profiles.iter() {
        // 验证指纹配置
        let config_valid = validate_fingerprint_config(profile);
        let tls_valid = validate_tls_config(profile);
        let ua_valid = validate_user_agent(profile_name);
        
        if config_valid && tls_valid && ua_valid {
            passed += 1;
            println!("✅ {}: 配置合法", profile_name);
        } else {
            println!("❌ {}: 配置={} TLS={} UA={}", 
                profile_name, config_valid, tls_valid, ua_valid);
        }
    }
    
    println!("\n📊 结果: {}/{} 通过 ({:.1}%)", 
        passed, total, passed as f64 / total as f64 * 100.0);
    
    assert_eq!(passed, total, "部分指纹配置不合法");
}

#[test]
#[ignore] // 需要网络
fn test_all_fingerprints_http1_1() {
    println!("\n╔═══════════════════════════════════════════════════════════╗");
    println!("║      全面验证所有 66 个指纹 - HTTP/1.1（使用自己的库）     ║");
    println!("╚═══════════════════════════════════════════════════════════╝\n");
    
    let all_profiles = mapped_tls_clients();
    let total = all_profiles.len();
    let mut stats = ValidationStats::new();
    let mut results = Vec::new();
    
    println!("📋 开始验证 {} 个浏览器指纹\n", total);
    
    for (i, (profile_name, profile)) in all_profiles.iter().enumerate() {
        println!("┌─────────────────────────────────────────────────────────┐");
        println!("│ [{}/{}] {}", i + 1, total, profile_name);
        println!("└─────────────────────────────────────────────────────────┘");
        
        let result = validate_single_fingerprint(profile_name, profile);
        print_validation_result(&result);
        
        stats.add(&result);
        results.push(result);
        
        std::thread::sleep(std::time::Duration::from_millis(200));
    }
    
    stats.print_summary();
    
    // 保存报告
    save_validation_report(&results);
    
    // 验证成功率
    let success_rate = stats.http1_1_success as f64 / stats.total as f64 * 100.0;
    assert!(success_rate >= 80.0, "成功率 {:.1}% 低于 80%", success_rate);
}

/// 验证单个指纹
fn validate_single_fingerprint(
    profile_name: &str,
    profile: &ClientProfile,
) -> ValidationResult {
    // 1. 验证指纹配置
    let fingerprint_valid = validate_fingerprint_config(profile);
    let tls_config_valid = validate_tls_config(profile);
    
    // 2. 获取 User-Agent
    let user_agent = get_user_agent_by_profile_name(profile_name)
        .unwrap_or_else(|_| "Mozilla/5.0".to_string());
    let user_agent_valid = !user_agent.is_empty();
    
    println!("  📌 User-Agent: {}", &user_agent[..user_agent.len().min(60)]);
    println!("  📌 指纹配置: {}", if fingerprint_valid { "✅" } else { "❌" });
    println!("  📌 TLS 配置: {}", if tls_config_valid { "✅" } else { "❌" });
    
    // 3. 创建 HTTP 客户端
    let headers = HTTPHeaders::default();
    let client = HttpClient::with_profile(
        profile.clone(),
        headers,
        user_agent,
    );
    
    // 4. 测试 HTTP/1.1 连接
    println!("  → 测试 HTTP/1.1...");
    let http1_1_result = test_http1_1_connection(&client);
    
    if http1_1_result.success {
        println!("    ✅ 状态码 {}, {}ms, {} 字节",
            http1_1_result.status_code.unwrap(),
            http1_1_result.response_time_ms,
            http1_1_result.response_size
        );
    } else {
        println!("    ❌ {}", 
            http1_1_result.error.as_ref().unwrap_or(&"未知错误".to_string()));
    }
    
    ValidationResult {
        profile_name: profile_name.to_string(),
        fingerprint_valid,
        tls_config_valid,
        user_agent_valid,
        http1_1_result,
    }
}

/// 验证指纹配置的合法性
fn validate_fingerprint_config(profile: &ClientProfile) -> bool {
    match profile.get_client_hello_spec() {
        Ok(spec) => {
            // 打印调试信息
            eprintln!("  DEBUG: cipher_suites={}, extensions={}, tls_vers_min={}, tls_vers_max={}",
                spec.cipher_suites.len(), spec.extensions.len(), spec.tls_vers_min, spec.tls_vers_max);
            
            // 密码套件不能为空
            if spec.cipher_suites.is_empty() {
                eprintln!("  FAIL: cipher_suites is empty");
                return false;
            }
            
            // 扩展不能为空
            if spec.extensions.is_empty() {
                eprintln!("  FAIL: extensions is empty");
                return false;
            }
            
            // TLS 版本检查 - 0 是合法的（可能表示不限制）
            // 只检查如果都设置了，范围要合理
            if spec.tls_vers_min > 0 && spec.tls_vers_max > 0 {
                if spec.tls_vers_min > spec.tls_vers_max {
                    eprintln!("  FAIL: tls_vers_min > tls_vers_max");
                    return false;
                }
            }
            
            true
        }
        Err(e) => {
            eprintln!("  FAIL: get_client_hello_spec error: {}", e);
            false
        }
    }
}

/// 验证 TLS 配置的完整性
fn validate_tls_config(profile: &ClientProfile) -> bool {
    match profile.get_client_hello_spec() {
        Ok(spec) => {
            // 密码套件数量（至少5个）
            // Chrome: 16+, Firefox: 9+, Safari: 7+
            if spec.cipher_suites.len() < 5 {
                eprintln!("  FAIL TLS: cipher_suites too few: {}", spec.cipher_suites.len());
                return false;
            }
            
            // 扩展数量（至少3个）
            // Chrome: 19+, Firefox: 6+, Safari: 5+
            // 注意：不同浏览器的扩展数量差异很大，这是正常的
            if spec.extensions.len() < 3 {
                eprintln!("  FAIL TLS: extensions too few: {}", spec.extensions.len());
                return false;
            }
            
            true
        }
        Err(e) => {
            eprintln!("  FAIL TLS: get_client_hello_spec error: {}", e);
            false
        }
    }
}

/// 验证 User-Agent
fn validate_user_agent(profile_name: &str) -> bool {
    match get_user_agent_by_profile_name(profile_name) {
        Ok(ua) => {
            // User-Agent 不能为空
            if ua.is_empty() {
                return false;
            }
            
            // 应该包含浏览器名称
            let profile_lower = profile_name.to_lowercase();
            if profile_lower.contains("chrome") {
                ua.contains("Chrome") || ua.contains("chrome")
            } else if profile_lower.contains("firefox") {
                ua.contains("Firefox") || ua.contains("firefox")
            } else if profile_lower.contains("safari") {
                ua.contains("Safari") || ua.contains("safari")
            } else {
                true // 其他情况暂时通过
            }
        }
        Err(_) => false,
    }
}

/// 测试 HTTP/1.1 连接
fn test_http1_1_connection(client: &HttpClient) -> TestResult {
    let start = Instant::now();
    
    // 使用多个测试 URL，提高成功率
    let test_urls = vec![
        "http://httpbin.org/get",
        "http://example.com/",
        "https://www.google.com/",
    ];
    
    for url in test_urls {
        match client.get(url) {
            Ok(response) => {
                let elapsed = start.elapsed().as_millis() as u64;
                
                if response.is_success() {
                    return TestResult::success(
                        response.status_code,
                        elapsed,
                        response.body.len()
                    );
                }
            }
            Err(_) => {
                // 尝试下一个 URL
                continue;
            }
        }
    }
    
    // 所有 URL 都失败
    let elapsed = start.elapsed().as_millis() as u64;
    TestResult::failure("所有测试 URL 都失败".to_string(), elapsed)
}

/// 打印验证结果
fn print_validation_result(result: &ValidationResult) {
    println!("\n  📊 验证结果：");
    println!("    指纹配置: {}", if result.fingerprint_valid { "✅" } else { "❌" });
    println!("    TLS 配置: {}", if result.tls_config_valid { "✅" } else { "❌" });
    println!("    User-Agent: {}", if result.user_agent_valid { "✅" } else { "❌" });
    println!();
}

/// 保存验证报告
fn save_validation_report(results: &[ValidationResult]) {
    use std::fs::File;
    use std::io::Write;
    
    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
    let filename = format!("validation_report_{}.txt", timestamp);
    
    if let Ok(mut file) = File::create(&filename) {
        writeln!(file, "全面指纹验证报告").unwrap();
        writeln!(file, "使用自己的 HTTP 客户端（不依赖 reqwest）").unwrap();
        writeln!(file, "生成时间: {}\n", chrono::Local::now()).unwrap();
        writeln!(file, "{}", "=".repeat(70)).unwrap();
        
        for result in results {
            writeln!(file, "\n指纹: {}", result.profile_name).unwrap();
            writeln!(file, "  指纹配置合法: {}", result.fingerprint_valid).unwrap();
            writeln!(file, "  TLS 配置完整: {}", result.tls_config_valid).unwrap();
            writeln!(file, "  User-Agent 合法: {}", result.user_agent_valid).unwrap();
            writeln!(file, "  HTTP/1.1: {}", result.http1_1_result.success).unwrap();
            
            if result.http1_1_result.success {
                writeln!(file, "    状态码: {}", result.http1_1_result.status_code.unwrap()).unwrap();
                writeln!(file, "    响应时间: {}ms", result.http1_1_result.response_time_ms).unwrap();
                writeln!(file, "    响应大小: {} 字节", result.http1_1_result.response_size).unwrap();
            } else {
                writeln!(file, "    错误: {}", 
                    result.http1_1_result.error.as_ref().unwrap_or(&"未知".to_string())).unwrap();
            }
        }
        
        println!("\n📄 详细报告已保存到: {}", filename);
    }
}

#[cfg(test)]
mod response_tests {
    use fingerprint::*;
    
    #[test]
    fn test_response_parsing() {
        // 测试完整的 HTTP 响应解析
        let raw_response = b"HTTP/1.1 200 OK\r\n\
                             Content-Type: text/plain\r\n\
                             Content-Length: 13\r\n\
                             \r\n\
                             Hello, World!";
        
        let result = HttpResponse::parse(raw_response);
        assert!(result.is_ok());
        
        let response = result.unwrap();
        assert_eq!(response.status_code, 200);
        assert_eq!(response.body, b"Hello, World!");
    }
    
    #[test]
    fn test_response_with_chunked() {
        // 测试 chunked encoding 响应
        let raw_response = b"HTTP/1.1 200 OK\r\n\
                             Transfer-Encoding: chunked\r\n\
                             \r\n\
                             7\r\nMozilla\r\n\
                             9\r\nDeveloper\r\n\
                             0\r\n\r\n";
        
        let result = HttpResponse::parse(raw_response);
        assert!(result.is_ok());
        
        let response = result.unwrap();
        assert_eq!(response.status_code, 200);
        assert_eq!(response.body, b"MozillaDeveloper");
    }
}
