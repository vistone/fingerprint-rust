//! HTTP 响应解析

use std::collections::HashMap;

/// HTTP 响应
#[derive(Debug, Clone)]
pub struct HttpResponse {
    pub status_code: u16,
    pub status_text: String,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
    pub http_version: String,
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
        }
    }

    /// 从原始响应解析
    pub fn parse(raw_response: &[u8]) -> Result<Self, String> {
        let response_str = String::from_utf8_lossy(raw_response);
        let parts: Vec<&str> = response_str.splitn(2, "\r\n\r\n").collect();
        
        if parts.len() < 2 {
            return Err("无效的 HTTP 响应".to_string());
        }

        let header_section = parts[0];
        let body_section = parts[1];

        // 解析状态行和 headers
        let mut lines = header_section.lines();
        
        // 解析状态行: HTTP/1.1 200 OK
        let status_line = lines.next().ok_or("缺少状态行")?;
        let status_parts: Vec<&str> = status_line.splitn(3, ' ').collect();
        
        if status_parts.len() < 2 {
            return Err("无效的状态行".to_string());
        }

        let http_version = status_parts[0].to_string();
        let status_code = status_parts[1]
            .parse::<u16>()
            .map_err(|_| "无效的状态码".to_string())?;
        let status_text = status_parts.get(2).unwrap_or(&"").to_string();

        // 解析 headers
        let mut headers = HashMap::new();
        for line in lines {
            if let Some(pos) = line.find(':') {
                let key = line[..pos].trim().to_string();
                let value = line[pos + 1..].trim().to_string();
                headers.insert(key, value);
            }
        }

        // Body
        let body = body_section.as_bytes().to_vec();

        Ok(Self {
            status_code,
            status_text,
            headers,
            body,
            http_version,
        })
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
