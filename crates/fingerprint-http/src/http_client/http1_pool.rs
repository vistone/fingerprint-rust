//! HTTP/1.1 with Connection Pool
//!
//! architectureexplain：
//! - HTTP/1.1 adopt netconnpool manage TCP connection pool
// ! - pool化pair象：TcpStream (裸 TCP connection)
//! - reusemethod：serialreuse (anconnectionsame when betweencan onlyprocessanrequest)
//! - protocollimit：HTTP/1.1 unable tomultiplereuse, needlarge numberconnectionsupportconcurrent
// ! - netconnpool negative责：connectionCreate, keepactive, 故障detect and recycle

#[cfg(feature = "connection-pool")]
use super::pool::ConnectionPoolManager;
use super::{HttpClientConfig, HttpClientError, HttpRequest, HttpResponse, Result};
#[cfg(feature = "connection-pool")]
use std::io::Write;
#[cfg(feature = "connection-pool")]
use std::sync::Arc;

/// useconnection poolsend HTTP/1.1 request
#[cfg(feature = "connection-pool")]
pub fn send_http1_request_with_pool(
    host: &str,
    port: u16,
    path: &str,
    request: &HttpRequest,
    config: &HttpClientConfig,
    pool_manager: &Arc<ConnectionPoolManager>,
) -> Result<HttpResponse> {
    // from connection poolGetconnection
    let pool = pool_manager.get_pool(host, port)?;

    // Get TCP connection
    let conn = pool.get_tcp().map_err(|e| {
        HttpClientError::ConnectionFailed(format!("Failed to get connection from pool: {:?}", e))
    })?;

    // from Connection in Extract TcpStream
    // PooledConnection implement了 Deref<Target = Connection>, candirectlyuse Connection method
    let tcp_stream = conn.tcp_conn().ok_or_else(|| {
        HttpClientError::ConnectionFailed("Expected TCP connection but got UDP".to_string())
    })?;

    // clone TcpStream so thatwecanuse它
    let mut stream = tcp_stream.try_clone().map_err(HttpClientError::Io)?;

    // Fix: Add Cookie to request ( if exists)
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

    // Build HTTP request
    let http_request = request_with_cookies.build_http1_request(host, path);

    // sendrequest
    stream
        .write_all(http_request.as_bytes())
        .map_err(HttpClientError::Io)?;

    // Fix: usecompleteresponsereadlogic (include body)
    // connectionwillautomatic归still to connection pool (through Drop)
    let buffer =
        super::io::read_http1_response_bytes(&mut stream, super::io::DEFAULT_MAX_RESPONSE_BYTES)
            .map_err(HttpClientError::Io)?;

    // Parseresponse
    HttpResponse::parse(&buffer).map_err(HttpClientError::InvalidResponse)
}

#[cfg(not(feature = "connection-pool"))]
pub fn send_http1_request_with_pool(
    _host: &str,
    _port: u16,
    _path: &str,
    _request: &HttpRequest,
    _config: &HttpClientConfig,
    _pool_manager: &std::sync::Arc<super::pool::ConnectionPoolManager>,
) -> Result<HttpResponse> {
    Err(HttpClientError::ConnectionFailed(
        "connection poolFeaturesnotenabled，请use --features connection-pool compile".to_string(),
    ))
}

#[cfg(all(test, feature = "connection-pool"))]
mod tests {
    #[allow(unused_imports)]
    use super::*;
    use crate::http_client::pool::PoolManagerConfig;
    use crate::http_client::request::HttpMethod;

    #[test]
    #[ignore] // neednetwork
    fn test_http1_with_pool() {
        let request = HttpRequest::new(HttpMethod::Get, "http://example.com/");
        let config = HttpClientConfig::default();
        let pool_manager = {
            #[allow(clippy::arc_with_non_send_sync)]
            {
                Arc::new(ConnectionPoolManager::new(PoolManagerConfig::default()))
            }
        };

        let result =
            send_http1_request_with_pool("example.com", 80, "/", &request, &config, &pool_manager);

        // maywillfailure (networkissue), but不should panic
        if let Ok(response) = result {
            println!("status code: {}", response.status_code);
            assert!(response.status_code > 0);
        }
    }

    #[test]
    #[ignore] // neednetwork
    fn test_connection_reuse() {
        let request = HttpRequest::new(HttpMethod::Get, "http://example.com/");
        let config = HttpClientConfig::default();
        let pool_manager = {
            #[allow(clippy::arc_with_non_send_sync)]
            {
                Arc::new(ConnectionPoolManager::new(PoolManagerConfig::default()))
            }
        };

        // 第oncerequest
        let _ =
            send_http1_request_with_pool("example.com", 80, "/", &request, &config, &pool_manager);

        // secondtimerequest (shouldreuseconnection)
        let _ =
            send_http1_request_with_pool("example.com", 80, "/", &request, &config, &pool_manager);

        // Checkstatisticsinfo
        let stats = pool_manager.get_stats();
        if !stats.is_empty() {
            println!("connection poolstatistics:");
            for stat in stats {
                stat.print();
            }
        }
    }
}
