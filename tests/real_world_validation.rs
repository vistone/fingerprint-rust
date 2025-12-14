//! 真实世界验证测试
//!
//! 这些测试需要访问真实的网站来验证指纹的有效性
//! 运行方式: cargo test --test real_world_validation -- --ignored --test-threads=1
//!
//! ⚠️ 警告：这些测试会访问互联网，可能需要较长时间

use fingerprint::*;

/// 测试说明
///
/// 这些测试标记为 #[ignore]，因为它们：
/// 1. 需要网络连接
/// 2. 依赖外部服务
/// 3. 运行时间较长
/// 4. 可能因网络问题失败
///
/// 运行方式：
/// ```bash
/// cargo test --test real_world_validation -- --ignored
/// ```

#[test]
#[ignore]
fn test_against_tls_fingerprint_api() {
    // TODO: 实现真实的 TLS 指纹验证
    //
    // 步骤：
    // 1. 使用这个库生成一个 Chrome 133 指纹
    // 2. 使用该指纹访问 https://tls.peet.ws/api/all
    // 3. 获取服务器返回的 JA4 指纹
    // 4. 对比与预期的 Chrome 133 JA4 指纹是否一致
    //
    // 预期结果：
    // - JA4 指纹应该与真实 Chrome 133 相似
    // - TLS 版本、密码套件等应该匹配
    
    let result = get_random_fingerprint_by_browser("chrome").unwrap();
    println!("生成的指纹: {}", result.hello_client_id);
    println!("User-Agent: {}", result.user_agent);
    
    // TODO: 需要实际的 HTTP 客户端来发送请求
    // 推荐使用: reqwest 或 hyper
    
    todo!("需要实现实际的 HTTP 请求和指纹验证");
}

#[test]
#[ignore]
fn test_chrome_fingerprint_authenticity() {
    // TODO: 验证 Chrome 指纹的真实性
    //
    // 对比项：
    // 1. TLS ClientHello 的字节序列
    // 2. 密码套件顺序
    // 3. 扩展顺序和内容
    // 4. HTTP/2 Settings
    // 5. Pseudo Header Order
    
    let profile = mapped_tls_clients().get("chrome_133").unwrap();
    let spec = profile.get_client_hello_spec().unwrap();
    
    println!("Chrome 133 配置:");
    println!("  密码套件数量: {}", spec.cipher_suites.len());
    println!("  扩展数量: {}", spec.extensions.len());
    
    // TODO: 对比真实 Chrome 133 的配置
    todo!("需要真实 Chrome 133 的指纹数据进行对比");
}

#[test]
#[ignore]
fn test_firefox_fingerprint_authenticity() {
    // TODO: 验证 Firefox 指纹的真实性
    let profile = mapped_tls_clients().get("firefox_133").unwrap();
    let spec = profile.get_client_hello_spec().unwrap();
    
    println!("Firefox 133 配置:");
    println!("  密码套件数量: {}", spec.cipher_suites.len());
    println!("  扩展数量: {}", spec.extensions.len());
    
    todo!("需要真实 Firefox 133 的指纹数据进行对比");
}

#[test]
#[ignore]
fn test_against_cloudflare_protection() {
    // TODO: 测试是否能绕过 Cloudflare 的机器人检测
    //
    // 测试网站：找一个有 Cloudflare 保护的网站
    // 预期：使用正确的指纹应该能正常访问
    
    todo!("需要实际访问受 Cloudflare 保护的网站");
}

#[test]
#[ignore]
fn test_ja4_fingerprint_matches_real_browser() {
    // TODO: 验证生成的 JA4 指纹是否与真实浏览器匹配
    //
    // 步骤：
    // 1. 收集真实 Chrome/Firefox 的 JA4 指纹
    // 2. 使用这个库生成对应的 JA4 指纹
    // 3. 对比两者是否一致
    
    let profile = mapped_tls_clients().get("chrome_133").unwrap();
    let spec = profile.get_client_hello_spec().unwrap();
    let signature = extract_signature(&spec);
    
    // 创建 JA4 签名
    let ja4_sig = Ja4Signature {
        version: signature.version,
        cipher_suites: signature.cipher_suites,
        extensions: signature.extensions,
        signature_algorithms: signature.signature_algorithms,
        sni: Some("example.com".to_string()),
        alpn: Some("h2".to_string()),
    };
    
    let ja4 = ja4_sig.generate_ja4();
    println!("生成的 JA4: {}", ja4.full.value());
    
    // TODO: 对比真实 Chrome 133 的 JA4 指纹
    // 真实 Chrome 133 的 JA4 指纹应该从实际浏览器中获取
    
    todo!("需要真实浏览器的 JA4 指纹数据");
}

#[test]
#[ignore]
fn test_http2_settings_match_real_browser() {
    // TODO: 验证 HTTP/2 Settings 是否与真实浏览器匹配
    //
    // 对比项：
    // 1. Settings 的值
    // 2. Settings 的顺序
    // 3. Pseudo Header Order
    // 4. Connection Flow
    
    let profile = mapped_tls_clients().get("chrome_133").unwrap();
    let settings = profile.get_settings();
    let order = profile.get_pseudo_header_order();
    
    println!("Chrome 133 HTTP/2 配置:");
    println!("  Settings: {:?}", settings);
    println!("  Pseudo Header Order: {:?}", order);
    
    // TODO: 对比真实 Chrome 的 HTTP/2 配置
    todo!("需要从真实浏览器抓包获取 HTTP/2 配置");
}

/// 验证建议
///
/// 为了真正验证这个库的有效性，建议：
///
/// 1. **使用 Wireshark 抓包**
///    - 抓取真实 Chrome/Firefox 的 TLS ClientHello
///    - 抓取使用这个库的 TLS ClientHello
///    - 对比两者的差异
///
/// 2. **使用指纹检测服务**
///    - https://tls.peet.ws/api/all
///    - https://kawayiyi.com/tls
///    - https://ja3er.com/
///    - https://browserleaks.com/ssl
///
/// 3. **测试反爬虫系统**
///    - 访问有反爬虫保护的网站
///    - 验证是否被识别为机器人
///    - 对比使用真实浏览器的结果
///
/// 4. **收集真实数据**
///    - 从 https://github.com/refraction-networking/utls 获取参考
///    - 从 https://github.com/biandratti/huginn-net 获取参考
///    - 自己抓取真实浏览器的数据
#[test]
fn test_validation_recommendations() {
    println!("\n=== 真实验证建议 ===\n");
    println!("1. 使用 Wireshark 抓包对比 TLS ClientHello");
    println!("2. 访问 TLS 指纹检测服务");
    println!("3. 测试反爬虫系统");
    println!("4. 收集真实浏览器数据");
    println!("\n详细说明请参考函数文档注释");
}
