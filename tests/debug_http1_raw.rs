//! 原始 HTTP/1.1 测试

use std::io::{Read, Write};
use std::net::TcpStream;

#[test]
#[ignore]
fn test_raw_http1_request() {
    use rustls::{ClientConfig, ClientConnection, RootCertStore, ServerName};
    use std::sync::Arc;

    println!("\n═══ 原始 HTTP/1.1 + TLS 测试 ═══\n");

    // 1. TCP 连接
    let tcp = TcpStream::connect("kh.google.com:443").expect("TCP 连接失败");
    println!("✅ TCP 连接成功");

    // 2. TLS 配置
    let mut root_store = RootCertStore::empty();
    root_store.add_trust_anchors(webpki_roots::TLS_SERVER_ROOTS.iter().map(|ta| {
        rustls::OwnedTrustAnchor::from_subject_spki_name_constraints(
            ta.subject,
            ta.spki,
            ta.name_constraints,
        )
    }));

    let mut tls_config = ClientConfig::builder()
        .with_safe_defaults()
        .with_root_certificates(root_store)
        .with_no_client_auth();

    // 设置 ALPN 为 http/1.1
    tls_config.alpn_protocols = vec![b"http/1.1".to_vec()];
    println!("设置 ALPN: http/1.1");

    let server_name = ServerName::try_from("kh.google.com").unwrap();
    let conn = ClientConnection::new(Arc::new(tls_config), server_name).unwrap();

    let mut tls_stream = rustls::StreamOwned::new(conn, tcp);
    println!("✅ TLS 握手开始");

    // 完成握手
    tls_stream.flush().expect("TLS 握手失败");
    println!("✅ TLS 握手完成");

    // 检查 ALPN
    if let Some(proto) = tls_stream.conn.alpn_protocol() {
        println!("✅ ALPN: {}", String::from_utf8_lossy(proto));
    }

    // 3. 构建 HTTP/1.1 请求
    let request = concat!(
        "GET /rt/earth/PlanetoidMetadata HTTP/1.1\r\n",
        "Host: kh.google.com\r\n",
        "User-Agent: Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/133.0.0.0 Safari/537.36\r\n",
        "Accept: */*\r\n",
        "Connection: close\r\n",
        "\r\n"
    );

    println!("\n发送请求:");
    println!("{}", request);

    // 4. 发送请求
    tls_stream.write_all(request.as_bytes()).expect("发送失败");
    tls_stream.flush().expect("Flush 失败");
    println!("✅ 请求已发送");

    // 5. 读取响应
    println!("\n开始读取响应...");
    let mut response = Vec::new();

    match tls_stream.read_to_end(&mut response) {
        Ok(n) => {
            println!("✅ 读取了 {} bytes", n);

            // 打印响应
            let response_str = String::from_utf8_lossy(&response);
            println!("\n响应:");
            println!("{}", &response_str[..response_str.len().min(500)]);

            assert!(n > 0, "响应不应该为空");
        }
        Err(e) => {
            println!("❌ 读取失败: {}", e);
            println!("错误类型: {:?}", e.kind());
            panic!("读取响应失败");
        }
    }
}

#[test]
#[ignore]
fn test_with_chunked_reading() {
    use rustls::{ClientConfig, ClientConnection, RootCertStore, ServerName};
    use std::sync::Arc;
    use std::time::Duration;

    println!("\n═══ HTTP/1.1 分块读取测试 ═══\n");

    let tcp = TcpStream::connect("kh.google.com:443").expect("TCP 连接失败");
    tcp.set_read_timeout(Some(Duration::from_secs(10))).unwrap();
    tcp.set_write_timeout(Some(Duration::from_secs(10)))
        .unwrap();

    let mut root_store = RootCertStore::empty();
    root_store.add_trust_anchors(webpki_roots::TLS_SERVER_ROOTS.iter().map(|ta| {
        rustls::OwnedTrustAnchor::from_subject_spki_name_constraints(
            ta.subject,
            ta.spki,
            ta.name_constraints,
        )
    }));

    let mut tls_config = ClientConfig::builder()
        .with_safe_defaults()
        .with_root_certificates(root_store)
        .with_no_client_auth();

    tls_config.alpn_protocols = vec![b"http/1.1".to_vec()];

    let server_name = ServerName::try_from("kh.google.com").unwrap();
    let conn = ClientConnection::new(Arc::new(tls_config), server_name).unwrap();
    let mut tls_stream = rustls::StreamOwned::new(conn, tcp);

    let request = concat!(
        "GET /rt/earth/PlanetoidMetadata HTTP/1.1\r\n",
        "Host: kh.google.com\r\n",
        "User-Agent: curl/7.68.0\r\n",
        "Accept: */*\r\n",
        "Connection: close\r\n",
        "\r\n"
    );

    tls_stream.write_all(request.as_bytes()).unwrap();
    tls_stream.flush().unwrap();
    println!("✅ 请求已发送");

    // 分块读取
    let mut response = Vec::new();
    let mut buffer = [0u8; 4096];
    let mut total_read = 0;

    println!("\n开始分块读取...");
    loop {
        match tls_stream.read(&mut buffer) {
            Ok(0) => {
                println!("✅ 连接关闭");
                break;
            }
            Ok(n) => {
                total_read += n;
                response.extend_from_slice(&buffer[..n]);
                println!("读取了 {} bytes (总计: {})", n, total_read);
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                println!("⚠️ WouldBlock");
                continue;
            }
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                println!("⚠️ UnexpectedEof at {} bytes", total_read);
                break;
            }
            Err(e) => {
                println!("❌ 错误: {} ({:?})", e, e.kind());
                panic!("读取失败");
            }
        }
    }

    println!("\n✅ 总共读取: {} bytes", total_read);

    if total_read > 0 {
        let response_str = String::from_utf8_lossy(&response);
        println!("\n响应（前 300 字符）:");
        println!("{}", &response_str[..response_str.len().min(300)]);
    }

    assert!(total_read > 0, "应该读取到数据");
}
