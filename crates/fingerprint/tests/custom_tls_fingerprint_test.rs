// ! custom TLS fingerprinttesting
//! Custom TLS fingerprint tests.
// ! validate我们真正use了自己of TLS fingerprintlibrary
// ! 不依赖 rustls/native-tls

use fingerprint::{mapped_tls_clients, tls_handshake::TLSHandshakeBuilder};
use std::io::{Read, Write};
use std::net::TcpStream;

#[test]
fn test_custom_tls_fingerprint_generation() {
    println!("\n╔══════════════════════════════════════════════════════════╗");
    println!("║        测试自定义 TLS 指纹生成（不使用外部库）           ║");
    println!("╚══════════════════════════════════════════════════════════╝\n");

    // getall浏览器configure
    let profiles = mapped_tls_clients();

    println!("📋 可用的浏览器指纹: {} 个\n", profiles.len());

    // testing几个主要浏览器
    let test_browsers = vec!["chrome_133", "firefox_133", "safari_18_2"];

    for browser_name in test_browsers {
        if let Some(profile) = profiles.get(browser_name) {
            println!("🔍 测试浏览器: {}", browser_name);

            let spec = &profile.tls_config;
            println!("  ✅ ClientHelloSpec 生成成功");
            println!("     - 密码套件: {}", spec.cipher_suites.len());
            println!("     - 扩展: {}", spec.extensions.len());

            // build TLS ClientHello
            match TLSHandshakeBuilder::build_client_hello(spec, "www.example.com") {
                Ok(client_hello_bytes) => {
                    println!(
                        "  ✅ ClientHello 构建成功: {} bytes",
                        client_hello_bytes.len()
                    );

                    // validate TLS record格式
                    assert_eq!(client_hello_bytes[0], 22, "应该是握手记录");
                    assert_eq!(client_hello_bytes[1], 0x03, "TLS 版本主版本号");

                    println!("  ✅ TLS 记录格式验证通过");
                }
                Err(e) => {
                    println!("  ❌ ClientHello 构建失败: {}", e);
                }
            }

            println!();
        }
    }
}

#[test]
#[ignore] // requirenetworkconnect
fn test_custom_tls_fingerprint_real_connection() {
    println!("\n╔══════════════════════════════════════════════════════════╗");
    println!("║      测试自定义 TLS 指纹与真实服务器建立连接             ║");
    println!("╚══════════════════════════════════════════════════════════╝\n");

    // get Chrome 133 configure
    let profiles = mapped_tls_clients();
    let chrome = profiles.get("chrome_133").expect("找不到 Chrome 133");

    println!("🔍 使用浏览器: Chrome 133");

    // generate ClientHelloSpec
    let spec = &chrome.tls_config;

    println!("  ✅ ClientHelloSpec 生成");
    println!("     - 密码套件: {}", spec.cipher_suites.len());
    println!("     - 扩展: {}", spec.extensions.len());

    // build TLS ClientHello
    let client_hello = TLSHandshakeBuilder::build_client_hello(spec, "www.example.com")
        .expect("无法构建 ClientHello");

    println!("  ✅ ClientHello 构建完成: {} bytes\n", client_hello.len());

    // connect到真实service器
    println!("🌐 连接到 www.example.com:443 ...");

    match TcpStream::connect("93.184.215.14:443") {
        // example.com of IP
        Ok(mut stream) => {
            println!("  ✅ TCP 连接建立\n");

            // send我们自己buildof ClientHello
            println!(
                "📤 发送自定义 TLS ClientHello ({} bytes)...",
                client_hello.len()
            );

            match stream.write_all(&client_hello) {
                Ok(_) => {
                    println!("  ✅ ClientHello 发送成功\n");

                    // 尝试读取service器响应
                    println!("📥 等待服务器响应...");

                    let mut response = vec![0u8; 5];
                    match stream.read_exact(&mut response) {
                        Ok(_) => {
                            // parse TLS record头
                            let record_type = response[0];
                            let version = u16::from_be_bytes([response[1], response[2]]);
                            let length = u16::from_be_bytes([response[3], response[4]]);

                            println!("  ✅ 收到服务器响应:");
                            println!("     - 记录类型: {}", record_type);
                            println!("     - TLS 版本: 0x{:04x}", version);
                            println!("     - 数据长度: {} bytes", length);

                            // 如果是握手record (22)，descriptionservice器接受了我们of ClientHello
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

        let spec = &profile.tls_config;
        match TLSHandshakeBuilder::build_client_hello(spec, "example.com") {
            Ok(bytes) => {
                // validate基本格式
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

    // 要求至少 90% success率
    assert!((success as f64 / total as f64) >= 0.9, "成功率低于 90%");
}
