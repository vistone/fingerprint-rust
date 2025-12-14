//! 导出 TLS 配置为 JSON
//!
//! 用法：
//! cargo run --example export_config <profile_name> [output_file]
//!
//! 示例：
//! cargo run --example export_config chrome_133 chrome_133.json

use fingerprint::{mapped_tls_clients, export::export_config_json};
use std::env;
use std::fs;
use std::io::Write;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <profile_name> [output_file]", args[0]);
        eprintln!("Available profiles:");
        let mut profiles: Vec<_> = mapped_tls_clients().keys().collect();
        profiles.sort();
        for name in profiles {
            eprintln!("  - {}", name);
        }
        std::process::exit(1);
    }

    let profile_name = &args[1];
    let output_file = if args.len() > 2 {
        Some(&args[2])
    } else {
        None
    };

    let profiles = mapped_tls_clients();
    let profile = profiles.get(profile_name.as_str())
        .ok_or_else(|| format!("Profile not found: {}", profile_name))?;

    let spec = profile.get_client_hello_spec()?;
    let json = export_config_json(&spec)?;

    if let Some(path) = output_file {
        let mut file = fs::File::create(path)?;
        file.write_all(json.as_bytes())?;
        println!("Exported configuration for '{}' to '{}'", profile_name, path);
    } else {
        println!("{}", json);
    }

    Ok(())
}
