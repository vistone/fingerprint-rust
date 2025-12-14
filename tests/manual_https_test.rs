//! 手动测试 HTTPS 请求构建

use fingerprint::{get_user_agent_by_profile_name, HttpRequest};

#[test]
fn test_request_building() {
    let url = "https://kh.google.com/rt/earth/PlanetoidMetadata";
    let user_agent = get_user_agent_by_profile_name("chrome_133").unwrap();

    let request = HttpRequest::new(fingerprint::http_client::request::HttpMethod::Get, url)
        .with_user_agent(&user_agent);

    let http_request = request.build_http1_request("kh.google.com", "/rt/earth/PlanetoidMetadata");

    println!(
        "\n═══ HTTP/1.1 请求格式 ═══\n{}\n═══════════════════════",
        http_request
    );

    // 检查必要的 headers
    assert!(http_request.contains("Host: kh.google.com"));
    assert!(http_request.contains("User-Agent:"));
    assert!(http_request.contains("Connection: close"));
    assert!(http_request.starts_with("GET /rt/earth/PlanetoidMetadata HTTP/1.1\r\n"));
}

// 测试添加更多标准 headers
#[test]
fn test_request_with_standard_headers() {
    let url = "https://kh.google.com/rt/earth/PlanetoidMetadata";
    let user_agent = get_user_agent_by_profile_name("chrome_133").unwrap();

    let mut headers = std::collections::HashMap::new();
    headers.insert("Accept".to_string(), "*/*".to_string());
    headers.insert(
        "Accept-Encoding".to_string(),
        "gzip, deflate, br".to_string(),
    );
    headers.insert("Accept-Language".to_string(), "en-US,en;q=0.9".to_string());

    let mut request = HttpRequest::new(fingerprint::http_client::request::HttpMethod::Get, url)
        .with_user_agent(&user_agent);

    // 添加 headers
    for (k, v) in headers {
        request = request.with_header(&k, &v);
    }

    let http_request = request.build_http1_request("kh.google.com", "/rt/earth/PlanetoidMetadata");

    println!(
        "\n═══ HTTP/1.1 请求格式（带标准 headers）═══\n{}\n═══════════════════════",
        http_request
    );

    assert!(http_request.contains("Accept:"));
    assert!(http_request.contains("Accept-Encoding:"));
}
