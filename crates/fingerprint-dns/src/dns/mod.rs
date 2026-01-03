//! DNS pre parsed library 
//!
//! provide DNS parsed service，regular parsed domainlist， and set成 IPInfo.io Get IP geographicinfo。
//!
//! ## Features
//!
//! - ✅ **concurrent DNS parsed **：supporthighconcurrent DNS query
//! - ✅ ** many formatconfiguration**：support JSON、YAML、TOML threeconfigurationformat
//! - ✅ **IP geographicinfo**：set成 IPInfo.io Getdetailedgeographicbit置 and ISP info
//! - ✅ **intelligentintervaladjust**：dis cover new IP when high频detect，otherwise指countbackoff
//! - ✅ ** many formatoutput**：support JSON、YAML、TOML threeoutputformat
//! - ✅ ** original child propertywrite**：usetemporaryfileensurecountdatasecurity
//! - ✅ **易于set成**：providesimple Start/Stop interface，supportas library use

mod collector;
mod config;
mod ipinfo;
mod resolver;
mod serverpool;
mod service;
mod storage;
mod types;

pub use collector::ServerCollector;
pub use config::load_config;
pub use ipinfo::IPInfoClient;
pub use resolver::DNSResolver;
pub use serverpool::ServerPool;
pub use service::Service;
pub use storage::{load_domain_ips, save_domain_ips};
pub use types::*;
