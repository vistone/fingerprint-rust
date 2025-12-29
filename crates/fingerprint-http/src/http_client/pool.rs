//! 连接池管理
//!
//! 基于 netconnpool 实现连接复用和生命周期管理

use super::{HttpClientError, Result};
use std::time::Duration;

#[cfg(feature = "connection-pool")]
use std::collections::HashMap;

#[cfg(feature = "connection-pool")]
use std::net::TcpStream;

#[cfg(feature = "connection-pool")]
use std::sync::{Arc, Mutex};

#[cfg(feature = "connection-pool")]
use netconnpool::{Config as PoolConfig, ConnectionType, Pool};

/// 连接池管理器
#[cfg(feature = "connection-pool")]
pub struct ConnectionPoolManager {
    /// 连接池实例（按 host:port 分组）
    pools: Arc<Mutex<HashMap<String, Arc<Pool>>>>,
    /// 默认配置
    config: PoolManagerConfig,
}

#[cfg(feature = "connection-pool")]
impl Default for ConnectionPoolManager {
    fn default() -> Self {
        Self::new(PoolManagerConfig::default())
    }
}

/// 连接池管理器（无连接池功能时的占位）
#[cfg(not(feature = "connection-pool"))]
pub struct ConnectionPoolManager {
    #[allow(dead_code)]
    config: PoolManagerConfig,
}

#[cfg(not(feature = "connection-pool"))]
impl Default for ConnectionPoolManager {
    fn default() -> Self {
        Self::new(PoolManagerConfig::default())
    }
}

/// 连接池管理器配置
#[derive(Debug, Clone)]
pub struct PoolManagerConfig {
    /// 最大连接数
    pub max_connections: usize,
    /// 最小空闲连接数
    pub min_idle: usize,
    /// 连接超时
    pub connect_timeout: Duration,
    /// 空闲超时
    pub idle_timeout: Duration,
    /// 最大生命周期
    pub max_lifetime: Duration,
    /// 是否启用连接复用
    pub enable_reuse: bool,
}

impl Default for PoolManagerConfig {
    fn default() -> Self {
        Self {
            max_connections: 100,
            min_idle: 10,
            connect_timeout: Duration::from_secs(30),
            idle_timeout: Duration::from_secs(90),
            max_lifetime: Duration::from_secs(600), // 10分钟
            enable_reuse: true,
        }
    }
}

impl ConnectionPoolManager {
    /// 创建新的连接池管理器
    #[cfg(feature = "connection-pool")]
    pub fn new(config: PoolManagerConfig) -> Self {
        Self {
            pools: Arc::new(Mutex::new(HashMap::new())),
            config,
        }
    }

    #[cfg(not(feature = "connection-pool"))]
    pub fn new(config: PoolManagerConfig) -> Self {
        Self { config }
    }

    /// 获取或创建连接池
    #[cfg(feature = "connection-pool")]
    pub fn get_pool(&self, host: &str, port: u16) -> Result<Arc<Pool>> {
        let key = format!("{}:{}", host, port);
        let mut pools = self
            .pools
            .lock()
            .map_err(|e| HttpClientError::ConnectionFailed(format!("连接池锁失败: {}", e)))?;

        if let Some(pool) = pools.get(&key) {
            return Ok(pool.clone());
        }

        // 创建新的连接池
        let pool_config = self.create_pool_config(host, port);
        let pool = Pool::new(pool_config)
            .map_err(|e| HttpClientError::ConnectionFailed(format!("创建连接池失败: {:?}", e)))?;

        let pool = Arc::new(pool);
        pools.insert(key, pool.clone());

        Ok(pool)
    }

    #[cfg(not(feature = "connection-pool"))]
    pub fn get_pool(&self, _host: &str, _port: u16) -> Result<()> {
        Err(HttpClientError::ConnectionFailed(
            "连接池功能未启用，请使用 --features connection-pool 编译".to_string(),
        ))
    }

    /// 创建连接池配置
    #[cfg(feature = "connection-pool")]
    fn create_pool_config(&self, host: &str, port: u16) -> PoolConfig {
        let host = host.to_string();
        let connect_timeout = self.config.connect_timeout;

        PoolConfig {
            mode: netconnpool::PoolMode::Client,
            max_connections: self.config.max_connections,
            min_connections: self.config.min_idle,
            max_idle_connections: self.config.max_connections,
            connection_timeout: self.config.connect_timeout,
            idle_timeout: self.config.idle_timeout,
            max_lifetime: self.config.max_lifetime,
            get_connection_timeout: self.config.connect_timeout,
            health_check_interval: Duration::from_secs(30),
            health_check_timeout: Duration::from_secs(3),
            connection_leak_timeout: Duration::from_secs(300),

            // 提供 Dialer 函数来创建 TCP 连接
            dialer: Some(Box::new(move |_protocol| {
                use std::net::{IpAddr, SocketAddr, ToSocketAddrs};

                let addrs: Vec<SocketAddr> = (host.as_str(), port)
                    .to_socket_addrs()
                    .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?
                    .collect();

                // 优先使用 IPv4，避免在“无 IPv6 路由”的环境中出现 `Network is unreachable`。
                let mut v4 = Vec::new();
                let mut v6 = Vec::new();
                for a in addrs {
                    match a.ip() {
                        IpAddr::V4(_) => v4.push(a),
                        IpAddr::V6(_) => v6.push(a),
                    }
                }

                let mut last_err: Option<std::io::Error> = None;
                for addr in v4.into_iter().chain(v6.into_iter()) {
                    match TcpStream::connect_timeout(&addr, connect_timeout) {
                        Ok(s) => return Ok(ConnectionType::Tcp(s)),
                        Err(e) => last_err = Some(e),
                    }
                }

                Err(Box::new(last_err.unwrap_or_else(|| {
                    std::io::Error::new(std::io::ErrorKind::Other, "no resolved addresses")
                }))
                    as Box<dyn std::error::Error + Send + Sync>)
            })),
            listener: None,
            acceptor: None,
            health_checker: None,
            close_conn: None,
            on_created: None,
            on_borrow: None,
            on_return: None,
            enable_stats: true,
            enable_health_check: true,
            clear_udp_buffer_on_return: false,
            max_buffer_clear_packets: 0,
            udp_buffer_clear_timeout: Duration::from_secs(0),
        }
    }

    /// 获取统计信息
    #[cfg(feature = "connection-pool")]
    pub fn get_stats(&self) -> Vec<PoolStats> {
        let pools = match self.pools.lock() {
            Ok(p) => p,
            Err(e) => {
                eprintln!("警告: 连接池锁失败: {}", e);
                return Vec::new();
            }
        };
        pools
            .iter()
            .map(|(key, pool)| {
                let stats = pool.stats();
                PoolStats {
                    endpoint: key.clone(),
                    total_connections: stats.total_connections_created,
                    active_connections: stats.current_active_connections,
                    idle_connections: stats.current_idle_connections,
                    total_requests: stats.total_get_requests,
                    successful_requests: stats.successful_gets,
                    failed_requests: stats.failed_gets,
                }
            })
            .collect()
    }

    #[cfg(not(feature = "connection-pool"))]
    pub fn get_stats(&self) -> Vec<PoolStats> {
        vec![]
    }

    /// 清理空闲连接
    #[cfg(feature = "connection-pool")]
    pub fn cleanup_idle(&self) {
        // netconnpool 会自动清理，这里只是提供接口
        if let Ok(pools) = self.pools.lock() {
            println!("连接池状态: {} 个端点", pools.len());
        }
    }

    #[cfg(not(feature = "connection-pool"))]
    pub fn cleanup_idle(&self) {}

    /// 关闭所有连接池
    #[cfg(feature = "connection-pool")]
    pub fn shutdown(&self) {
        if let Ok(mut pools) = self.pools.lock() {
            for (_, pool) in pools.iter() {
                let _ = pool.close();
            }
            pools.clear();
            println!("所有连接池已关闭");
        }
    }

    #[cfg(not(feature = "connection-pool"))]
    pub fn shutdown(&self) {}
}

/// 连接池统计信息
#[derive(Debug, Clone)]
pub struct PoolStats {
    pub endpoint: String,
    pub total_connections: i64,
    pub active_connections: i64,
    pub idle_connections: i64,
    pub total_requests: i64,
    pub successful_requests: i64,
    pub failed_requests: i64,
}

impl PoolStats {
    /// 获取成功率
    pub fn success_rate(&self) -> f64 {
        if self.total_requests == 0 {
            return 0.0;
        }
        (self.successful_requests as f64 / self.total_requests as f64) * 100.0
    }

    /// 打印统计信息
    pub fn print(&self) {
        println!("\n📊 连接池统计: {}", self.endpoint);
        println!("  总连接数: {}", self.total_connections);
        println!("  活跃连接: {}", self.active_connections);
        println!("  空闲连接: {}", self.idle_connections);
        println!("  总请求数: {}", self.total_requests);
        println!("  成功请求: {}", self.successful_requests);
        println!("  失败请求: {}", self.failed_requests);
        println!("  成功率: {:.2}%", self.success_rate());
    }
}

#[cfg(all(test, not(feature = "connection-pool")))]
mod tests {
    use super::*;

    #[test]
    fn test_pool_manager_creation() {
        let manager = ConnectionPoolManager::default();
        // 连接池功能未启用时，无需检查内部状态
        assert_eq!(manager.get_stats().len(), 0);
    }

    #[test]
    fn test_pool_config() {
        let config = PoolManagerConfig::default();
        assert_eq!(config.max_connections, 100);
        assert_eq!(config.min_idle, 10);
        assert!(config.enable_reuse);
    }
}

#[cfg(all(test, feature = "connection-pool"))]
mod pool_tests {
    use super::*;

    #[test]
    #[ignore] // 需要网络
    fn test_pool_creation_with_connection() {
        let manager = ConnectionPoolManager::default();
        let result = manager.get_pool("example.com", 80);
        assert!(result.is_ok());

        let pool = result.unwrap();

        // 获取一个连接
        let conn_result = pool.get();
        // 可能会失败（如果无法连接），但不应该 panic
        if let Ok(_conn) = conn_result {
            println!("成功获取连接");
        }
    }

    #[test]
    fn test_pool_stats() {
        let manager = ConnectionPoolManager::default();
        let stats = manager.get_stats();
        // 初始应该没有连接池
        assert_eq!(stats.len(), 0);
    }
}
