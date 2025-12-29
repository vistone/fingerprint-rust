//! DNS 解析器模块
//!
//! 提供并发 DNS 解析功能，使用自定义 DNS 服务器列表

use crate::dns::serverpool::ServerPool;
use crate::dns::types::{DNSError, DNSResult, DomainIPs, IPInfo};
use std::net::IpAddr;
use std::sync::Arc;
use std::time::Duration;

use hickory_resolver::proto::rr::{RData, RecordType};
use hickory_resolver::{
    config::{NameServerConfig, Protocol, ResolverConfig, ResolverOpts},
    TokioAsyncResolver,
};

/// DNS 解析器
pub struct DNSResolver {
    /// DNS 查询超时时间
    timeout: Duration,
    /// DNS 服务器池
    server_pool: Arc<ServerPool>,
}

impl DNSResolver {
    /// 创建新的 DNS 解析器（使用默认 DNS 服务器）
    pub fn new(timeout: Duration) -> Self {
        Self {
            timeout,
            server_pool: Arc::new(ServerPool::default()),
        }
    }

    /// 使用指定的 DNS 服务器池创建解析器
    pub fn with_server_pool(timeout: Duration, server_pool: Arc<ServerPool>) -> Self {
        Self {
            timeout,
            server_pool,
        }
    }

    /// 解析域名的所有 IP 地址（IPv4 和 IPv6）
    pub async fn resolve(&self, domain: &str) -> Result<DNSResult, DNSError> {
        eprintln!(
            "[DNS Resolver] ========== 开始解析域名: {} ==========",
            domain
        );
        let mut domain_ips = DomainIPs::new();

        // 解析 IPv4
        eprintln!("[DNS Resolver] 开始解析 IPv4 地址...");
        if let Ok(ipv4_addrs) = self.resolve_aaaa_or_a(domain, false).await {
            eprintln!(
                "[DNS Resolver] IPv4 解析成功，获得 {} 个地址",
                ipv4_addrs.len()
            );
            domain_ips.ipv4 = ipv4_addrs;
        } else {
            eprintln!("[DNS Resolver] IPv4 解析失败");
        }

        // 解析 IPv6
        eprintln!("[DNS Resolver] 开始解析 IPv6 地址...");
        if let Ok(ipv6_addrs) = self.resolve_aaaa_or_a(domain, true).await {
            eprintln!(
                "[DNS Resolver] IPv6 解析成功，获得 {} 个地址",
                ipv6_addrs.len()
            );
            domain_ips.ipv6 = ipv6_addrs;
        } else {
            eprintln!("[DNS Resolver] IPv6 解析失败");
        }

        eprintln!(
            "[DNS Resolver] ========== 域名解析完成: {} (IPv4: {} 个, IPv6: {} 个) ==========",
            domain,
            domain_ips.ipv4.len(),
            domain_ips.ipv6.len()
        );

        Ok(DNSResult {
            domain: domain.to_string(),
            ips: domain_ips,
        })
    }

    /// 解析 IPv4 (A) 或 IPv6 (AAAA) 记录
    /// 使用收集到的全球 DNS 服务器进行查询
    async fn resolve_aaaa_or_a(&self, domain: &str, ipv6: bool) -> Result<Vec<IPInfo>, DNSError> {
        self.resolve_with_hickory(domain, ipv6).await
    }

    /// 使用 hickory-resolver 进行 DNS 查询，并发查询多个 DNS 服务器以获取所有可能的 IP
    async fn resolve_with_hickory(
        &self,
        domain: &str,
        ipv6: bool,
    ) -> Result<Vec<IPInfo>, DNSError> {
        use futures::stream::{self, StreamExt};
        use std::collections::HashSet;
        use std::net::SocketAddr;
        use std::str::FromStr;

        // 从服务器池中获取 DNS 服务器列表
        let servers = self.server_pool.servers();
        eprintln!("[DNS Resolver] 开始解析域名: {} (IPv6: {})", domain, ipv6);
        eprintln!("[DNS Resolver] 服务器池总数量: {}", servers.len());

        // 使用所有服务器并发查询（不限制数量）
        // Go 项目的 ResolveDomain 使用 pool.GetAllServers() 获取所有服务器，并发查询
        // 失败的服务器会被忽略，成功的服务器返回的 IP 会被收集并去重
        eprintln!("[DNS Resolver] 将查询所有 {} 个服务器", servers.len());

        let servers_with_sockets: Vec<_> = servers
            .iter()
            .filter_map(|server_str| {
                // 解析服务器地址格式：可以是 "ip:port" 或只有 "ip"（默认端口 53）
                let (ip_str, port) = if let Some(colon_pos) = server_str.find(':') {
                    let ip = &server_str[..colon_pos];
                    let port = server_str[colon_pos + 1..].parse::<u16>().unwrap_or(53);
                    (ip.to_string(), port)
                } else {
                    (server_str.to_string(), 53u16)
                };

                // 解析 IP 地址
                if let Ok(ip_addr) = IpAddr::from_str(&ip_str) {
                    Some((server_str.to_string(), SocketAddr::new(ip_addr, port)))
                } else {
                    None
                }
            })
            .collect();

        let total_servers = servers_with_sockets.len();
        eprintln!("[DNS Resolver] 解析后的服务器地址数量: {}", total_servers);

        if servers_with_sockets.is_empty() {
            eprintln!("[DNS Resolver] 没有可用的服务器地址，使用系统 DNS");
            return self.resolve_with_system(domain, ipv6).await;
        }

        // 记录类型
        let record_type = if ipv6 {
            RecordType::AAAA
        } else {
            RecordType::A
        };
        eprintln!("[DNS Resolver] 查询记录类型: {:?}", record_type);

        // 配置解析选项
        let mut opts = ResolverOpts::default();
        opts.timeout = Duration::from_millis(1000); // 单个服务器超时时间 1 秒
        opts.attempts = 1; // 每个服务器只尝试一次，因为我们并发查询多个
        eprintln!(
            "[DNS Resolver] 单个服务器超时: {:?}, 总体超时: {:?}",
            opts.timeout, self.timeout
        );

        // 并发查询多个 DNS 服务器
        // 使用超时包装，避免单个慢服务器阻塞整个查询
        let server_pool = self.server_pool.clone();
        let query_timeout = self.timeout; // 使用 resolver 的总体超时时间
        let query_tasks = stream::iter(servers_with_sockets)
            .map(move |(server_str, socket_addr)| {
                let domain = domain.to_string();
                let opts = opts.clone();
                let record_type = record_type;
                let server_str = server_str.clone();
                let server_pool = server_pool.clone();
                let query_timeout = query_timeout;

                async move {
                    let start_time = std::time::Instant::now();

                    // 使用超时包装查询，避免单个服务器阻塞
                    let query_result = tokio::time::timeout(query_timeout, async {
                        // 为每个服务器创建独立的 resolver
                        let mut config = ResolverConfig::new();
                        let name_server = NameServerConfig {
                            socket_addr,
                            protocol: Protocol::Udp,
                            tls_dns_name: None,
                            trust_negative_responses: false,
                            bind_addr: None,
                        };
                        config.add_name_server(name_server);

                        let resolver = TokioAsyncResolver::tokio(config, opts);
                        resolver.lookup(&domain, record_type).await
                    }).await;

                    // 执行查询
                    match query_result {
                        Ok(Ok(lookup)) => {
                            let mut ips = Vec::new();
                            let mut record_count = 0usize;

                            // 遍历所有记录，收集所有 IP 地址
                            for record in lookup.record_iter() {
                                record_count += 1;
                                if let Some(rdata) = record.data() {
                                    let ip_str = match rdata {
                                        RData::A(ipv4) if !ipv6 => {
                                            ipv4.to_string()
                                        }
                                        RData::AAAA(ipv6_addr) if ipv6 => {
                                            ipv6_addr.to_string()
                                        }
                                        _ => continue,
                                    };
                                    ips.push(ip_str);
                                }
                            }

                            // 记录成功响应时间
                            let response_time = start_time.elapsed();
                            if !ips.is_empty() {
                                // 打印详细日志，显示返回的所有 IP
                                eprintln!("[DNS Query] ✅ 服务器 {} 成功，返回 {} 个 IP（共 {} 条记录），耗时: {:?}",
                                         server_str, ips.len(), record_count, response_time);
                                if ips.len() > 1 {
                                    eprintln!("  [DNS Query] 返回的 IP 列表: {}", ips.join(", "));
                                }
                                server_pool.record_success(&server_str, response_time);
                            } else {
                                eprintln!("[DNS Query] ⚠️  服务器 {} 查询成功但未返回 IP（共 {} 条记录，但类型不匹配），耗时: {:?}",
                                         server_str, record_count, response_time);
                                server_pool.record_failure(&server_str);
                            }
                            Ok(ips)
                        }
                        Ok(Err(_)) | Err(_) => {
                            // 记录失败（查询失败或超时），不打印日志以减少输出
                            server_pool.record_failure(&server_str);
                            // 单个服务器失败不影响整体，返回空结果
                            Ok::<Vec<String>, DNSError>(Vec::new())
                        }
                    }
                }
            })
            .buffer_unordered(1000); // 增加并发数到 1000，加快查询速度

        eprintln!("[DNS Resolver] 开始并发查询，并发数: 1000");

        // 流式收集结果，等待所有服务器响应，收集尽可能多的 IP
        // 对于大量服务器，增加总体超时时间
        let overall_timeout = Duration::from_secs(30); // 总体超时 30 秒，确保所有服务器都有机会响应
        let mut all_ips = HashSet::new(); // 使用 HashSet 自动去重，相同的 IP 只会保留一个
        let mut query_tasks = query_tasks;
        let mut success_count = 0usize;
        let mut failure_count = 0usize;
        let mut total_ips_received = 0usize; // 统计收到的总 IP 数量（去重前）

        // 使用超时和流式处理，收集尽可能多的结果
        let timeout_future = tokio::time::sleep(overall_timeout);
        tokio::pin!(timeout_future);
        let start_time = std::time::Instant::now();
        let mut last_log_time = std::time::Instant::now();
        let log_interval = Duration::from_millis(500); // 每500ms打印一次进度

        eprintln!(
            "[DNS Resolver] 开始收集查询结果（总体超时: {:?}，总服务器数: {}）",
            overall_timeout, total_servers
        );

        loop {
            tokio::select! {
                // 检查是否有新的查询结果
                result = query_tasks.next() => {
                    match result {
                        Some(Ok(ips)) => {
                            success_count += 1;
                            let ips_count = ips.len(); // 先保存 IP 数量，避免移动后无法访问
                            total_ips_received += ips_count; // 统计收到的总 IP 数量（去重前）

                            let before_count = all_ips.len();
                            for ip in ips {
                                all_ips.insert(ip); // HashSet 自动去重，相同的 IP 只会保留一个
                            }
                            let after_count = all_ips.len();
                            let new_ips_count = after_count - before_count;

                            // 如果这个服务器返回的 IP 中有重复的，会在日志中显示
                            if ips_count > new_ips_count {
                                eprintln!("[DNS Resolver] 服务器返回 {} 个 IP，其中 {} 个是新 IP，{} 个是重复的（已自动去重）",
                                         ips_count, new_ips_count, ips_count - new_ips_count);
                            }

                            // 定期打印进度，显示去重统计
                            if last_log_time.elapsed() >= log_interval {
                                let duplicate_count = total_ips_received - all_ips.len();
                                eprintln!("[DNS Resolver] 进度: {}/{} 服务器完成，成功 {} 个，失败 {} 个",
                                         success_count + failure_count, total_servers, success_count, failure_count);
                                eprintln!("[DNS Resolver] IP 统计: 收到 {} 个 IP，去重后 {} 个唯一 IP，过滤了 {} 个重复 IP",
                                         total_ips_received, all_ips.len(), duplicate_count);
                                last_log_time = std::time::Instant::now();
                            }
                        }
                        Some(Err(_)) => {
                            failure_count += 1;
                            // 定期打印进度
                            if last_log_time.elapsed() >= log_interval {
                                eprintln!("[DNS Resolver] 进度: {}/{} 服务器完成，成功 {} 个，失败 {} 个，已收集 IP: {} 个",
                                         success_count + failure_count, total_servers, success_count, failure_count, all_ips.len());
                                last_log_time = std::time::Instant::now();
                            }
                            // 单个查询失败，继续
                        }
                        None => {
                            // 所有查询完成
                            let duplicate_count = total_ips_received - all_ips.len();
                            eprintln!("[DNS Resolver] ✅ 所有查询完成: 成功 {} 个，失败 {} 个",
                                     success_count, failure_count);
                            eprintln!("[DNS Resolver] IP 去重统计: 收到 {} 个 IP，去重后 {} 个唯一 IP，过滤了 {} 个重复 IP",
                                     total_ips_received, all_ips.len(), duplicate_count);
                            break;
                        }
                    }
                }
                // 超时
                _ = &mut timeout_future => {
                    let duplicate_count = total_ips_received - all_ips.len();
                    eprintln!("[DNS Resolver] ⏱️  查询总体超时（{}秒），完成 {}/{} 服务器，成功 {} 个，失败 {} 个",
                             overall_timeout.as_secs(), success_count + failure_count, total_servers, success_count, failure_count);
                    eprintln!("[DNS Resolver] IP 去重统计: 收到 {} 个 IP，去重后 {} 个唯一 IP，过滤了 {} 个重复 IP",
                             total_ips_received, all_ips.len(), duplicate_count);
                    break;
                }
            }
        }

        let total_time = start_time.elapsed();
        let duplicate_count = total_ips_received - all_ips.len();
        eprintln!("[DNS Resolver] 查询完成，总耗时: {:?}", total_time);
        eprintln!("[DNS Resolver] 最终 IP 去重统计: 收到 {} 个 IP，去重后 {} 个唯一 IP，过滤了 {} 个重复 IP（去重率: {:.2}%）",
                 total_ips_received, all_ips.len(), duplicate_count,
                 if total_ips_received > 0 { (duplicate_count as f64 / total_ips_received as f64) * 100.0 } else { 0.0 });

        // 转换为 IPInfo 列表
        // 注意：all_ips 是 HashSet，已经自动去重，相同的 IP 只会保留一个
        let ip_infos: Vec<IPInfo> = all_ips.into_iter().map(IPInfo::new).collect();

        eprintln!(
            "[DNS Resolver] 转换为 IPInfo，最终返回 {} 个唯一 IP 地址（已去重）",
            ip_infos.len()
        );

        if ip_infos.is_empty() {
            // 如果所有查询都失败，回退到系统 DNS
            eprintln!("[DNS Resolver] ⚠️  所有查询都失败，回退到系统 DNS");
            self.resolve_with_system(domain, ipv6).await
        } else {
            Ok(ip_infos)
        }
    }

    /// 使用系统 DNS 解析（回退方案）
    async fn resolve_with_system(&self, domain: &str, ipv6: bool) -> Result<Vec<IPInfo>, DNSError> {
        use std::net::ToSocketAddrs;

        let addr_str = format!("{}:80", domain);
        let mut ip_infos = Vec::new();

        if let Ok(addrs) = addr_str.to_socket_addrs() {
            for addr in addrs {
                let ip = addr.ip();
                // 根据 ipv6 参数过滤地址类型
                match (ipv6, ip) {
                    (true, IpAddr::V6(_)) => {
                        ip_infos.push(IPInfo::new(ip.to_string()));
                    }
                    (false, IpAddr::V4(_)) => {
                        ip_infos.push(IPInfo::new(ip.to_string()));
                    }
                    _ => {
                        // 不匹配的类型，跳过
                    }
                }
            }
        }

        Ok(ip_infos)
    }

    /// 批量解析域名（并发）
    pub async fn resolve_many(
        &self,
        domains: Vec<String>,
        max_concurrency: usize,
    ) -> Vec<(String, Result<DNSResult, DNSError>)> {
        use futures::stream::{self, StreamExt};

        let tasks = stream::iter(domains)
            .map(|domain| {
                let resolver = self;
                async move {
                    let result = resolver.resolve(&domain).await;
                    (domain, result)
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
    async fn test_resolve() {
        let resolver = DNSResolver::new(Duration::from_secs(4));
        let result = resolver.resolve("google.com").await;
        assert!(result.is_ok());
        let dns_result = result.unwrap();
        assert!(!dns_result.ips.ipv4.is_empty() || !dns_result.ips.ipv6.is_empty());
    }
}
