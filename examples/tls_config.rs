//! TLS 配置使用示例
//!
//! 展示如何使用真实的 TLS Client Hello 配置和 HTTP/2 Settings

use fingerprint::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== TLS 指纹配置示例 ===\n");

    // 1. 获取指纹配置
    println!("1. 获取 Chrome 133 指纹配置：");
    let profile = mapped_tls_clients().get("chrome_133").unwrap();
    
    // 2. 获取 TLS Client Hello Spec（真正的 TLS 指纹配置）
    println!("2. 获取 TLS Client Hello Spec：");
    let client_hello_spec = profile.get_client_hello_spec()?;
    println!("   密码套件数量: {}", client_hello_spec.cipher_suites.len());
    println!("   扩展数量: {}", client_hello_spec.extensions.len());
    println!("   TLS 版本范围: {}-{}", client_hello_spec.tls_vers_min, client_hello_spec.tls_vers_max);

    // 3. 获取 HTTP/2 Settings
    println!("\n3. HTTP/2 Settings：");
    let settings = profile.get_settings();
    for (id, value) in settings.iter() {
        println!("   Setting {}: {}", id, value);
    }

    // 4. 获取 Pseudo Header Order
    println!("\n4. Pseudo Header Order：");
    let order = profile.get_pseudo_header_order();
    for (i, header) in order.iter().enumerate() {
        println!("   {}: {}", i + 1, header);
    }

    // 5. 比较不同浏览器的配置
    println!("\n5. 比较不同浏览器的配置：");
    
    let chrome_profile = mapped_tls_clients().get("chrome_133").unwrap();
    let firefox_profile = mapped_tls_clients().get("firefox_133").unwrap();
    let safari_profile = mapped_tls_clients().get("safari_16_0").unwrap();

    println!("   Chrome Pseudo Header Order: {:?}", chrome_profile.get_pseudo_header_order());
    println!("   Firefox Pseudo Header Order: {:?}", firefox_profile.get_pseudo_header_order());
    println!("   Safari Pseudo Header Order: {:?}", safari_profile.get_pseudo_header_order());

    // 6. 展示如何使用 ClientHelloSpec
    println!("\n6. ClientHelloSpec 详细信息：");
    let chrome_spec = chrome_profile.get_client_hello_spec()?;
    println!("   TLS 版本范围: {}-{}", chrome_spec.tls_vers_min, chrome_spec.tls_vers_max);
    println!("   前5个密码套件: {:?}", &chrome_spec.cipher_suites[..chrome_spec.cipher_suites.len().min(5)]);
    println!("   扩展数量: {}", chrome_spec.extensions.len());
    println!("   压缩方法: {:?}", chrome_spec.compression_methods);

    // 7. HTTP/2 Header Priority
    println!("\n7. HTTP/2 Header Priority：");
    if let Some(priority) = chrome_profile.get_header_priority() {
        println!("   Weight: {}", priority.weight);
        println!("   Stream Dependency: {}", priority.stream_dependency);
        println!("   Exclusive: {}", priority.exclusive);
    }

    println!("\n✅ 所有配置已成功获取！");
    println!("\n注意：这些配置可以用于：");
    println!("  - 配置 TLS 客户端库（如 rustls）的 Client Hello");
    println!("  - 配置 HTTP/2 客户端的 Settings 和 Header Order");
    println!("  - 实现真实的浏览器指纹模拟");

    Ok(())
}
