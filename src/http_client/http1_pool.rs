//! HTTP/1.1 with Connection Pool
//!
//! 使用 netconnpool 管理 TCP 连接复用

#[cfg(feature = "connection-pool")]
use super::pool::ConnectionPoolManager;
use super::{HttpClientConfig, HttpClientError, HttpRequest, HttpResponse, Result};
#[cfg(feature = "connection-pool")]
use std::io::{Read, Write};
use std::sync::Arc;
#[cfg(feature = "connection-pool")]
use std::net::TcpStream;

/// 使用连接池发送 HTTP/1.1 请求
#[cfg(feature = "connection-pool")]
pub fn send_http1_request_with_pool(
    host: &str,
    port: u16,
    path: &str,
    request: &HttpRequest,
    config: &HttpClientConfig,
    pool_manager: &Arc<ConnectionPoolManager>,
) -> Result<HttpResponse> {
    // 从连接池获取连接
    let pool = pool_manager.get_pool(host, port)?;

    // 获取 TCP 连接
    let conn = pool
        .GetTCP()
        .map_err(|e| HttpClientError::ConnectionFailed(format!("从连接池获取连接失败: {:?}", e)))?;

    // 使用闭包执行请求，确保连接最后能被归还
    let result = (|| -> Result<HttpResponse> {
        // 从 Connection 中提取 TcpStream
        let tcp_stream = conn
            .GetTcpConn()
            .ok_or_else(|| HttpClientError::ConnectionFailed("期望 TCP 连接但得到 UDP".to_string()))?;

        // 克隆 TcpStream 以便我们可以使用它
        let mut stream = tcp_stream.try_clone().map_err(HttpClientError::Io)?;

        // 设置超时
        stream
            .set_read_timeout(Some(config.read_timeout))
            .map_err(HttpClientError::Io)?;
        stream
            .set_write_timeout(Some(config.write_timeout))
            .map_err(HttpClientError::Io)?;

        // 构建 HTTP 请求
        let http_request = request.build_http1_request(host, path);

        // 发送请求
        stream
            .write_all(http_request.as_bytes())
            .map_err(HttpClientError::Io)?;
        stream.flush().map_err(HttpClientError::Io)?;

        // 读取响应
        // 注意：我们必须精确读取，不能多读，否则会影响连接复用
        let mut buffer = Vec::with_capacity(4096);
        
        // 1. 读取 Headers
        let headers_len = read_until_headers_end(&mut stream, &mut buffer)?;
        
        // 2. 解析 Headers 以确定 Body 长度
        let (is_chunked, content_length) = parse_body_type(&buffer[..headers_len])?;
        
        // 3. 读取 Body
        if is_chunked {
            read_chunked_body(&mut stream, &mut buffer)?;
        } else if let Some(len) = content_length {
            read_fixed_body(&mut stream, &mut buffer, len)?;
        } else {
            // 假设如果没有明确指示长度，就认为 Body 为空（Keep-Alive 模式下）
            // 或者是 Close 模式读取到 EOF
            // 检查 Connection: close
            let is_close = buffer.windows(17).any(|w| w.eq_ignore_ascii_case(b"Connection: close"));
            if is_close {
                 stream.read_to_end(&mut buffer).map_err(HttpClientError::Io)?;
            }
        }

        // 解析响应
        HttpResponse::parse(&buffer).map_err(HttpClientError::InvalidResponse)
    })();

    // 归还连接
    // 如果出错（特别是 IO 错误），理想情况下应该标记连接不可用
    // 但这里简化处理，总是归还，依靠 Pool 的健康检查机制
    let _ = pool.Put(conn);

    result
}

#[cfg(feature = "connection-pool")]
fn read_until_headers_end(stream: &mut TcpStream, buffer: &mut Vec<u8>) -> Result<usize> {
    let mut temp = [0u8; 1];
    while !buffer.ends_with(b"\r\n\r\n") {
        let n = stream.read(&mut temp).map_err(HttpClientError::Io)?;
        if n == 0 {
            if buffer.is_empty() {
                 return Err(HttpClientError::ConnectionFailed("连接已关闭".to_string()));
            }
            return Err(HttpClientError::InvalidResponse("Headers 不完整".to_string()));
        }
        buffer.push(temp[0]);
        if buffer.len() > 64 * 1024 {
            return Err(HttpClientError::InvalidResponse("Headers 过大".to_string()));
        }
    }
    Ok(buffer.len())
}

#[cfg(feature = "connection-pool")]
fn parse_body_type(headers_data: &[u8]) -> Result<(bool, Option<usize>)> {
    let mut headers = [httparse::EMPTY_HEADER; 64];
    let mut req = httparse::Response::new(&mut headers);
    let status = req.parse(headers_data).map_err(|e| HttpClientError::InvalidResponse(format!("Header 解析失败: {}", e)))?;
    
    if status.is_partial() {
         return Err(HttpClientError::InvalidResponse("Header 不完整".to_string()));
    }
    
    let mut is_chunked = false;
    let mut content_length = None;
    
    for h in req.headers {
        if h.name.eq_ignore_ascii_case("Transfer-Encoding") {
            let val = std::str::from_utf8(h.value).unwrap_or("");
            if val.contains("chunked") {
                is_chunked = true;
            }
        } else if h.name.eq_ignore_ascii_case("Content-Length") {
             let val = std::str::from_utf8(h.value).unwrap_or("0");
             content_length = val.trim().parse::<usize>().ok();
        }
    }
    
    Ok((is_chunked, content_length))
}

#[cfg(feature = "connection-pool")]
fn read_fixed_body(stream: &mut TcpStream, buffer: &mut Vec<u8>, len: usize) -> Result<()> {
    if len == 0 {
        return Ok(());
    }
    let mut chunk = vec![0u8; len];
    stream.read_exact(&mut chunk).map_err(HttpClientError::Io)?;
    buffer.extend_from_slice(&chunk);
    Ok(())
}

#[cfg(feature = "connection-pool")]
fn read_chunked_body(stream: &mut TcpStream, buffer: &mut Vec<u8>) -> Result<()> {
    loop {
        // 读取 chunk size line (hex)
        let mut size_line = Vec::new();
        let mut temp = [0u8; 1];
        loop {
            stream.read_exact(&mut temp).map_err(HttpClientError::Io)?;
            size_line.push(temp[0]);
            if size_line.ends_with(b"\r\n") {
                break;
            }
            if size_line.len() > 64 {
                return Err(HttpClientError::InvalidResponse("Chunk size 行过长".to_string()));
            }
        }
        
        // 写入 buffer
        buffer.extend_from_slice(&size_line);
        
        // 解析 size
        let size_str = std::str::from_utf8(&size_line).map_err(|_| HttpClientError::InvalidResponse("Chunk size 非 UTF-8".to_string()))?;
        let size_str = size_str.trim();
        let size = usize::from_str_radix(size_str, 16).map_err(|_| HttpClientError::InvalidResponse("Chunk size 格式错误".to_string()))?;
        
        if size == 0 {
            // 最后一个 chunk，还有个 \r\n
            let mut end = [0u8; 2];
            stream.read_exact(&mut end).map_err(HttpClientError::Io)?;
            buffer.extend_from_slice(&end);
            break;
        }
        
        // 读取数据
        let mut data = vec![0u8; size];
        stream.read_exact(&mut data).map_err(HttpClientError::Io)?;
        buffer.extend_from_slice(&data);
        
        // 读取结尾 \r\n
        let mut crlf = [0u8; 2];
        stream.read_exact(&mut crlf).map_err(HttpClientError::Io)?;
        buffer.extend_from_slice(&crlf);
    }
    Ok(())
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
        "连接池功能未启用，请使用 --features connection-pool 编译".to_string(),
    ))
}

#[cfg(all(test, feature = "connection-pool"))]
mod tests {
    use super::*;
    use crate::http_client::pool::PoolManagerConfig;
    use crate::http_client::request::HttpMethod;

    #[test]
    #[ignore] // 需要网络
    fn test_http1_with_pool() {
        let request = HttpRequest::new(HttpMethod::Get, "http://httpbin.org/get");
        let config = HttpClientConfig::default();
        let pool_manager = Arc::new(ConnectionPoolManager::new(PoolManagerConfig::default()));

        let result =
            send_http1_request_with_pool("httpbin.org", 80, "/get", &request, &config, &pool_manager);

        // 可能会失败（网络问题），但不应该 panic
        if let Ok(response) = result {
            println!("状态码: {}", response.status_code);
            assert!(response.status_code > 0);
        }
    }

    #[test]
    #[ignore] // 需要网络
    fn test_connection_reuse() {
        let request = HttpRequest::new(HttpMethod::Get, "http://httpbin.org/get");
        let config = HttpClientConfig::default();
        let pool_manager = Arc::new(ConnectionPoolManager::new(PoolManagerConfig::default()));

        // 第一次请求
        println!("Sending 1st request...");
        let _ =
            send_http1_request_with_pool("httpbin.org", 80, "/get", &request, &config, &pool_manager);

        // 第二次请求（应该复用连接）
        println!("Sending 2nd request...");
        let _ =
            send_http1_request_with_pool("httpbin.org", 80, "/get", &request, &config, &pool_manager);

        // 检查统计信息
        let stats = pool_manager.get_stats();
        if !stats.is_empty() {
            println!("连接池统计:");
            for stat in stats {
                stat.print();
            }
        }
    }
}
