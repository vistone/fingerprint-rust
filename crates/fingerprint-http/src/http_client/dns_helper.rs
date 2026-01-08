//! DNS 解析辅助模块
//!
//! 提供 DNS 预解析和缓存功能集成到 HTTP 客户端

use std::net::{IpAddr, SocketAddr, ToSocketAddrs};
use std::sync::Arc;
use std::time::Duration;

/// DNS 解析辅助器
///
/// 提供带缓存的 DNS 解析功能，可选择性地集成到 HTTP 客户端
#[derive(Clone, Debug)]
pub struct DNSHelper {
    /// 缓存的域名到 IP 地址映射
    cache: Arc<std::sync::RwLock<std::collections::HashMap<String, Vec<IpAddr>>>>,
    /// 缓存 TTL
    ttl: Duration,
    /// 缓存时间戳
    cache_time: Arc<std::sync::RwLock<std::collections::HashMap<String, std::time::Instant>>>,
}

impl DNSHelper {
    /// 创建新的 DNS 辅助器
    ///
    /// # Arguments
    /// * `ttl` - 缓存 TTL，默认 300 秒（5 分钟）
    pub fn new(ttl: Duration) -> Self {
        Self {
            cache: Arc::new(std::sync::RwLock::new(std::collections::HashMap::new())),
            ttl,
            cache_time: Arc::new(std::sync::RwLock::new(std::collections::HashMap::new())),
        }
    }

    /// 解析域名到 IP 地址（带缓存）
    ///
    /// # Arguments
    /// * `host` - 主机名或域名
    /// * `port` - 端口号
    ///
    /// # Returns
    /// 解析后的 SocketAddr 列表
    pub fn resolve(&self, host: &str, port: u16) -> std::io::Result<Vec<SocketAddr>> {
        // 如果是 IP 地址，直接返回
        if let Ok(ip) = host.parse::<IpAddr>() {
            return Ok(vec![SocketAddr::new(ip, port)]);
        }

        // 尝试从缓存获取
        if let Some(cached_ips) = self.get_cached(host) {
            return Ok(cached_ips.iter().map(|ip| SocketAddr::new(*ip, port)).collect());
        }

        // 缓存未命中，执行实际解析
        let addr = format!("{}:{}", host, port);
        let addrs: Vec<SocketAddr> = addr.to_socket_addrs()?.collect();
        
        // 提取 IP 地址并缓存
        let ips: Vec<IpAddr> = addrs.iter().map(|addr| addr.ip()).collect();
        self.put_cache(host, ips);

        Ok(addrs)
    }

    /// 从缓存获取 IP 地址
    fn get_cached(&self, host: &str) -> Option<Vec<IpAddr>> {
        // 检查缓存是否过期
        if let Ok(cache_time) = self.cache_time.read() {
            if let Some(time) = cache_time.get(host) {
                if time.elapsed() > self.ttl {
                    // 缓存已过期
                    return None;
                }
            } else {
                return None;
            }
        }

        // 从缓存获取
        if let Ok(cache) = self.cache.read() {
            cache.get(host).cloned()
        } else {
            None
        }
    }

    /// 将 IP 地址存入缓存
    fn put_cache(&self, host: &str, ips: Vec<IpAddr>) {
        if let Ok(mut cache) = self.cache.write() {
            cache.insert(host.to_string(), ips);
        }
        if let Ok(mut cache_time) = self.cache_time.write() {
            cache_time.insert(host.to_string(), std::time::Instant::now());
        }
    }

    /// 预热缓存（预先解析一组域名）
    ///
    /// # Arguments
    /// * `domains` - 域名列表
    pub fn warmup(&self, domains: &[&str]) {
        for domain in domains {
            let _ = self.resolve(domain, 443); // 默认使用 HTTPS 端口
        }
    }

    /// 清除缓存
    pub fn clear_cache(&self) {
        if let Ok(mut cache) = self.cache.write() {
            cache.clear();
        }
        if let Ok(mut cache_time) = self.cache_time.write() {
            cache_time.clear();
        }
    }

    /// 使特定域名的缓存失效
    pub fn invalidate(&self, host: &str) {
        if let Ok(mut cache) = self.cache.write() {
            cache.remove(host);
        }
        if let Ok(mut cache_time) = self.cache_time.write() {
            cache_time.remove(host);
        }
    }

    /// 获取缓存统计信息
    ///
    /// # Returns
    /// (cached_domains, expired_domains)
    pub fn stats(&self) -> (usize, usize) {
        let total = if let Ok(cache) = self.cache.read() {
            cache.len()
        } else {
            0
        };

        let expired = if let Ok(cache_time) = self.cache_time.read() {
            cache_time.values().filter(|time| time.elapsed() > self.ttl).count()
        } else {
            0
        };

        (total, expired)
    }

    /// 清理过期的缓存条目
    ///
    /// # Returns
    /// 清理的条目数量
    pub fn cleanup_expired(&self) -> usize {
        let mut expired_keys = Vec::new();

        // 找出过期的键
        if let Ok(cache_time) = self.cache_time.read() {
            for (key, time) in cache_time.iter() {
                if time.elapsed() > self.ttl {
                    expired_keys.push(key.clone());
                }
            }
        }

        // 删除过期的条目
        let count = expired_keys.len();
        for key in expired_keys {
            self.invalidate(&key);
        }

        count
    }
}

impl Default for DNSHelper {
    fn default() -> Self {
        Self::new(Duration::from_secs(300)) // 默认 5 分钟 TTL
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dns_helper_basic() {
        let helper = DNSHelper::new(Duration::from_secs(60));

        // 解析 localhost
        let addrs = helper.resolve("localhost", 8080).unwrap();
        assert!(!addrs.is_empty());
        assert_eq!(addrs[0].port(), 8080);

        // 验证缓存
        let (cached, _) = helper.stats();
        assert_eq!(cached, 1);
    }

    #[test]
    fn test_dns_helper_ip_address() {
        let helper = DNSHelper::new(Duration::from_secs(60));

        // 直接使用 IP 地址
        let addrs = helper.resolve("127.0.0.1", 8080).unwrap();
        assert_eq!(addrs.len(), 1);
        assert_eq!(addrs[0].port(), 8080);

        // IP 地址不应该被缓存
        let (cached, _) = helper.stats();
        assert_eq!(cached, 0);
    }

    #[test]
    fn test_dns_helper_cache_expiration() {
        let helper = DNSHelper::new(Duration::from_millis(100));

        // 解析域名
        let _ = helper.resolve("localhost", 8080);

        // 立即检查：应该被缓存
        let (cached, expired) = helper.stats();
        assert_eq!(cached, 1);
        assert_eq!(expired, 0);

        // 等待过期
        std::thread::sleep(Duration::from_millis(150));

        // 检查过期
        let (cached, expired) = helper.stats();
        assert_eq!(cached, 1);
        assert_eq!(expired, 1);

        // 清理过期条目
        let cleaned = helper.cleanup_expired();
        assert_eq!(cleaned, 1);

        let (cached, _) = helper.stats();
        assert_eq!(cached, 0);
    }

    #[test]
    fn test_dns_helper_warmup() {
        let helper = DNSHelper::new(Duration::from_secs(60));

        // 预热缓存
        helper.warmup(&["localhost"]);

        // 验证缓存
        let (cached, _) = helper.stats();
        assert_eq!(cached, 1);
    }

    #[test]
    fn test_dns_helper_clear_cache() {
        let helper = DNSHelper::new(Duration::from_secs(60));

        // 添加缓存
        let _ = helper.resolve("localhost", 8080);
        let (cached, _) = helper.stats();
        assert_eq!(cached, 1);

        // 清除缓存
        helper.clear_cache();
        let (cached, _) = helper.stats();
        assert_eq!(cached, 0);
    }
}
