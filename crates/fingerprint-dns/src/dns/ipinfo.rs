//! IPInfo.io setbecomemodule
//!
//! from IPInfo.io API Get IP addressdetailedinfo (geographicbitplace, ISP etc.)

use crate::dns::types::{DNSError, IPInfo};
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

    /// Get IP addressdetailedinfo
    pub async fn get_ip_info(&self, ip: &str) -> Result<IPInfo, DNSError> {
        // securityFix: use HTTP Header pass token, 而is not URL parameter
        // this waycanavoid token leak to log, errormessage, proxyserver etc.
        let url = format!("https://ipinfo.io/{}", ip);

        // use standard HTTP client to avoid Sync issues
        let client = reqwest::Client::builder()
            .timeout(self.timeout)
            .build()
            .map_err(|e| DNSError::Http(format!("failed to create HTTP client: {}", e)))?;

        let response = client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .send()
            .await
            .map_err(|e| DNSError::Http(format!("HTTP request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(DNSError::IPInfo(format!(
                "IPInfo API returned error: {}",
                response.status()
            )));
        }

        // Parse JSON response
        let json: serde_json::Value = response
            .json()
            .await
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

    /// bulkGet IP addressinfo (concurrent)
    /// automaticdeduplicate, ensureeach IP onlyqueryonce
    pub async fn get_ip_infos(
        &self,
        ips: Vec<String>,
    ) -> Result<std::collections::HashMap<String, IPInfo>, DNSError> {
        use futures::stream::{self, StreamExt};

        // deduplicateinput IP
        let unique_ips: std::collections::HashSet<String> = ips.into_iter().collect();

        // concurrentquery (limit 10 concurrency)
        let results: Result<std::collections::HashMap<String, IPInfo>, DNSError> =
            stream::iter(unique_ips)
                .map(|ip| {
                    let client = self.clone();
                    async move { client.get_ip_info(&ip).await.map(|info| (ip.clone(), info)) }
                })
                .buffer_unordered(10)
                .collect::<Vec<Result<(String, IPInfo), DNSError>>>()
                .await
                .into_iter()
                .collect();

        results
    }
}

impl Clone for IPInfoClient {
    fn clone(&self) -> Self {
        Self {
            token: self.token.clone(),
            timeout: self.timeout,
        }
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
