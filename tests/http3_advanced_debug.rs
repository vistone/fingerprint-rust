//! HTTP/3 高级调试 - 深入 QUIC 层
//! 针对 Google Earth API 进行极致优化

#[cfg(feature = "http3")]
#[tokio::test]
#[ignore]
async fn test_http3_step_by_step() {
    use bytes::Buf;
    use h3_quinn::quinn;
    use std::time::Instant;

    println!("\n╔══════════════════════════════════════════════════════════╗");
    println!("║  HTTP/3 逐步调试 - Google Earth API                    ║");
    println!("╚══════════════════════════════════════════════════════════╝\n");

    let start_total = Instant::now();

    // 1. 配置 QUIC - 使用优化的传输参数
    println!("【步骤 1】配置 QUIC 客户端");
    let start = Instant::now();

    let mut roots = rustls::RootCertStore::empty();
    roots.add_trust_anchors(webpki_roots::TLS_SERVER_ROOTS.iter().map(|ta| {
        rustls::OwnedTrustAnchor::from_subject_spki_name_constraints(
            ta.subject,
            ta.spki,
            ta.name_constraints,
        )
    }));

    let mut tls_config = rustls::ClientConfig::builder()
        .with_safe_defaults()
        .with_root_certificates(roots)
        .with_no_client_auth();

    tls_config.alpn_protocols = vec![b"h3".to_vec()];

    let mut client_config = quinn::ClientConfig::new(std::sync::Arc::new(tls_config));

    // 优化传输配置
    let mut transport = quinn::TransportConfig::default();

    // 增加初始窗口大小
    transport.initial_rtt(std::time::Duration::from_millis(100));
    transport.max_idle_timeout(Some(std::time::Duration::from_secs(60).try_into().unwrap()));
    transport.keep_alive_interval(Some(std::time::Duration::from_secs(10)));

    // 调整流控制窗口 - 使用 VarInt
    transport.stream_receive_window((1024 * 1024u32).into()); // 1MB
    transport.receive_window((10 * 1024 * 1024u32).into()); // 10MB

    // 允许更多并发流
    transport.max_concurrent_bidi_streams(100u32.into());
    transport.max_concurrent_uni_streams(100u32.into());

    client_config.transport_config(std::sync::Arc::new(transport));

    let mut endpoint = quinn::Endpoint::client("0.0.0.0:0".parse().unwrap()).unwrap();
    endpoint.set_default_client_config(client_config);

    println!("  ✅ QUIC 配置完成 ({:?})", start.elapsed());

    // 2. DNS 解析和连接
    println!("\n【步骤 2】建立 QUIC 连接");
    let start = Instant::now();

    let addr = "kh.google.com:443";
    println!("  目标: {}", addr);

    // DNS 解析
    use std::net::ToSocketAddrs;
    let socket_addr = addr
        .to_socket_addrs()
        .unwrap()
        .next()
        .expect("DNS 解析失败");

    println!("  ✅ DNS 解析: {:?}", socket_addr);

    let connection = match endpoint.connect(socket_addr, "kh.google.com") {
        Ok(connecting) => {
            println!("  ✅ 开始连接...");
            match connecting.await {
                Ok(conn) => {
                    println!("  ✅ QUIC 连接成功 ({:?})", start.elapsed());

                    // 打印连接信息
                    let stats = conn.stats();
                    println!("  📊 连接统计:");
                    println!("     RTT: {:?}", stats.path.rtt);
                    println!("     拥塞窗口: {} bytes", stats.path.cwnd);

                    conn
                }
                Err(e) => {
                    println!("  ❌ QUIC 握手失败: {:?}", e);
                    panic!("QUIC 握手失败");
                }
            }
        }
        Err(e) => {
            println!("  ❌ 无法初始化连接: {:?}", e);
            panic!("连接初始化失败");
        }
    };

    // 3. HTTP/3 握手
    println!("\n【步骤 3】HTTP/3 握手");
    let start = Instant::now();

    let h3_conn = match h3::client::new(h3_quinn::Connection::new(connection)).await {
        Ok(conn) => {
            println!("  ✅ HTTP/3 握手成功 ({:?})", start.elapsed());
            conn
        }
        Err(e) => {
            println!("  ❌ HTTP/3 握手失败: {:?}", e);
            panic!("HTTP/3 握手失败");
        }
    };

    let (driver, mut send_request) = h3_conn;

    // 在后台驱动连接 - 关键！
    // h3 的 driver 需要持续运行以处理底层 QUIC 连接
    let driver_handle = tokio::spawn(async move {
        let mut driver = driver;
        let _ = std::future::poll_fn(|cx| driver.poll_close(cx)).await;
    });

    // 4. 构建并发送请求
    println!("\n【步骤 4】发送 HTTP/3 请求");
    let start = Instant::now();

    let req = http::Request::builder()
        .method(http::Method::GET)
        .uri("https://kh.google.com/rt/earth/PlanetoidMetadata")
        .version(http::Version::HTTP_3)
        .header(
            "user-agent",
            "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36",
        )
        .header("accept", "*/*")
        .body(())
        .unwrap();

    println!("  📤 发送请求...");
    let mut stream = match send_request.send_request(req).await {
        Ok(s) => {
            println!("  ✅ 请求已发送 ({:?})", start.elapsed());
            s
        }
        Err(e) => {
            println!("  ❌ 发送请求失败: {:?}", e);
            driver_handle.abort();
            panic!("发送请求失败");
        }
    };

    // 完成请求发送
    println!("  📤 完成请求...");
    if let Err(e) = stream.finish().await {
        println!("  ❌ 完成请求失败: {:?}", e);
        driver_handle.abort();
        panic!("完成请求失败");
    }
    println!("  ✅ 请求完成");

    // 5. 接收响应
    println!("\n【步骤 5】接收响应");
    let start = Instant::now();

    let resp = match stream.recv_response().await {
        Ok(r) => {
            println!("  ✅ 收到响应头 ({:?})", start.elapsed());
            println!("  📊 状态: {}", r.status());
            println!("  📊 Headers:");
            for (k, v) in r.headers().iter() {
                println!("     {}: {:?}", k, v);
            }
            r
        }
        Err(e) => {
            println!("  ❌ 接收响应失败: {:?}", e);
            driver_handle.abort();
            panic!("接收响应失败");
        }
    };

    // 6. 读取 Body
    println!("\n【步骤 6】读取响应体");
    let start = Instant::now();

    let mut body_data = Vec::new();
    let mut chunk_count = 0;

    while let Ok(Some(mut chunk)) = stream.recv_data().await {
        chunk_count += 1;
        let len = chunk.remaining();
        println!("  📦 数据块 {}: {} bytes", chunk_count, len);

        let mut chunk_bytes = vec![0u8; len];
        chunk.copy_to_slice(&mut chunk_bytes);
        body_data.extend_from_slice(&chunk_bytes);
    }

    println!("  ✅ Body 读取完成 ({:?})", start.elapsed());
    println!("  📊 总大小: {} bytes", body_data.len());
    println!("  📊 总块数: {}", chunk_count);

    // 总结
    println!("\n╔══════════════════════════════════════════════════════════╗");
    println!("║  测试结果                                                ║");
    println!("╚══════════════════════════════════════════════════════════╝");
    println!("✅ 状态码: {}", resp.status());
    println!("✅ Body 大小: {} bytes", body_data.len());
    println!("✅ 总耗时: {:?}", start_total.elapsed());

    // 清理
    driver_handle.abort();

    assert_eq!(resp.status(), 200);
    assert!(!body_data.is_empty());
}

#[cfg(not(feature = "http3"))]
#[test]
fn test_http3_feature_required() {
    println!("需要启用 http3 feature");
}
