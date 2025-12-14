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

/// 连接池管理器（无连接池功能时的占位）
#[cfg(not(feature = "connection-pool"))]
pub struct ConnectionPoolManager {
    config: PoolManagerConfig,
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

    /// 创建默认管理器
    pub fn default() -> Self {
        Self::new(PoolManagerConfig::default())
    }

    /// 获取或创建连接池
    #[cfg(feature = "connection-pool")]
    pub fn get_pool(&self, host: &str, port: u16) -> Result<Arc<Pool>> {
        let key = format!("{}:{}", host, port);
        let mut pools = self.pools.lock().unwrap();

        if let Some(pool) = pools.get(&key) {
            return Ok(pool.clone());
        }

        // 创建新的连接池
        let pool_config = self.create_pool_config(host, port);
        let pool = Pool::NewPool(pool_config)
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
        let port = port;

        PoolConfig {
            Mode: netconnpool::PoolMode::Client,
            MaxConnections: self.config.max_connections,
            MinConnections: self.config.min_idle,
            MaxIdleConnections: self.config.max_connections,
            ConnectionTimeout: self.config.connect_timeout,
            IdleTimeout: self.config.idle_timeout,
            MaxLifetime: self.config.max_lifetime,
            GetConnectionTimeout: self.config.connect_timeout,
            HealthCheckInterval: Duration::from_secs(30),
            HealthCheckTimeout: Duration::from_secs(3),
            ConnectionLeakTimeout: Duration::from_secs(300),

            // 提供 Dialer 函数来创建 TCP 连接
            Dialer: Some(Box::new(move || {
                let addr = format!("{}:{}", host, port);
                TcpStream::connect(&addr)
                    .map(ConnectionType::Tcp)
                    .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
            })),

            Listener: None,
            Acceptor: None,
            HealthChecker: None,
            CloseConn: None,
            OnCreated: None,
            OnBorrow: None,
            OnReturn: None,
            EnableStats: true,
            EnableHealthCheck: true,
            ClearUDPBufferOnReturn: true,
            UDPBufferClearTimeout: Duration::from_millis(100),
            MaxBufferClearPackets: 100,
        }
    }

    /// 获取统计信息
    #[cfg(feature = "connection-pool")]
    pub fn get_stats(&self) -> Vec<PoolStats> {
        let pools = self.pools.lock().unwrap();
        pools
            .iter()
            .map(|(key, pool)| {
                let stats = pool.Stats();
                PoolStats {
                    endpoint: key.clone(),
                    total_connections: stats.TotalConnectionsCreated,
                    active_connections: stats.CurrentActiveConnections,
                    idle_connections: stats.CurrentIdleConnections,
                    total_requests: stats.TotalGetRequests,
                    successful_requests: stats.SuccessfulGets,
                    failed_requests: stats.FailedGets,
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
        let pools = self.pools.lock().unwrap();
        println!("连接池状态: {} 个端点", pools.len());
    }

    #[cfg(not(feature = "connection-pool"))]
    pub fn cleanup_idle(&self) {}

    /// 关闭所有连接池
    #[cfg(feature = "connection-pool")]
    pub fn shutdown(&self) {
        let mut pools = self.pools.lock().unwrap();
        for (_, pool) in pools.iter() {
            let _ = pool.Close();
        }
        pools.clear();
        println!("所有连接池已关闭");
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
        let conn_result = pool.GetTCP();
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
