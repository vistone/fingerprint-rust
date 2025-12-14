//! HTTP 响应解析
//! 
//! 支持：
//! - chunked encoding
//! - gzip/deflate 压缩
//! - 完整的 HTTP/1.1 响应解析

use std::collections::HashMap;
use std::io::Read;

/// HTTP 响应
#[derive(Debug, Clone)]
pub struct HttpResponse {
    pub status_code: u16,
    pub status_text: String,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
    pub http_version: String,
    pub response_time_ms: u64,  // 响应时间
}

impl HttpResponse {
    /// 创建新的响应
    pub fn new(status_code: u16) -> Self {
        Self {
            status_code,
            status_text: String::new(),
            headers: HashMap::new(),
            body: Vec::new(),
            http_version: "HTTP/1.1".to_string(),
            response_time_ms: 0,
        }
    }

    /// 从原始响应解析（完整版本）
    pub fn parse(raw_response: &[u8]) -> Result<Self, String> {
        let start = std::time::Instant::now();
        
        // 1. 分离 headers 和 body
        let (headers_end, body_start) = Self::find_headers_end(raw_response)?;
        
        let header_bytes = &raw_response[..headers_end];
        let body_bytes = &raw_response[body_start..];
        
        // 2. 解析 headers
        let header_str = String::from_utf8_lossy(header_bytes);
        let mut lines = header_str.lines();
        
        // 3. 解析状态行: HTTP/1.1 200 OK
        let status_line = lines.next().ok_or("缺少状态行")?;
        let (http_version, status_code, status_text) = Self::parse_status_line(status_line)?;
        
        // 4. 解析 headers
        let headers = Self::parse_headers(lines)?;
        
        // 5. 处理 body
        let body = Self::process_body(body_bytes, &headers)?;
        
        let response_time_ms = start.elapsed().as_millis() as u64;
        
        Ok(Self {
            status_code,
            status_text,
            headers,
            body,
            http_version,
            response_time_ms,
        })
    }
    
    /// 查找 headers 结束位置（\r\n\r\n）
    fn find_headers_end(data: &[u8]) -> Result<(usize, usize), String> {
        for i in 0..data.len().saturating_sub(3) {
            if &data[i..i + 4] == b"\r\n\r\n" {
                return Ok((i, i + 4));
            }
        }
        Err("未找到 headers 结束标记".to_string())
    }
    
    /// 解析状态行
    fn parse_status_line(line: &str) -> Result<(String, u16, String), String> {
        let parts: Vec<&str> = line.splitn(3, ' ').collect();
        
        if parts.len() < 2 {
            return Err(format!("无效的状态行: {}", line));
        }
        
        let http_version = parts[0].to_string();
        let status_code = parts[1]
            .parse::<u16>()
            .map_err(|_| format!("无效的状态码: {}", parts[1]))?;
        let status_text = parts.get(2).unwrap_or(&"").to_string();
        
        Ok((http_version, status_code, status_text))
    }
    
    /// 解析 headers
    fn parse_headers<'a, I>(lines: I) -> Result<HashMap<String, String>, String>
    where
        I: Iterator<Item = &'a str>,
    {
        let mut headers = HashMap::new();
        
        for line in lines {
            if line.is_empty() {
                continue;
            }
            
            if let Some(pos) = line.find(':') {
                let key = line[..pos].trim().to_lowercase(); // 转小写便于查找
                let value = line[pos + 1..].trim().to_string();
                headers.insert(key, value);
            }
        }
        
        Ok(headers)
    }
    
    /// 处理响应体（支持 chunked 和压缩）
    fn process_body(body_bytes: &[u8], headers: &HashMap<String, String>) -> Result<Vec<u8>, String> {
        let mut body = body_bytes.to_vec();
        
        // 1. 处理 Transfer-Encoding: chunked
        if let Some(te) = headers.get("transfer-encoding") {
            if te.contains("chunked") {
                body = Self::parse_chunked(&body)?;
            }
        }
        
        // 2. 处理 Content-Encoding
        if let Some(ce) = headers.get("content-encoding") {
            body = Self::decompress(&body, ce)?;
        }
        
        Ok(body)
    }
    
    /// 解析 chunked encoding
    fn parse_chunked(data: &[u8]) -> Result<Vec<u8>, String> {
        let mut result = Vec::new();
        let mut pos = 0;
        
        loop {
            // 查找 chunk size 行的结束（\r\n）
            let size_line_end = data[pos..]
                .windows(2)
                .position(|w| w == b"\r\n")
                .ok_or("Invalid chunked encoding: missing CRLF after size")?;
            
            // 解析 chunk size（十六进制）
            let size_str = std::str::from_utf8(&data[pos..pos + size_line_end])
                .map_err(|_| "Invalid chunk size: not UTF-8")?;
            
            // 移除可能的扩展参数（如 "3b; name=value"）
            let size_str = size_str.split(';').next().unwrap_or(size_str).trim();
            
            let size = usize::from_str_radix(size_str, 16)
                .map_err(|e| format!("Invalid chunk size '{}': {}", size_str, e))?;
            
            // size = 0 表示最后一个 chunk
            if size == 0 {
                break;
            }
            
            // 跳过 size 行和 \r\n
            pos += size_line_end + 2;
            
            // 检查是否有足够的数据
            if pos + size > data.len() {
                return Err(format!("Chunk size {} exceeds available data", size));
            }
            
            // 提取 chunk data
            result.extend_from_slice(&data[pos..pos + size]);
            pos += size;
            
            // 跳过 chunk 后面的 \r\n
            if pos + 2 <= data.len() && &data[pos..pos + 2] == b"\r\n" {
                pos += 2;
            } else {
                return Err("Invalid chunked encoding: missing CRLF after chunk data".to_string());
            }
        }
        
        Ok(result)
    }
    
    /// 解压缩响应体
    fn decompress(data: &[u8], encoding: &str) -> Result<Vec<u8>, String> {
        match encoding.to_lowercase().as_str() {
            "gzip" => Self::decompress_gzip(data),
            "deflate" => Self::decompress_deflate(data),
            "br" => Self::decompress_brotli(data),
            "identity" | "" => Ok(data.to_vec()),
            _ => Err(format!("不支持的编码: {}", encoding)),
        }
    }
    
    /// 解压 gzip
    fn decompress_gzip(data: &[u8]) -> Result<Vec<u8>, String> {
        use flate2::read::GzDecoder;
        
        let mut decoder = GzDecoder::new(data);
        let mut result = Vec::new();
        decoder
            .read_to_end(&mut result)
            .map_err(|e| format!("gzip 解压失败: {}", e))?;
        Ok(result)
    }
    
    /// 解压 deflate
    fn decompress_deflate(data: &[u8]) -> Result<Vec<u8>, String> {
        use flate2::read::DeflateDecoder;
        
        let mut decoder = DeflateDecoder::new(data);
        let mut result = Vec::new();
        decoder
            .read_to_end(&mut result)
            .map_err(|e| format!("deflate 解压失败: {}", e))?;
        Ok(result)
    }
    
    /// 解压 brotli
    fn decompress_brotli(_data: &[u8]) -> Result<Vec<u8>, String> {
        // TODO: 添加 brotli 支持
        // 需要添加 brotli crate 依赖
        Err("brotli 解压暂未实现".to_string())
    }

    /// 获取响应体为字符串
    pub fn body_as_string(&self) -> Result<String, std::string::FromUtf8Error> {
        String::from_utf8(self.body.clone())
    }

    /// 检查是否成功
    pub fn is_success(&self) -> bool {
        self.status_code >= 200 && self.status_code < 300
    }

    /// 获取 header
    pub fn get_header(&self, key: &str) -> Option<&String> {
        self.headers.get(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_response() {
        let raw = b"HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: 11\r\n\r\nHello World";
        
        let response = HttpResponse::parse(raw).unwrap();
        
        assert_eq!(response.status_code, 200);
        assert_eq!(response.status_text, "OK");
        assert_eq!(response.get_header("Content-Type"), Some(&"text/html".to_string()));
        assert_eq!(response.body_as_string().unwrap(), "Hello World");
        assert!(response.is_success());
    }

    #[test]
    fn test_parse_error_response() {
        let raw = b"HTTP/1.1 404 Not Found\r\n\r\n";
        
        let response = HttpResponse::parse(raw).unwrap();
        
        assert_eq!(response.status_code, 404);
        assert_eq!(response.status_text, "Not Found");
        assert!(!response.is_success());
    }
}
