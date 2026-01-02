//! IPInfo.io set成module
//!
//!  from  IPInfo.io API Get IP address的detailedinfo（地理bit置、ISP 等）

use crate::dns::types::{DNSError, IPInfo};
use fingerprint_http::http_client::{HttpClient, HttpClientConfig};
use std::time::Duration;

/// IPInfo.io client
pub struct IPInfoClient {
    token: String,
    timeout: Duration,
}

impl IPInfoClient {
    /// Create a new IPInfo client
    pub fn new(token: String, timeout: Duration) -> Self {
        Self { token, timeout }
    }

    /// Get IP address的detailedinfo
    pub async fn get_ip_info(&self, ip: &str) -> Result<IPInfo, DNSError> {
        // securityFix: use HTTP Header 传递 token，而is not URL parameter
        // 这样canavoid token 泄露 to log、errormessage、proxyserver等
        let url = format!("https://ipinfo.io/{}", ip);

        // useiteminside部 HttpClient
        let config = HttpClientConfig {
            connect_timeout: self.timeout,
            read_timeout: self.timeout,
            write_timeout: self.timeout,
            ..Default::default()
        };
        let client = HttpClient::new(config);

        // Createbring有 Authorization header 的request
        use fingerprint_http::http_client::{HttpMethod, HttpRequest};
        let request = HttpRequest::new(HttpMethod::Get, &url)
            .with_header("Authorization", &format!("Bearer {}", self.token));

        //  in asyncupdown文中executesync HTTP request
        let response = tokio::task::spawn_blocking({
            let request = request.clone();
            move || client.send_request(&request)
        })
        .await
        .map_err(|e| DNSError::Http(format!("task join error: {}", e)))?
        .map_err(|e| DNSError::Http(format!("HTTP request failed: {}", e)))?;

        if !response.is_success() {
            return Err(DNSError::IPInfo(format!(
                "IPInfo API returned error: {}",
                response.status_code
            )));
        }

        // Parse JSON response
        let body_str = String::from_utf8_lossy(&response.body);
        let json: serde_json::Value = serde_json::from_str(&body_str)
            .map_err(|e| DNSError::Http(format!("failed to parse JSON: {}", e)))?;

        // Parseresponse
        Ok(IPInfo {
            ip: json["ip"].as_str().unwrap_or(ip).to_string(),
            hostname: json["hostname"].as_str().map(|s| s.to_string()),
            city: json["city"].as_str().map(|s| s.to_string()),
            region: json["region"].as_str().map(|s| s.to_string()),
            country: json["country"].as_str().map(|s| s.to_string()),
            loc: json["loc"].as_str().map(|s| s.to_string()),
            org: json["org"].as_str().map(|s| s.to_string()),
            timezone: json["timezone"].as_str().map(|s| s.to_string()),
        })
    }

    /// bulkGet IP addressinfo（concurrent）
    /// automaticdeduplicate，ensureeach IP 只queryonce
    pub async fn get_ip_infos(
        &self,
        ips: Vec<String>,
        max_concurrency: usize,
    ) -> Vec<(String, Result<IPInfo, DNSError>)> {
        use futures::stream::{self, StreamExt};
        use std::collections::HashSet;

        // pair IP listdeduplicate，ensureeach IP 只queryonce
        let unique_ips: Vec<String> = ips
            .into_iter()
            .collect::<HashSet<String>>()
            .into_iter()
            .collect();

        let tasks = stream::iter(unique_ips)
            .map(|ip| {
                let client = &self;
                async move {
                    let result = client.get_ip_info(&ip).await;
                    (ip, result)
                }
            })
            .buffer_unordered(max_concurrency);

        tasks.collect().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // needreal token，defaultskip
    async fn test_get_ip_info() {
        let _client = IPInfoClient::new("test-token".to_string(), Duration::from_secs(20));

        // thistestneedreal token
        // let result = client.get_ip_info("8.8.8.8").await;
        // assert!(result.is_ok());
    }
}
