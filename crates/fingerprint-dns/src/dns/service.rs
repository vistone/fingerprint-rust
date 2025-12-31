//! DNS Service 模块
//!
//! 提供 DNS 解析服务的 Start/Stop 接口

use crate::dns::collector::ServerCollector;
use crate::dns::config::load_config;
use crate::dns::ipinfo::IPInfoClient;
use crate::dns::resolver::DNSResolver;
use crate::dns::serverpool::ServerPool;
use crate::dns::storage::{load_domain_ips, save_domain_ips};
use crate::dns::types::IPInfo;
use crate::dns::types::{DNSConfig, DNSError, DomainIPs};
use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{oneshot, RwLock};
use tokio::time::sleep;

/// DNS Service（对应 Go 版本的 Service）
pub struct Service {
    config: Arc<DNSConfig>,
    resolver: Arc<RwLock<DNSResolver>>, // 使用 RwLock 以便在 start 时更新
    ipinfo_client: Arc<IPInfoClient>,
    running: Arc<RwLock<bool>>,
    stop_tx: Arc<RwLock<Option<oneshot::Sender<()>>>>,
}

impl Service {
    /// 创建新的 Service 实例
    pub fn new(config: DNSConfig) -> Result<Self, DNSError> {
        config.validate()?;

        // 解析超时时间
        let dns_timeout = parse_duration(&config.dns_timeout).unwrap_or(Duration::from_secs(4));

        // HTTP 超时时间
        let http_timeout = parse_duration(&config.http_timeout).unwrap_or(Duration::from_secs(20));

        // 使用默认 DNS 服务器创建 resolver（将在 start 时替换为收集到的服务器）
        let resolver = Arc::new(RwLock::new(DNSResolver::new(dns_timeout)));
        let ipinfo_client = Arc::new(IPInfoClient::new(config.ipinfo_token.clone(), http_timeout));

        Ok(Self {
            config: Arc::new(config),
            resolver,
            ipinfo_client,
            running: Arc::new(RwLock::new(false)),
            stop_tx: Arc::new(RwLock::new(None)),
        })
    }

    /// 创建新的 Service 实例，并使用指定的 DNS 服务器池
    pub async fn with_server_pool(
        config: DNSConfig,
        server_pool: Arc<ServerPool>,
    ) -> Result<Self, DNSError> {
        config.validate()?;

        // 解析超时时间
        let dns_timeout = parse_duration(&config.dns_timeout).unwrap_or(Duration::from_secs(4));

        // HTTP 超时时间
        let http_timeout = parse_duration(&config.http_timeout).unwrap_or(Duration::from_secs(20));

        // 使用指定的 DNS 服务器池创建 resolver
        let resolver = Arc::new(RwLock::new(DNSResolver::with_server_pool(
            dns_timeout,
            server_pool,
        )));
        let ipinfo_client = Arc::new(IPInfoClient::new(config.ipinfo_token.clone(), http_timeout));

        Ok(Self {
            config: Arc::new(config),
            resolver,
            ipinfo_client,
            running: Arc::new(RwLock::new(false)),
            stop_tx: Arc::new(RwLock::new(None)),
        })
    }

    /// 从配置文件创建 Service
    pub fn from_config_file<P: AsRef<std::path::Path>>(path: P) -> Result<Self, DNSError> {
        let config = load_config(path)?;
        Self::new(config)
    }

    /// 启动服务（在后台线程运行，不阻塞主线程）
    /// 自动维护 DNS 服务器池（dnsservernames.json），无需人工干预
    pub async fn start(&self) -> Result<(), DNSError> {
        // 检查是否已经在运行
        {
            let mut running = self.running.write().await;
            if *running {
                return Err(DNSError::Config("service is already running".to_string()));
            }
            *running = true;
        }

        // 加载/收集 DNS 服务器池（对应 Go 的 NewServerPool）
        // 优先从本地文件加载，如果不存在或为空，才从网络收集
        // collect_all 已经处理了：
        //   - 如果文件存在且不为空：直接使用，不进行检查
        //   - 如果文件不存在或为空：从网络收集并进行健康检查后保存
        let mut server_pool = ServerCollector::collect_all(Some(Duration::from_secs(30))).await;
        eprintln!("当前服务器池有 {} 个 DNS 服务器", server_pool.len());

        // 如果服务器池为空，使用默认服务器
        if server_pool.is_empty() {
            eprintln!("Warning: 没有可用的 DNS 服务器，使用默认服务器");
            server_pool = ServerPool::default();
            eprintln!("使用默认 DNS 服务器: {} 个", server_pool.len());
        }

        // 使用通过健康检查的服务器池更新 resolver
        // 解析时使用所有通过健康检查的服务器并发查询
        let dns_timeout =
            parse_duration(&self.config.dns_timeout).unwrap_or(Duration::from_secs(4));
        let server_pool_arc = Arc::new(server_pool);
        let new_resolver = DNSResolver::with_server_pool(dns_timeout, server_pool_arc.clone());

        // 替换 resolver
        {
            let mut resolver_guard = self.resolver.write().await;
            *resolver_guard = new_resolver;
        }

        // 创建停止通道
        let (tx, mut rx) = oneshot::channel();
        {
            let mut stop_tx = self.stop_tx.write().await;
            *stop_tx = Some(tx);
        }

        // 启动后台任务：定期淘汰慢的DNS服务器（不阻塞主线程）
        // 参考 Go 项目的实现：在解析过程中记录性能，后台定期清理慢节点
        let resolver_for_cleanup = self.resolver.clone();
        let server_pool_for_cleanup = server_pool_arc.clone();
        let dns_timeout_for_cleanup = dns_timeout;
        tokio::spawn(async move {
            let cleanup_interval = Duration::from_secs(300); // 每5分钟清理一次（对应 Go 项目的定期清理）
            let max_avg_response_time_ms = 2000.0; // 平均响应时间超过2秒的淘汰
            let max_failure_rate = 0.5; // 失败率超过50%的淘汰

            loop {
                tokio::time::sleep(cleanup_interval).await;

                // 淘汰慢的服务器（对应 Go 项目的 RemoveSlowServers）
                let old_count = server_pool_for_cleanup.len();
                let min_active_servers = 5; // 生产环境下建议至少保留 5 个服务器作为保底
                let optimized_pool = server_pool_for_cleanup.remove_slow_servers(
                    max_avg_response_time_ms,
                    max_failure_rate,
                    min_active_servers,
                );
                let new_count = optimized_pool.len();
                let removed_count = old_count - new_count;

                if removed_count > 0 {
                    eprintln!(
                        "[DNS Service] 后台清理：淘汰了 {} 个慢的DNS服务器（剩余 {} 个）",
                        removed_count, new_count
                    );

                    // 更新 resolver 的服务器池（对应 Go 项目的更新服务器池）
                    let new_resolver = DNSResolver::with_server_pool(
                        dns_timeout_for_cleanup,
                        Arc::new(optimized_pool),
                    );
                    {
                        let mut resolver_guard = resolver_for_cleanup.write().await;
                        *resolver_guard = new_resolver;
                    }
                }
            }
        });

        // 在后台线程启动服务主循环（不阻塞主线程）
        // 使用 Arc 包装的字段，可以直接在闭包中使用
        let config = self.config.clone();
        let resolver = self.resolver.clone();
        let ipinfo_client = self.ipinfo_client.clone();
        let running = self.running.clone();

        tokio::spawn(async move {
            // 解析间隔
            let base_interval =
                parse_duration(&config.interval).unwrap_or(Duration::from_secs(120));

            eprintln!("[DNS Service] 服务已启动，将在后台运行（不阻塞主线程）");
            eprintln!(
                "[DNS Service] 配置: 域名列表 {} 个，检查间隔: {}，数据目录: {}",
                config.domain_list.len(),
                config.interval,
                config.domain_ips_dir
            );

            // 创建临时的 Service 实例用于调用 resolve_and_save_all
            // 注意：resolve_and_save_all 需要 &self，所以我们需要创建一个辅助函数
            // 或者直接在这里实现解析逻辑

            // 动态间隔调整
            let mut current_interval = base_interval;
            let mut last_has_new_ips = false;

            loop {
                // 检查停止信号
                if rx.try_recv().is_ok() {
                    eprintln!("[DNS Service] 收到停止信号，正在停止服务...");
                    break;
                }

                // 执行解析（使用辅助函数）- 等待解析完成后再等待间隔
                eprintln!("[DNS Service] 开始执行 DNS 解析...");
                let resolve_start = std::time::Instant::now();
                match resolve_and_save_all_internal(&resolver, &ipinfo_client, &config).await {
                    Ok(has_new_ips) => {
                        let resolve_duration = resolve_start.elapsed();
                        eprintln!(
                            "[DNS Service] DNS 解析完成，耗时: {:.2}秒",
                            resolve_duration.as_secs_f64()
                        );

                        // 智能间隔调整：发现新 IP 时高频检测，否则指数退避
                        if has_new_ips {
                            current_interval = base_interval;
                            last_has_new_ips = true;
                            eprintln!(
                                "[DNS Service] 发现新 IP，下次将在 {} 后执行",
                                format_duration(&current_interval)
                            );
                        } else {
                            if last_has_new_ips {
                                // 之前有新 IP，现在没有了，逐步增加间隔
                                current_interval = base_interval;
                                last_has_new_ips = false;
                            } else {
                                // 指数退避，但不超过 10 倍基础间隔
                                current_interval = (current_interval * 2).min(base_interval * 10);
                            }
                            eprintln!(
                                "[DNS Service] 未发现新 IP，下次将在 {} 后执行",
                                format_duration(&current_interval)
                            );
                        }
                    }
                    Err(e) => {
                        let resolve_duration = resolve_start.elapsed();
                        eprintln!(
                            "[DNS Service] DNS 解析出错（耗时: {:.2}秒）: {}",
                            resolve_duration.as_secs_f64(),
                            e
                        );
                        // 出错时使用基础间隔
                        current_interval = base_interval;
                    }
                }

                // 检查停止信号（在等待间隔前）
                if rx.try_recv().is_ok() {
                    eprintln!("[DNS Service] 收到停止信号，正在停止服务...");
                    break;
                }

                // 等待当前间隔（在解析完成后）
                eprintln!(
                    "[DNS Service] 等待 {} 后执行下一次解析...",
                    format_duration(&current_interval)
                );
                sleep(current_interval).await;
            }

            // 停止服务
            {
                let mut running = running.write().await;
                *running = false;
            }

            eprintln!("[DNS Service] 服务已停止");
        });

        eprintln!("[DNS Service] 服务已在后台启动，主线程不会被阻塞");
        Ok(())
    }

    /// 停止服务
    pub async fn stop(&self) -> Result<(), DNSError> {
        let mut stop_tx = self.stop_tx.write().await;
        if let Some(tx) = stop_tx.take() {
            let _ = tx.send(());
        }
        Ok(())
    }

    /// 检查服务是否在运行
    pub async fn is_running(&self) -> bool {
        *self.running.read().await
    }

    /// 设置基础执行间隔
    pub fn set_interval(&self, _interval: Duration) {
        // 注意：动态调整模式下，实际间隔会根据是否发现新IP而变化
        // 这个函数主要用于静态模式，目前暂不支持
    }

    /// 解析并保存所有域名的 IP 信息
    /// 注意：此方法目前未直接使用，实际使用的是 resolve_and_save_all_internal
    #[allow(dead_code)]
    async fn resolve_and_save_all(&self) -> Result<bool, DNSError> {
        resolve_and_save_all_internal(&self.resolver, &self.ipinfo_client, &self.config).await
    }
}

/// 辅助函数：解析并保存所有域名的 IP 信息（可以在闭包中使用）
async fn resolve_and_save_all_internal(
    resolver: &Arc<RwLock<DNSResolver>>,
    ipinfo_client: &Arc<IPInfoClient>,
    config: &Arc<DNSConfig>,
) -> Result<bool, DNSError> {
    let mut has_new_ips = false;

    // 并发解析所有域名（使用收集到的 DNS 服务器）
    let resolver_guard = resolver.read().await;
    let results = resolver_guard
        .resolve_many(config.domain_list.clone(), config.max_concurrency)
        .await;
    drop(resolver_guard);

    // 为每个域名的 IP 地址获取详细信息
    for (domain, dns_result) in results {
        match dns_result {
            Ok(result) => {
                // 获取现有数据
                let existing = load_domain_ips(&domain, &config.domain_ips_dir)?;

                // 提取所有解析到的 IP（已去重）
                let all_ipv4: HashSet<String> = result
                    .ips
                    .ipv4
                    .iter()
                    .map(|ip_info| ip_info.ip.clone())
                    .collect();
                let all_ipv6: HashSet<String> = result
                    .ips
                    .ipv6
                    .iter()
                    .map(|ip_info| ip_info.ip.clone())
                    .collect();

                // 从现有数据中提取已存在的 IP
                let existing_ipv4: HashSet<String> = existing
                    .as_ref()
                    .map(|e| e.ipv4.iter().map(|ip| ip.ip.clone()).collect())
                    .unwrap_or_default();
                let existing_ipv6: HashSet<String> = existing
                    .as_ref()
                    .map(|e| e.ipv6.iter().map(|ip| ip.ip.clone()).collect())
                    .unwrap_or_default();

                // 找出新发现的 IP（只查询这些）
                let new_ipv4: Vec<String> = all_ipv4.difference(&existing_ipv4).cloned().collect();
                let new_ipv6: Vec<String> = all_ipv6.difference(&existing_ipv6).cloned().collect();

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

                // 只查询新发现的 IPv4 的详细信息
                if !new_ipv4.is_empty() {
                    eprintln!(
                        "[DNS Service] 发现 {} 个新的 IPv4 地址，正在获取详细信息...",
                        new_ipv4.len()
                    );
                    let ipv4_results = ipinfo_client
                        .get_ip_infos(new_ipv4.clone(), config.max_ip_fetch_conc)
                        .await;

                    for (ip, ip_result) in ipv4_results {
                        match ip_result {
                            Ok(mut ip_info) => {
                                // 保留原始 IP（因为 IPInfo 可能返回不同的格式）
                                ip_info.ip = ip.clone();
                                domain_ips.ipv4.push(ip_info);
                            }
                            Err(e) => {
                                eprintln!("[DNS Service] Failed to get IP info for {}: {}", ip, e);
                                // 即使失败，也保存基本 IP 信息
                                domain_ips.ipv4.push(IPInfo::new(ip));
                            }
                        }
                    }
                }

                // 只查询新发现的 IPv6 的详细信息
                if !new_ipv6.is_empty() {
                    eprintln!(
                        "[DNS Service] 发现 {} 个新的 IPv6 地址，正在获取详细信息...",
                        new_ipv6.len()
                    );
                    let ipv6_results = ipinfo_client
                        .get_ip_infos(new_ipv6.clone(), config.max_ip_fetch_conc)
                        .await;

                    for (ip, ip_result) in ipv6_results {
                        match ip_result {
                            Ok(mut ip_info) => {
                                ip_info.ip = ip.clone();
                                domain_ips.ipv6.push(ip_info);
                            }
                            Err(e) => {
                                eprintln!("[DNS Service] Failed to get IP info for {}: {}", ip, e);
                                domain_ips.ipv6.push(IPInfo::new(ip));
                            }
                        }
                    }
                }

                // 检查是否有新 IP
                if !new_ipv4.is_empty() || !new_ipv6.is_empty() {
                    has_new_ips = true;
                }

                // 保存结果
                save_domain_ips(&domain, &domain_ips, &config.domain_ips_dir)?;
            }
            Err(e) => {
                eprintln!("[DNS Service] Failed to resolve {}: {}", domain, e);
            }
        }
    }

    Ok(has_new_ips)
}

/// 格式化 Duration 为可读字符串
fn format_duration(d: &Duration) -> String {
    let secs = d.as_secs();
    if secs < 60 {
        format!("{}秒", secs)
    } else if secs < 3600 {
        format!("{}分{}秒", secs / 60, secs % 60)
    } else {
        format!("{}小时{}分{}秒", secs / 3600, (secs % 3600) / 60, secs % 60)
    }
}

/// 解析时间字符串（如 "2m", "30s", "1h"）
fn parse_duration(s: &str) -> Option<Duration> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }

    let (num, unit) = if let Some(stripped) = s.strip_suffix("ns") {
        (stripped.parse::<u64>().ok()?, "ns")
    } else if let Some(stripped) = s.strip_suffix("us") {
        (stripped.parse::<u64>().ok()?, "us")
    } else if let Some(stripped) = s.strip_suffix("µs") {
        (stripped.parse::<u64>().ok()?, "us")
    } else if let Some(stripped) = s.strip_suffix("ms") {
        (stripped.parse::<u64>().ok()?, "ms")
    } else if let Some(stripped) = s.strip_suffix('s') {
        (stripped.parse::<u64>().ok()?, "s")
    } else if let Some(stripped) = s.strip_suffix('m') {
        (stripped.parse::<u64>().ok()?, "m")
    } else if let Some(stripped) = s.strip_suffix('h') {
        (stripped.parse::<u64>().ok()?, "h")
    } else {
        // 尝试作为秒数解析
        (s.parse::<u64>().ok()?, "s")
    };

    Some(match unit {
        "ns" => Duration::from_nanos(num),
        "us" | "µs" => Duration::from_micros(num),
        "ms" => Duration::from_millis(num),
        "s" => Duration::from_secs(num),
        "m" => Duration::from_secs(num * 60),
        "h" => Duration::from_secs(num * 3600),
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_duration() {
        assert_eq!(parse_duration("30s"), Some(Duration::from_secs(30)));
        assert_eq!(parse_duration("2m"), Some(Duration::from_secs(120)));
        assert_eq!(parse_duration("1h"), Some(Duration::from_secs(3600)));
        assert_eq!(parse_duration("500ms"), Some(Duration::from_millis(500)));
    }
}
