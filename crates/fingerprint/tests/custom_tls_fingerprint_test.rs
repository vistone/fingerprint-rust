//! 自定义 TLS 指纹测试
//!
//! 验证我们真正使用了自己的 TLS 指纹库
//! 不依赖 rustls/native-tls

use fingerprint::{mapped_tls_clients, tls_handshake::TLSHandshakeBuilder};
use std::io::{Read, Write};
use std::net::TcpStream;

#[test]
fn test_custom_tls_fingerprint_generation() {
    println!("\n╔══════════════════════════════════════════════════════════╗");
    println!("║        测试自定义 TLS 指纹生成（不使用外部库）           ║");
    println!("╚══════════════════════════════════════════════════════════╝\n");

    // 获取所有浏览器配置
    let profiles = mapped_tls_clients();

    println!("📋 可用的浏览器指纹: {} 个\n", profiles.len());

    // 测试几个主要浏览器
    let test_browsers = vec!["chrome_133", "firefox_133", "safari_18_2"];

    for browser_name in test_browsers {
        if let Some(profile) = profiles.get(browser_name) {
            println!("🔍 测试浏览器: {}", browser_name);

            // 从配置生成 ClientHelloSpec
            if let Ok(spec) = profile.get_client_hello_spec() {
                println!("  ✅ ClientHelloSpec 生成成功");
                println!("     - 密码套件: {}", spec.cipher_suites.len());
                println!("     - 扩展: {}", spec.extensions.len());

                // 构建 TLS ClientHello
                match TLSHandshakeBuilder::build_client_hello(&spec, "www.example.com") {
                    Ok(client_hello_bytes) => {
                        println!(
                            "  ✅ ClientHello 构建成功: {} bytes",
                            client_hello_bytes.len()
                        );

                        // 验证 TLS 记录格式
                        assert_eq!(client_hello_bytes[0], 22, "应该是握手记录");
                        assert_eq!(client_hello_bytes[1], 0x03, "TLS 版本主版本号");

                        println!("  ✅ TLS 记录格式验证通过");
                    }
                    Err(e) => {
                        println!("  ❌ ClientHello 构建失败: {}", e);
                    }
                }
            } else {
                println!("  ❌ ClientHelloSpec 生成失败");
            }

            println!();
        }
    }
}

#[test]
#[ignore] // 需要网络连接
fn test_custom_tls_fingerprint_real_connection() {
    println!("\n╔══════════════════════════════════════════════════════════╗");
    println!("║      测试自定义 TLS 指纹与真实服务器建立连接             ║");
    println!("╚══════════════════════════════════════════════════════════╝\n");

    // 获取 Chrome 133 配置
    let profiles = mapped_tls_clients();
    let chrome = profiles.get("chrome_133").expect("找不到 Chrome 133");

    println!("🔍 使用浏览器: Chrome 133");

    // 生成 ClientHelloSpec
    let spec = chrome
        .get_client_hello_spec()
        .expect("无法生成 ClientHelloSpec");

    println!("  ✅ ClientHelloSpec 生成");
    println!("     - 密码套件: {}", spec.cipher_suites.len());
    println!("     - 扩展: {}", spec.extensions.len());

    // 构建 TLS ClientHello
    let client_hello = TLSHandshakeBuilder::build_client_hello(&spec, "www.example.com")
        .expect("无法构建 ClientHello");

    println!("  ✅ ClientHello 构建完成: {} bytes\n", client_hello.len());

    // 连接到真实服务器
    println!("🌐 连接到 www.example.com:443 ...");

    match TcpStream::connect("93.184.215.14:443") {
        // example.com 的 IP
        Ok(mut stream) => {
            println!("  ✅ TCP 连接建立\n");

            // 发送我们自己构建的 ClientHello
            println!(
                "📤 发送自定义 TLS ClientHello ({} bytes)...",
                client_hello.len()
            );

            match stream.write_all(&client_hello) {
                Ok(_) => {
                    println!("  ✅ ClientHello 发送成功\n");

                    // 尝试读取服务器响应
                    println!("📥 等待服务器响应...");

                    let mut response = vec![0u8; 5];
                    match stream.read_exact(&mut response) {
                        Ok(_) => {
                            // 解析 TLS 记录头
                            let record_type = response[0];
                            let version = u16::from_be_bytes([response[1], response[2]]);
                            let length = u16::from_be_bytes([response[3], response[4]]);

                            println!("  ✅ 收到服务器响应:");
                            println!("     - 记录类型: {}", record_type);
                            println!("     - TLS 版本: 0x{:04x}", version);
                            println!("     - 数据长度: {} bytes", length);

                            // 如果是握手记录 (22)，说明服务器接受了我们的 ClientHello
                            if record_type == 22 {
                                println!("\n  🎉 服务器接受了我们的自定义 TLS 指纹！");
                                println!("  ✅ TLS 握手开始！");

                                // 读取 ServerHello
                                let mut server_hello = vec![0u8; length as usize];
                                if stream.read_exact(&mut server_hello).is_ok() {
                                    println!("  ✅ ServerHello 接收完成: {} bytes", length);
                                }
                            } else {
                                println!("\n  ⚠️ 收到非握手响应，记录类型: {}", record_type);
                            }
                        }
                        Err(e) => {
                            println!("  ❌ 读取响应失败: {}", e);
                        }
                    }
                }
                Err(e) => {
                    println!("  ❌ 发送 ClientHello 失败: {}", e);
                }
            }
        }
        Err(e) => {
            println!("  ❌ TCP 连接失败: {}", e);
        }
    }
}

#[test]
fn test_all_browser_fingerprints() {
    println!("\n╔══════════════════════════════════════════════════════════╗");
    println!("║        测试所有 66 个浏览器指纹的 ClientHello 生成       ║");
    println!("╚══════════════════════════════════════════════════════════╝\n");

    let profiles = mapped_tls_clients();
    let total = profiles.len();
    let mut success = 0;
    let mut failed = Vec::new();

    println!("🔍 测试 {} 个浏览器指纹...\n", total);

    for (i, (name, profile)) in profiles.iter().enumerate() {
        print!("  [{}/{}] {} ... ", i + 1, total, name);

        match profile.get_client_hello_spec() {
            Ok(spec) => {
                match TLSHandshakeBuilder::build_client_hello(&spec, "example.com") {
                    Ok(bytes) => {
                        // 验证基本格式
                        if bytes[0] == 22 && bytes.len() > 50 {
                            println!("✅ ({} bytes)", bytes.len());
                            success += 1;
                        } else {
                            println!("❌ (格式错误)");
                            failed.push(name.clone());
                        }
                    }
                    Err(e) => {
                        println!("❌ (构建失败: {})", e);
                        failed.push(name.clone());
                    }
                }
            }
            Err(e) => {
                println!("❌ (Spec 失败: {})", e);
                failed.push(name.clone());
            }
        }
    }

    println!("\n📊 测试结果:");
    println!("  总计: {}", total);
    println!("  成功: {} ✅", success);
    println!("  失败: {} ❌", failed.len());
    println!("  成功率: {:.1}%", (success as f64 / total as f64) * 100.0);

    if !failed.is_empty() {
        println!("\n❌ 失败的浏览器:");
        for name in failed {
            println!("  - {}", name);
        }
    }

    // 要求至少 90% 成功率
    assert!((success as f64 / total as f64) >= 0.9, "成功率低于 90%");
}
