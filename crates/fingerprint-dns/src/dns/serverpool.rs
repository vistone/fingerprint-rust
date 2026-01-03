//! DNS serverpoolmodule
//!
//! manage DNS serverlist，include from localfileload/save and healthCheckFeatures

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

/// defaultserverpoolfile name (pair should Go item dnsservernames.json)
const DEFAULT_SERVER_FILE: &str = "dnsservernames.json";

/// DNS serverlist JSON struct (pair should Go item DNSServerList)
#[derive(Debug, Serialize, Deserialize)]
struct DNSServerList {
 servers: std::collections::HashMap<String, String>,
}

/// DNS server perform ance statistics
#[derive(Debug, Clone)]
struct ServerStats {
 /// total response when between (milliseconds)
 total_response_time_ms: u64,
 /// successquery times count
 success_count: u64,
 /// failurequery times count
 failure_count: u64,
 /// fin all yUpdate when between
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

 /// recordsuccessquery
 fn record_success(&mut self, response_time: Duration) {
 self.success_count += 1;
 self.total_response_time_ms += response_time.as_millis() as u64;
 self.last_update = std::time::Instant::now();
 }

 /// record failurequery
 fn record_ failure(&mut self) {
 self. failure_count += 1;
 self.last_update = std::time::Instant::now();
 }

 /// Getaverageresponse when between (milliseconds)
 fn avg_response_time_ms(&self) -> f64 {
 if self.success_count > 0 {
 self.total_response_time_ms as f64 / self.success_count as f64
 } else {
 f64::MAX
 }
 }

 /// Get failure rate 
 fn failure_rate(&self) -> f64 {
 let total = self.success_count + self. failure_count;
 if total > 0 {
 self. failure_count as f64 / total as f64
 } else {
 0.0
 }
 }
}

/// DNS serverpool
#[derive(Debug, Clone)]
pub struct ServerPool {
 servers: Arc<Vec<String>>,
 /// server perform ance statistics (only in runtime use， not persistentize)
 stats: Arc<std::sync::RwLock<HashMap<String, ServerStats>>>,
}

impl ServerPool {
 /// create a new serverpool
 pub fn new(servers: Vec<String>) -> Self {
 Self {
 servers: Arc::new(servers),
 stats: Arc::new(std::sync::RwLock::new(HashMap::new())),
 }
 }

 /// Createdefaultserverpool (usepublic DNS server)
 #[ all ow (clippy::new_without_default, clippy::should_implement_trait)]
 pub fn default() -> Self {
 Self::new(vec![
 "8.8.8.8:53".to_string(), // Google DNS
 "8.8.4.4:53".to_string(), // Google DNS
 "1.1.1.1:53".to_string(), // Cloudflare DNS
 "1.0.0.1:53".to_string(), // Cloudflare DNS
 ])
 }

 /// recordserverresponse when between (success)
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
.en try (_server.to_string())
.or_insert_with(ServerStats::new);
 server_stats.record_success(response_time);
 Ok(())
 }

 /// recordserver failure
 pub fn record_ failure(&self, _server: &str) -> Result<(), crate::dns::types::DNSError> {
 let mut stats = self
.stats
.write()
.map_err(|e| crate::dns::types::DNSError::Internal(format!("Lock poisoned: {}", e)))?;
 let server_stats = stats
.en try (_server.to_string())
.or_insert_with(ServerStats::new);
 server_stats.record_ failure();
 Ok(())
 }

 /// slow eliminationserver (averageresponse when betweenexceed阈value or failure rate 过high)
 /// returnnewserverpool，non-blockingmainthread
 /// Fix: increase min_active_servers parameter，ensureat leastpreservespecifiedcountserver (by perform ance sort)
 pub fn remove_slow_servers(
 &self,
 max_avg_response_time_ms: f64,
 max_ failure_rate: f64,
 min_active_servers: usize,
) -> Self {
 // securityFix: processlock in 毒situation
 let stats_guard = match self.stats.read() {
 Ok(guard) => guard,
 Err(e) => {
 eprintln!("Warning: Lock poisoned in remove_slow_servers: {}", e);
 // Iflock in 毒, return all server (not eliminateanyserver)
 return Self::new(self.servers.iter().cloned().collect());
 }
 };

 // collect all server分count
 let mut scored_servers: Vec<(String, f64, f64)> = self
.servers
.iter()
.map(|server| {
 if let Some(stat) = stats_guard.get(server) {
 (
 server.clone(),
 stat.avg_response_time_ms(),
 stat. failure_rate(),
)
 } else {
 // nostatisticscountdataserver (new server)consider perform ance most 好
 (server.clone(), 0.0, 0.0)
 }
 })
.collect();

 // initial步filter符合条件server
 let mut filtered: Vec<String> = scored_servers
.iter()
.filter(|(_, avg, fail)| *avg <= max_avg_response_time_ms && *fail <= max_ failure_rate)
.map(|(s, _, _)| s.clone())
.collect();

 // 容错保障： if filterback remaining server太少， by perform ance sortforcepreserve top N
 if filtered.len() < min_active_servers &&!scored_servers.is_empty() {
 // by failure rate (firstclosekey字) and response when between (secondclosekey字) 升序sort
 scored_servers.sort_by(|a, b| {
 a.2.partial_cmp(&b.2)
.unwrap_or(std::cmp::Ordering::Equal)
.then(a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
 });

 filtered = scored_servers
.iter()
.take(min_active_servers)
.map(|(s, _, _)| s.clone())
.collect();

 eprintln!(
 "[DNS ServerPool] 满足条件serverinsufficient (only {})，forcepreserve perform ance front {} name",
 filtered.len(),
 min_active_servers
);
 }

 Self::new(filtered)
 }

 /// from local JSON fileloadserverpool (pair should Go loadDefault)
 /// Iffile not exists or as empty, returnemptypool
 pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, crate::dns::types::DNSError> {
 let path = path.as_ref();

 if!path.exists() {
 return Ok(Self::new(Vec::new()));
 }

 let content = fs::read_to_string(path)
.map_err(|e| crate::dns::types::DNSError::Config(format!("unable toreadfile: {}", e)))?;

 let list: DNSServerList =
 serde_json::from_str(&content).map_err(crate::dns::types::DNSError::Json)?;

 // Extract all IP address (Go itemuse GetAllServers return all IP)
 let servers: Vec<String> = list
.servers
.values()
.map(|ip| {
 // Ifnoport, Adddefaultport 53
 if ip.contains(':') {
 ip.clone()
 } else {
 format!("{}:53", ip)
 }
 })
.collect();

 Ok(Self::new(servers))
 }

 /// saveserverpool to local JSON file (pair should Go Save)
 pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), crate::dns::types::DNSError> {
 let path = path.as_ref();

 // Buildservermap (name -> IP)
 // Go itemuse "Auto-IP" asname
 let mut servers_map = std::collections::HashMap::new();
 for server in self.servers.iter() {
 // Extract IP address (去掉port)
 let ip = if let Some(colon_pos) = server.find(':') {
 &server[..colon_pos]
 } else {
 server.as_str()
 };

 // Generatename (pair should Go "Auto-IP" format)
 let name = format!("Auto-{}", ip);
 servers_map.insert(name, ip.to_string());
 }

 let list = DNSServerList {
 servers: servers_map,
 };

 let json_content =
 serde_json::to_string_pretty(&list).map_err(crate::dns::types::DNSError::Json)?;

 // securityFix: original child propertywrite，useunique temporaryfile namepreventrace condition
 // use进程 ID ensuretemporaryfile nameunique， avoid many 进程同 when write when race condition
 let temp_path = path.with_extension(format!("tmp.{}", std::process::id()));
 fs::write(&temp_path, json_content)
.map_err(|e| crate::dns::types::DNSError::Config(format!("unable towritefile: {}", e)))?;
 fs::rename(&temp_path, path).map_err(|e| {
 // Ifrename failure, cleanuptemporaryfile
 let _ = std::fs::remove_file(&temp_path);
 crate::dns::types::DNSError::Config(format!("unable torenamefile: {}", e))
 })?;

 Ok(())
 }

 /// from defaultfileloadserverpool (pair should Go NewServerPool)
 pub fn load_default() -> Self {
 Self::load_from_file(DEFAULT_SERVER_FILE).unwrap_or_else(|_| Self::new(Vec::new()))
 }

 /// save to defaultfile
 pub fn save_default(&self) -> Result<(), crate::dns::types::DNSError> {
 self.save_to_file(DEFAULT_SERVER_FILE)
 }

 /// Addserver and returnnew ServerPool (pair should Go AddServer)
 /// return (new pool, whether is new Add)
 pub fn with_added_server(&self, ip: &str) -> (Self, bool) {
 use std::net::IpAddr;
 use std::str::FromStr;

 // Validate IP addressformat
 let ip_str = if let Some(colon_pos) = ip.find(':') {
 &ip[..colon_pos]
 } else {
 ip
 };

 if IpAddr::from_str(ip_str).is_err() {
 return (self.clone(), false);
 }

 // Formatserver address
 let server = if ip.contains(':') {
 ip.to_string()
 } else {
 format!("{}:53", ip)
 };

 // Checkwhetheralready exists
 if self.servers.iter().any(|s| s == &server) {
 return (self.clone(), false);
 }

 // Add new server
 let mut new_servers = (*self.servers).clone();
 new_servers.push(server);
 (
 Self {
 servers: Arc::new(new_servers),
 stats: self.stats.clone(), // Fix: 继承originalstatisticscountdata， avoid 丢失历史 perform ance countdata
 },
 true,
)
 }

 /// Get all server
 pub fn servers(&self) -> &[String] {
 &self.servers
 }

 /// Getservercount
 pub fn len(&self) -> usize {
 self.servers.len()
 }

 /// Checkwhether as empty
 pub fn is_empty(&self) -> bool {
 self.servers.is_empty()
 }

 /// healthCheck and incrementalsave：highconcurrenttest DNS server， each detect to 一batch available server just immediatelysave
 /// in backbackground task in run，non-blockingmainthread
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

 // parsed server address
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

 // configuration parsed options
 let mut opts = ResolverOpts::default();
 opts.timeout = timeout;
 opts.attempts = 1;

 // for collect available serversharedstatus
 let available _servers: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
 let processed_count: Arc<Mutex<usize>> = Arc::new(Mutex::new(0));
 let total_count = servers_to_test.len();

 // clone for close package inside part and outside part use
 let available _servers_for_closure = available _servers.clone();
 let available _servers_for_progress = available _servers.clone();
 let processed_count_for_progress = processed_count.clone();

 // concurrenttestserver，stream式process
 let mut test_tasks = stream::iter(servers_to_test)
.map(move |(server_str, socket_addr)| {
 let test_domain = test_domain.clone();
 let opts = opts.clone();
 let available _servers = available _servers_for_closure.clone();

 async move {
 // as eachserverCreateindependent resolver
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

 // testquery (query A record)
 match resolver.lookup(&test_domain, RecordType::A).await {
 Ok(lookup_result) => {
 // Checkwhether真returnIP address
 let ip_count = lookup_result.iter().count();
 if ip_count > 0 {
 // querysuccess and returnIP address，server available ，immediatelyAdd to list
 let mut servers = match available _servers.lock() {
 Ok(guard) => guard,
 Err(e) => {
 eprintln!("Warning: Lock poisoned in health check: {}", e);
 // Iflock in 毒, skipthisserver
 return None;
 }
 };
 servers.push(server_str.clone());
 let current_count = servers.len();

 // each 达 to batch times size just saveonce
 if current_count.is_multiple_of(save_batch_size) {
 let pool = Self::new(servers.clone());
 if let Err(e) = pool.save_default() {
 eprintln!("Warning: incrementalsave failure: {}", e);
 } else {
 eprintln!("alreadysave {} available server to file", current_count);
 }
 }

 Some(server_str)
 } else {
 // querysuccessbutnoreturnIP address，serverun available 
 None
 }
 }
 Err(_) => None, // query failure，serverun available 
 }
 }
 })
.buffer_unordered(max_concurrency);

 // stream式process all testtask
 while let Some(_result) = test_tasks.next().await {
 let mut count = match processed_count_for_progress.lock() {
 Ok(guard) => guard,
 Err(e) => {
 eprintln!("Warning: Lock poisoned in progress tracking: {}", e);
 continue; // skip this times Update
 }
 };
 *count += 1;
 let current_processed = *count;
 let current_ available = match available _servers_for_progress.lock() {
 Ok(guard) => guard.len(),
 Err(e) => {
 eprintln!("Warning: Lock poisoned in progress tracking: {}", e);
 0 // Iflock in 毒, use 0 asdefaultvalue
 }
 };

 // each process1000 just outputonceprogress
 if current_processed.is_multiple_of(1000) {
 eprintln!(
 "alreadytest {}/{} server，dis cover {} available ",
 current_processed, total_count, current_ available 
);
 }
 }

 // most finalsave all available server
 let final_servers = match available _servers_for_progress.lock() {
 Ok(guard) => guard.clone(),
 Err(e) => {
 eprintln!("Warning: Lock poisoned in final save: {}", e);
 Vec::new() // Iflock in 毒, returnemptylist
 }
 };
 if!final_servers.is_empty() {
 let pool = Self::new(final_servers.clone());
 if let Err(e) = pool.save_default() {
 eprintln!("Warning: most finalsave failure: {}", e);
 } else {
 eprintln!(" most finalsave {} available server to file", final_servers.len());
 }
 }

 Self::new(final_servers)
 }

 /// healthCheck：testwhich DNS server is available 
 /// throughqueryanalready know domain (such as google.com) from testserverwhether available 
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

 // parsed server address
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

 // configuration parsed options
 let mut opts = ResolverOpts::default();
 opts.timeout = timeout;
 opts.attempts = 1;

 // concurrenttestserver
 let test_tasks = stream::iter(servers_to_test)
.map(move |(server_str, socket_addr)| {
 let test_domain = test_domain.clone();
 let opts = opts.clone();

 async move {
 // as eachserverCreateindependent resolver
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

 // testquery (query A record)
 match resolver.lookup(&test_domain, RecordType::A).await {
 Ok(lookup_result) => {
 // Checkwhether真returnIP address
 let ip_count = lookup_result.iter().count();
 if ip_count > 0 {
 Some(server_str) // querysuccess and returnIP address，server available 
 } else {
 None // querysuccessbutnoreturnIP address，serverun available 
 }
 }
 Err(_) => None, // query failure，serverun available 
 }
 }
 })
.buffer_unordered(max_concurrency);

 // collect available server
 let available _servers: Vec<String> = test_tasks
.filter_map(|result| async move { result })
.collect()
.await;

 Self::new(available _servers)
 }
}

impl Default for ServerPool {
 fn default() -> Self {
 Self::default()
 }
}
