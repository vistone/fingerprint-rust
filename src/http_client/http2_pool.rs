//! HTTP/2 with Connection Pool
//!
//! 使用 netconnpool 管理 TCP 连接复用，支持 HTTP/2

#[cfg(all(feature = "connection-pool", feature = "http2"))]
use super::pool::ConnectionPoolManager;
use super::{HttpClientConfig, HttpClientError, HttpRequest, HttpResponse, Result};
#[cfg(all(feature = "connection-pool", feature = "http2"))]
use std::sync::Arc;

/// 使用连接池发送 HTTP/2 请求
#[cfg(all(feature = "connection-pool", feature = "http2"))]
pub async fn send_http2_request_with_pool(
    host: &str,
    port: u16,
    path: &str,
    request: &HttpRequest,
    config: &HttpClientConfig,
    pool_manager: &Arc<ConnectionPoolManager>,
) -> Result<HttpResponse> {
    use h2::client;
    use http::{Request as HttpRequest2, Version};
    use tokio_rustls::TlsConnector;

    // 从连接池获取连接
    let pool = pool_manager.get_pool(host, port)?;

    // 获取 TCP 连接
    let conn = pool
        .GetTCP()
        .map_err(|e| HttpClientError::ConnectionFailed(format!("从连接池获取连接失败: {:?}", e)))?;

    // 从 Connection 中提取 TcpStream
    let tcp_stream = conn
        .GetTcpConn()
        .ok_or_else(|| HttpClientError::ConnectionFailed("期望 TCP 连接但得到 UDP".to_string()))?;

    // 克隆 TcpStream 以便我们可以使用它
    let tcp_stream = tcp_stream.try_clone().map_err(HttpClientError::Io)?;

    // 转换为 tokio TcpStream
    tcp_stream
        .set_nonblocking(true)
        .map_err(HttpClientError::Io)?;
    let tcp_stream =
        tokio::net::TcpStream::from_std(tcp_stream).map_err(HttpClientError::Io)?;

    // TLS 握手
    let mut root_store = rustls::RootCertStore::empty();
    root_store.add_trust_anchors(webpki_roots::TLS_SERVER_ROOTS.iter().map(|ta| {
        rustls::OwnedTrustAnchor::from_subject_spki_name_constraints(
            ta.subject,
            ta.spki,
            ta.name_constraints,
        )
    }));

    let mut tls_config = rustls::ClientConfig::builder()
        .with_safe_defaults()
        .with_root_certificates(root_store)
        .with_no_client_auth();

    // 设置 ALPN 为 h2
    tls_config.alpn_protocols = vec![b"h2".to_vec()];

    let connector = TlsConnector::from(std::sync::Arc::new(tls_config));
    let server_name = rustls::ServerName::try_from(host)
        .map_err(|_| HttpClientError::TlsError("无效的服务器名称".to_string()))?;

    let tls_stream = connector
        .connect(server_name, tcp_stream)
        .await
        .map_err(|e| HttpClientError::TlsError(format!("TLS 握手失败: {}", e)))?;

    // 建立 HTTP/2 连接
    let (mut client, h2_conn) = client::handshake(tls_stream)
        .await
        .map_err(|e| HttpClientError::Http2Error(format!("HTTP/2 握手失败: {}", e)))?;

    // 在后台运行连接
    tokio::spawn(async move {
        if let Err(e) = h2_conn.await {
            eprintln!("HTTP/2 连接错误: {:?}", e);
        }
    });

    // 构建 HTTP/2 请求
    let uri: http::Uri = format!("https://{}:{}{}", host, port, path)
        .parse()
        .map_err(|e| HttpClientError::InvalidRequest(format!("无效的 URI: {}", e)))?;

    let http2_request = HttpRequest2::builder()
        .method(match request.method {
            super::request::HttpMethod::Get => http::Method::GET,
            super::request::HttpMethod::Post => http::Method::POST,
            super::request::HttpMethod::Put => http::Method::PUT,
            super::request::HttpMethod::Delete => http::Method::DELETE,
            super::request::HttpMethod::Head => http::Method::HEAD,
            super::request::HttpMethod::Options => http::Method::OPTIONS,
            super::request::HttpMethod::Patch => http::Method::PATCH,
        })
        .uri(uri)
        .version(Version::HTTP_2)
        // 不要手动添加 host header，h2 会自动从 URI 提取
        .header("user-agent", &config.user_agent);

    let http2_request = request
        .headers
        .iter()
        // 跳过 host header
        .filter(|(k, _)| k.to_lowercase() != "host")
        .fold(http2_request, |builder, (k, v)| builder.header(k, v))
        .body(())
        .map_err(|e| HttpClientError::InvalidRequest(format!("构建请求失败: {}", e)))?;

    // 发送请求
    let (response, _send_stream) = client
        .send_request(http2_request, true)
        .map_err(|e| HttpClientError::Http2Error(format!("发送请求失败: {}", e)))?;

    // 等待响应头
    let response = response
        .await
        .map_err(|e| HttpClientError::Http2Error(format!("接收响应失败: {}", e)))?;

    // 先提取 status 和 headers
    let status_code = response.status().as_u16();
    let status_text = http::StatusCode::from_u16(status_code)
        .ok()
        .and_then(|s| s.canonical_reason())
        .unwrap_or("Unknown")
        .to_string();
    let headers: std::collections::HashMap<String, String> = response
        .headers()
        .iter()
        .map(|(k, v)| (k.as_str().to_string(), v.to_str().unwrap_or("").to_string()))
        .collect();

    // 读取响应体
    let mut body_stream = response.into_body();
    let mut body_data = Vec::new();

    while let Some(chunk) = body_stream.data().await {
        let chunk = chunk.map_err(|e| {
            HttpClientError::Io(std::io::Error::other(format!("读取 body 失败: {}", e)))
        })?;
        body_data.extend_from_slice(&chunk);

        // 释放流控制窗口
        let _ = body_stream.flow_control().release_capacity(chunk.len());
    }

    Ok(HttpResponse {
        http_version: "HTTP/2".to_string(),
        status_code,
        status_text,
        headers,
        body: body_data,
        response_time_ms: 0, // TODO: 添加计时
    })
}

#[cfg(test)]
#[cfg(all(feature = "connection-pool", feature = "http2"))]
mod tests {
    use super::*;
    use crate::http_client::pool::PoolManagerConfig;
    use crate::http_client::request::HttpMethod;

    #[tokio::test]
    #[ignore] // 需要网络连接
    async fn test_http2_with_pool() {
        let user_agent = "TestClient/1.0".to_string();
        let config = HttpClientConfig {
            user_agent,
            prefer_http2: true,
            ..Default::default()
        };

        let pool_manager = Arc::new(ConnectionPoolManager::new(PoolManagerConfig::default()));

        let request = HttpRequest::new(HttpMethod::Get, "https://httpbin.org/get");

        let result = send_http2_request_with_pool(
            "httpbin.org",
            443,
            "/get",
            &request,
            &config,
            &pool_manager,
        )
        .await;

        // 可能会失败（网络问题），但不应该 panic
        if let Ok(response) = result {
            assert_eq!(response.status_code, 200);
            assert_eq!(response.http_version, "HTTP/2");
        }
    }
}
