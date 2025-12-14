//! 深度调试 HTTP/2 实现
//! 逐步验证每个环节

#[cfg(feature = "http2")]
#[tokio::test]
#[ignore]
async fn test_http2_handshake_only() {
    use rustls::{ClientConfig, RootCertStore, ServerName};
    use std::sync::Arc;
    use tokio::net::TcpStream;
    use tokio_rustls::TlsConnector;

    println!("\n═══ 测试 1: TCP 连接 ═══");
    let addr = "kh.google.com:443";
    let tcp = TcpStream::connect(addr).await.expect("TCP 连接失败");
    println!("✅ TCP 连接成功");

    println!("\n═══ 测试 2: TLS 握手（带 ALPN）═══");
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

    // 设置 ALPN
    tls_config.alpn_protocols = vec![b"h2".to_vec()];
    println!("设置 ALPN: h2");

    let connector = TlsConnector::from(Arc::new(tls_config));
    let server_name = ServerName::try_from("kh.google.com").unwrap();

    let tls_stream = connector
        .connect(server_name, tcp)
        .await
        .expect("TLS 握手失败");

    println!("✅ TLS 握手成功");

    // 检查 ALPN 结果
    let (_, session) = tls_stream.get_ref();
    if let Some(proto) = session.alpn_protocol() {
        println!("✅ ALPN 协商结果: {}", String::from_utf8_lossy(proto));
    } else {
        println!("⚠️ 没有协商 ALPN");
    }

    println!("\n═══ 测试 3: HTTP/2 握手 ═══");
    let (mut h2_client, h2_conn) = h2::client::handshake(tls_stream)
        .await
        .expect("HTTP/2 握手失败");

    println!("✅ HTTP/2 握手成功");

    // 在后台运行连接
    tokio::spawn(async move {
        if let Err(e) = h2_conn.await {
            eprintln!("HTTP/2 连接错误: {:?}", e);
        }
    });

    println!("\n═══ 测试 4: 发送 HTTP/2 请求 ═══");

    // 构建正确的 URI
    let uri = "https://kh.google.com/rt/earth/PlanetoidMetadata"
        .parse::<http::Uri>()
        .expect("URI 解析失败");

    let request = http::Request::builder()
        .method(http::Method::GET)
        .uri(uri)
        .version(http::Version::HTTP_2)
        // 不需要手动添加 host header，h2 会从 URI 自动提取
        .header("user-agent", "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/133.0.0.0 Safari/537.36")
        .header("accept", "*/*")
        .body(())
        .expect("构建请求失败");

    println!("请求详情:");
    println!("  Method: {}", request.method());
    println!("  URI: {}", request.uri());
    println!("  Headers:");
    for (k, v) in request.headers() {
        println!("    {}: {:?}", k, v);
    }

    let (response_future, _send_stream) =
        h2_client.send_request(request, true).expect("发送请求失败");

    println!("✅ 请求已发送");

    println!("\n═══ 测试 5: 接收响应 ═══");
    match response_future.await {
        Ok(response) => {
            println!("✅ 收到响应！");
            println!("  状态码: {}", response.status());
            println!("  Headers:");
            for (k, v) in response.headers() {
                println!("    {}: {:?}", k, v);
            }

            let mut body_stream = response.into_body();
            let mut body_data = Vec::new();

            while let Some(chunk_result) = body_stream.data().await {
                match chunk_result {
                    Ok(chunk) => {
                        println!("  收到数据块: {} bytes", chunk.len());
                        body_data.extend_from_slice(&chunk);
                        let _ = body_stream.flow_control().release_capacity(chunk.len());
                    }
                    Err(e) => {
                        println!("  ❌ 读取数据块失败: {:?}", e);
                        break;
                    }
                }
            }

            println!("✅ Body 总大小: {} bytes", body_data.len());

            assert!(!body_data.is_empty(), "Body 不应该为空");
        }
        Err(e) => {
            println!("❌ 接收响应失败: {:?}", e);
            println!("错误类型: {}", std::any::type_name_of_val(&e));
            panic!("HTTP/2 请求失败");
        }
    }
}

#[cfg(feature = "http2")]
#[tokio::test]
#[ignore]
async fn test_http2_with_www_google() {
    use rustls::{ClientConfig, RootCertStore, ServerName};
    use std::sync::Arc;
    use tokio::net::TcpStream;
    use tokio_rustls::TlsConnector;

    println!("\n═══ 测试 www.google.com (HTTP/2) ═══");

    let tcp = TcpStream::connect("www.google.com:443")
        .await
        .expect("TCP 连接失败");

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

    tls_config.alpn_protocols = vec![b"h2".to_vec()];

    let connector = TlsConnector::from(Arc::new(tls_config));
    let server_name = ServerName::try_from("www.google.com").unwrap();

    let tls_stream = connector.connect(server_name, tcp).await.expect("TLS 失败");

    let (mut h2_client, h2_conn) = h2::client::handshake(tls_stream)
        .await
        .expect("HTTP/2 握手失败");

    tokio::spawn(async move {
        if let Err(e) = h2_conn.await {
            eprintln!("连接错误: {:?}", e);
        }
    });

    let request = http::Request::builder()
        .method("GET")
        .uri("https://www.google.com/")
        .header("host", "www.google.com")
        .header("user-agent", "Mozilla/5.0")
        .body(())
        .unwrap();

    let (response_future, _) = h2_client.send_request(request, true).unwrap();

    match response_future.await {
        Ok(response) => {
            println!("✅ 成功: {}", response.status());
            assert!(response.status().is_success());
        }
        Err(e) => {
            println!("❌ 失败: {:?}", e);
            panic!("测试失败");
        }
    }
}

#[cfg(not(feature = "http2"))]
#[test]
fn test_http2_feature_required() {
    println!("需要启用 http2 feature");
}
