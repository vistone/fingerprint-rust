//! DNS preParselibrary
//!
//! provide DNS Parse service, regularParsedomainlist, 并setbecome IPInfo.io Get IP geographicinfo.
//!
//! ## Features
//!
//! - ✅ **concurrent DNS Parse**：supporthighconcurrent DNS query
//! - ✅ **multipleformatconfiguration**：support JSON, YAML, TOML threeconfigurationformat
//! - ✅ **IP geographicinfo**：setbecome IPInfo.io Getdetailedgeographicbitplace and ISP info
//! - ✅ **intelligentintervaladjust**：discovernew IP when highfrequencydetect, otherwisepointcountbackoff
//! - ✅ **multipleformatoutput**：support JSON, YAML, TOML threeoutputformat
//! - ✅ **originalchildpropertywrite**：usetemporaryfileensurecountdatasecurity
//! - ✅ **easy于setbecome**：providesimple Start/Stop interface, supportaslibraryuse
//! - ✅ **DNS 缓存**：内存缓存功能，减少重复解析，提高性能

mod cache;
mod collector;
mod config;
mod ipinfo;
mod resolver;
mod serverpool;
mod service;
mod storage;
mod types;

pub use cache::{CachedDNSResolver, DNSCache};
pub use collector::ServerCollector;
pub use config::load_config;
pub use ipinfo::IPInfoClient;
pub use resolver::{DNSResolver, DNSResolverTrait};
pub use serverpool::ServerPool;
pub use service::Service;
pub use storage::{load_domain_ips, save_domain_ips};
pub use types::*;
