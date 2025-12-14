//! 调试 ClientHello 格式

use fingerprint::{mapped_tls_clients, tls_handshake::TLSHandshakeBuilder};

fn main() {
    println!("╔══════════════════════════════════════════════════════════╗");
    println!("║           ClientHello 格式调试工具                       ║");
    println!("╚══════════════════════════════════════════════════════════╝\n");

    let profiles = mapped_tls_clients();
    let chrome = profiles.get("chrome_133").unwrap();
    let spec = chrome.get_client_hello_spec().unwrap();

    println!("📋 ClientHelloSpec 信息:");
    println!("  - 密码套件: {}", spec.cipher_suites.len());
    println!("  - 扩展: {}", spec.extensions.len());
    println!("  - 压缩方法: {:?}", spec.compression_methods);

    println!("\n🔧 扩展列表:");
    for (i, ext) in spec.extensions.iter().enumerate() {
        let ext_id = ext.extension_id();
        let ext_len = ext.len();
        println!("  [{}] ID=0x{:04x}, Len={}", i, ext_id, ext_len);
    }

    println!("\n🔨 构建 ClientHello...");
    let client_hello = TLSHandshakeBuilder::build_client_hello(&spec, "kh.google.com").unwrap();

    println!("\n📦 ClientHello hex dump (前 150 bytes):");
    for (i, chunk) in client_hello.chunks(16).take(10).enumerate() {
        print!("  {:04x}: ", i * 16);
        for byte in chunk {
            print!("{:02x} ", byte);
        }
        print!("  |  ");
        for byte in chunk {
            let ch = if *byte >= 0x20 && *byte < 0x7f {
                *byte as char
            } else {
                '.'
            };
            print!("{}", ch);
        }
        println!();
    }

    // 解析结构
    println!("\n📊 TLS 记录层:");
    println!("  - 类型: {} ({})", client_hello[0], match client_hello[0] {
        22 => "Handshake",
        21 => "Alert",
        23 => "Application Data",
        _ => "Unknown",
    });
    println!("  - 版本: 0x{:02x}{:02x} (TLS {})", 
        client_hello[1], 
        client_hello[2],
        match (client_hello[1], client_hello[2]) {
            (0x03, 0x01) => "1.0",
            (0x03, 0x02) => "1.1",
            (0x03, 0x03) => "1.2",
            (0x03, 0x04) => "1.3",
            _ => "Unknown",
        }
    );
    let record_len = u16::from_be_bytes([client_hello[3], client_hello[4]]);
    println!("  - 长度: {} bytes", record_len);

    println!("\n📊 握手层:");
    println!("  - 类型: {} ({})", client_hello[5], match client_hello[5] {
        1 => "ClientHello",
        2 => "ServerHello",
        _ => "Other",
    });
    let handshake_len = ((client_hello[6] as u32) << 16) 
        | ((client_hello[7] as u32) << 8) 
        | (client_hello[8] as u32);
    println!("  - 长度: {} bytes", handshake_len);

    println!("\n📊 ClientHello:");
    println!("  - 客户端版本: 0x{:02x}{:02x}", client_hello[9], client_hello[10]);
    println!("  - 随机数: 32 bytes (offset 11-42)");
    
    let session_id_len = client_hello[43];
    println!("  - 会话 ID 长度: {}", session_id_len);
    
    let mut offset = 44 + session_id_len as usize;
    
    if offset + 2 <= client_hello.len() {
        let cipher_suites_len = u16::from_be_bytes([
            client_hello[offset], 
            client_hello[offset + 1]
        ]);
        println!("  - 密码套件长度: {} bytes ({} suites)", 
            cipher_suites_len, 
            cipher_suites_len / 2
        );
        
        // 打印密码套件
        println!("\n    密码套件:");
        offset += 2;
        for i in 0..(cipher_suites_len / 2).min(5) {
            let cs = u16::from_be_bytes([
                client_hello[offset + i as usize * 2], 
                client_hello[offset + i as usize * 2 + 1]
            ]);
            println!("      [{}] 0x{:04x}", i, cs);
        }
        if cipher_suites_len / 2 > 5 {
            println!("      ... ({} more)", cipher_suites_len / 2 - 5);
        }
        
        offset += cipher_suites_len as usize;
    }
    
    if offset + 1 <= client_hello.len() {
        let compression_len = client_hello[offset];
        println!("\n  - 压缩方法长度: {}", compression_len);
        offset += 1;
        
        if compression_len > 0 && offset + compression_len as usize <= client_hello.len() {
            print!("    压缩方法: [");
            for i in 0..compression_len {
                print!("{:02x}", client_hello[offset + i as usize]);
                if i < compression_len - 1 {
                    print!(", ");
                }
            }
            println!("]");
            offset += compression_len as usize;
        }
    }
    
    if offset + 2 <= client_hello.len() {
        let extensions_len = u16::from_be_bytes([
            client_hello[offset], 
            client_hello[offset + 1]
        ]);
        println!("\n  - 扩展总长度: {} bytes", extensions_len);
        offset += 2;
        
        // 解析扩展
        println!("\n    扩展列表:");
        let mut ext_offset = offset;
        let mut ext_count = 0;
        
        while ext_offset + 4 <= offset + extensions_len as usize && ext_count < 20 {
            let ext_type = u16::from_be_bytes([
                client_hello[ext_offset], 
                client_hello[ext_offset + 1]
            ]);
            let ext_len = u16::from_be_bytes([
                client_hello[ext_offset + 2], 
                client_hello[ext_offset + 3]
            ]);
            
            let ext_name = match ext_type {
                0 => "server_name (SNI)",
                10 => "supported_groups",
                11 => "ec_point_formats",
                13 => "signature_algorithms",
                16 => "application_layer_protocol_negotiation",
                23 => "extended_master_secret",
                35 => "session_ticket",
                43 => "supported_versions",
                45 => "psk_key_exchange_modes",
                51 => "key_share",
                _ => "unknown",
            };
            
            println!("      [{}] Type=0x{:04x} ({}), Len={}", 
                ext_count, ext_type, ext_name, ext_len
            );
            
            // 如果是 SNI 扩展，显示服务器名称
            if ext_type == 0 && ext_len > 5 {
                let sni_list_len = u16::from_be_bytes([
                    client_hello[ext_offset + 4], 
                    client_hello[ext_offset + 5]
                ]);
                if sni_list_len > 3 && ext_offset + 9 < client_hello.len() {
                    let name_type = client_hello[ext_offset + 6];
                    let name_len = u16::from_be_bytes([
                        client_hello[ext_offset + 7], 
                        client_hello[ext_offset + 8]
                    ]);
                    if name_type == 0 && name_len > 0 {
                        let name_start = ext_offset + 9;
                        let name_end = (name_start + name_len as usize).min(client_hello.len());
                        if let Ok(name) = std::str::from_utf8(&client_hello[name_start..name_end]) {
                            println!("           └─ 服务器名称: {}", name);
                        }
                    }
                }
            }
            
            ext_offset += 4 + ext_len as usize;
            ext_count += 1;
        }
    }
    
    println!("\n✅ ClientHello 总大小: {} bytes", client_hello.len());
    
    // 验证格式
    println!("\n🔍 格式验证:");
    let mut issues = Vec::new();
    
    if client_hello[0] != 22 {
        issues.push(format!("❌ 记录类型应该是 22 (Handshake), 实际是 {}", client_hello[0]));
    } else {
        println!("  ✅ 记录类型正确 (Handshake)");
    }
    
    if client_hello[5] != 1 {
        issues.push(format!("❌ 握手类型应该是 1 (ClientHello), 实际是 {}", client_hello[5]));
    } else {
        println!("  ✅ 握手类型正确 (ClientHello)");
    }
    
    let expected_record_len = client_hello.len() - 5;
    if record_len as usize != expected_record_len {
        issues.push(format!("❌ 记录长度不匹配: 声明 {}, 实际 {}", 
            record_len, expected_record_len));
    } else {
        println!("  ✅ 记录长度正确");
    }
    
    let expected_handshake_len = client_hello.len() - 9;
    if handshake_len as usize != expected_handshake_len {
        issues.push(format!("❌ 握手长度不匹配: 声明 {}, 实际 {}", 
            handshake_len, expected_handshake_len));
    } else {
        println!("  ✅ 握手长度正确");
    }
    
    if !issues.is_empty() {
        println!("\n⚠️  发现 {} 个问题:", issues.len());
        for issue in &issues {
            println!("  {}", issue);
        }
    } else {
        println!("\n✅ 所有格式验证通过！");
    }
}
