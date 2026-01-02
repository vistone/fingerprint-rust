//! DNS Service module
//!
//! provide DNS Parseservice Start/Stop interface

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

/// DNS Service（Corresponds to Go version's Service）
pub struct Service {
    config: Arc<DNSConfig>,
    resolver: Arc<RwLock<DNSResolver>>, // use RwLock 以便 in start  when Update
    ipinfo_client: Arc<IPInfoClient>,
    running: Arc<RwLock<bool>>,
    stop_tx: Arc<RwLock<Option<oneshot::Sender<()>>>>,
}

impl Service {
    /// Create a new Service instance
    pub fn new(config: DNSConfig) -> Result<Self, DNSError> {
        config.validate()?;

        // Parsetimeout duration
        let dns_timeout = parse_duration(&config.dns_timeout).unwrap_or(Duration::from_secs(4));

        // HTTP timeout duration
        let http_timeout = parse_duration(&config.http_timeout).unwrap_or(Duration::from_secs(20));

        // usedefault DNS serverCreate resolver（will in start  when replace为collect to 的server）
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

    /// Create a new Service instance，并usespecified DNS serverpool
    pub async fn with_server_pool(
        config: DNSConfig,
        server_pool: Arc<ServerPool>,
    ) -> Result<Self, DNSError> {
        config.validate()?;

        // Parsetimeout duration
        let dns_timeout = parse_duration(&config.dns_timeout).unwrap_or(Duration::from_secs(4));

        // HTTP timeout duration
        let http_timeout = parse_duration(&config.http_timeout).unwrap_or(Duration::from_secs(20));

        // usespecified DNS serverpoolCreate resolver
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

    ///  from configurationfileCreate Service
    pub fn from_config_file<P: AsRef<std::path::Path>>(path: P) -> Result<Self, DNSError> {
        let config = load_config(path)?;
        Self::new(config)
    }

    /// startservice（ in back台线程run，不阻塞main线程）
    /// automatic维护 DNS serverpool（dnsservernames.json），无需manual干预
    pub async fn start(&self) -> Result<(), DNSError> {
        // Checkwhetheralready in run
        {
            let mut running = self.running.write().await;
            if *running {
                return Err(DNSError::Config("service is already running".to_string()));
            }
            *running = true;
        }

        // load/collect DNS serverpool（pair应 Go  NewServerPool）
        // 优先 from localfileload， if 不 exists or 为empty，才 from networkcollect
        // collect_all alreadyprocess了：
        //   -  if file exists且is notempty：directlyuse，不进行Check
        //   -  if file不 exists or 为empty： from networkcollect并进行健康Checkbacksave
        let mut server_pool = ServerCollector::collect_all(Some(Duration::from_secs(30))).await;
        eprintln!("currentserverpool有 {} 个 DNS server", server_pool.len());

        // Ifserverpool为empty, usedefaultserver
        if server_pool.is_empty() {
            eprintln!("Warning: noavailable DNS server，usedefaultserver");
            server_pool = ServerPool::default();
            eprintln!("usedefault DNS server: {} 个", server_pool.len());
        }

        // usethrough健康Check的serverpoolUpdate resolver
        // Parse when useallthrough健康Check的serverconcurrentquery
        let dns_timeout =
            parse_duration(&self.config.dns_timeout).unwrap_or(Duration::from_secs(4));
        let server_pool_arc = Arc::new(server_pool);
        let new_resolver = DNSResolver::with_server_pool(dns_timeout, server_pool_arc.clone());

        // replace resolver
        {
            let mut resolver_guard = self.resolver.write().await;
            *resolver_guard = new_resolver;
        }

        // Createstop通道
        let (tx, mut rx) = oneshot::channel();
        {
            let mut stop_tx = self.stop_tx.write().await;
            *stop_tx = Some(tx);
        }

        // startback台任务：定期淘汰慢的DNSserver（不阻塞main线程）
        // reference Go 项destinationimplement： in Parse过程中record性能，back台定期清理慢node
        let resolver_for_cleanup = self.resolver.clone();
        let server_pool_for_cleanup = server_pool_arc.clone();
        let dns_timeout_for_cleanup = dns_timeout;
        tokio::spawn(async move {
            let cleanup_interval = Duration::from_secs(300); // 每5分钟清理一次（pair应 Go 项destination定期清理）
            let max_avg_response_time_ms = 2000.0; // averageresponse when 间超过2秒的淘汰
            let max_failure_rate = 0.5; // failure率超过50%的淘汰

            loop {
                tokio::time::sleep(cleanup_interval).await;

                // 淘汰慢的server（pair应 Go item RemoveSlowServers）
                let old_count = server_pool_for_cleanup.len();
                let min_active_servers = 5; // 生产environmentdown建议至少preserve 5serveras保bottom
                let optimized_pool = server_pool_for_cleanup.remove_slow_servers(
                    max_avg_response_time_ms,
                    max_failure_rate,
                    min_active_servers,
                );
                let new_count = optimized_pool.len();
                let removed_count = old_count - new_count;

                if removed_count > 0 {
                    eprintln!(
                        "[DNS Service] back台清理：淘汰了 {} 个慢的DNSserver（剩余 {} 个）",
                        removed_count, new_count
                    );

                    // Update resolver 的serverpool（pair应 Go 项destinationUpdateserverpool）
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

        //  in back台线程startservicemain循环（不阻塞main线程）
        // use Arc wrap的field，candirectly in 闭包中use
        let config = self.config.clone();
        let resolver = self.resolver.clone();
        let ipinfo_client = self.ipinfo_client.clone();
        let running = self.running.clone();

        tokio::spawn(async move {
            // Parse间隔
            let base_interval =
                parse_duration(&config.interval).unwrap_or(Duration::from_secs(120));

            eprintln!("[DNS Service] servicealreadystart，will in back台run（不阻塞main线程）");
            eprintln!(
                "[DNS Service] configuration: domainlist {} 个，Check间隔: {}，count据目录: {}",
                config.domain_list.len(),
                config.interval,
                config.domain_ips_dir
            );

            // Createtemporary Service instance for call resolve_and_save_all
            // Note: resolve_and_save_all need &self，so我们needCreateanauxiliaryfunction
            //  or 者directly in 这里implementParse逻辑

            // 动态间隔adjust
            let mut current_interval = base_interval;
            let mut last_has_new_ips = false;

            loop {
                // Checkstop信号
                if rx.try_recv().is_ok() {
                    eprintln!("[DNS Service] 收 to stop信号，正 in stopservice...");
                    break;
                }

                // executeParse（useauxiliaryfunction）- waitParsecompleteback再wait间隔
                eprintln!("[DNS Service] startexecute DNS Parse...");
                let resolve_start = std::time::Instant::now();
                match resolve_and_save_all_internal(&resolver, &ipinfo_client, &config).await {
                    Ok(has_new_ips) => {
                        let resolve_duration = resolve_start.elapsed();
                        eprintln!(
                            "[DNS Service] DNS Parsecomplete，耗 when : {:.2}秒",
                            resolve_duration.as_secs_f64()
                        );

                        // 智能间隔adjust：发现new IP  when 高频detect，otherwise指count退避
                        if has_new_ips {
                            current_interval = base_interval;
                            last_has_new_ips = true;
                            eprintln!(
                                "[DNS Service] 发现new IP，down次will in {} backexecute",
                                format_duration(&current_interval)
                            );
                        } else {
                            if last_has_new_ips {
                                // before有new IP，现 in no了，逐步increase间隔
                                current_interval = base_interval;
                                last_has_new_ips = false;
                            } else {
                                // 指count退避，but不超过 10 倍基础间隔
                                current_interval = (current_interval * 2).min(base_interval * 10);
                            }
                            eprintln!(
                                "[DNS Service] not发现new IP，down次will in {} backexecute",
                                format_duration(&current_interval)
                            );
                        }
                    }
                    Err(e) => {
                        let resolve_duration = resolve_start.elapsed();
                        eprintln!(
                            "[DNS Service] DNS Parse出错（耗 when : {:.2}秒）: {}",
                            resolve_duration.as_secs_f64(),
                            e
                        );
                        // 出错 when use基础间隔
                        current_interval = base_interval;
                    }
                }

                // Checkstop信号（ in wait间隔front）
                if rx.try_recv().is_ok() {
                    eprintln!("[DNS Service] 收 to stop信号，正 in stopservice...");
                    break;
                }

                // waitcurrent间隔（ in Parsecompleteback）
                eprintln!(
                    "[DNS Service] wait {} backexecutedown一次Parse...",
                    format_duration(&current_interval)
                );
                sleep(current_interval).await;
            }

            // stopservice
            {
                let mut running = running.write().await;
                *running = false;
            }

            eprintln!("[DNS Service] servicealreadystop");
        });

        eprintln!("[DNS Service] servicealready in back台start，main线程不will被阻塞");
        Ok(())
    }

    /// stopservice
    pub async fn stop(&self) -> Result<(), DNSError> {
        let mut stop_tx = self.stop_tx.write().await;
        if let Some(tx) = stop_tx.take() {
            let _ = tx.send(());
        }
        Ok(())
    }

    /// Checkservicewhether in run
    pub async fn is_running(&self) -> bool {
        *self.running.read().await
    }

    /// settings基础execute间隔
    pub fn set_interval(&self, _interval: Duration) {
        // Note: 动态adjustpatterndown，actual间隔willBased onwhether发现新IP而变化
        // 这个functionmain for 静态pattern，目front暂不support
    }

    /// Parse并savealldomain IP info
    /// Note: 此method目frontnotdirectlyuse，actualuse的是 resolve_and_save_all_internal
    #[allow(dead_code)]
    async fn resolve_and_save_all(&self) -> Result<bool, DNSError> {
        resolve_and_save_all_internal(&self.resolver, &self.ipinfo_client, &self.config).await
    }
}

/// auxiliaryfunction：Parse并savealldomain IP info（can in 闭包中use）
async fn resolve_and_save_all_internal(
    resolver: &Arc<RwLock<DNSResolver>>,
    ipinfo_client: &Arc<IPInfoClient>,
    config: &Arc<DNSConfig>,
) -> Result<bool, DNSError> {
    let mut has_new_ips = false;

    // concurrentParsealldomain（usecollect to  DNS server）
    let resolver_guard = resolver.read().await;
    let results = resolver_guard
        .resolve_many(config.domain_list.clone(), config.max_concurrency)
        .await;
    drop(resolver_guard);

    // 为eachdomain IP addressGet详细info
    for (domain, dns_result) in results {
        match dns_result {
            Ok(result) => {
                // Get现有count据
                let existing = load_domain_ips(&domain, &config.domain_ips_dir)?;

                // ExtractallParse to  IP（alreadydeduplicate）
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

                //  from 现有count据中Extractalready exists IP
                let existing_ipv4: HashSet<String> = existing
                    .as_ref()
                    .map(|e| e.ipv4.iter().map(|ip| ip.ip.clone()).collect())
                    .unwrap_or_default();
                let existing_ipv6: HashSet<String> = existing
                    .as_ref()
                    .map(|e| e.ipv6.iter().map(|ip| ip.ip.clone()).collect())
                    .unwrap_or_default();

                // 找出新发现 IP（只query这些）
                let new_ipv4: Vec<String> = all_ipv4.difference(&existing_ipv4).cloned().collect();
                let new_ipv6: Vec<String> = all_ipv6.difference(&existing_ipv6).cloned().collect();

                // Build最final domain_ips，先copyalready exists的count据
                let mut domain_ips = DomainIPs::new();

                // copyalready exists IPv4 info
                if let Some(existing) = &existing {
                    for existing_ip in &existing.ipv4 {
                        if all_ipv4.contains(&existing_ip.ip) {
                            domain_ips.ipv4.push(existing_ip.clone());
                        }
                    }
                }

                // copyalready exists IPv6 info
                if let Some(existing) = &existing {
                    for existing_ip in &existing.ipv6 {
                        if all_ipv6.contains(&existing_ip.ip) {
                            domain_ips.ipv6.push(existing_ip.clone());
                        }
                    }
                }

                // 只query新发现 IPv4 的详细info
                if !new_ipv4.is_empty() {
                    eprintln!(
                        "[DNS Service] 发现 {} 个new IPv4 address，正 in Get详细info...",
                        new_ipv4.len()
                    );
                    let ipv4_results = ipinfo_client
                        .get_ip_infos(new_ipv4.clone(), config.max_ip_fetch_conc)
                        .await;

                    for (ip, ip_result) in ipv4_results {
                        match ip_result {
                            Ok(mut ip_info) => {
                                // preserve原beginning IP（because IPInfo mayreturn不同的format）
                                ip_info.ip = ip.clone();
                                domain_ips.ipv4.push(ip_info);
                            }
                            Err(e) => {
                                eprintln!("[DNS Service] Failed to get IP info for {}: {}", ip, e);
                                // 即使failure，alsosave基本 IP info
                                domain_ips.ipv4.push(IPInfo::new(ip));
                            }
                        }
                    }
                }

                // 只query新发现 IPv6 的详细info
                if !new_ipv6.is_empty() {
                    eprintln!(
                        "[DNS Service] 发现 {} 个new IPv6 address，正 in Get详细info...",
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

                // Checkwhether有new IP
                if !new_ipv4.is_empty() || !new_ipv6.is_empty() {
                    has_new_ips = true;
                }

                // saveresult
                save_domain_ips(&domain, &domain_ips, &config.domain_ips_dir)?;
            }
            Err(e) => {
                eprintln!("[DNS Service] Failed to resolve {}: {}", domain, e);
            }
        }
    }

    Ok(has_new_ips)
}

/// Format Duration 为可读string
fn format_duration(d: &Duration) -> String {
    let secs = d.as_secs();
    if secs < 60 {
        format!("{}秒", secs)
    } else if secs < 3600 {
        format!("{}分{}秒", secs / 60, secs % 60)
    } else {
        format!("{}小 when {}分{}秒", secs / 3600, (secs % 3600) / 60, secs % 60)
    }
}

/// Parse when 间string（如 "2m", "30s", "1h"）
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
        // tryas秒countParse
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
