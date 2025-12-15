//! DNS 域名解析示例
//!
//! 使用方法：
//!   cargo run --example resolve_domains --features dns,rustls-tls

#[cfg(feature = "dns")]
use fingerprint::dns::{
    load_domain_ips, save_domain_ips, DNSResolver, DomainIPs, IPInfoClient, ServerCollector,
};
#[cfg(feature = "dns")]
use std::collections::HashSet;
#[cfg(feature = "dns")]
use std::path::PathBuf;
#[cfg(feature = "dns")]
use std::sync::Arc;
#[cfg(feature = "dns")]
use std::time::Duration;

#[cfg(feature = "dns")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 DNS 域名解析示例");
    println!("==================\n");

    // IPInfo token
    let token = "f6babc99a5ec26";
    let ipinfo_client = IPInfoClient::new(token.to_string(), Duration::from_secs(20));

    // 要解析的域名列表
    let domains = vec!["kh.google.com", "khmdb.google.com"];

    // 收集全球 DNS 服务器
    println!("📡 正在收集全球 DNS 服务器...");
    let server_pool = ServerCollector::collect_all(Some(Duration::from_secs(30))).await;
    println!("✅ 已收集 {} 个 DNS 服务器", server_pool.len());

    // 使用收集到的 DNS 服务器创建解析器
    let resolver = DNSResolver::with_server_pool(Duration::from_secs(4), Arc::new(server_pool));

    // 创建输出目录
    let output_dir = PathBuf::from("./dns_output");
    std::fs::create_dir_all(&output_dir)?;

    println!("📡 开始解析域名...\n");

    // 解析每个域名
    for domain in &domains {
        println!("解析域名: {}", domain);

        // DNS 解析
        let dns_result = resolver.resolve(domain).await?;
        println!(
            "  ✅ DNS 解析完成: {} 个 IPv4, {} 个 IPv6",
            dns_result.ips.ipv4.len(),
            dns_result.ips.ipv6.len()
        );

        // 加载已存在的 IP 信息（用于去重，避免重复查询 IPInfo）
        println!("  📂 加载本地存储的 IP 信息...");
        let existing = load_domain_ips(domain, &output_dir)?;

        // 提取所有解析到的 IP（DNS 解析结果已去重）
        let all_ipv4: HashSet<String> = dns_result
            .ips
            .ipv4
            .iter()
            .map(|ip_info| ip_info.ip.clone())
            .collect();
        let all_ipv6: HashSet<String> = dns_result
            .ips
            .ipv6
            .iter()
            .map(|ip_info| ip_info.ip.clone())
            .collect();

        // 从本地存储中提取已存在的 IP
        let existing_ipv4: HashSet<String> = existing
            .as_ref()
            .map(|e| e.ipv4.iter().map(|ip| ip.ip.clone()).collect())
            .unwrap_or_default();
        let existing_ipv6: HashSet<String> = existing
            .as_ref()
            .map(|e| e.ipv6.iter().map(|ip| ip.ip.clone()).collect())
            .unwrap_or_default();

        // 找出新发现的 IP（与本地存储去重后，只查询这些新 IP）
        let new_ipv4: Vec<String> = all_ipv4.difference(&existing_ipv4).cloned().collect();
        let new_ipv6: Vec<String> = all_ipv6.difference(&existing_ipv6).cloned().collect();

        println!("  📊 IP 统计（已与本地存储去重）:");
        println!(
            "     IPv4: 总数 {} 个，本地已存在 {} 个，新发现 {} 个（将查询这 {} 个）",
            all_ipv4.len(),
            existing_ipv4.len(),
            new_ipv4.len(),
            new_ipv4.len()
        );
        println!(
            "     IPv6: 总数 {} 个，本地已存在 {} 个，新发现 {} 个（将查询这 {} 个）",
            all_ipv6.len(),
            existing_ipv6.len(),
            new_ipv6.len(),
            new_ipv6.len()
        );

        // 构建最终的 domain_ips，先复制已存在的数据
        let mut domain_ips = DomainIPs::new();

        // 复制已存在的 IPv4 信息
        if let Some(existing) = &existing {
            for existing_ip in &existing.ipv4 {
                if all_ipv4.contains(&existing_ip.ip) {
                    domain_ips.ipv4.push(existing_ip.clone());
                }
            }
        }

        // 复制已存在的 IPv6 信息
        if let Some(existing) = &existing {
            for existing_ip in &existing.ipv6 {
                if all_ipv6.contains(&existing_ip.ip) {
                    domain_ips.ipv6.push(existing_ip.clone());
                }
            }
        }

        // 只查询新发现的 IPv4 的详细信息（已与本地存储去重）
        if !new_ipv4.is_empty() {
            println!(
                "  📡 获取新发现的 IPv4 详细信息（{} 个 IP，已去重，并发处理）...",
                new_ipv4.len()
            );
            let ipv4_results = ipinfo_client.get_ip_infos(new_ipv4.clone(), 50).await;
            eprintln!("  [IPInfo] IPv4 查询完成: {} 个结果", ipv4_results.len());
            for (ip, ip_result) in ipv4_results {
                match ip_result {
                    Ok(mut ip_info) => {
                        // 保留原始 IP（因为 IPInfo 可能返回不同的格式）
                        ip_info.ip = ip.clone();
                        domain_ips.ipv4.push(ip_info);
                    }
                    Err(e) => {
                        eprintln!("  [IPInfo] ⚠️  获取 {} 的详细信息失败: {}", ip, e);
                        // 即使失败，也保存基本 IP 信息
                        domain_ips.ipv4.push(fingerprint::dns::IPInfo::new(ip));
                    }
                }
            }
            eprintln!(
                "  [IPInfo] IPv4 详细信息获取完成: {} 个",
                domain_ips.ipv4.len()
            );
        } else {
            println!("  ✅ IPv4 没有新发现的 IP，跳过 IPInfo 查询");
        }

        // 只查询新发现的 IPv6 的详细信息（已与本地存储去重）
        if !new_ipv6.is_empty() {
            println!(
                "  📡 获取新发现的 IPv6 详细信息（{} 个 IP，已去重，并发处理）...",
                new_ipv6.len()
            );
            let ipv6_results = ipinfo_client.get_ip_infos(new_ipv6.clone(), 50).await;
            eprintln!("  [IPInfo] IPv6 查询完成: {} 个结果", ipv6_results.len());
            for (ip, ip_result) in ipv6_results {
                match ip_result {
                    Ok(mut ip_info) => {
                        ip_info.ip = ip.clone();
                        domain_ips.ipv6.push(ip_info);
                    }
                    Err(e) => {
                        eprintln!("  [IPInfo] ⚠️  获取 {} 的详细信息失败: {}", ip, e);
                        domain_ips.ipv6.push(fingerprint::dns::IPInfo::new(ip));
                    }
                }
            }
            eprintln!(
                "  [IPInfo] IPv6 详细信息获取完成: {} 个",
                domain_ips.ipv6.len()
            );
        } else {
            println!("  ✅ IPv6 没有新发现的 IP，跳过 IPInfo 查询");
        }

        println!(
            "  ✅ IP 信息获取完成: {} 个 IPv4, {} 个 IPv6\n",
            domain_ips.ipv4.len(),
            domain_ips.ipv6.len()
        );

        // 保存为三种格式（save_domain_ips 会同时保存 JSON、YAML、TOML）
        save_domain_ips(domain, &domain_ips, &output_dir)?;
        println!("  ✅ JSON 已保存: {}/{}.json", output_dir.display(), domain);
        println!("  ✅ YAML 已保存: {}/{}.yaml", output_dir.display(), domain);
        println!("  ✅ TOML 已保存: {}/{}.toml", output_dir.display(), domain);

        println!();
    }

    println!("🎉 所有域名解析完成！");
    println!("📁 输出目录: {}", output_dir.display());

    // 显示 JSON 示例内容
    println!("\n📄 JSON 格式示例:");
    let json_path = output_dir.join("kh.google.com.json");
    if json_path.exists() {
        let content = std::fs::read_to_string(&json_path)?;
        println!("{}", content);
    }

    Ok(())
}

#[cfg(not(feature = "dns"))]
fn main() {
    println!("此示例需要启用 'dns' feature");
    println!("使用方法: cargo run --example resolve_domains --features dns,rustls-tls");
}
