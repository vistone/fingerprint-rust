// ! 独立testing DNS service器收集器（不依赖libraryofothermodule）

#[cfg(feature = "dns")]
#[tokio::test]
#[ignore = "requires external network access to public-dns.info"]
async fn test_collect_public_dns_standalone() {
    use std::net::{IpAddr, SocketAddr};
    use std::time::Duration;

    async fn collect_public_dns() -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let timeout = Duration::from_secs(30);
        let url = "https://public-dns.info/nameservers.txt";

        // use项目内部of HttpClient
        let config = fingerprint::HttpClientConfig {
            connect_timeout: timeout,
            read_timeout: timeout,
            write_timeout: timeout,
            ..Default::default()
        };
        let client = fingerprint::HttpClient::new(config);

        // 在async上下文中执行sync的 HTTP 请求
        let response = client
            .get(url)
            .map_err(|e| format!("HTTP request failed: {}", e))?;

        if !response.is_success() {
            return Err(format!("HTTP {}", response.status_code).into());
        }

        // 读取响应文本
        let text = String::from_utf8_lossy(&response.body).to_string();

        // parse文本，每行一个 IP address
        let mut servers = Vec::new();
        for line in text.lines() {
            let line = line.trim();

            // 跳过空行andcomment
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            // validate是否to有效of IP address
            fn is_valid_ip(s: &str) -> bool {
                // 如果includeport号，尝试parse SocketAddr
                if s.contains(':') && s.matches(':').count() <= 2 {
                    if s.parse::<SocketAddr>().is_ok() {
                        return true;
                    }
                    if s.starts_with('[') {
                        return s.parse::<SocketAddr>().is_ok();
                    }
                }
                // 尝试parseto IP address
                s.parse::<IpAddr>().is_ok()
            }

            if is_valid_ip(line) {
                // 如果没有port，添加defaultport 53
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

    // runtesting
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
