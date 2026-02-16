//! connection poolmanage
//!
//! based on netconnpool implementconnectionreuse and lifecyclemanage

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

/// connection poolmanageer
#[cfg(feature = "connection-pool")]
pub struct ConnectionPoolManager {
    /// connection poolinstance ( by  host:port group)
    pools: Arc<Mutex<HashMap<String, Arc<Pool>>>>,
    /// defaultconfiguration
    config: PoolManagerConfig,
    /// HTTP/2 sessionpool (Fix: implementtrue multiplexreuse)
    #[cfg(feature = "http2")]
    h2_session_pool: Arc<super::h2_session_pool::H2SessionPool>,
    /// HTTP/3 sessionpool
    #[cfg(feature = "http3")]
    h3_session_pool: Arc<super::h3_session_pool::H3SessionPool>,
}

#[cfg(feature = "connection-pool")]
impl Default for ConnectionPoolManager {
    fn default() -> Self {
        Self::new(PoolManagerConfig::default())
    }
}

/// connection poolmanageer (noneconnection poolFeatures when 占bit)
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

/// connection poolmanageerconfiguration
#[derive(Debug, Clone)]
pub struct PoolManagerConfig {
    /// maximumconnectioncount
    pub max_connections: usize,
    /// minimumempty闲connectioncount
    pub min_idle: usize,
    /// connectiontimeout
    pub connect_timeout: Duration,
    /// empty闲timeout
    pub idle_timeout: Duration,
    /// maximumlifecycle
    pub max_lifetime: Duration,
    /// whetherenabledconnectionreuse
    pub enable_reuse: bool,
    /// TCP Profile
    pub profile: Option<std::sync::Arc<fingerprint_profiles::BrowserProfile>>,
}

// 连接池配置默认值常量
const DEFAULT_MAX_CONNECTIONS: usize = 100; // 最大连接数
const DEFAULT_MIN_IDLE: usize = 10; // 最小空闲连接数
const DEFAULT_CONNECT_TIMEOUT_SECS: u64 = 30; // 连接超时（秒）
const DEFAULT_IDLE_TIMEOUT_SECS: u64 = 90; // 空闲超时（秒）
const DEFAULT_MAX_LIFETIME_SECS: u64 = 600; // 最大生命周期（秒）

impl Default for PoolManagerConfig {
    fn default() -> Self {
        Self {
            max_connections: DEFAULT_MAX_CONNECTIONS,
            min_idle: DEFAULT_MIN_IDLE,
            connect_timeout: Duration::from_secs(DEFAULT_CONNECT_TIMEOUT_SECS),
            idle_timeout: Duration::from_secs(DEFAULT_IDLE_TIMEOUT_SECS),
            max_lifetime: Duration::from_secs(DEFAULT_MAX_LIFETIME_SECS),
            enable_reuse: true,
            profile: None,
        }
    }
}

impl ConnectionPoolManager {
    /// Create a newconnection poolmanageer
    #[cfg(feature = "connection-pool")]
    pub fn new(config: PoolManagerConfig) -> Self {
        Self {
            pools: Arc::new(Mutex::new(HashMap::new())),
            config,
            #[cfg(feature = "http2")]
            h2_session_pool: Arc::new(super::h2_session_pool::H2SessionPool::default()),
            #[cfg(feature = "http3")]
            h3_session_pool: Arc::new(super::h3_session_pool::H3SessionPool::default()),
        }
    }

    #[cfg(not(feature = "connection-pool"))]
    pub fn new(config: PoolManagerConfig) -> Self {
        Self { config }
    }

    /// Get HTTP/2 sessionpool
    #[cfg(all(feature = "connection-pool", feature = "http2"))]
    pub fn h2_session_pool(&self) -> &Arc<super::h2_session_pool::H2SessionPool> {
        &self.h2_session_pool
    }

    /// Get HTTP/3 sessionpool
    #[cfg(all(feature = "connection-pool", feature = "http3"))]
    pub fn h3_session_pool(&self) -> &Arc<super::h3_session_pool::H3SessionPool> {
        &self.h3_session_pool
    }

    /// Get or Createconnection pool
    #[cfg(feature = "connection-pool")]
    pub fn get_pool(&self, host: &str, port: u16) -> Result<Arc<Pool>> {
        let key = format!("{}:{}", host, port);
        let mut pools = self.pools.lock().map_err(|e| {
            HttpClientError::ConnectionFailed(format!("connection poollockfailure: {}", e))
        })?;

        if let Some(pool) = pools.get(&key) {
            return Ok(pool.clone());
        }

        // Create a newconnection pool
        let pool_config = self.create_pool_config(host, port);
        let pool = Pool::new(pool_config).map_err(|e| {
            HttpClientError::ConnectionFailed(format!("Createconnection poolfailure: {:?}", e))
        })?;

        let pool = Arc::new(pool);
        pools.insert(key, pool.clone());

        Ok(pool)
    }

    #[cfg(not(feature = "connection-pool"))]
    pub fn get_pool(&self, _host: &str, _port: u16) -> Result<()> {
        Err(HttpClientError::ConnectionFailed(
            "connection poolFeaturesnotenabled，请use --features connection-pool compile"
                .to_string(),
        ))
    }

    /// Createconnection poolconfiguration
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

            // provide Dialer functionfromCreate TCP connection
            // Note: hereunable todirectlyaccess config.profile, because dialer is closepackage
            // TCP Profile should in Createconnection poolbeforethenapplication to config in
            dialer: Some(Box::new(move |_protocol| {
                use std::net::{IpAddr, SocketAddr, ToSocketAddrs};

                let addrs: Vec<SocketAddr> = (host.as_str(), port)
                    .to_socket_addrs()
                    .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?
                    .collect();

                // priorityuse IPv4, avoid in "none IPv6 route"environment in appear `Network is unreachable`.
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
                    // Note: Currently using standard connect method
                    // Future enhancement: Apply TCP profile configuration in connection pool
                    match TcpStream::connect_timeout(&addr, connect_timeout) {
                        Ok(s) => return Ok(ConnectionType::Tcp(s)),
                        Err(e) => last_err = Some(e),
                    }
                }

                Err(Box::new(
                    last_err.unwrap_or_else(|| std::io::Error::other("no resolved addresses")),
                )
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

    /// Getstatisticsinfo
    #[cfg(feature = "connection-pool")]
    pub fn get_stats(&self) -> Vec<PoolStats> {
        let pools = match self.pools.lock() {
            Ok(p) => p,
            Err(e) => {
                eprintln!("warning: connection poollockfailure: {}", e);
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

    /// cleanupempty闲connection
    #[cfg(feature = "connection-pool")]
    pub fn cleanup_idle(&self) {
        // netconnpool willautomaticcleanup, hereonly is provideinterface
        if let Ok(pools) = self.pools.lock() {
            println!("connection poolstatus: {} 端点", pools.len());
        }
    }

    #[cfg(not(feature = "connection-pool"))]
    pub fn cleanup_idle(&self) {}

    /// closeallconnection pool
    #[cfg(feature = "connection-pool")]
    pub fn shutdown(&self) {
        if let Ok(mut pools) = self.pools.lock() {
            for (_, pool) in pools.iter() {
                let _ = pool.close();
            }
            pools.clear();
            println!("allconnection poolalreadyclose");
        }
    }

    #[cfg(not(feature = "connection-pool"))]
    pub fn shutdown(&self) {}
}

/// connection poolstatisticsinfo
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
    /// Getsuccess率
    pub fn success_rate(&self) -> f64 {
        if self.total_requests == 0 {
            return 0.0;
        }
        (self.successful_requests as f64 / self.total_requests as f64) * 100.0
    }

    /// printstatisticsinfo
    pub fn print(&self) {
        println!("\n📊 connection poolstatistics: {}", self.endpoint);
        println!(" 总connectioncount: {}", self.total_connections);
        println!(" activeconnection: {}", self.active_connections);
        println!(" empty闲connection: {}", self.idle_connections);
        println!(" 总requestcount: {}", self.total_requests);
        println!(" successrequest: {}", self.successful_requests);
        println!(" failurerequest: {}", self.failed_requests);
        println!(" success率: {:.2}%", self.success_rate());
    }
}

#[cfg(all(test, not(feature = "connection-pool")))]
mod tests {
    use super::*;

    #[test]
    fn test_pool_manager_creation() {
        let manager = ConnectionPoolManager::default();
        // connection poolFeaturesnotenabled when , no needCheckinside部status
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
    #[ignore] // neednetwork
    fn test_pool_creation_with_connection() {
        let manager = ConnectionPoolManager::default();
        let result = manager.get_pool("example.com", 80);
        assert!(result.is_ok());

        let pool = result.unwrap();

        // Getanconnection
        let conn_result = pool.get();
        // maywillfailure ( if unable toconnection), but不should panic
        if let Ok(_conn) = conn_result {
            println!("successGetconnection");
        }
    }

    #[test]
    fn test_pool_stats() {
        let manager = ConnectionPoolManager::default();
        let stats = manager.get_stats();
        // initialbeginningshouldnoconnection pool
        assert_eq!(stats.len(), 0);
    }
}
