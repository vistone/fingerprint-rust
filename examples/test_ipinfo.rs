//! 测试 IPInfo.io 集成
//!
//! 使用方法：
//!   cargo run --example test_ipinfo --features dns,rustls-tls

#[cfg(feature = "dns")]
use std::time::Duration;

#[cfg(feature = "dns")]
// 内联 IPInfoClient 实现（避免依赖 resolver）
mod ipinfo_test {
    use super::*;
    use fingerprint::http_client::{HttpClient, HttpClientConfig};
    use fingerprint::IPInfo;

    pub struct IPInfoClient {
        token: String,
        timeout: Duration,
    }

    impl IPInfoClient {
        pub fn new(token: String, timeout: Duration) -> Self {
            Self { token, timeout }
        }

        pub async fn get_ip_info(&self, ip: &str) -> Result<IPInfo, Box<dyn std::error::Error>> {
            let url = format!("https://ipinfo.io/{}?token={}", ip, self.token);

            let config = HttpClientConfig {
                connect_timeout: self.timeout,
                read_timeout: self.timeout,
                write_timeout: self.timeout,
                ..Default::default()
            };
            let client = HttpClient::new(config);

            let response = tokio::task::spawn_blocking(move || client.get(&url)).await??;

            if !response.is_success() {
                return Err(format!(
                    "HTTP {}: {}",
                    response.status_code,
                    String::from_utf8_lossy(&response.body)
                )
                .into());
            }

            let body_str = String::from_utf8_lossy(&response.body);
            let json: serde_json::Value = serde_json::from_str(&body_str)?;

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
    }
}

#[cfg(feature = "dns")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    use ipinfo_test::IPInfoClient;

    // 使用测试 token
    let token = "f6babc99a5ec26";

    println!("🔍 测试 IPInfo.io 集成...\n");
    println!("📡 Token: {}\n", token);

    let client = IPInfoClient::new(token.to_string(), Duration::from_secs(20));

    // 测试 1: 获取 Google DNS 的 IP 信息
    println!("测试 1: 获取 8.8.8.8 的 IP 信息");
    match client.get_ip_info("8.8.8.8").await {
        Ok(info) => {
            println!("✅ 成功获取 IP 信息:");
            println!("   IP: {}", info.ip);
            if let Some(ref hostname) = info.hostname {
                println!("   主机名: {}", hostname);
            }
            if let Some(ref city) = info.city {
                println!("   城市: {}", city);
            }
            if let Some(ref region) = info.region {
                println!("   地区: {}", region);
            }
            if let Some(ref country) = info.country {
                println!("   国家: {}", country);
            }
            if let Some(ref org) = info.org {
                println!("   组织: {}", org);
            }
            if let Some(ref loc) = info.loc {
                println!("   坐标: {}", loc);
            }
            if let Some(ref timezone) = info.timezone {
                println!("   时区: {}", timezone);
            }
        }
        Err(e) => {
            println!("❌ 获取失败: {}", e);
        }
    }

    println!("\n测试 2: 获取 Cloudflare DNS 的 IP 信息");
    match client.get_ip_info("1.1.1.1").await {
        Ok(info) => {
            println!("✅ 成功获取 IP 信息:");
            println!("   IP: {}", info.ip);
            if let Some(ref city) = info.city {
                println!("   城市: {}", city);
            }
            if let Some(ref country) = info.country {
                println!("   国家: {}", country);
            }
            if let Some(ref org) = info.org {
                println!("   组织: {}", org);
            }
        }
        Err(e) => {
            println!("❌ 获取失败: {}", e);
        }
    }

    println!("\n✅ 测试完成！");
    Ok(())
}

#[cfg(not(feature = "dns"))]
fn main() {
    println!("此示例需要启用 'dns' feature");
    println!("使用方法: cargo run --example test_ipinfo --features dns,rustls-tls");
}
