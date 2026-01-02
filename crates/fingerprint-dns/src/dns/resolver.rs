//! DNS Parseermodule
//!
//! provideconcurrent DNS ParseFeatures，usecustom DNS serverlist

use crate::dns::serverpool::ServerPool;
use crate::dns::types::{DNSError, DNSResult, DomainIPs, IPInfo};
use std::net::IpAddr;
use std::sync::Arc;
use std::time::Duration;

use hickory_resolver::proto::rr::{RData, RecordType};
use hickory_resolver::{
 config::{NameServerConfig, Protocol, ResolverConfig, ResolverOpts},
 TokioAsyncResolver,
};

/// DNS Parseer
pub struct DNSResolver {
 /// DNS querytimeout duration
 timeout: Duration,
 /// DNS serverpool
 server_pool: Arc<ServerPool>,
 /// Fix: cache resolver instance，avoidfrequentCreate and destroy
 /// use Arc<Mutex<HashMap>> storeeach DNS server resolver
 resolver_cache:
 Arc<std::sync::Mutex<std::collections::HashMap<String, Arc<TokioAsyncResolver>>>>,
}

impl DNSResolver {
 /// Create a new DNS Parseer（usedefault DNS server）
 pub fn new(timeout: Duration) -> Self {
 Self {
 timeout,
 server_pool: Arc::new(ServerPool::default()),
 resolver_cache: Arc::new(std::sync::Mutex::new(std::collections::HashMap::new())),
 }
 }

 /// usespecified DNS serverpoolCreateParseer
 pub fn with_server_pool(timeout: Duration, server_pool: Arc<ServerPool>) -> Self {
 Self {
 timeout,
 server_pool,
 resolver_cache: Arc::new(std::sync::Mutex::new(std::collections::HashMap::new())),
 }
 }

 /// Parsedomainall IP address（IPv4 and IPv6）
 pub async fn resolve(&self, domain: &str) -> Result<DNSResult, DNSError> {
 eprintln!(
 "[DNS Resolver] ========== startParsedomain: {} ==========",
 domain
 );
 let mut domain_ips = DomainIPs::new();

 // Parse IPv4
 eprintln!("[DNS Resolver] startParse IPv4 address...");
 if let Ok(ipv4_addrs) = self.resolve_aaaa_or_a(domain, false).await {
 eprintln!(
 "[DNS Resolver] IPv4 Parsesuccess，obtain {} address",
 ipv4_addrs.len()
 );
 domain_ips.ipv4 = ipv4_addrs;
 } else {
 eprintln!("[DNS Resolver] IPv4 Parsefailure");
 }

 // Parse IPv6
 eprintln!("[DNS Resolver] startParse IPv6 address...");
 if let Ok(ipv6_addrs) = self.resolve_aaaa_or_a(domain, true).await {
 eprintln!(
 "[DNS Resolver] IPv6 Parsesuccess，obtain {} address",
 ipv6_addrs.len()
 );
 domain_ips.ipv6 = ipv6_addrs;
 } else {
 eprintln!("[DNS Resolver] IPv6 Parsefailure");
 }

 eprintln!(
 "[DNS Resolver] ========== domainParsecomplete: {} (IPv4: {}, IPv6: {} ) ==========",
 domain,
 domain_ips.ipv4.len(),
 domain_ips.ipv6.len()
 );

 Ok(DNSResult {
 domain: domain.to_string(),
 ips: domain_ips,
 })
 }

 /// Parse IPv4 (A) or IPv6 (AAAA) record
 /// usecollect to 全球 DNS serverperformquery
 async fn resolve_aaaa_or_a(&self, domain: &str, ipv6: bool) -> Result<Vec<IPInfo>, DNSError> {
 self.resolve_with_hickory(domain, ipv6).await
 }

 /// use hickory-resolver perform DNS query，concurrentquerymultiple DNS server以Getallmay IP
 async fn resolve_with_hickory(
 &self,
 domain: &str,
 ipv6: bool,
 ) -> Result<Vec<IPInfo>, DNSError> {
 use futures::stream::{self, StreamExt};
 use std::collections::HashSet;
 use std::net::SocketAddr;
 use std::str::FromStr;

 // from serverpool in Get DNS serverlist
 let servers = self.server_pool.servers();
 eprintln!("[DNS Resolver] startParsedomain: {} (IPv6: {})", domain, ipv6);
 eprintln!("[DNS Resolver] serverpool总count: {}", servers.len());

 // useallserverconcurrentquery（不limitcount）
 // Go item ResolveDomain use pool.GetAllServers() Getallserver，concurrentquery
 // failureserverwill被ignore，successserverreturn IP will被collect并deduplicate
 eprintln!("[DNS Resolver] willqueryall {} server", servers.len());

 let servers_with_sockets: Vec<_> = servers
.iter()
.filter_map(|server_str| {
 // Parseserveraddressformat：can is "ip:port" or only "ip"（defaultport 53）
 let (ip_str, port) = if let Some(colon_pos) = server_str.find(':') {
 let ip = &server_str[..colon_pos];
 let port = server_str[colon_pos + 1..].parse::<u16>().unwrap_or(53);
 (ip.to_string(), port)
 } else {
 (server_str.to_string(), 53u16)
 };

 // Parse IP address
 if let Ok(ip_addr) = IpAddr::from_str(&ip_str) {
 Some((server_str.to_string(), SocketAddr::new(ip_addr, port)))
 } else {
 None
 }
 })
.collect();

 let total_servers = servers_with_sockets.len();
 eprintln!("[DNS Resolver] Parsebackserveraddresscount: {}", total_servers);

 if servers_with_sockets.is_empty() {
 eprintln!("[DNS Resolver] noavailableserveraddress，usesystem DNS");
 return self.resolve_with_system(domain, ipv6).await;
 }

 // recordtype
 let record_type = if ipv6 {
 RecordType::AAAA
 } else {
 RecordType::A
 };
 eprintln!("[DNS Resolver] queryrecordtype: {:?}", record_type);

 // configurationParseoptions
 let mut opts = ResolverOpts::default();
 opts.timeout = Duration::from_millis(1000); // singleservertimeout duration 1 seconds
 opts.attempts = 1; // eachserver只tryonce，becauseweconcurrentquerymultiple
 eprintln!(
 "[DNS Resolver] singleservertimeout: {:?}, overalltimeout: {:?}",
 opts.timeout, self.timeout
 );

 // concurrentquerymultiple DNS server
 // usetimeoutwrap，avoidsingle慢server阻塞整query
 let server_pool = self.server_pool.clone();
 let query_timeout = self.timeout; // use resolver overalltimeout duration
 // Fix: shared resolver cache
 let resolver_cache = self.resolver_cache.clone();
 let query_tasks = stream::iter(servers_with_sockets)
.map(move |(server_str, socket_addr)| {
 let domain = domain.to_string();
 let opts = opts.clone();
 let record_type = record_type;
 let server_str = server_str.clone();
 let server_pool = server_pool.clone();
 let query_timeout = query_timeout;
 let resolver_cache = resolver_cache.clone();

 async move {
 let start_time = std::time::Instant::now();

 // usetimeoutwrapquery，avoidsingleserver阻塞
 let query_result = tokio::time::timeout(query_timeout, async {
 // Fix: reuse resolver instance，avoidfrequentCreate and destroy
 // use server_str as key 来cache resolver
 let resolver = {
 let mut cache = resolver_cache.lock().unwrap_or_else(|e| {
 eprintln!("warning: resolver cachelockfailure: {}", e);
 // Iflockfailure, Createannewempty HashMap 并relock定
 drop(e.into_inner());
 resolver_cache.lock().expect("unable toGet resolver cachelock")
 });

 if let Some(cached) = cache.get(&server_str) {
 cached.clone()
 } else {
 // Create a new resolver 并cache
 let mut config = ResolverConfig::new();
 let name_server = NameServerConfig {
 socket_addr,
 protocol: Protocol::Udp,
 tls_dns_name: None,
 trust_negative_responses: false,
 bind_addr: None,
 };
 config.add_name_server(name_server);

 let resolver = Arc::new(TokioAsyncResolver::tokio(config, opts.clone()));
 cache.insert(server_str.clone(), resolver.clone());
 resolver
 }
 };

 resolver.lookup(&domain, record_type).await
 }).await;

 // executequery
 match query_result {
 Ok(Ok(lookup)) => {
 let mut ips = Vec::new();
 let mut record_count = 0usize;

 // 遍历allrecord，collectall IP address
 for record in lookup.record_iter() {
 record_count += 1;
 if let Some(rdata) = record.data() {
 let ip_str = match rdata {
 RData::A(ipv4) if !ipv6 => {
 ipv4.to_string()
 }
 RData::AAAA(ipv6_addr) if ipv6 => {
 ipv6_addr.to_string()
 }
 _ => continue,
 };
 ips.push(ip_str);
 }
 }

 // recordsuccessresponse when between
 let response_time = start_time.elapsed();
 if !ips.is_empty() {
 // printdetailedlog，displayreturnall IP
 eprintln!("[DNS Query] ✅ server {} success，return {} IP（共 {} 条record），耗 when: {:?}",
 server_str, ips.len(), record_count, response_time);
 if ips.len() > 1 {
 eprintln!(" [DNS Query] return IP list: {}", ips.join(", "));
 }
 if let Err(e) = server_pool.record_success(&server_str, response_time) {
 eprintln!("Warning: recordserversuccessstatisticsfailure: {}", e);
 }
 } else {
 eprintln!("[DNS Query] ⚠️ server {} querysuccessbutnotreturn IP（共 {} 条record，buttypedoes not match），耗 when: {:?}",
 server_str, record_count, response_time);
 if let Err(e) = server_pool.record_failure(&server_str) {
 eprintln!("Warning: recordserverfailurestatisticsfailure: {}", e);
 }
 }
 Ok(ips)
 }
 Ok(Err(_)) | Err(_) => {
 // recordfailure（queryfailure or timeout），不printlog以decreaseoutput
 let _ = server_pool.record_failure(&server_str);
 // singleserverfailure不影响whole，returnemptyresult
 Ok::<Vec<String>, DNSError>(Vec::new())
 }
 }
 }
 })
.buffer_unordered(50); // Fix: 降低concurrentcount to 50，avoidfiledescribe符耗尽 and 资source爆炸

 eprintln!("[DNS Resolver] startconcurrentquery，concurrentcount: 50");

 // stream式collectresult，waitallserverresponse，collect尽may多 IP
 // for 大量server，increaseoveralltimeout duration
 let overall_timeout = Duration::from_secs(30); // overalltimeout 30 seconds，ensureallserver都有机willresponse
 let mut all_ips = HashSet::new(); // use HashSet automaticdeduplicate，same IP 只willpreservean
 let mut query_tasks = query_tasks;
 let mut success_count = 0usize;
 let mut failure_count = 0usize;
 let mut total_ips_received = 0usize; // statistics收 to 总 IP count（deduplicatefront）

 // usetimeout and stream式process，collect尽may多result
 let timeout_future = tokio::time::sleep(overall_timeout);
 tokio::pin!(timeout_future);
 let start_time = std::time::Instant::now();
 let mut last_log_time = std::time::Instant::now();
 let log_interval = Duration::from_millis(500); // 每500msprintonce进度

 eprintln!(
 "[DNS Resolver] startcollectqueryresult（overalltimeout: {:?}，总servercount: {}）",
 overall_timeout, total_servers
 );

 loop {
 tokio::select! {
 // Checkwhether有newqueryresult
 result = query_tasks.next() => {
 match result {
 Some(Ok(ips)) => {
 success_count += 1;
 let ips_count = ips.len(); // 先save IP count，avoidmovebackunable toaccess
 total_ips_received += ips_count; // statistics收 to 总 IP count（deduplicatefront）

 let before_count = all_ips.len();
 for ip in ips {
 all_ips.insert(ip); // HashSet automaticdeduplicate，same IP 只willpreservean
 }
 let after_count = all_ips.len();
 let new_ips_count = after_count - before_count;

 // Ifthisserverreturn IP in 有duplicate的, will in log in display
 if ips_count > new_ips_count {
 eprintln!("[DNS Resolver] serverreturn {} IP，其 in {} is new IP，{} is duplicate的（alreadyautomaticdeduplicate）",
 ips_count, new_ips_count, ips_count - new_ips_count);
 }

 // regularprint进度，displaydeduplicatestatistics
 if last_log_time.elapsed() >= log_interval {
 let duplicate_count = total_ips_received - all_ips.len();
 eprintln!("[DNS Resolver] 进度: {}/{} servercomplete，success {} ，failure {} ",
 success_count + failure_count, total_servers, success_count, failure_count);
 eprintln!("[DNS Resolver] IP statistics: 收 to {} IP，deduplicateback {} 唯一 IP，filter了 {} duplicate IP",
 total_ips_received, all_ips.len(), duplicate_count);
 last_log_time = std::time::Instant::now();
 }
 }
 Some(Err(_)) => {
 failure_count += 1;
 // regularprint进度
 if last_log_time.elapsed() >= log_interval {
 eprintln!("[DNS Resolver] 进度: {}/{} servercomplete，success {} ，failure {} ，alreadycollect IP: {} ",
 success_count + failure_count, total_servers, success_count, failure_count, all_ips.len());
 last_log_time = std::time::Instant::now();
 }
 // singlequeryfailure，continue
 }
 None => {
 // allquerycomplete
 let duplicate_count = total_ips_received - all_ips.len();
 eprintln!("[DNS Resolver] ✅ allquerycomplete: success {} ，failure {} ",
 success_count, failure_count);
 eprintln!("[DNS Resolver] IP deduplicatestatistics: 收 to {} IP，deduplicateback {} 唯一 IP，filter了 {} duplicate IP",
 total_ips_received, all_ips.len(), duplicate_count);
 break;
 }
 }
 }
 // timeout
 _ = &mut timeout_future => {
 let duplicate_count = total_ips_received - all_ips.len();
 eprintln!("[DNS Resolver] ⏱️ queryoveralltimeout（{}seconds），complete {}/{} server，success {} ，failure {} ",
 overall_timeout.as_secs(), success_count + failure_count, total_servers, success_count, failure_count);
 eprintln!("[DNS Resolver] IP deduplicatestatistics: 收 to {} IP，deduplicateback {} 唯一 IP，filter了 {} duplicate IP",
 total_ips_received, all_ips.len(), duplicate_count);
 break;
 }
 }
 }

 let total_time = start_time.elapsed();
 let duplicate_count = total_ips_received - all_ips.len();
 eprintln!("[DNS Resolver] querycomplete，总耗 when: {:?}", total_time);
 eprintln!("[DNS Resolver] 最final IP deduplicatestatistics: 收 to {} IP，deduplicateback {} 唯一 IP，filter了 {} duplicate IP（deduplicate率: {:.2}%）",
 total_ips_received, all_ips.len(), duplicate_count,
 if total_ips_received > 0 { (duplicate_count as f64 / total_ips_received as f64) * 100.0 } else { 0.0 });

 // convert to IPInfo list
 // Note: all_ips is HashSet，alreadyautomaticdeduplicate，same IP 只willpreservean
 let ip_infos: Vec<IPInfo> = all_ips.into_iter().map(IPInfo::new).collect();

 eprintln!(
 "[DNS Resolver] convert to IPInfo，最finalreturn {} 唯一 IP address（alreadydeduplicate）",
 ip_infos.len()
 );

 if ip_infos.is_empty() {
 // Ifallquery都failure, back to system DNS
 eprintln!("[DNS Resolver] ⚠️ allquery都failure，back to system DNS");
 self.resolve_with_system(domain, ipv6).await
 } else {
 Ok(ip_infos)
 }
 }

 /// usesystem DNS Parse（back方案）
 async fn resolve_with_system(&self, domain: &str, ipv6: bool) -> Result<Vec<IPInfo>, DNSError> {
 use std::net::ToSocketAddrs;

 let addr_str = format!("{}:80", domain);
 let mut ip_infos = Vec::new();

 if let Ok(addrs) = addr_str.to_socket_addrs() {
 for addr in addrs {
 let ip = addr.ip();
 // Based on ipv6 parameterfilteraddresstype
 match (ipv6, ip) {
 (true, IpAddr::V6(_)) => {
 ip_infos.push(IPInfo::new(ip.to_string()));
 }
 (false, IpAddr::V4(_)) => {
 ip_infos.push(IPInfo::new(ip.to_string()));
 }
 _ => {
 // does not matchtype，skip
 }
 }
 }
 }

 Ok(ip_infos)
 }

 /// bulkParsedomain（concurrent）
 pub async fn resolve_many(
 &self,
 domains: Vec<String>,
 max_concurrency: usize,
 ) -> Vec<(String, Result<DNSResult, DNSError>)> {
 use futures::stream::{self, StreamExt};

 let tasks = stream::iter(domains)
.map(|domain| {
 let resolver = self;
 async move {
 let result = resolver.resolve(&domain).await;
 (domain, result)
 }
 })
.buffer_unordered(max_concurrency);

 tasks.collect().await
 }
}

#[cfg(test)]
mod tests {
 use super::*;

 #[tokio::test]
 async fn test_resolve() {
 let resolver = DNSResolver::new(Duration::from_secs(4));
 let result = resolver.resolve("google.com").await;
 assert!(result.is_ok());
 let dns_result = result.unwrap();
 assert!(!dns_result.ips.ipv4.is_empty() || !dns_result.ips.ipv6.is_empty());
 }
}
