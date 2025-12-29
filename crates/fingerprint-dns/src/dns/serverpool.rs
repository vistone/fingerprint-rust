//! DNS 服务器池模块
//!
//! 管理 DNS 服务器列表，包括从本地文件加载/保存和健康检查功能

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

/// 默认服务器池文件名（对应 Go 项目的 dnsservernames.json）
const DEFAULT_SERVER_FILE: &str = "dnsservernames.json";

/// DNS 服务器列表的 JSON 结构（对应 Go 项目的 DNSServerList）
#[derive(Debug, Serialize, Deserialize)]
struct DNSServerList {
    servers: std::collections::HashMap<String, String>,
}

/// DNS 服务器性能统计
#[derive(Debug, Clone)]
struct ServerStats {
    /// 总响应时间（毫秒）
    total_response_time_ms: u64,
    /// 成功查询次数
    success_count: u64,
    /// 失败查询次数
    failure_count: u64,
    /// 最后更新时间
    last_update: std::time::Instant,
}

impl ServerStats {
    fn new() -> Self {
        Self {
            total_response_time_ms: 0,
            success_count: 0,
            failure_count: 0,
            last_update: std::time::Instant::now(),
        }
    }

    /// 记录成功查询
    fn record_success(&mut self, response_time: Duration) {
        self.success_count += 1;
        self.total_response_time_ms += response_time.as_millis() as u64;
        self.last_update = std::time::Instant::now();
    }

    /// 记录失败查询
    fn record_failure(&mut self) {
        self.failure_count += 1;
        self.last_update = std::time::Instant::now();
    }

    /// 获取平均响应时间（毫秒）
    fn avg_response_time_ms(&self) -> f64 {
        if self.success_count > 0 {
            self.total_response_time_ms as f64 / self.success_count as f64
        } else {
            f64::MAX
        }
    }

    /// 获取失败率
    fn failure_rate(&self) -> f64 {
        let total = self.success_count + self.failure_count;
        if total > 0 {
            self.failure_count as f64 / total as f64
        } else {
            0.0
        }
    }
}

/// DNS 服务器池
#[derive(Debug, Clone)]
pub struct ServerPool {
    servers: Arc<Vec<String>>,
    /// 服务器性能统计（仅在运行时使用，不持久化）
    stats: Arc<std::sync::RwLock<HashMap<String, ServerStats>>>,
}

impl ServerPool {
    /// 创建新的服务器池
    pub fn new(servers: Vec<String>) -> Self {
        Self {
            servers: Arc::new(servers),
            stats: Arc::new(std::sync::RwLock::new(HashMap::new())),
        }
    }

    /// 创建默认服务器池（使用公共 DNS 服务器）
    #[allow(clippy::new_without_default, clippy::should_implement_trait)]
    pub fn default() -> Self {
        Self::new(vec![
            "8.8.8.8:53".to_string(), // Google DNS
            "8.8.4.4:53".to_string(), // Google DNS
            "1.1.1.1:53".to_string(), // Cloudflare DNS
            "1.0.0.1:53".to_string(), // Cloudflare DNS
        ])
    }

    /// 记录服务器响应时间（成功）
    pub fn record_success(
        &self,
        _server: &str,
        response_time: Duration,
    ) -> Result<(), crate::dns::types::DNSError> {
        let mut stats = self
            .stats
            .write()
            .map_err(|e| crate::dns::types::DNSError::Internal(format!("Lock poisoned: {}", e)))?;
        let server_stats = stats
            .entry(_server.to_string())
            .or_insert_with(ServerStats::new);
        server_stats.record_success(response_time);
        Ok(())
    }

    /// 记录服务器失败
    pub fn record_failure(&self, _server: &str) -> Result<(), crate::dns::types::DNSError> {
        let mut stats = self
            .stats
            .write()
            .map_err(|e| crate::dns::types::DNSError::Internal(format!("Lock poisoned: {}", e)))?;
        let server_stats = stats
            .entry(_server.to_string())
            .or_insert_with(ServerStats::new);
        server_stats.record_failure();
        Ok(())
    }

    /// 淘汰慢的服务器（平均响应时间超过阈值或失败率过高）
    /// 返回新的服务器池，不阻塞主线程
    pub fn remove_slow_servers(
        &self,
        max_avg_response_time_ms: f64,
        max_failure_rate: f64,
    ) -> Self {
        // 安全修复：处理锁中毒情况
        let stats_guard = match self.stats.read() {
            Ok(guard) => guard,
            Err(e) => {
                eprintln!("Warning: Lock poisoned in remove_slow_servers: {}", e);
                // 如果锁中毒，返回所有服务器（不淘汰任何服务器）
                return Self::new(self.servers.iter().cloned().collect());
            }
        };
        let servers: Vec<String> = self
            .servers
            .iter()
            .filter(|server| {
                if let Some(server_stat) = stats_guard.get(*server) {
                    let avg_time = server_stat.avg_response_time_ms();
                    let failure_rate = server_stat.failure_rate();
                    // 保留响应时间快且失败率低的服务器
                    avg_time <= max_avg_response_time_ms && failure_rate <= max_failure_rate
                } else {
                    // 没有统计数据的服务器保留（新服务器）
                    true
                }
            })
            .cloned()
            .collect();

        Self::new(servers)
    }

    /// 从本地 JSON 文件加载服务器池（对应 Go 的 loadDefault）
    /// 如果文件不存在或为空，返回空池
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, crate::dns::types::DNSError> {
        let path = path.as_ref();

        if !path.exists() {
            return Ok(Self::new(Vec::new()));
        }

        let content = fs::read_to_string(path)
            .map_err(|e| crate::dns::types::DNSError::Config(format!("无法读取文件: {}", e)))?;

        let list: DNSServerList =
            serde_json::from_str(&content).map_err(crate::dns::types::DNSError::Json)?;

        // 提取所有 IP 地址（Go 项目使用 GetAllServers 返回所有 IP）
        let servers: Vec<String> = list
            .servers
            .values()
            .map(|ip| {
                // 如果没有端口，添加默认端口 53
                if ip.contains(':') {
                    ip.clone()
                } else {
                    format!("{}:53", ip)
                }
            })
            .collect();

        Ok(Self::new(servers))
    }

    /// 保存服务器池到本地 JSON 文件（对应 Go 的 Save）
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), crate::dns::types::DNSError> {
        let path = path.as_ref();

        // 构建服务器映射（名称 -> IP）
        // Go 项目使用 "Auto-IP" 作为名称
        let mut servers_map = std::collections::HashMap::new();
        for server in self.servers.iter() {
            // 提取 IP 地址（去掉端口）
            let ip = if let Some(colon_pos) = server.find(':') {
                &server[..colon_pos]
            } else {
                server.as_str()
            };

            // 生成名称（对应 Go 的 "Auto-IP" 格式）
            let name = format!("Auto-{}", ip);
            servers_map.insert(name, ip.to_string());
        }

        let list = DNSServerList {
            servers: servers_map,
        };

        let json_content =
            serde_json::to_string_pretty(&list).map_err(crate::dns::types::DNSError::Json)?;

        // 安全修复：原子性写入，使用唯一的临时文件名防止竞态条件
        // 使用进程 ID 确保临时文件名唯一，避免多进程同时写入时的竞态条件
        let temp_path = path.with_extension(format!("tmp.{}", std::process::id()));
        fs::write(&temp_path, json_content)
            .map_err(|e| crate::dns::types::DNSError::Config(format!("无法写入文件: {}", e)))?;
        fs::rename(&temp_path, path).map_err(|e| {
            // 如果重命名失败，清理临时文件
            let _ = std::fs::remove_file(&temp_path);
            crate::dns::types::DNSError::Config(format!("无法重命名文件: {}", e))
        })?;

        Ok(())
    }

    /// 从默认文件加载服务器池（对应 Go 的 NewServerPool）
    pub fn load_default() -> Self {
        Self::load_from_file(DEFAULT_SERVER_FILE).unwrap_or_else(|_| Self::new(Vec::new()))
    }

    /// 保存到默认文件
    pub fn save_default(&self) -> Result<(), crate::dns::types::DNSError> {
        self.save_to_file(DEFAULT_SERVER_FILE)
    }

    /// 添加服务器并返回新的 ServerPool（对应 Go 的 AddServer）
    /// 返回 (新池, 是否是新添加的)
    pub fn with_added_server(&self, ip: &str) -> (Self, bool) {
        use std::net::IpAddr;
        use std::str::FromStr;

        // 验证 IP 地址格式
        let ip_str = if let Some(colon_pos) = ip.find(':') {
            &ip[..colon_pos]
        } else {
            ip
        };

        if IpAddr::from_str(ip_str).is_err() {
            return (self.clone(), false);
        }

        // 格式化服务器地址
        let server = if ip.contains(':') {
            ip.to_string()
        } else {
            format!("{}:53", ip)
        };

        // 检查是否已存在
        if self.servers.iter().any(|s| s == &server) {
            return (self.clone(), false);
        }

        // 添加新服务器
        let mut new_servers = (*self.servers).clone();
        new_servers.push(server);
        (
            Self {
                servers: Arc::new(new_servers),
                stats: self.stats.clone(), // 修复：继承原有的统计数据，避免丢失历史性能数据
            },
            true,
        )
    }

    /// 获取所有服务器
    pub fn servers(&self) -> &[String] {
        &self.servers
    }

    /// 获取服务器数量
    pub fn len(&self) -> usize {
        self.servers.len()
    }

    /// 检查是否为空
    pub fn is_empty(&self) -> bool {
        self.servers.is_empty()
    }

    /// 健康检查并增量保存：高并发测试 DNS 服务器，每检测到一批可用服务器就立即保存
    /// 在后台任务中运行，不阻塞主线程
    pub async fn health_check_and_save_incremental(
        &self,
        test_domain: &str,
        timeout: Duration,
        max_concurrency: usize,
        save_batch_size: usize,
    ) -> Self {
        use futures::stream::{self, StreamExt};
        use hickory_resolver::proto::rr::RecordType;
        use hickory_resolver::{
            config::{NameServerConfig, Protocol, ResolverConfig, ResolverOpts},
            TokioAsyncResolver,
        };
        use std::net::{IpAddr, SocketAddr};
        use std::str::FromStr;
        use std::sync::{Arc, Mutex};

        let servers = self.servers();
        let test_domain = test_domain.to_string();

        // 解析服务器地址
        let servers_to_test: Vec<_> = servers
            .iter()
            .filter_map(|server_str| {
                let (ip_str, port) = if let Some(colon_pos) = server_str.find(':') {
                    let ip = &server_str[..colon_pos];
                    let port = server_str[colon_pos + 1..].parse::<u16>().unwrap_or(53);
                    (ip.to_string(), port)
                } else {
                    (server_str.clone(), 53)
                };

                if let Ok(ip_addr) = IpAddr::from_str(&ip_str) {
                    Some((server_str.clone(), SocketAddr::new(ip_addr, port)))
                } else {
                    None
                }
            })
            .collect();

        // 配置解析选项
        let mut opts = ResolverOpts::default();
        opts.timeout = timeout;
        opts.attempts = 1;

        // 用于收集可用服务器的共享状态
        let available_servers: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
        let processed_count: Arc<Mutex<usize>> = Arc::new(Mutex::new(0));
        let total_count = servers_to_test.len();

        // 克隆用于闭包内部和外部使用
        let available_servers_for_closure = available_servers.clone();
        let available_servers_for_progress = available_servers.clone();
        let processed_count_for_progress = processed_count.clone();

        // 并发测试服务器，流式处理
        let mut test_tasks = stream::iter(servers_to_test)
            .map(move |(server_str, socket_addr)| {
                let test_domain = test_domain.clone();
                let opts = opts.clone();
                let available_servers = available_servers_for_closure.clone();

                async move {
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

                    // 测试查询（查询 A 记录）
                    match resolver.lookup(&test_domain, RecordType::A).await {
                        Ok(lookup_result) => {
                            // 检查是否真的返回了IP地址
                            let ip_count = lookup_result.iter().count();
                            if ip_count > 0 {
                                // 查询成功且返回了IP地址，服务器可用，立即添加到列表
                                let mut servers = match available_servers.lock() {
                                    Ok(guard) => guard,
                                    Err(e) => {
                                        eprintln!("Warning: Lock poisoned in health check: {}", e);
                                        // 如果锁中毒，跳过这个服务器
                                        return None;
                                    }
                                };
                                servers.push(server_str.clone());
                                let current_count = servers.len();

                                // 每达到批次大小就保存一次
                                if current_count.is_multiple_of(save_batch_size) {
                                    let pool = Self::new(servers.clone());
                                    if let Err(e) = pool.save_default() {
                                        eprintln!("Warning: 增量保存失败: {}", e);
                                    } else {
                                        eprintln!("已保存 {} 个可用服务器到文件", current_count);
                                    }
                                }

                                Some(server_str)
                            } else {
                                // 查询成功但没有返回IP地址，服务器不可用
                                None
                            }
                        }
                        Err(_) => None, // 查询失败，服务器不可用
                    }
                }
            })
            .buffer_unordered(max_concurrency);

        // 流式处理所有测试任务
        while let Some(_result) = test_tasks.next().await {
            let mut count = match processed_count_for_progress.lock() {
                Ok(guard) => guard,
                Err(e) => {
                    eprintln!("Warning: Lock poisoned in progress tracking: {}", e);
                    continue; // 跳过这次更新
                }
            };
            *count += 1;
            let current_processed = *count;
            let current_available = match available_servers_for_progress.lock() {
                Ok(guard) => guard.len(),
                Err(e) => {
                    eprintln!("Warning: Lock poisoned in progress tracking: {}", e);
                    0 // 如果锁中毒，使用 0 作为默认值
                }
            };

            // 每处理1000个就输出一次进度
            if current_processed.is_multiple_of(1000) {
                eprintln!(
                    "已测试 {}/{} 个服务器，发现 {} 个可用",
                    current_processed, total_count, current_available
                );
            }
        }

        // 最终保存所有可用服务器
        let final_servers = match available_servers_for_progress.lock() {
            Ok(guard) => guard.clone(),
            Err(e) => {
                eprintln!("Warning: Lock poisoned in final save: {}", e);
                Vec::new() // 如果锁中毒，返回空列表
            }
        };
        if !final_servers.is_empty() {
            let pool = Self::new(final_servers.clone());
            if let Err(e) = pool.save_default() {
                eprintln!("Warning: 最终保存失败: {}", e);
            } else {
                eprintln!("最终保存了 {} 个可用服务器到文件", final_servers.len());
            }
        }

        Self::new(final_servers)
    }

    /// 健康检查：测试哪些 DNS 服务器是可用的
    /// 通过查询一个已知域名（如 google.com）来测试服务器是否可用
    pub async fn health_check(
        &self,
        test_domain: &str,
        timeout: Duration,
        max_concurrency: usize,
    ) -> Self {
        use futures::stream::{self, StreamExt};
        use hickory_resolver::proto::rr::RecordType;
        use hickory_resolver::{
            config::{NameServerConfig, Protocol, ResolverConfig, ResolverOpts},
            TokioAsyncResolver,
        };
        use std::net::{IpAddr, SocketAddr};
        use std::str::FromStr;

        let servers = self.servers();
        let test_domain = test_domain.to_string();

        // 解析服务器地址
        let servers_to_test: Vec<_> = servers
            .iter()
            .filter_map(|server_str| {
                let (ip_str, port) = if let Some(colon_pos) = server_str.find(':') {
                    let ip = &server_str[..colon_pos];
                    let port = server_str[colon_pos + 1..].parse::<u16>().unwrap_or(53);
                    (ip.to_string(), port)
                } else {
                    (server_str.clone(), 53)
                };

                if let Ok(ip_addr) = IpAddr::from_str(&ip_str) {
                    Some((server_str.clone(), SocketAddr::new(ip_addr, port)))
                } else {
                    None
                }
            })
            .collect();

        // 配置解析选项
        let mut opts = ResolverOpts::default();
        opts.timeout = timeout;
        opts.attempts = 1;

        // 并发测试服务器
        let test_tasks = stream::iter(servers_to_test)
            .map(move |(server_str, socket_addr)| {
                let test_domain = test_domain.clone();
                let opts = opts.clone();

                async move {
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

                    // 测试查询（查询 A 记录）
                    match resolver.lookup(&test_domain, RecordType::A).await {
                        Ok(lookup_result) => {
                            // 检查是否真的返回了IP地址
                            let ip_count = lookup_result.iter().count();
                            if ip_count > 0 {
                                Some(server_str) // 查询成功且返回了IP地址，服务器可用
                            } else {
                                None // 查询成功但没有返回IP地址，服务器不可用
                            }
                        }
                        Err(_) => None, // 查询失败，服务器不可用
                    }
                }
            })
            .buffer_unordered(max_concurrency);

        // 收集可用的服务器
        let available_servers: Vec<String> = test_tasks
            .filter_map(|result| async move { result })
            .collect()
            .await;

        Self::new(available_servers)
    }
}

impl Default for ServerPool {
    fn default() -> Self {
        Self::default()
    }
}
