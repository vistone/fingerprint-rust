//! 独立测试 DNS 服务器收集器（不依赖 resolver）
//!
//! 使用方法：
//!   cargo run --example test_collector_only --features dns

//! 独立测试 DNS 服务器收集器（不依赖 resolver）
//!
//! 使用方法：
//!   cargo run --example test_collector_only --features dns,rustls-tls

#[cfg(feature = "dns")]

// 由于 resolver 模块有编译问题，我们直接复制 collector 的核心逻辑来测试
#[cfg(feature = "dns")]
async fn test_collect_public_dns() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    use std::time::Duration;
    let timeout = Duration::from_secs(30);
    let url = "https://public-dns.info/nameservers.txt";

    let client = reqwest::Client::builder()
        .timeout(timeout)
        .build()?;

    let response = client.get(url).send().await?;

    if !response.status().is_success() {
        return Err(format!("HTTP {}: {}", response.status(), response.status().canonical_reason().unwrap_or("")).into());
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
        if is_valid_ip_address(line) {
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

#[allow(dead_code)]
fn is_valid_ip_address(s: &str) -> bool {
    use std::net::{IpAddr, SocketAddr};
    
    // 如果包含端口号，先解析 SocketAddr
    if s.contains(':') && s.matches(':').count() <= 2 {
        // 可能是 IPv4:port 格式
        if s.parse::<SocketAddr>().is_ok() {
            return true;
        }
        // 也可能是 IPv6:port，但格式更复杂，需要特殊处理
        // 简化处理：如果包含 []，尝试解析
        if s.starts_with('[') {
            return s.parse::<SocketAddr>().is_ok();
        }
    }
    
    // 尝试解析为 IP 地址
    s.parse::<IpAddr>().is_ok()
}

#[cfg(feature = "dns")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 测试从 public-dns.info 获取 DNS 服务器列表...\n");

    match test_collect_public_dns().await {
        Ok(servers) => {
            println!("✅ 成功获取 DNS 服务器列表");
            println!("   服务器数量: {}\n", servers.len());
            
            // 显示前 20 个服务器
            let display_count = servers.len().min(20);
            println!("前 {} 个服务器:", display_count);
            for (i, server) in servers.iter().take(display_count).enumerate() {
                println!("  {}. {}", i + 1, server);
            }
            if servers.len() > display_count {
                println!("  ... (还有 {} 个)", servers.len() - display_count);
            }

            // 验证 IP 地址格式
            println!("\n📊 统计信息:");
            let ipv4_count = servers.iter()
                .filter(|s| s.parse::<std::net::Ipv4Addr>().is_ok() || 
                           s.starts_with(|c: char| c.is_ascii_digit()))
                .count();
            println!("   IPv4 服务器: {} (估算)", ipv4_count);
            println!("   总服务器数: {}", servers.len());
        }
        Err(e) => {
            println!("❌ 获取失败: {}", e);
            return Err(e);
        }
    }

    println!("\n✅ 测试完成！");
    Ok(())
}

#[cfg(not(feature = "dns"))]
fn main() {
    println!("此示例需要启用 'dns' feature");
    println!("使用方法: cargo run --example test_collector_only --features dns,rustls-tls");
}

