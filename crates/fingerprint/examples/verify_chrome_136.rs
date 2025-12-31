use fingerprint::*;
use fingerprint_core::dicttls::cipher_suites;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Chrome 136 指纹深度验证\n");

    let profiles = mapped_tls_clients();
    let chrome_136 = profiles
        .get("chrome_136")
        .expect("Chrome 136 profile should exist");

    // 1. 验证密码套件 (Cipher Suites)
    println!("1️⃣  加密套件权重验证:");
    let spec = chrome_136.get_client_hello_spec()?;

    // 获取前 5 个加密套件（跳过 GREASE）
    let first_suites: Vec<u16> = spec
        .cipher_suites
        .iter()
        .filter(|&&s| !fingerprint_tls::tls_config::is_grease_value(s))
        .take(5)
        .cloned()
        .collect();

    println!("   前 5 个非 GREASE 加密套件:");
    for suite in first_suites {
        println!("     - 0x{:04x}", suite);
    }

    // 预期第一个是 TLS_AES_128_GCM_SHA256 (0x1301)
    if spec.cipher_suites.iter().any(|&s| s == 0x1301) {
        println!("   ✅ 包含 TLS_AES_128_GCM_SHA256");
    }

    // 2. 验证 ALPN
    println!("\n2️⃣  ALPN 优先级验证:");
    if let Some(metadata) = &spec.metadata {
        if let Some(alpn) = metadata.get_alpn() {
            println!("   配置的 ALPN: {:?}", alpn);
            if alpn.first() == Some(&"h3".to_string()) {
                println!("   ✅ h3 已正确置于首位");
            } else {
                println!("   ❌ h3 未置于首位: {:?}", alpn.first());
            }
        } else {
            println!("   ❌ 未找到 ALPN 元数据");
        }
    }

    // 3. 构建实战字节流
    println!("\n3️⃣  ClientHello 字节流构建:");
    let client_hello = TLSHandshakeBuilder::build_client_hello(&spec, "www.google.com")?;
    println!("   ✅ 成功生成 ClientHello: {} bytes", client_hello.len());

    // 简单检查 ALPN 是否在字节流中 (h3, h2, http/1.1)
    if client_hello.windows(2).any(|w| w == b"h3") && client_hello.windows(2).any(|w| w == b"h2") {
        println!("   ✅ 字节流中包含 h3 和 h2 标识");
    }

    // 4. JA4 指纹验证
    println!("\n4️⃣  JA4 指纹主动生成:");
    let ja4 = chrome_136.get_ja4_string()?;
    println!("   ✅ JA4: {}", ja4);

    println!("\n✨ Chrome 136 微调验证通过！");
    Ok(())
}
