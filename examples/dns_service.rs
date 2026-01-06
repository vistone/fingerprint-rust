//! DNS 服务示例程序
//!
//! Usage:
//! cargo run --example dns_service --features dns -- -config config/config.json #[cfg(feature = "dns")]
use fingerprint::dns::{load_config, Service}; #[cfg(feature = "dns")]
use std::env; #[cfg(feature = "dns")]
use std::process; #[cfg(feature = "dns")]
#[tokio::main]
async fn main() { // 解析命令行参数 let args: Vec<String> = env::args().collect(); let config_path = if args.len() > 2 && args[1] == "-config" { &args[2] } else { eprintln!("使用方法: {} -config <config_file>", args[0]); process::exit(1); }; // 加载配置（验证配has性） let _config = match load_config(config_path) { Ok(cfg) => cfg, Err(e) => { eprintln!("加载配置失败: {}", e); process::exit(1); } }; // 创建服务 let service = match Service::from_config_file(config_path) { Ok(svc) => svc, Err(e) => { eprintln!("创建服务失败: {}", e); process::exit(1); } }; // 启动服务 println!("启动 DNS 服务..."); println!("按 Ctrl+C 停止服务"); if let Err(e) = service.start().await { eprintln!("Service error: {}", e); process::exit(1); } println!("DNS 服务已停止");
} #[cfg(not(feature = "dns"))]
fn main() { println!("This example requires enabling 'dns' feature"); println!( "使用方法: cargo run --example dns_service --features dns -- -config config/config.json" );
}
