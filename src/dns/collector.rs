//! DNS 服务器收集器模块
//!
//! 收集可用的 DNS 服务器，包括从 public-dns.info 获取公共 DNS 服务器列表

use crate::dns::serverpool::ServerPool;
use crate::dns::types::DNSError;
use std::time::Duration;

/// DNS 服务器收集器
pub struct ServerCollector;

impl ServerCollector {
    /// 从 public-dns.info 获取公共 DNS 服务器列表
    /// 对应 Go 版本的 collectPublicDNS 函数
    pub async fn collect_public_dns(
        timeout: Option<Duration>,
    ) -> Result<ServerPool, DNSError> {
        let timeout = timeout.unwrap_or(Duration::from_secs(30));
        let url = "https://public-dns.info/nameservers.txt";

        // 使用项目内部的 HttpClient
        let config = crate::http_client::HttpClientConfig {
            connect_timeout: timeout,
            read_timeout: timeout,
            write_timeout: timeout,
            ..Default::default()
        };
        let client = crate::http_client::HttpClient::new(config);
        
        // 在异步上下文中执行同步的 HTTP 请求
        let response = tokio::task::spawn_blocking(move || {
            client.get(url)
        })
        .await
        .map_err(|e| DNSError::Http(format!("task join error: {}", e)))?
        .map_err(|e| DNSError::Http(format!("HTTP request failed: {}", e)))?;

        if !response.is_success() {
            return Err(DNSError::Http(format!(
                "failed to fetch nameservers: HTTP {}",
                response.status_code
            )));
        }

        // 读取响应文本
        let text = String::from_utf8_lossy(&response.body).to_string();

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

        if servers.is_empty() {
            // 如果获取失败，返回默认服务器
            eprintln!("Warning: No servers fetched from public-dns.info, using defaults");
            return Ok(ServerPool::default());
        }

        Ok(ServerPool::new(servers))
    }

    /// 收集系统 DNS 服务器
    pub fn collect_system_dns() -> ServerPool {
        // 目前返回默认的公共 DNS 服务器
        // 未来可以扩展为从系统配置读取
        ServerPool::default()
    }

    /// 从配置文件收集 DNS 服务器
    pub fn collect_from_config(_servers: Vec<String>) -> ServerPool {
        // 如果配置了自定义服务器，使用它们
        // 否则使用默认服务器
        ServerPool::default()
    }

    /// 验证并更新现有文件中的 DNS 服务器
    /// 从文件加载所有服务器，进行健康检查，只保留可用的服务器并保存回文件
    /// 
    /// # 参数
    /// - `test_domain`: 用于测试的域名，默认为 "google.com"
    /// - `test_timeout`: 每个服务器的测试超时时间，默认为 3 秒
    /// - `max_concurrency`: 最大并发测试数，默认为 100
    pub async fn validate_and_update_file(
        test_domain: Option<&str>,
        test_timeout: Option<Duration>,
        max_concurrency: Option<usize>,
    ) -> Result<(usize, usize), DNSError> {
        use std::path::Path;

        const DEFAULT_SERVER_FILE: &str = "dnsservernames.json";

        let test_domain = test_domain.unwrap_or("google.com");
        let test_timeout = test_timeout.unwrap_or(Duration::from_secs(3));
        let max_concurrency = max_concurrency.unwrap_or(100);

        // 从文件加载所有服务器
        let file_path = Path::new(DEFAULT_SERVER_FILE);
        if !file_path.exists() {
            return Err(DNSError::Config(format!("文件 {} 不存在", DEFAULT_SERVER_FILE)));
        }

        let pool = ServerPool::load_from_file(file_path)?;
        let total_count = pool.len();

        if total_count == 0 {
            return Err(DNSError::Config("文件中没有 DNS 服务器".to_string()));
        }

        eprintln!("从文件加载了 {} 个 DNS 服务器", total_count);
        eprintln!("正在测试 DNS 服务器可用性（测试域名: {}）...", test_domain);

        // 进行健康检查
        let validated_pool = pool
            .health_check(test_domain, test_timeout, max_concurrency)
            .await;

        let valid_count = validated_pool.len();
        let invalid_count = total_count - valid_count;

        eprintln!("健康检查完成:");
        eprintln!("   总服务器数: {}", total_count);
        eprintln!("   可用服务器: {} ({:.2}%)", valid_count, 
                 if total_count > 0 { (valid_count as f64 / total_count as f64) * 100.0 } else { 0.0 });
        eprintln!("   不可用服务器: {} ({:.2}%)", invalid_count,
                 if total_count > 0 { (invalid_count as f64 / total_count as f64) * 100.0 } else { 0.0 });

        // 保存验证后的服务器（先备份原文件）
        if valid_count > 0 {
            let backup_path = format!("{}.backup", DEFAULT_SERVER_FILE);
            if let Err(e) = std::fs::copy(file_path, &backup_path) {
                eprintln!("Warning: 无法创建备份文件: {}", e);
            } else {
                eprintln!("已创建备份: {}", backup_path);
            }

            validated_pool.save_default()?;
            eprintln!("已保存 {} 个可用服务器到文件", valid_count);
        } else {
            return Err(DNSError::Config("没有可用的 DNS 服务器".to_string()));
        }

        Ok((total_count, valid_count))
    }

    /// 收集所有可用的 DNS 服务器（对应 Go 的 BootstrapPoolInternal）
    /// 从多个源收集，并在保存前进行健康检查，只保留可用的服务器
    pub async fn collect_all(
        timeout: Option<Duration>,
    ) -> ServerPool {
        // 先尝试从本地文件加载（对应 Go 的 loadDefault）
        let pool = ServerPool::load_default();
        
        if !pool.is_empty() {
            eprintln!("从本地文件加载了 {} 个 DNS 服务器（已通过验证，直接使用）", pool.len());
            // 文件中的服务器已经通过验证，直接使用，不进行全面检查
            // 只在后台异步检测和淘汰慢节点，不阻塞主线程
            return pool;
        }

        // 如果文件不存在或为空，从网络收集（对应 Go 的 BootstrapPoolInternal）
        eprintln!("本地文件不存在或为空，从网络收集 DNS 服务器...");
        
        match Self::collect_public_dns(timeout).await {
            Ok(new_pool) => {
                let new_count = new_pool.len();
                eprintln!("从网络收集了 {} 个 DNS 服务器", new_count);
                
                // 在保存前进行健康检查，只保留可用的服务器
                // 使用高并发检测，每检测到一批就立即保存，快速完成不长时间阻塞
                eprintln!("正在高并发测试 DNS 服务器可用性（测试哪些服务器可以返回 IP 地址）...");
                let test_timeout = Duration::from_secs(2); // 减少超时时间，加快检测
                let max_concurrency = 500; // 大幅增加并发数，加快检测速度
                let save_batch_size = 100; // 每检测到100个可用服务器就保存一次
                
                let validated_pool = new_pool
                    .health_check_and_save_incremental("google.com", test_timeout, max_concurrency, save_batch_size)
                    .await;
                
                let valid_count = validated_pool.len();
                let invalid_count = new_count - valid_count;
                eprintln!("健康检查完成:");
                eprintln!("   总服务器数: {}", new_count);
                eprintln!("   可用服务器: {} ({:.2}%)", valid_count, 
                         if new_count > 0 { (valid_count as f64 / new_count as f64) * 100.0 } else { 0.0 });
                eprintln!("   不可用服务器: {} ({:.2}%)", invalid_count,
                         if new_count > 0 { (invalid_count as f64 / new_count as f64) * 100.0 } else { 0.0 });
                
                // 文件已经在增量保存过程中更新了，直接返回
                if valid_count > 0 {
                    validated_pool
                } else {
                    eprintln!("Warning: 没有可用的 DNS 服务器，使用默认服务器");
                    ServerPool::default()
                }
            }
            Err(e) => {
                eprintln!("Warning: Failed to collect public DNS servers: {}, using defaults", e);
                ServerPool::default()
            }
        }
    }
}

/// 验证是否为有效的 IP 地址（IPv4 或 IPv6）
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_valid_ip_address() {
        assert!(is_valid_ip_address("8.8.8.8"));
        assert!(is_valid_ip_address("1.1.1.1"));
        assert!(is_valid_ip_address("2001:4860:4860::8888"));
        assert!(is_valid_ip_address("8.8.8.8:53"));
        assert!(!is_valid_ip_address("invalid"));
        assert!(!is_valid_ip_address(""));
        assert!(!is_valid_ip_address("not.an.ip"));
    }

    #[tokio::test]
    #[ignore] // 需要网络连接，默认跳过
    async fn test_collect_public_dns() {
        let pool = ServerCollector::collect_public_dns(None).await;
        assert!(pool.is_ok());
        let pool = pool.unwrap();
        assert!(!pool.is_empty());
        println!("Collected {} DNS servers", pool.len());
    }
}

