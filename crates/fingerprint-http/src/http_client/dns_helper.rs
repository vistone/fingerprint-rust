// ! DNS parse辅助module
//! DNS helper utilities for the HTTP client.
// ! provide DNS 预parseandcache functionality集成到 HTTP client

use std::net::{IpAddr, SocketAddr, ToSocketAddrs};
use std::sync::Arc;
use std::time::Duration;

// / DNS parse辅助器
/// DNS resolution helper with caching.
// / provide带cacheof DNS parse functionality，optional择性地集成到 HTTP client
#[derive(Clone, Debug)]
pub struct DNSHelper {
    // / cacheofdomain到 IP addressmap
    cache: Arc<std::sync::RwLock<std::collections::HashMap<String, Vec<IpAddr>>>>,
    // / cache TTL
    ttl: Duration,
    // / cachetime戳
    cache_time: Arc<std::sync::RwLock<std::collections::HashMap<String, std::time::Instant>>>,
}

impl DNSHelper {
    // / createnew DNS 辅助器
    ///
    /// # Arguments
    // / * `ttl` - cache TTL，default 300 秒（5 分钟）
    pub fn new(ttl: Duration) -> Self {
        Self {
            cache: Arc::new(std::sync::RwLock::new(std::collections::HashMap::new())),
            ttl,
            cache_time: Arc::new(std::sync::RwLock::new(std::collections::HashMap::new())),
        }
    }

    // / parsedomain到 IP address（带cache）
    ///
    /// # Arguments
    // / * `host` - host名或domain
    // / * `port` - port号
    ///
    /// # Returns
    // / parse后of SocketAddr list
    pub fn resolve(&self, host: &str, port: u16) -> std::io::Result<Vec<SocketAddr>> {
        // 如果是 IP address，直接return
        if let Ok(ip) = host.parse::<IpAddr>() {
            return Ok(vec![SocketAddr::new(ip, port)]);
        }

        // 尝试从cacheget
        if let Some(cached_ips) = self.get_cached(host) {
            return Ok(cached_ips
                .iter()
                .map(|ip| SocketAddr::new(*ip, port))
                .collect());
        }

        // cache未命中，执行实际parse
        let addr = format!("{}:{}", host, port);
        let addrs: Vec<SocketAddr> = addr.to_socket_addrs()?.collect();

        // extract IP address并cache
        let ips: Vec<IpAddr> = addrs.iter().map(|addr| addr.ip()).collect();
        self.put_cache(host, ips);

        Ok(addrs)
    }

    // / 从cacheget IP address
    fn get_cached(&self, host: &str) -> Option<Vec<IpAddr>> {
        // checkcache是否过期
        if let Ok(cache_time) = self.cache_time.read() {
            if let Some(time) = cache_time.get(host) {
                if time.elapsed() > self.ttl {
                    // cache已过期
                    return None;
                }
            } else {
                return None;
            }
        }

        // 从cacheget
        if let Ok(cache) = self.cache.read() {
            cache.get(host).cloned()
        } else {
            None
        }
    }

    // / 将 IP address存入cache
    fn put_cache(&self, host: &str, ips: Vec<IpAddr>) {
        if let Ok(mut cache) = self.cache.write() {
            cache.insert(host.to_string(), ips);
        }
        if let Ok(mut cache_time) = self.cache_time.write() {
            cache_time.insert(host.to_string(), std::time::Instant::now());
        }
    }

    // / warm upcache（预先parse一组domain）
    ///
    /// # Arguments
    // / * `domains` - domainlist
    pub fn warmup(&self, domains: &[&str]) {
        for domain in domains {
            let _ = self.resolve(domain, 443); // defaultuse HTTPS port
        }
    }

    // / 清除cache
    pub fn clear_cache(&self) {
        if let Ok(mut cache) = self.cache.write() {
            cache.clear();
        }
        if let Ok(mut cache_time) = self.cache_time.write() {
            cache_time.clear();
        }
    }

    // / 使特定domainofcache失效
    pub fn invalidate(&self, host: &str) {
        if let Ok(mut cache) = self.cache.write() {
            cache.remove(host);
        }
        if let Ok(mut cache_time) = self.cache_time.write() {
            cache_time.remove(host);
        }
    }

    // / getcachestatisticsinfo
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
            cache_time
                .values()
                .filter(|time| time.elapsed() > self.ttl)
                .count()
        } else {
            0
        };

        (total, expired)
    }

    // / cleanup过期ofcache条目
    ///
    /// # Returns
    // / cleanupof条目count
    pub fn cleanup_expired(&self) -> usize {
        let mut expired_keys = Vec::new();

        // 找出过期of键
        if let Ok(cache_time) = self.cache_time.read() {
            for (key, time) in cache_time.iter() {
                if time.elapsed() > self.ttl {
                    expired_keys.push(key.clone());
                }
            }
        }

        // delete过期of条目
        let count = expired_keys.len();
        for key in expired_keys {
            self.invalidate(&key);
        }

        count
    }
}

impl Default for DNSHelper {
    fn default() -> Self {
        Self::new(Duration::from_secs(300)) // default 5 分钟 TTL
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dns_helper_basic() {
        let helper = DNSHelper::new(Duration::from_secs(60));

        // parse localhost
        let addrs = helper.resolve("localhost", 8080).unwrap();
        assert!(!addrs.is_empty());
        assert_eq!(addrs[0].port(), 8080);

        // validatecache
        let (cached, _) = helper.stats();
        assert_eq!(cached, 1);
    }

    #[test]
    fn test_dns_helper_ip_address() {
        let helper = DNSHelper::new(Duration::from_secs(60));

        // 直接use IP address
        let addrs = helper.resolve("127.0.0.1", 8080).unwrap();
        assert_eq!(addrs.len(), 1);
        assert_eq!(addrs[0].port(), 8080);

        // IP address不应该被cache
        let (cached, _) = helper.stats();
        assert_eq!(cached, 0);
    }

    #[test]
    fn test_dns_helper_cache_expiration() {
        let helper = DNSHelper::new(Duration::from_millis(100));

        // parsedomain
        let _ = helper.resolve("localhost", 8080);

        // 立即check：应该被cache
        let (cached, expired) = helper.stats();
        assert_eq!(cached, 1);
        assert_eq!(expired, 0);

        // 等待过期
        std::thread::sleep(Duration::from_millis(150));

        // check过期
        let (cached, expired) = helper.stats();
        assert_eq!(cached, 1);
        assert_eq!(expired, 1);

        // cleanup过期条目
        let cleaned = helper.cleanup_expired();
        assert_eq!(cleaned, 1);

        let (cached, _) = helper.stats();
        assert_eq!(cached, 0);
    }

    #[test]
    fn test_dns_helper_warmup() {
        let helper = DNSHelper::new(Duration::from_secs(60));

        // warm upcache
        helper.warmup(&["localhost"]);

        // validatecache
        let (cached, _) = helper.stats();
        assert_eq!(cached, 1);
    }

    #[test]
    fn test_dns_helper_clear_cache() {
        let helper = DNSHelper::new(Duration::from_secs(60));

        // 添加cache
        let _ = helper.resolve("localhost", 8080);
        let (cached, _) = helper.stats();
        assert_eq!(cached, 1);

        // 清除cache
        helper.clear_cache();
        let (cached, _) = helper.stats();
        assert_eq!(cached, 0);
    }
}
