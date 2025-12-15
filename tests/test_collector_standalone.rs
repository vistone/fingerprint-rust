//! 独立测试 DNS 服务器收集器（不依赖库的其他模块）

#[cfg(feature = "dns")]
#[tokio::test]
async fn test_collect_public_dns_standalone() {
    use std::time::Duration;
    use std::net::{IpAddr, SocketAddr};

    async fn collect_public_dns() -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let timeout = Duration::from_secs(30);
        let url = "https://public-dns.info/nameservers.txt";

        let client = reqwest::Client::builder()
            .timeout(timeout)
            .build()?;

        let response = client.get(url).send().await?;

        if !response.status().is_success() {
            return Err(format!("HTTP {}", response.status()).into());
        }

        let text = response.text().await?;

        // 解析文本，每行一个 IP 地址
        let mut servers = Vec::new();
        for line in text.lines() {
            let line = line.trim();
            
            // 跳过空行和注释
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            // 验证是否为有效的 IP 地址
            fn is_valid_ip(s: &str) -> bool {
                // 如果包含端口号，尝试解析 SocketAddr
                if s.contains(':') && s.matches(':').count() <= 2 {
                    if s.parse::<SocketAddr>().is_ok() {
                        return true;
                    }
                    if s.starts_with('[') {
                        return s.parse::<SocketAddr>().is_ok();
                    }
                }
                // 尝试解析为 IP 地址
                s.parse::<IpAddr>().is_ok()
            }

            if is_valid_ip(line) {
                // 如果没有端口，添加默认端口 53
                let server = if line.contains(':') {
                    line.to_string()
                } else {
                    format!("{}:53", line)
                };
                servers.push(server);
            }
        }

        Ok(servers)
    }

    // 运行测试
    match collect_public_dns().await {
        Ok(servers) => {
            println!("✅ 成功获取 DNS 服务器列表");
            println!("   服务器数量: {}", servers.len());
            assert!(!servers.is_empty(), "应该至少获取到一个 DNS 服务器");
            
            // 显示前 10 个
            println!("   前 10 个服务器:");
            for (i, server) in servers.iter().take(10).enumerate() {
                println!("     {}. {}", i + 1, server);
            }
        }
        Err(e) => {
            panic!("获取 DNS 服务器失败: {}", e);
        }
    }
}

