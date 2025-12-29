//! DNS 预解析库
//!
//! 提供 DNS 解析服务，定期解析域名列表，并集成 IPInfo.io 获取 IP 地理信息。
//!
//! ## 特性
//!
//! - ✅ **并发 DNS 解析**：支持高并发 DNS 查询
//! - ✅ **多格式配置**：支持 JSON、YAML、TOML 三种配置格式
//! - ✅ **IP 地理信息**：集成 IPInfo.io 获取详细的地理位置和 ISP 信息
//! - ✅ **智能间隔调整**：发现新 IP 时高频检测，否则指数退避
//! - ✅ **多格式输出**：支持 JSON、YAML、TOML 三种输出格式
//! - ✅ **原子性写入**：使用临时文件确保数据安全
//! - ✅ **易于集成**：提供简单的 Start/Stop 接口，支持作为库使用

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
