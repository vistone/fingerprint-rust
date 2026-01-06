//! TLS 指纹示例
//!
//! 展示如何使用 TLS 指纹配置和生成 ClientHello
//!
//! 运行方式:
//! ```bash
//! cargo run --example tls_fingerprint
//! ```

use fingerprint::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("╔══════════════════════════════════════════════════════════╗");
    println!("║            TLS 指纹配置和使用示例                       ║");
    println!("╚══════════════════════════════════════════════════════════╝\n");

    let profiles = mapped_tls_clients();
    println!("📚 可用浏览器指纹: {} 个\n", profiles.len());

    // ========================================================================
    // 1. 获取 TLS Client Hello Spec
    // ========================================================================
    println!("1️⃣  获取 TLS Client Hello Spec\n");
    let chrome = profiles.get("chrome_133").unwrap();
    let spec = chrome.get_client_hello_spec()?;

    println!("   Chrome 133 配置:");
    println!("     - 密码套件: {}", spec.cipher_suites.len());
    println!("     - 扩展: {}", spec.extensions.len());
    println!("     - TLS 版本: 0x{:04x} - 0x{:04x}", spec.tls_vers_min, spec.tls_vers_max);
    println!("     - 压缩方法: {:?}", spec.compression_methods);

    // ========================================================================
    // 2. 生成 ClientHello
    // ========================================================================
    println!("\n2️⃣  生成 ClientHello\n");
    #[cfg(feature = "crypto")]
    {
        match TLSHandshakeBuilder::build_client_hello(&spec, "www.google.com") {
            Ok(bytes) => {
                println!("   ✅ ClientHello 生成成功！");
                println!("     - 总大小: {} bytes", bytes.len());
                println!("     - 前 10 bytes: {:02x?}", &bytes[..10.min(bytes.len())]);

                // 验证 TLS 记录格式
                println!("\n   TLS 记录格式:");
                println!("     - 类型: {} (Handshake)", bytes[0]);
                println!("     - 版本: 0x{:02x}{:02x}", bytes[1], bytes[2]);
                let length = u16::from_be_bytes([bytes[3], bytes[4]]);
                println!("     - 长度: {} bytes", length);
            }
            Err(e) => {
                println!("   ❌ ClientHello 生成失败: {}", e);
            }
        }
    }
    #[cfg(not(feature = "crypto"))]
    {
        println!("   ⚠️  需要启用 crypto feature 才能生成 ClientHello");
    }

    // ========================================================================
    // 3. HTTP/2 Settings
    // ========================================================================
    println!("\n3️⃣  HTTP/2 Settings\n");
    let settings = chrome.get_settings();
    println!("   Settings 数量: {}", settings.len());
    for (id, value) in settings.iter().take(5) {
        println!("     - Setting {}: {}", id, value);
    }

    // ========================================================================
    // 4. Pseudo Header Order
    // ========================================================================
    println!("\n4️⃣  Pseudo Header Order\n");
    let order = chrome.get_pseudo_header_order();
    println!("   Chrome 顺序: {:?}", order);

    let firefox = profiles.get("firefox_133").unwrap();
    let firefox_order = firefox.get_pseudo_header_order();
    println!("   Firefox 顺序: {:?}", firefox_order);

    let safari = profiles.get("safari_16_0").unwrap();
    let safari_order = safari.get_pseudo_header_order();
    println!("   Safari 顺序: {:?}", safari_order);

    // ========================================================================
    // 5. 对比不同浏览器的配置
    // ========================================================================
    println!("\n5️⃣  对比不同浏览器的 ClientHello 大小\n");
    let browsers = vec!["chrome_133", "firefox_133", "safari_ios_18_0", "opera_91"];

    for browser_name in browsers {
        if let Some(profile) = profiles.get(browser_name) {
            if let Ok(spec) = profile.get_client_hello_spec() {
                #[cfg(feature = "crypto")]
                {
                    if let Ok(bytes) = TLSHandshakeBuilder::build_client_hello(&spec, "example.com") {
                        println!(
                            "   {:20} : {:3} bytes ({} 密码套件, {} 扩展)",
                            browser_name,
                            bytes.len(),
                            spec.cipher_suites.len(),
                            spec.extensions.len()
                        );
                    }
                }
                #[cfg(not(feature = "crypto"))]
                {
                    println!(
                        "   {:20} : {} 密码套件, {} 扩展",
                        browser_name,
                        spec.cipher_suites.len(),
                        spec.extensions.len()
                    );
                }
            }
        }
    }

    // ========================================================================
    // 6. HTTP/2 Header Priority
    // ========================================================================
    println!("\n6️⃣  HTTP/2 Header Priority\n");
    if let Some(priority) = chrome.get_header_priority() {
        println!("   Chrome Priority:");
        println!("     - Weight: {}", priority.weight);
        println!("     - Stream Dependency: {}", priority.stream_dependency);
        println!("     - Exclusive: {}", priority.exclusive);
    }

    println!("\n✅ 所有示例执行完成！\n");
    println!("💡 关键要点:");
    println!("   1. 使用自己的 TLS 指纹库，不依赖外部 TLS 库");
    println!("   2. 生成的 ClientHello 符合 TLS 标准");
    println!("   3. 支持 66 种不同的浏览器指纹");
    println!("   4. 包含 HTTP/2 Settings 和 Header Order 配置\n");

    Ok(())
}
