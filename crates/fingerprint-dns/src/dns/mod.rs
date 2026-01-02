//! DNS 预Parse库
//!
//! provide DNS Parseservice，regularParsedomainlist，并set成 IPInfo.io Get IP 地理info。
//!
//! ## Features
//!
//! - ✅ **concurrent DNS Parse**：support高concurrent DNS query
//! - ✅ **多formatconfiguration**：support JSON、YAML、TOML 三种configurationformat
//! - ✅ **IP 地理info**：set成 IPInfo.io Getdetailed的地理bit置 and ISP info
//! - ✅ **智能intervaladjust**：discovernew IP  when 高频detect，otherwise指count退避
//! - ✅ **多formatoutput**：support JSON、YAML、TOML 三种outputformat
//! - ✅ **原child性write**：usetemporaryfileensurecount据security
//! - ✅ **易于set成**：providesimple Start/Stop interface，supportas库use

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
