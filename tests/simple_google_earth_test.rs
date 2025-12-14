//! 简单的 Google Earth API 测试（使用 reqwest）
//! 用来验证端点是否可访问

#[tokio::test]
#[ignore]
async fn test_with_reqwest() {
    let url = "https://kh.google.com/rt/earth/PlanetoidMetadata";

    println!("\n测试使用 reqwest 访问: {}", url);

    let client = reqwest::Client::builder().use_rustls_tls().build().unwrap();

    match client.get(url).send().await {
        Ok(response) => {
            println!("✅ 成功！");
            println!("  状态码: {}", response.status());
            println!("  HTTP 版本: {:?}", response.version());

            if let Some(content_type) = response.headers().get("content-type") {
                println!("  Content-Type: {:?}", content_type);
            }

            match response.text().await {
                Ok(body) => {
                    println!("  Body 大小: {} bytes", body.len());
                }
                Err(e) => {
                    println!("  读取 body 失败: {}", e);
                }
            }
        }
        Err(e) => {
            println!("❌ 失败: {}", e);
            panic!("reqwest 请求失败");
        }
    }
}
