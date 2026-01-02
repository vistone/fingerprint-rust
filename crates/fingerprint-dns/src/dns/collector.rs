//! DNS servercollect器module
//!
//! collectavailable DNS server，include from  public-dns.info Getpublic DNS serverlist

use crate::dns::serverpool::ServerPool;
use crate::dns::types::DNSError;
use std::time::Duration;

/// DNS servercollect器
pub struct ServerCollector;

impl ServerCollector {
    ///  from  public-dns.info Getpublic DNS serverlist
    /// Corresponds to Go version's collectPublicDNS function
    pub async fn collect_public_dns(timeout: Option<Duration>) -> Result<ServerPool, DNSError> {
        let timeout = timeout.unwrap_or(Duration::from_secs(30));
        let url = "https://public-dns.info/nameservers.txt";

        // useiteminside部 HttpClient
        let config = fingerprint_http::http_client::HttpClientConfig {
            connect_timeout: timeout,
            read_timeout: timeout,
            write_timeout: timeout,
            ..Default::default()
        };
        let client = fingerprint_http::http_client::HttpClient::new(config);

        //  in asyncupdown文中executesync HTTP request
        let response = tokio::task::spawn_blocking(move || client.get(url))
            .await
            .map_err(|e| DNSError::Http(format!("task join error: {}", e)))?
            .map_err(|e| DNSError::Http(format!("HTTP request failed: {}", e)))?;

        if !response.is_success() {
            return Err(DNSError::Http(format!(
                "failed to fetch nameservers: HTTP {}",
                response.status_code
            )));
        }

        // readresponse文本
        let text = String::from_utf8_lossy(&response.body).to_string();

        // Parse文本，每行an IP address
        let mut servers = Vec::new();
        for line in text.lines() {
            let line = line.trim();

            // skipempty行 and comment
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            // Validatewhether为valid IP address
            if is_valid_ip_address(line) {
                // Ifnoport, Adddefaultport 53
                let server = if line.contains(':') {
                    line.to_string()
                } else {
                    format!("{}:53", line)
                };
                servers.push(server);
            }
        }

        if servers.is_empty() {
            // IfGetfailure, returndefaultserver
            eprintln!("Warning: No servers fetched from public-dns.info, using defaults");
            return Ok(ServerPool::default());
        }

        Ok(ServerPool::new(servers))
    }

    /// collectsystem DNS server
    pub fn collect_system_dns() -> ServerPool {
        // 目frontreturndefault的public DNS server
        // not来canextension为 from systemconfigurationread
        ServerPool::default()
    }

    ///  from configurationfilecollect DNS server
    pub fn collect_from_config(_servers: Vec<String>) -> ServerPool {
        // Ifconfiguration了customserver, use它们
        // otherwiseusedefaultserver
        ServerPool::default()
    }

    /// Validate并Updateexistingfile中 DNS server
    ///  from fileloadallserver，performhealthCheck，只preserveavailable的server并save回file
    ///
    /// # Parameters
    /// - `test_domain`:  for test的domain，default为 "google.com"
    /// - `test_timeout`: eachserver的testtimeout duration，default为 3 秒
    /// - `max_concurrency`: maximumconcurrenttestcount，default为 100
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

        //  from fileloadallserver
        let file_path = Path::new(DEFAULT_SERVER_FILE);
        if !file_path.exists() {
            return Err(DNSError::Config(format!(
                "file {} 不 exists",
                DEFAULT_SERVER_FILE
            )));
        }

        let pool = ServerPool::load_from_file(file_path)?;
        let total_count = pool.len();

        if total_count == 0 {
            return Err(DNSError::Config("file中no DNS server".to_string()));
        }

        eprintln!(" from fileload了 {} 个 DNS server", total_count);
        eprintln!("正 in test DNS serveravailable性（testdomain: {}）...", test_domain);

        // performhealthCheck
        let validated_pool = pool
            .health_check(test_domain, test_timeout, max_concurrency)
            .await;

        let valid_count = validated_pool.len();
        let invalid_count = total_count - valid_count;

        eprintln!("healthCheckcomplete:");
        eprintln!("   总servercount: {}", total_count);
        eprintln!(
            "   availableserver: {} ({:.2}%)",
            valid_count,
            if total_count > 0 {
                (valid_count as f64 / total_count as f64) * 100.0
            } else {
                0.0
            }
        );
        eprintln!(
            "   unavailableserver: {} ({:.2}%)",
            invalid_count,
            if total_count > 0 {
                (invalid_count as f64 / total_count as f64) * 100.0
            } else {
                0.0
            }
        );

        // saveValidateback的server（先backup原file）
        if valid_count > 0 {
            let backup_path = format!("{}.backup", DEFAULT_SERVER_FILE);
            if let Err(e) = std::fs::copy(file_path, &backup_path) {
                eprintln!("Warning: unable toCreatebackupfile: {}", e);
            } else {
                eprintln!("alreadyCreatebackup: {}", backup_path);
            }

            validated_pool.save_default()?;
            eprintln!("alreadysave {} 个availableserver to file", valid_count);
        } else {
            return Err(DNSError::Config("noavailable DNS server".to_string()));
        }

        Ok((total_count, valid_count))
    }

    /// collectallavailable DNS server（pair应 Go  BootstrapPoolInternal）
    ///  from multiplesourcecollect，并 in savefrontperformhealthCheck，只preserveavailable的server
    pub async fn collect_all(timeout: Option<Duration>) -> ServerPool {
        // 先try from localfileload（pair应 Go  loadDefault）
        let pool = ServerPool::load_default();

        if !pool.is_empty() {
            eprintln!(
                " from localfileload了 {} 个 DNS server（alreadythroughValidate，directlyuse）",
                pool.len()
            );
            // fileinserveralreadythroughValidate，directlyuse，不perform全面Check
            // 只 in back台asyncdetect and 淘汰慢node，non-blockingmainthread
            return pool;
        }

        // Iffile不 exists or 为empty,  from networkcollect（pair应 Go  BootstrapPoolInternal）
        eprintln!("localfile不 exists or 为empty， from networkcollect DNS server...");

        match Self::collect_public_dns(timeout).await {
            Ok(new_pool) => {
                let new_count = new_pool.len();
                eprintln!(" from networkcollect了 {} 个 DNS server", new_count);

                //  in savefrontperformhealthCheck，只preserveavailable的server
                // use高concurrentdetect，每detect to 一batch就immediatelysave，fastcomplete不长 when 间阻塞
                eprintln!("正 in 高concurrenttest DNS serveravailable性（test哪些servercanreturn IP address）...");
                let test_timeout = Duration::from_secs(2); // decreasetimeout duration，加快detect
                let max_concurrency = 500; // 大幅increaseconcurrentcount，加快detectspeed
                let save_batch_size = 100; // 每detect to 100个availableserver就saveonce

                let validated_pool = new_pool
                    .health_check_and_save_incremental(
                        "google.com",
                        test_timeout,
                        max_concurrency,
                        save_batch_size,
                    )
                    .await;

                let valid_count = validated_pool.len();
                let invalid_count = new_count - valid_count;
                eprintln!("healthCheckcomplete:");
                eprintln!("   总servercount: {}", new_count);
                eprintln!(
                    "   availableserver: {} ({:.2}%)",
                    valid_count,
                    if new_count > 0 {
                        (valid_count as f64 / new_count as f64) * 100.0
                    } else {
                        0.0
                    }
                );
                eprintln!(
                    "   unavailableserver: {} ({:.2}%)",
                    invalid_count,
                    if new_count > 0 {
                        (invalid_count as f64 / new_count as f64) * 100.0
                    } else {
                        0.0
                    }
                );

                // filealready in 增量save过程中Update了，directlyreturn
                if valid_count > 0 {
                    validated_pool
                } else {
                    eprintln!("Warning: noavailable DNS server，usedefaultserver");
                    ServerPool::default()
                }
            }
            Err(e) => {
                eprintln!(
                    "Warning: Failed to collect public DNS servers: {}, using defaults",
                    e
                );
                ServerPool::default()
            }
        }
    }
}

/// Validatewhether为valid IP address（IPv4  or  IPv6）
fn is_valid_ip_address(s: &str) -> bool {
    use std::net::{IpAddr, SocketAddr};

    // Ifincludingport号, 先Parse SocketAddr
    if s.contains(':') && s.matches(':').count() <= 2 {
        // may是 IPv4:port format
        if s.parse::<SocketAddr>().is_ok() {
            return true;
        }
        // alsomay是 IPv6:port，butformat更复杂，needspecialprocess
        // simplifyprocess： if including []，tryParse
        if s.starts_with('[') {
            return s.parse::<SocketAddr>().is_ok();
        }
    }

    // tryParse为 IP address
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
    #[ignore] // neednetworkconnection，defaultskip
    async fn test_collect_public_dns() {
        let pool = ServerCollector::collect_public_dns(None).await;
        assert!(pool.is_ok());
        let pool = pool.unwrap();
        assert!(!pool.is_empty());
        println!("Collected {} DNS servers", pool.len());
    }
}
