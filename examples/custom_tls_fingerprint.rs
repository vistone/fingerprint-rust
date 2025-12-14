//! 自定义 TLS 指纹示例
//!
//! 演示如何使用我们自己的 TLS 指纹库生成 ClientHello
//! 不依赖 rustls/native-tls

use fingerprint::{mapped_tls_clients, tls_handshake::TLSHandshakeBuilder};

fn main() {
    println!("\n╔══════════════════════════════════════════════════════════╗");
    println!("║           自定义 TLS 指纹生成示例                       ║");
    println!("║        使用我们自己的指纹库，不依赖外部 TLS 库           ║");
    println!("╚══════════════════════════════════════════════════════════╝\n");

    // 获取所有浏览器配置
    let profiles = mapped_tls_clients();
    println!("📚 可用浏览器指纹: {} 个\n", profiles.len());

    // 示例 1: 生成 Chrome 133 的 ClientHello
    println!("🔍 示例 1: 生成 Chrome 133 ClientHello\n");
    if let Some(chrome) = profiles.get("chrome_133") {
        match chrome.get_client_hello_spec() {
            Ok(spec) => {
                println!("  ClientHelloSpec:");
                println!("    - 密码套件: {}", spec.cipher_suites.len());
                println!("    - 扩展: {}", spec.extensions.len());
                println!(
                    "    - TLS 版本: 0x{:04x} - 0x{:04x}",
                    spec.tls_vers_min, spec.tls_vers_max
                );

                // 构建 ClientHello
                match TLSHandshakeBuilder::build_client_hello(&spec, "www.google.com") {
                    Ok(bytes) => {
                        println!("\n  ✅ ClientHello 生成成功！");
                        println!("    - 总大小: {} bytes", bytes.len());
                        println!("    - 前 10 bytes: {:02x?}", &bytes[..10.min(bytes.len())]);

                        // 验证 TLS 记录格式
                        println!("\n  TLS 记录格式:");
                        println!("    - 类型: {} (Handshake)", bytes[0]);
                        println!(
                            "    - 版本: 0x{:02x}{:02x} (TLS 1.0 for compatibility)",
                            bytes[1], bytes[2]
                        );
                        let length = u16::from_be_bytes([bytes[3], bytes[4]]);
                        println!("    - 长度: {} bytes", length);
                    }
                    Err(e) => {
                        println!("  ❌ ClientHello 生成失败: {}", e);
                    }
                }
            }
            Err(e) => {
                println!("  ❌ ClientHelloSpec 生成失败: {}", e);
            }
        }
    }

    // 示例 2: 生成 Firefox 133 的 ClientHello
    println!("\n\n🔍 示例 2: 生成 Firefox 133 ClientHello\n");
    if let Some(firefox) = profiles.get("firefox_133") {
        if let Ok(spec) = firefox.get_client_hello_spec() {
            if let Ok(bytes) = TLSHandshakeBuilder::build_client_hello(&spec, "www.mozilla.org") {
                println!("  ✅ Firefox ClientHello: {} bytes", bytes.len());
            }
        }
    }

    // 示例 3: 生成 Safari 18.2 的 ClientHello
    println!("\n🔍 示例 3: 生成 Safari iOS 18.0 ClientHello\n");
    if let Some(safari) = profiles.get("safari_ios_18_0") {
        if let Ok(spec) = safari.get_client_hello_spec() {
            if let Ok(bytes) = TLSHandshakeBuilder::build_client_hello(&spec, "www.apple.com") {
                println!("  ✅ Safari ClientHello: {} bytes", bytes.len());
            }
        }
    }

    // 示例 4: 对比不同浏览器的 ClientHello 大小
    println!("\n\n📊 示例 4: 对比不同浏览器的 ClientHello 大小\n");
    let browsers_to_compare = vec!["chrome_133", "firefox_133", "safari_ios_18_0", "opera_91"];

    for browser_name in browsers_to_compare {
        if let Some(profile) = profiles.get(browser_name) {
            if let Ok(spec) = profile.get_client_hello_spec() {
                if let Ok(bytes) = TLSHandshakeBuilder::build_client_hello(&spec, "example.com") {
                    println!(
                        "  {:20} : {:3} bytes ({} 密码套件, {} 扩展)",
                        browser_name,
                        bytes.len(),
                        spec.cipher_suites.len(),
                        spec.extensions.len()
                    );
                }
            }
        }
    }

    // 示例 5: 使用调试模式查看详细信息
    println!("\n\n🔍 示例 5: 使用调试模式构建 ClientHello\n");
    if let Some(chrome) = profiles.get("chrome_133") {
        if let Ok(spec) = chrome.get_client_hello_spec() {
            let _ = TLSHandshakeBuilder::build_with_debug(&spec, "www.google.com");
        }
    }

    println!("\n✅ 所有示例执行完成！\n");
    println!("💡 关键要点:");
    println!("   1. 我们完全使用自己的 TLS 指纹库");
    println!("   2. 不依赖 rustls/native-tls");
    println!("   3. 生成的 ClientHello 符合 TLS 标准");
    println!("   4. 支持 66 种不同的浏览器指纹\n");
}
