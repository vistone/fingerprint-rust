//! DNS 预Parse库
//!
//! provide DNS Parse服务，定期Parsedomainlist，并集成 IPInfo.io Get IP 地理info。
//!
//! ## Features
//!
//! - ✅ **并发 DNS Parse**：support高并发 DNS query
//! - ✅ **多formatconfiguration**：support JSON、YAML、TOML 三种configurationformat
//! - ✅ **IP 地理info**：集成 IPInfo.io Get详细的地理bit置 and ISP info
//! - ✅ **智能间隔调整**：发现new IP  when 高频检测，otherwise指count退避
//! - ✅ **多formatoutput**：support JSON、YAML、TOML 三种outputformat
//! - ✅ **原child性write**：usetemporaryfile确保count据security
//! - ✅ **易于集成**：provide简单 Start/Stop interface，support作为库use

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
