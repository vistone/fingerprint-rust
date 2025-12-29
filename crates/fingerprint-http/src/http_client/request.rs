//! HTTP 请求构建器

use fingerprint_headers::headers::HTTPHeaders;
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
    ///
    /// 注意：该方法会把 body 当作 UTF-8 文本拼接到字符串中，**不适用于二进制 body**。
    /// 如需发送二进制数据，请使用 `build_http1_request_bytes`。
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
        if !self
            .headers
            .keys()
            .any(|k| k.eq_ignore_ascii_case("connection"))
        {
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

    /// 构建 HTTP/1.1 请求字节（推荐）
    ///
    /// - **输入**：`host`（用于 Host 头）、`path`（请求路径，包含 query 也可）
    /// - **输出**：完整的 HTTP/1.1 请求 bytes（headers + body）
    ///
    /// 相比 `build_http1_request`，该方法不会对 body 做 UTF-8 假设，适用于二进制 body。
    pub fn build_http1_request_bytes(&self, host: &str, path: &str) -> Vec<u8> {
        let mut head = format!("{} {} HTTP/1.1\r\n", self.method.as_str(), path);

        // Host header (必需)
        head.push_str(&format!("Host: {}\r\n", host));

        // 添加其他 headers（排除 Host）
        for (key, value) in &self.headers {
            if !key.eq_ignore_ascii_case("host") {
                head.push_str(&format!("{}: {}\r\n", key, value));
            }
        }

        // Content-Length (如果有 body)
        let body_len = self.body.as_ref().map(|b| b.len()).unwrap_or(0);
        if body_len > 0 {
            head.push_str(&format!("Content-Length: {}\r\n", body_len));
        }

        // Connection: close (默认)
        if !self
            .headers
            .keys()
            .any(|k| k.eq_ignore_ascii_case("connection"))
        {
            head.push_str("Connection: close\r\n");
        }

        // 结束 headers
        head.push_str("\r\n");

        let mut out = head.into_bytes();
        if let Some(ref body) = self.body {
            out.extend_from_slice(body);
        }
        out
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
