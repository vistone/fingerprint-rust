//! DNS Service module
//!
//! provide DNS Parse service Start/Stop interface

use crate::dns::collector::ServerCollector;
use crate::dns::config::load_config;
use crate::dns::ipinfo::IPInfoClient;
use crate::dns::resolver::DNSResolver;
use crate::dns::serverpool::ServerPool;
use crate::dns::storage::{load_domain_ips, save_domain_ips};
use crate::dns::types::IPInfo;
use crate::dns::types::{DNSConfig, DNSError, DomainIPs};
use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{oneshot, RwLock};
use tokio::time::sleep;

/// DNS Service (Corresponds to Go version's Service)
pub struct Service {
 config: Arc<DNSConfig>,
 resolver: Arc<RwLock<DNSResolver>>, // use RwLock so that in start when Update
 ipinfo_client: Arc<IPInfoClient>,
 running: Arc<RwLock<bool>>,
 stop_tx: Arc<RwLock<Option<oneshot::Sender<()>>>>,
}

impl Service {
 /// Create a new Service instance
 pub fn new(config: DNSConfig) -> Result<Self, DNSError> {
 config.validate()?;

 // Parse timeout duration
 let dns_timeout = parse_duration(&config.dns_timeout).unwrap_or(Duration::from_secs(4));

 // HTTP timeout duration
 let http_timeout = parse_duration(&config.http_timeout).unwrap_or(Duration::from_secs(20));

 // use default DNS server Create resolver (will in start when replace as collected server)
 let resolver = Arc::new(RwLock::new(DNSResolver::new(dns_timeout)));
 let ipinfo_client = Arc::new(IPInfoClient::new(config.ipinfo_token.clone(), http_timeout));

 Ok(Self {
 config: Arc::new(config),
 resolver,
 ipinfo_client,
 running: Arc::new(RwLock::new(false)),
 stop_tx: Arc::new(RwLock::new(None)),
 })
 }

 /// Create a new Service instance，并usespecified DNS serverpool
 pub async fn with_server_pool(
 config: DNSConfig,
 server_pool: Arc<ServerPool>,
 ) -> Result<Self, DNSError> {
 config.validate()?;

 // Parse timeout duration
 let dns_timeout = parse_duration(&config.dns_timeout).unwrap_or(Duration::from_secs(4));

 // HTTP timeout duration
 let http_timeout = parse_duration(&config.http_timeout).unwrap_or(Duration::from_secs(20));

 // usespecified DNS serverpoolCreate resolver
 let resolver = Arc::new(RwLock::new(DNSResolver::with_server_pool(
 dns_timeout,
 server_pool,
 )));
 let ipinfo_client = Arc::new(IPInfoClient::new(config.ipinfo_token.clone(), http_timeout));

 Ok(Self {
 config: Arc::new(config),
 resolver,
 ipinfo_client,
 running: Arc::new(RwLock::new(false)),
 stop_tx: Arc::new(RwLock::new(None)),
 })
 }

 /// from configurationfileCreate Service
 pub fn from_config_file<P: AsRef<std::path::Path>>(path: P) -> Result<Self, DNSError> {
 let config = load_config(path)?;
 Self::new(config)
 }

 /// startservice ( in back台threadrun，non-blockingmainthread)
 /// automaticmaintain DNS serverpool (dnsservernames.json)，no needmanualinterferepre
 pub async fn start(&self) -> Result<(), DNSError> {
 // Checkwhetheralready in run
 {
 let mut running = self.running.write().await;
 if *running {
 return Err(DNSError::Config("service is already running".to_string()));
 }
 *running = true;
 }

 // load/collect DNS serverpool (pairshould Go NewServerPool)
 // priority from localfileload， if 不 exists or as empty，才 from networkcollect
 // collect_all alreadyprocess了：
 // - if file exists and is notempty：directlyuse，不performCheck
 // - if file不 exists or as empty： from networkcollect并performhealthCheckbacksave
 let mut server_pool = ServerCollector::collect_all(Some(Duration::from_secs(30))).await;
 eprintln!("currentserverpool有 {} DNS server", server_pool.len());

 // Ifserverpool as empty, use defaultserver
 if server_pool.is_empty() {
 eprintln!("Warning: noavailable DNS server，use defaultserver");
 server_pool = ServerPool::default();
 eprintln!("use default DNS server: {} ", server_pool.len());
 }

 // usethroughhealthCheckserverpoolUpdate resolver
 // Parse when useallthroughhealthCheckserverconcurrentquery
 let dns_timeout =
 parse_duration(&self.config.dns_timeout).unwrap_or(Duration::from_secs(4));
 let server_pool_arc = Arc::new(server_pool);
 let new_resolver = DNSResolver::with_server_pool(dns_timeout, server_pool_arc.clone());

 // replace resolver
 {
 let mut resolver_guard = self.resolver.write().await;
 *resolver_guard = new_resolver;
 }

 // Createstopchannel
 let (tx, mut rx) = oneshot::channel();
 {
 let mut stop_tx = self.stop_tx.write().await;
 *stop_tx = Some(tx);
 }

 // startbackbackground task：regularslow eliminationDNSserver (non-blockingmainthread)
 // reference Go itemdestinationimplement： in Parseprocess in recordperformance，back台regularcleanup慢node
 let resolver_for_cleanup = self.resolver.clone();
 let server_pool_for_cleanup = server_pool_arc.clone();
 let dns_timeout_for_cleanup = dns_timeout;
 tokio::spawn(async move {
 let cleanup_interval = Duration::from_secs(300); // 每5minutescleanuponce (pair应 Go 项destinationregularcleanup)
 let max_avg_response_time_ms = 2000.0; // averageresponse when betweenexceed2secondseliminate
 let max_failure_rate = 0.5; // failure率exceed50%eliminate

 loop {
 tokio::time::sleep(cleanup_interval).await;

 // slow eliminationserver (pairshould Go item RemoveSlowServers)
 let old_count = server_pool_for_cleanup.len();
 let min_active_servers = 5; // productionenvironmentdownsuggestat leastpreserve 5serveras保bottom
 let optimized_pool = server_pool_for_cleanup.remove_slow_servers(
 max_avg_response_time_ms,
 max_failure_rate,
 min_active_servers,
 );
 let new_count = optimized_pool.len();
 let removed_count = old_count - new_count;

 if removed_count > 0 {
 eprintln!(
 "[DNS Service] back台cleanup：eliminate了 {} 慢DNSserver (remaining {} )",
 removed_count, new_count
 );

 // Update resolver serverpool (pairshould Go itemdestinationUpdateserverpool)
 let new_resolver = DNSResolver::with_server_pool(
 dns_timeout_for_cleanup,
 Arc::new(optimized_pool),
 );
 {
 let mut resolver_guard = resolver_for_cleanup.write().await;
 *resolver_guard = new_resolver;
 }
 }
 }
 });

 // in back台threadstartservicemainloop (non-blockingmainthread)
 // use Arc wrapfield，candirectly in closepackage in use
 let config = self.config.clone();
 let resolver = self.resolver.clone();
 let ipinfo_client = self.ipinfo_client.clone();
 let running = self.running.clone();

 tokio::spawn(async move {
 // Parseinterval
 let base_interval =
 parse_duration(&config.interval).unwrap_or(Duration::from_secs(120));

 eprintln!("[DNS Service] servicealreadystart，will in back台run (non-blockingmainthread)");
 eprintln!(
 "[DNS Service] configuration: domainlist {} ，Checkinterval: {}，countdatadirectory: {}",
 config.domain_list.len(),
 config.interval,
 config.domain_ips_dir
 );

 // Createtemporary Service instance for call resolve_and_save_all
 // Note: resolve_and_save_all need &self，soweneedCreateanauxiliaryfunction
 // or one whodirectly in hereimplementParselogic

 // dynamicintervaladjust
 let mut current_interval = base_interval;
 let mut last_has_new_ips = false;

 loop {
 // Checkstopsignal
 if rx.try_recv().is_ok() {
 eprintln!("[DNS Service] 收 to stopsignal，正 in stopservice...");
 break;
 }

 // executeParse (useauxiliaryfunction)- waitParsecompleteback再waitinterval
 eprintln!("[DNS Service] startexecute DNS Parse...");
 let resolve_start = std::time::Instant::now();
 match resolve_and_save_all_internal(&resolver, &ipinfo_client, &config).await {
 Ok(has_new_ips) => {
 let resolve_duration = resolve_start.elapsed();
 eprintln!(
 "[DNS Service] DNS Parsecomplete，耗 when: {:.2}seconds",
 resolve_duration.as_secs_f64()
 );

 // intelligentintervaladjust：discovernew IP when highfrequencydetect，otherwisepointcountbackoff
 if has_new_ips {
 current_interval = base_interval;
 last_has_new_ips = true;
 eprintln!(
 "[DNS Service] discovernew IP，down次will in {} backexecute",
 format_duration(&current_interval)
 );
 } else {
 if last_has_new_ips {
 // before有new IP，current in no了，逐stepincreaseinterval
 current_interval = base_interval;
 last_has_new_ips = false;
 } else {
 // pointcountbackoff，but不exceed 10 倍basicinterval
 current_interval = (current_interval * 2).min(base_interval * 10);
 }
 eprintln!(
 "[DNS Service] notdiscovernew IP，down次will in {} backexecute",
 format_duration(&current_interval)
 );
 }
 }
 Err(e) => {
 let resolve_duration = resolve_start.elapsed();
 eprintln!(
 "[DNS Service] DNS Parseerror (耗 when: {:.2}seconds): {}",
 resolve_duration.as_secs_f64(),
 e
 );
 // error when usebasicinterval
 current_interval = base_interval;
 }
 }

 // Checkstopsignal ( in waitintervalfront)
 if rx.try_recv().is_ok() {
 eprintln!("[DNS Service] 收 to stopsignal，正 in stopservice...");
 break;
 }

 // waitcurrentinterval ( in Parsecompleteback)
 eprintln!(
 "[DNS Service] wait {} backexecutedownonceParse...",
 format_duration(&current_interval)
 );
 sleep(current_interval).await;
 }

 // stopservice
 {
 let mut running = running.write().await;
 *running = false;
 }

 eprintln!("[DNS Service] servicealreadystop");
 });

 eprintln!("[DNS Service] servicealready in back台start，mainthread不will被blocking");
 Ok(())
 }

 /// stopservice
 pub async fn stop(&self) -> Result<(), DNSError> {
 let mut stop_tx = self.stop_tx.write().await;
 if let Some(tx) = stop_tx.take() {
 let _ = tx.send(());
 }
 Ok(())
 }

 /// Checkservicewhether in run
 pub async fn is_running(&self) -> bool {
 *self.running.read().await
 }

 /// settingsbasicexecuteinterval
 pub fn set_interval(&self, _interval: Duration) {
 // Note: dynamicadjustpatterndown，actualintervalwillBased onwhetherdiscovernewIP而change
 // thisfunctionmain for staticpattern，itemfronttemporary不support
 }

 /// Parse并savealldomain IP info
 /// Note: 此methoditemfrontnotdirectlyuse，actualuse的 is resolve_and_save_all_internal
 #[allow(dead_code)]
 async fn resolve_and_save_all(&self) -> Result<bool, DNSError> {
 resolve_and_save_all_internal(&self.resolver, &self.ipinfo_client, &self.config).await
 }
}

/// auxiliaryfunction：Parse并savealldomain IP info (can in closepackage in use)
async fn resolve_and_save_all_internal(
 resolver: &Arc<RwLock<DNSResolver>>,
 ipinfo_client: &Arc<IPInfoClient>,
 config: &Arc<DNSConfig>,
) -> Result<bool, DNSError> {
 let mut has_new_ips = false;

 // concurrentParsealldomain (usecollect to DNS server)
 let resolver_guard = resolver.read().await;
 let results = resolver_guard
.resolve_many(config.domain_list.clone(), config.max_concurrency)
.await;
 drop(resolver_guard);

 // as eachdomain IP addressGetdetailedinfo
 for (domain, dns_result) in results {
 match dns_result {
 Ok(result) => {
 // Getexistingcountdata
 let existing = load_domain_ips(&domain, &config.domain_ips_dir)?;

 // ExtractallParse to IP (alreadydeduplicate)
 let all_ipv4: HashSet<String> = result
.ips
.ipv4
.iter()
.map(|ip_info| ip_info.ip.clone())
.collect();
 let all_ipv6: HashSet<String> = result
.ips
.ipv6
.iter()
.map(|ip_info| ip_info.ip.clone())
.collect();

 // from existingcountdata in Extractalready exists IP
 let existing_ipv4: HashSet<String> = existing
.as_ref()
.map(|e| e.ipv4.iter().map(|ip| ip.ip.clone()).collect())
.unwrap_or_default();
 let existing_ipv6: HashSet<String> = existing
.as_ref()
.map(|e| e.ipv6.iter().map(|ip| ip.ip.clone()).collect())
.unwrap_or_default();

 // find outnewdiscover IP (onlyquerythese)
 let new_ipv4: Vec<String> = all_ipv4.difference(&existing_ipv4).cloned().collect();
 let new_ipv6: Vec<String> = all_ipv6.difference(&existing_ipv6).cloned().collect();

 // Buildmostfinal domain_ips，先copyalready existscountdata
 let mut domain_ips = DomainIPs::new();

 // copyalready exists IPv4 info
 if let Some(existing) = &existing {
 for existing_ip in &existing.ipv4 {
 if all_ipv4.contains(&existing_ip.ip) {
 domain_ips.ipv4.push(existing_ip.clone());
 }
 }
 }

 // copyalready exists IPv6 info
 if let Some(existing) = &existing {
 for existing_ip in &existing.ipv6 {
 if all_ipv6.contains(&existing_ip.ip) {
 domain_ips.ipv6.push(existing_ip.clone());
 }
 }
 }

 // onlyquerynewdiscover IPv4 detailedinfo
 if !new_ipv4.is_empty() {
 eprintln!(
 "[DNS Service] discover {} new IPv4 address，正 in Getdetailedinfo...",
 new_ipv4.len()
 );
 let ipv4_results = ipinfo_client
.get_ip_infos(new_ipv4.clone(), config.max_ip_fetch_conc)
.await;

 for (ip, ip_result) in ipv4_results {
 match ip_result {
 Ok(mut ip_info) => {
 // preserveoriginalbeginning IP (because IPInfo mayreturndifferentformat)
 ip_info.ip = ip.clone();
 domain_ips.ipv4.push(ip_info);
 }
 Err(e) => {
 eprintln!("[DNS Service] Failed to get IP info for {}: {}", ip, e);
 // that is使failure，alsosavebasic IP info
 domain_ips.ipv4.push(IPInfo::new(ip));
 }
 }
 }
 }

 // onlyquerynewdiscover IPv6 detailedinfo
 if !new_ipv6.is_empty() {
 eprintln!(
 "[DNS Service] discover {} new IPv6 address，正 in Getdetailedinfo...",
 new_ipv6.len()
 );
 let ipv6_results = ipinfo_client
.get_ip_infos(new_ipv6.clone(), config.max_ip_fetch_conc)
.await;

 for (ip, ip_result) in ipv6_results {
 match ip_result {
 Ok(mut ip_info) => {
 ip_info.ip = ip.clone();
 domain_ips.ipv6.push(ip_info);
 }
 Err(e) => {
 eprintln!("[DNS Service] Failed to get IP info for {}: {}", ip, e);
 domain_ips.ipv6.push(IPInfo::new(ip));
 }
 }
 }
 }

 // Checkwhether有new IP
 if !new_ipv4.is_empty() || !new_ipv6.is_empty() {
 has_new_ips = true;
 }

 // save result
 save_domain_ips(&domain, &domain_ips, &config.domain_ips_dir)?;
 }
 Err(e) => {
 eprintln!("[DNS Service] Failed to resolve {}: {}", domain, e);
 }
 }
 }

 Ok(has_new_ips)
}

/// Format Duration as readablestring
fn format_duration(d: &Duration) -> String {
 let secs = d.as_secs();
 if secs < 60 {
 format!("{}seconds", secs)
 } else if secs < 3600 {
 format!("{}分{}seconds", secs / 60, secs % 60)
 } else {
 format!("{}小 when {}分{}seconds", secs / 3600, (secs % 3600) / 60, secs % 60)
 }
}

/// Parse when betweenstring (如 "2m", "30s", "1h")
fn parse_duration(s: &str) -> Option<Duration> {
 let s = s.trim();
 if s.is_empty() {
 return None;
 }

 let (num, unit) = if let Some(stripped) = s.strip_suffix("ns") {
 (stripped.parse::<u64>().ok()?, "ns")
 } else if let Some(stripped) = s.strip_suffix("us") {
 (stripped.parse::<u64>().ok()?, "us")
 } else if let Some(stripped) = s.strip_suffix("µs") {
 (stripped.parse::<u64>().ok()?, "us")
 } else if let Some(stripped) = s.strip_suffix("ms") {
 (stripped.parse::<u64>().ok()?, "ms")
 } else if let Some(stripped) = s.strip_suffix('s') {
 (stripped.parse::<u64>().ok()?, "s")
 } else if let Some(stripped) = s.strip_suffix('m') {
 (stripped.parse::<u64>().ok()?, "m")
 } else if let Some(stripped) = s.strip_suffix('h') {
 (stripped.parse::<u64>().ok()?, "h")
 } else {
 // tryassecondscountParse
 (s.parse::<u64>().ok()?, "s")
 };

 Some(match unit {
 "ns" => Duration::from_nanos(num),
 "us" | "µs" => Duration::from_micros(num),
 "ms" => Duration::from_millis(num),
 "s" => Duration::from_secs(num),
 "m" => Duration::from_secs(num * 60),
 "h" => Duration::from_secs(num * 3600),
 _ => return None,
 })
}

#[cfg(test)]
mod tests {
 use super::*;

 #[test]
 fn test_parse_duration() {
 assert_eq!(parse_duration("30s"), Some(Duration::from_secs(30)));
 assert_eq!(parse_duration("2m"), Some(Duration::from_secs(120)));
 assert_eq!(parse_duration("1h"), Some(Duration::from_secs(3600)));
 assert_eq!(parse_duration("500ms"), Some(Duration::from_millis(500)));
 }
}
