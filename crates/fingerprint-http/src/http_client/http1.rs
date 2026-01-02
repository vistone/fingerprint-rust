//! HTTP/1.1 implement
//!
//! use netconnpool 管理 TCP connection，send HTTP/1.1 request

use super::{HttpClientConfig, HttpClientError, HttpRequest, HttpResponse, Result};
use std::io::Write;
use std::net::TcpStream;

/// send HTTP/1.1 request
pub fn send_http1_request(
    host: &str,
    port: u16,
    path: &str,
    request: &HttpRequest,
    config: &HttpClientConfig,
) -> Result<HttpResponse> {
    // connectionserver
    let addr = format!("{}:{}", host, port);
    let mut stream = TcpStream::connect(&addr)
        .map_err(|e| HttpClientError::ConnectionFailed(format!("Connection failed {}: {}", addr, e)))?;

    // settingstimeout
    stream
        .set_read_timeout(Some(config.read_timeout))
        .map_err(HttpClientError::Io)?;
    stream
        .set_write_timeout(Some(config.write_timeout))
        .map_err(HttpClientError::Io)?;

    // Fix: Add Cookie  to request（ if  exists）
    let mut request_with_cookies = request.clone();
    if let Some(cookie_store) = &config.cookie_store {
        super::request::add_cookies_to_request(
            &mut request_with_cookies,
            cookie_store,
            host,
            path,
            false, // HTTP is notsecurityconnection
        );
    }

    // Build并send HTTP/1.1 request
    let header_order = config.profile.as_ref().map(|p| p.header_order.as_slice());
    let http_request = request_with_cookies.build_http1_request_bytes(host, path, header_order);
    stream
        .write_all(&http_request)
        .map_err(HttpClientError::Io)?;
    stream.flush().map_err(HttpClientError::Io)?;

    // readresponse
    let mut stream = stream;
    let buffer =
        super::io::read_http1_response_bytes(&mut stream, super::io::DEFAULT_MAX_RESPONSE_BYTES)
            .map_err(HttpClientError::Io)?;

    // Parseresponse
    HttpResponse::parse(&buffer).map_err(HttpClientError::InvalidResponse)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore] // neednetworkconnection
    fn test_send_http1_request() {
        let request = HttpRequest::new(
            crate::http_client::request::HttpMethod::Get,
            "http://example.com",
        )
        .with_user_agent("TestClient/1.0");

        let config = HttpClientConfig::default();
        let response = send_http1_request("example.com", 80, "/", &request, &config).unwrap();

        if response.status_code == 200 {
            assert!(response.is_success());
        } else {
            println!("⚠️  Server returned {}", response.status_code);
        }
    }
}
