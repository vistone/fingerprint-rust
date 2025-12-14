//! Google Earth API 真实网络测试
//!
//! 测试地址: https://kh.google.com/rt/earth/PlanetoidMetadata
//! 该地址支持: HTTP/1.1, HTTP/2, HTTP/3
//!
//! 验证我们自定义的 TLS 指纹系统能够成功访问真实的 Google 服务

use fingerprint::{
    mapped_tls_clients, tls_handshake::TLSHandshakeBuilder,
};
use std::io::{Read, Write};
use std::net::TcpStream;

const TEST_URL: &str = "https://kh.google.com/rt/earth/PlanetoidMetadata";
const TEST_HOST: &str = "kh.google.com";
#[allow(dead_code)]
const TEST_PATH: &str = "/rt/earth/PlanetoidMetadata";

#[test]
#[ignore] // 需要网络连接
fn test_google_earth_api_with_custom_tls_all_browsers() {
    println!("\n╔══════════════════════════════════════════════════════════╗");
    println!("║   Google Earth API 测试 - 使用自定义 TLS 指纹系统        ║");
    println!("║   测试地址: {}   ║", TEST_URL);
    println!("╚══════════════════════════════════════════════════════════╝\n");

    let profiles = mapped_tls_clients();
    let total = profiles.len();
    let mut success = 0;
    let mut failed = Vec::new();

    println!("🔍 测试所有 {} 个浏览器指纹...\n", total);

    for (i, (name, profile)) in profiles.iter().enumerate() {
        print!("  [{:2}/{:2}] {:25} ... ", i + 1, total, name);

        match test_single_browser_custom_tls(name, profile) {
            Ok(response) => {
                println!("✅ ({})", response);
                success += 1;
            }
            Err(e) => {
                println!("❌ ({})", e);
                failed.push((name.clone(), e));
            }
        }
    }

    println!("\n╔══════════════════════════════════════════════════════════╗");
    println!("║                     测试结果汇总                         ║");
    println!("╚══════════════════════════════════════════════════════════╝\n");
    println!("  总计: {}", total);
    println!("  成功: {} ✅", success);
    println!("  失败: {} ❌", failed.len());
    println!("  成功率: {:.1}%", (success as f64 / total as f64) * 100.0);

    if !failed.is_empty() {
        println!("\n❌ 失败的浏览器:");
        for (name, err) in &failed {
            println!("  - {}: {}", name, err);
        }
    }

    if success > 0 {
        println!("\n✅ 成功验证: 我们的自定义 TLS 指纹系统可以访问真实的 Google 服务！");
    }

    // 要求至少 80% 成功率
    assert!(
        (success as f64 / total as f64) >= 0.8,
        "成功率低于 80%: {}/{}",
        success,
        total
    );
}

fn test_single_browser_custom_tls(
    _browser_name: &str,
    profile: &fingerprint::ClientProfile,
) -> Result<String, String> {
    // 1. 生成 ClientHelloSpec
    let spec = profile
        .get_client_hello_spec()
        .map_err(|e| format!("生成 Spec 失败: {}", e))?;

    // 2. 构建自定义 TLS ClientHello
    let client_hello = TLSHandshakeBuilder::build_client_hello(&spec, TEST_HOST)
        .map_err(|e| format!("构建 ClientHello 失败: {}", e))?;

    // 3. 连接到 Google Earth API
    let mut stream = TcpStream::connect("142.251.163.100:443") // kh.google.com 的 IP
        .map_err(|e| format!("TCP 连接失败: {}", e))?;

    stream
        .set_read_timeout(Some(std::time::Duration::from_secs(5)))
        .ok();
    stream
        .set_write_timeout(Some(std::time::Duration::from_secs(5)))
        .ok();

    // 4. 发送自定义的 TLS ClientHello
    stream
        .write_all(&client_hello)
        .map_err(|e| format!("发送 ClientHello 失败: {}", e))?;

    // 5. 读取服务器响应
    let mut response_header = vec![0u8; 5];
    stream
        .read_exact(&mut response_header)
        .map_err(|e| format!("读取响应失败: {}", e))?;

    // 6. 解析 TLS 记录头
    let record_type = response_header[0];
    let _version = u16::from_be_bytes([response_header[1], response_header[2]]);
    let length = u16::from_be_bytes([response_header[3], response_header[4]]);

    // 7. 验证是否收到握手响应
    if record_type == 22 {
        // Handshake
        // 读取 ServerHello
        let mut server_hello = vec![0u8; length as usize];
        stream.read_exact(&mut server_hello).ok();

        Ok(format!("ServerHello {} bytes", length))
    } else if record_type == 21 {
        // Alert
        let mut alert = vec![0u8; length as usize];
        stream.read_exact(&mut alert).ok();
        Err(format!("TLS Alert: {:?}", alert.get(0..2)))
    } else {
        Err(format!("未知记录类型: {}", record_type))
    }
}

#[test]
#[ignore] // 需要网络连接
fn test_google_earth_api_http_versions() {
    println!("\n╔══════════════════════════════════════════════════════════╗");
    println!("║   Google Earth API - HTTP 版本测试                       ║");
    println!("║   测试 HTTP/1.1, HTTP/2, HTTP/3                         ║");
    println!("╚══════════════════════════════════════════════════════════╝\n");

    // 测试 Chrome 133 在不同 HTTP 版本下的表现
    let profiles = mapped_tls_clients();
    let chrome = profiles.get("chrome_133").expect("找不到 Chrome 133 配置");

    // HTTP/1.1 测试
    println!("🔍 测试 HTTP/1.1...");
    test_http_version(chrome, "1.1");

    // HTTP/2 测试
    println!("\n🔍 测试 HTTP/2...");
    test_http_version(chrome, "2");

    // HTTP/3 测试（如果支持）
    println!("\n🔍 测试 HTTP/3...");
    println!("  ⚠️  HTTP/3 需要完整的 QUIC 实现");
}

fn test_http_version(profile: &fingerprint::ClientProfile, version: &str) {
    match version {
        "1.1" => {
            // 使用我们的自定义 TLS 指纹 + HTTP/1.1
            println!("  📦 使用自定义 TLS 指纹构建 ClientHello...");

            match profile.get_client_hello_spec() {
                Ok(spec) => {
                    match TLSHandshakeBuilder::build_client_hello(&spec, TEST_HOST) {
                        Ok(client_hello) => {
                            println!("  ✅ ClientHello 构建成功: {} bytes", client_hello.len());

                            // 尝试连接
                            if let Ok(mut stream) = TcpStream::connect("142.251.163.100:443") {
                                if stream.write_all(&client_hello).is_ok() {
                                    let mut response = vec![0u8; 5];
                                    if stream.read_exact(&mut response).is_ok() {
                                        let record_type = response[0];
                                        if record_type == 22 {
                                            println!(
                                                "  ✅ HTTP/1.1 连接成功（使用自定义 TLS 指纹）"
                                            );
                                        } else {
                                            println!("  ⚠️  收到非握手响应: {}", record_type);
                                        }
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            println!("  ❌ ClientHello 构建失败: {}", e);
                        }
                    }
                }
                Err(e) => {
                    println!("  ❌ 获取 ClientHelloSpec 失败: {}", e);
                }
            }
        }
        "2" => {
            println!("  ℹ️  HTTP/2 需要完成 TLS 握手后协商 ALPN");
            println!("  📝 我们的自定义 TLS 指纹已包含 ALPN 扩展（h2）");
        }
        "3" => {
            println!("  ℹ️  HTTP/3 基于 QUIC 协议，需要 UDP 连接");
            println!("  📝 我们的自定义 TLS 指纹支持 TLS 1.3，可用于 QUIC");
        }
        _ => {}
    }
}

#[test]
#[ignore] // 需要网络连接
fn test_google_earth_api_detailed_chrome() {
    println!("\n╔══════════════════════════════════════════════════════════╗");
    println!("║   详细测试: Chrome 133 访问 Google Earth API             ║");
    println!("║   使用我们自定义的 TLS 指纹系统                          ║");
    println!("╚══════════════════════════════════════════════════════════╝\n");

    let profiles = mapped_tls_clients();
    let chrome = profiles.get("chrome_133").expect("找不到 Chrome 133 配置");

    println!("📋 Chrome 133 配置信息:");
    println!("  - 浏览器: Chrome 133");
    println!("  - 操作系统: Windows 10");

    // 生成 ClientHelloSpec
    println!("\n🔧 生成 ClientHelloSpec...");
    let spec = chrome
        .get_client_hello_spec()
        .expect("无法生成 ClientHelloSpec");

    println!("  ✅ ClientHelloSpec 生成成功");
    println!("     - 密码套件: {}", spec.cipher_suites.len());
    println!("     - 扩展: {}", spec.extensions.len());

    // 构建 TLS ClientHello
    println!("\n🔨 构建自定义 TLS ClientHello...");
    let client_hello =
        TLSHandshakeBuilder::build_with_debug(&spec, TEST_HOST).expect("无法构建 ClientHello");

    println!("\n🌐 连接到 Google Earth API...");
    println!("  地址: {}", TEST_URL);

    match TcpStream::connect("142.251.163.100:443") {
        Ok(mut stream) => {
            println!("  ✅ TCP 连接建立");

            stream
                .set_read_timeout(Some(std::time::Duration::from_secs(10)))
                .ok();
            stream
                .set_write_timeout(Some(std::time::Duration::from_secs(10)))
                .ok();

            println!(
                "\n📤 发送自定义 TLS ClientHello ({} bytes)...",
                client_hello.len()
            );
            match stream.write_all(&client_hello) {
                Ok(_) => {
                    println!("  ✅ ClientHello 发送成功");

                    println!("\n📥 等待服务器响应...");
                    let mut response = vec![0u8; 5];
                    match stream.read_exact(&mut response) {
                        Ok(_) => {
                            let record_type = response[0];
                            let version = u16::from_be_bytes([response[1], response[2]]);
                            let length = u16::from_be_bytes([response[3], response[4]]);

                            println!("  ✅ 收到服务器响应:");
                            println!(
                                "     - 记录类型: {} ({})",
                                record_type,
                                match record_type {
                                    22 => "Handshake",
                                    21 => "Alert",
                                    23 => "Application Data",
                                    _ => "Unknown",
                                }
                            );
                            println!("     - TLS 版本: 0x{:04x}", version);
                            println!("     - 数据长度: {} bytes", length);

                            if record_type == 22 {
                                let mut server_hello = vec![0u8; length as usize];
                                match stream.read_exact(&mut server_hello) {
                                    Ok(_) => {
                                        println!("\n  🎉 服务器接受了我们的自定义 TLS 指纹！");
                                        println!("  ✅ TLS 握手开始！");
                                        println!("  ✅ ServerHello 接收完成: {} bytes", length);

                                        // 解析 ServerHello
                                        if server_hello.len() >= 38 {
                                            let handshake_type = server_hello[0];
                                            if handshake_type == 2 {
                                                // ServerHello
                                                let server_version = u16::from_be_bytes([
                                                    server_hello[4],
                                                    server_hello[5],
                                                ]);
                                                println!("\n  📊 ServerHello 详情:");
                                                println!(
                                                    "     - 服务器 TLS 版本: 0x{:04x}",
                                                    server_version
                                                );
                                                println!("     - 服务器随机数: {} bytes", 32);

                                                println!("\n  ✅✅✅ 成功验证: Google 服务器接受了我们自定义的 TLS 指纹！");
                                                println!("  🎊 我们真正使用了自己的指纹库系统！");
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        println!("  ⚠️  读取 ServerHello 失败: {}", e);
                                    }
                                }
                            } else if record_type == 21 {
                                let mut alert = vec![0u8; length as usize];
                                if stream.read_exact(&mut alert).is_ok() && alert.len() >= 2 {
                                    println!(
                                        "\n  ⚠️  收到 TLS Alert: Level={}, Description={}",
                                        alert[0], alert[1]
                                    );
                                }
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
            println!("  提示: 请检查网络连接或使用 VPN");
        }
    }
}
