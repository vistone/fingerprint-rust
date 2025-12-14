//! HTTP 请求构建器

use crate::HTTPHeaders;
use std::collections::HashMap;

/// HTTP 方法
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
    Head,
    Options,
    Patch,
}

impl HttpMethod {
    pub fn as_str(&self) -> &str {
        match self {
            HttpMethod::Get => "GET",
            HttpMethod::Post => "POST",
            HttpMethod::Put => "PUT",
            HttpMethod::Delete => "DELETE",
            HttpMethod::Head => "HEAD",
            HttpMethod::Options => "OPTIONS",
            HttpMethod::Patch => "PATCH",
        }
    }
}

/// HTTP 请求
#[derive(Debug, Clone)]
pub struct HttpRequest {
    pub method: HttpMethod,
    pub url: String,
    pub headers: HashMap<String, String>,
    pub body: Option<Vec<u8>>,
}

impl HttpRequest {
    /// 创建新的请求
    pub fn new(method: HttpMethod, url: &str) -> Self {
        Self {
            method,
            url: url.to_string(),
            headers: HashMap::new(),
            body: None,
        }
    }

    /// 添加 User-Agent
    pub fn with_user_agent(mut self, user_agent: &str) -> Self {
        self.headers
            .insert("User-Agent".to_string(), user_agent.to_string());
        self
    }

    /// 添加自定义 header
    pub fn with_header(mut self, key: &str, value: &str) -> Self {
        self.headers.insert(key.to_string(), value.to_string());
        self
    }

    /// 批量添加 headers
    pub fn with_headers(mut self, headers: &HTTPHeaders) -> Self {
        let headers_map = headers.to_map();
        for (key, value) in headers_map {
            self.headers.insert(key, value);
        }
        self
    }

    /// 设置请求体
    pub fn with_body(mut self, body: Vec<u8>) -> Self {
        self.body = Some(body);
        self
    }

    /// 设置 JSON 请求体
    pub fn with_json_body(mut self, json: &str) -> Self {
        self.headers
            .insert("Content-Type".to_string(), "application/json".to_string());
        self.body = Some(json.as_bytes().to_vec());
        self
    }

    /// 构建 HTTP/1.1 请求字符串
    pub fn build_http1_request(&self, host: &str, path: &str) -> String {
        let mut request = format!("{} {} HTTP/1.1\r\n", self.method.as_str(), path);

        // Host header (必需)
        request.push_str(&format!("Host: {}\r\n", host));

        // 添加其他 headers
        for (key, value) in &self.headers {
            if key.to_lowercase() != "host" {
                request.push_str(&format!("{}: {}\r\n", key, value));
            }
        }

        // Content-Length (如果有 body)
        if let Some(ref body) = self.body {
            request.push_str(&format!("Content-Length: {}\r\n", body.len()));
        }

        // Connection: close (HTTP/1.1)
        if !self.headers.contains_key("Connection") {
            request.push_str("Connection: close\r\n");
        }

        // 结束 headers
        request.push_str("\r\n");

        // 添加 body
        if let Some(ref body) = self.body {
            request.push_str(&String::from_utf8_lossy(body));
        }

        request
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_http1_request() {
        let request = HttpRequest::new(HttpMethod::Get, "http://example.com/test")
            .with_user_agent("TestAgent/1.0")
            .with_header("Accept", "text/html");

        let http1_request = request.build_http1_request("example.com", "/test");

        assert!(http1_request.contains("GET /test HTTP/1.1"));
        assert!(http1_request.contains("Host: example.com"));
        assert!(http1_request.contains("User-Agent: TestAgent/1.0"));
        assert!(http1_request.contains("Accept: text/html"));
    }

    #[test]
    fn test_post_with_body() {
        let body = b"test data";
        let request =
            HttpRequest::new(HttpMethod::Post, "http://example.com/api").with_body(body.to_vec());

        let http1_request = request.build_http1_request("example.com", "/api");

        assert!(http1_request.contains("POST /api HTTP/1.1"));
        assert!(http1_request.contains("Content-Length: 9"));
        assert!(http1_request.contains("test data"));
    }
}
