//! 打印 p0f 所有数据

use fingerprint_defense::passive::p0f::P0fDatabase;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║              打印 p0f 所有数据                                ║");
    println!("╚════════════════════════════════════════════════════════════════╝\n");

    // 尝试从常见位置加载 p0f 数据库
    let p0f_paths = vec![
        "p0f.fp",
        "/etc/p0f/p0f.fp",
        "/usr/share/p0f/p0f.fp",
        "crates/fingerprint-defense/p0f.fp",
        "fingerprint-defense/p0f.fp",
    ];

    let mut db: Option<P0fDatabase> = None;
    let mut loaded_path = String::new();

    for path in &p0f_paths {
        if Path::new(path).exists() {
            println!("📂 找到 p0f 数据库文件: {}", path);
            match P0fDatabase::load_from_file(path) {
                Ok(database) => {
                    db = Some(database);
                    loaded_path = path.to_string();
                    println!("✅ 成功加载 p0f 数据库\n");
                    break;
                }
                Err(e) => {
                    println!("❌ 加载失败: {}\n", e);
                }
            }
        }
    }

    if db.is_none() {
        println!("⚠️  未找到 p0f 数据库文件");
        println!("   请确保 p0f.fp 文件存在于以下位置之一：");
        for path in &p0f_paths {
            println!("     - {}", path);
        }
        return Ok(());
    }

    let db = db.unwrap();

    // 打印统计信息
    let stats = db.stats();
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("【p0f 数据库统计】");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  TCP 请求签名: {} 个", stats.tcp_request_count);
    println!("  TCP 响应签名: {} 个", stats.tcp_response_count);
    println!("  HTTP 请求签名: {} 个", stats.http_request_count);
    println!("  HTTP 响应签名: {} 个", stats.http_response_count);
    println!();

    // 打印所有 TCP 请求签名
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("【TCP 请求签名】");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let tcp_requests = db.get_all_tcp_request();
    println!("总数: {} 个签名\n", tcp_requests.len());

    for (i, sig) in tcp_requests.iter().enumerate() {
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("签名 #{}: {}", i + 1, sig.id);
        println!("  操作系统: {}", sig.os);
        println!("  版本: {}", sig.version);
        println!("  TTL: {:?} (初始: {})", sig.ttl_pattern, sig.initial_ttl);
        println!("  窗口大小: {:?} (模式: {:?})", sig.window_value, sig.window_mode);
        println!("  MSS: {:?}", sig.mss_pattern);
        println!("  TCP 选项顺序: {:?}", sig.options_order);
        println!("  IP 标志: DF={}, ID+={}, ID-={}", sig.ip_flags.df, sig.ip_flags.id_plus, sig.ip_flags.id_minus);
        println!();
    }

    // 打印所有 TCP 响应签名
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("【TCP 响应签名】");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let tcp_responses = db.get_all_tcp_response();
    println!("总数: {} 个签名\n", tcp_responses.len());

    for (i, sig) in tcp_responses.iter().enumerate() {
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("签名 #{}: {}", i + 1, sig.id);
        println!("  操作系统: {}", sig.os);
        println!("  版本: {}", sig.version);
        println!("  TTL: {:?} (初始: {})", sig.ttl_pattern, sig.initial_ttl);
        println!("  窗口大小: {:?} (模式: {:?})", sig.window_value, sig.window_mode);
        println!("  MSS: {:?}", sig.mss_pattern);
        println!("  TCP 选项顺序: {:?}", sig.options_order);
        println!("  IP 标志: DF={}, ID+={}, ID-={}", sig.ip_flags.df, sig.ip_flags.id_plus, sig.ip_flags.id_minus);
        println!();
    }

    // 打印所有 HTTP 请求签名
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("【HTTP 请求签名】");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let http_requests = db.get_all_http_request();
    println!("总数: {} 个签名\n", http_requests.len());

    for (i, sig) in http_requests.iter().enumerate() {
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("签名 #{}: {}", i + 1, sig.id);
        println!("  标签: {}", sig.label);
        println!("  User-Agent 模式: {:?}", sig.user_agent_pattern);
        println!("  Headers: {:?}", sig.headers);
        println!();
    }

    // 打印所有 HTTP 响应签名
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("【HTTP 响应签名】");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let http_responses = db.get_all_http_response();
    println!("总数: {} 个签名\n", http_responses.len());

    for (i, sig) in http_responses.iter().enumerate() {
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("签名 #{}: {}", i + 1, sig.id);
        println!("  标签: {}", sig.label);
        println!("  User-Agent 模式: {:?}", sig.user_agent_pattern);
        println!("  Headers: {:?}", sig.headers);
        println!();
    }

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("✅ 所有 p0f 数据打印完成！");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    Ok(())
}
