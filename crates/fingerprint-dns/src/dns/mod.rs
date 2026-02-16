//! DNS 预解析库
//!
//! 提供 DNS 解析服务，定期解析域名列表，并通过 IPInfo.io 获取 IP 地理信息。
//!
//! ## 功能特性
//!
//! - ✅ **并发 DNS 解析**：支持高并发 DNS 查询
//! - ✅ **多种格式配置**：支持 JSON、YAML、TOML 三种配置格式
//! - ✅ **IP 地理信息**：通过 IPInfo.io 获取详细的地理位置和 ISP 信息
//! - ✅ **智能间隔调整**：发现新 IP 时高频检测，否则逐步退避
//! - ✅ **多种格式输出**：支持 JSON、YAML、TOML 三种输出格式
//! - ✅ **原子性写入**：使用临时文件确保数据安全
//! - ✅ **易于集成**：提供简单的 Start/Stop 接口，支持作为库使用
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
