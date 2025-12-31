//! p0f 签名数据库解析
//!
//! 解析 p0f.fp 格式的签名数据库文件。

use crate::passive::p0f_parser;
use crate::passive::tcp::TcpSignature;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use thiserror::Error;

/// p0f 签名数据库
pub struct P0fDatabase {
    /// TCP 请求签名
    tcp_request: HashMap<String, TcpSignature>,

    /// TCP 响应签名
    tcp_response: HashMap<String, TcpSignature>,

    /// HTTP 请求签名
    http_request: HashMap<String, P0fHttpSignature>,

    /// HTTP 响应签名
    http_response: HashMap<String, P0fHttpSignature>,
}

/// HTTP 签名（p0f 格式）
#[derive(Debug, Clone)]
pub struct P0fHttpSignature {
    pub id: String,
    pub label: String,
    pub user_agent_pattern: Option<String>,
    pub headers: Vec<String>,
}

impl P0fDatabase {
    /// 从文件加载 p0f 数据库
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, P0fError> {
        let content = fs::read_to_string(path)?;
        Self::parse(&content)
    }

    /// 解析 p0f 数据库内容
    pub fn parse(content: &str) -> Result<Self, P0fError> {
        let mut db = Self {
            tcp_request: HashMap::new(),
            tcp_response: HashMap::new(),
            http_request: HashMap::new(),
            http_response: HashMap::new(),
        };

        let mut current_section: Option<&str> = None;
        let mut current_label: Option<String> = None;

        for line in content.lines() {
            let line = line.trim();

            // 跳过空行和注释
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            // 检查是否是新的部分
            if line.starts_with('[') && line.ends_with(']') {
                current_section = Some(&line[1..line.len() - 1]);
                current_label = None;
                continue;
            }

            // 解析 label
            if let Some(stripped) = line.strip_prefix("label = ") {
                current_label = Some(stripped.trim().to_string());
                continue;
            }

            // 解析 sig
            if let Some(stripped) = line.strip_prefix("sig = ") {
                let sig_value = stripped.trim().to_string();

                // 如果有 label 和 sig，尝试解析
                if let Some(label) = &current_label {
                    if let Some(section) = current_section {
                        match section {
                            "tcp:request" => {
                                if let Ok(tcp_sig) = Self::parse_tcp_signature(label, &sig_value) {
                                    db.tcp_request.insert(tcp_sig.id.clone(), tcp_sig);
                                }
                            }
                            "tcp:response" => {
                                if let Ok(tcp_sig) = Self::parse_tcp_signature(label, &sig_value) {
                                    db.tcp_response.insert(tcp_sig.id.clone(), tcp_sig);
                                }
                            }
                            "http:request" => {
                                if let Ok(http_sig) = Self::parse_http_signature(label, &sig_value)
                                {
                                    db.http_request.insert(http_sig.id.clone(), http_sig);
                                }
                            }
                            "http:response" => {
                                if let Ok(http_sig) = Self::parse_http_signature(label, &sig_value)
                                {
                                    db.http_response.insert(http_sig.id.clone(), http_sig);
                                }
                            }
                            _ => {}
                        }
                    }
                }
                continue;
            }
        }

        Ok(db)
    }

    /// 解析 TCP 签名（使用详细解析器）
    fn parse_tcp_signature(label: &str, sig: &str) -> Result<TcpSignature, P0fError> {
        // 使用详细的解析器
        let p0f_sig = p0f_parser::parse_tcp_signature(label, sig)
            .map_err(|e| P0fError::Parse(e.to_string()))?;

        // 转换为 TcpSignature
        Ok(p0f_sig.into())
    }

    /// 解析 HTTP 签名
    fn parse_http_signature(label: &str, _sig: &str) -> Result<P0fHttpSignature, P0fError> {
        // 简化解析
        Ok(P0fHttpSignature {
            id: format!("http-{}", label.replace(':', "-")),
            label: label.to_string(),
            user_agent_pattern: None,
            headers: Vec::new(),
        })
    }

    /// 获取 TCP 请求签名
    pub fn get_tcp_request(&self, id: &str) -> Option<&TcpSignature> {
        self.tcp_request.get(id)
    }

    /// 获取所有 TCP 请求签名
    pub fn get_all_tcp_request(&self) -> Vec<&TcpSignature> {
        self.tcp_request.values().collect()
    }

    /// 获取所有 TCP 响应签名
    pub fn get_all_tcp_response(&self) -> Vec<&TcpSignature> {
        self.tcp_response.values().collect()
    }

    /// 获取所有 HTTP 请求签名
    pub fn get_all_http_request(&self) -> Vec<&P0fHttpSignature> {
        self.http_request.values().collect()
    }

    /// 获取所有 HTTP 响应签名
    pub fn get_all_http_response(&self) -> Vec<&P0fHttpSignature> {
        self.http_response.values().collect()
    }

    /// 获取统计信息
    pub fn stats(&self) -> P0fStats {
        P0fStats {
            tcp_request_count: self.tcp_request.len(),
            tcp_response_count: self.tcp_response.len(),
            http_request_count: self.http_request.len(),
            http_response_count: self.http_response.len(),
        }
    }
}

/// p0f 数据库统计信息
#[derive(Debug)]
pub struct P0fStats {
    pub tcp_request_count: usize,
    pub tcp_response_count: usize,
    pub http_request_count: usize,
    pub http_response_count: usize,
}

/// p0f 错误
#[derive(Debug, Error)]
pub enum P0fError {
    #[error("IO 错误: {0}")]
    Io(#[from] std::io::Error),

    #[error("无效的格式")]
    InvalidFormat,

    #[error("解析错误: {0}")]
    Parse(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_print_all_p0f_data() {
        println!("\n╔════════════════════════════════════════════════════════════════╗");
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

        for path in &p0f_paths {
            if Path::new(path).exists() {
                println!("📂 找到 p0f 数据库文件: {}", path);
                match P0fDatabase::load_from_file(path) {
                    Ok(database) => {
                        db = Some(database);
                        println!("✅ 成功加载 p0f 数据库 (路径: {})\n", path);
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
            println!("\n   或者创建一个示例数据库进行测试");

            // 创建一个示例数据库用于演示
            println!("\n【创建示例数据库用于演示】\n");
            let example_content = r#"
[tcp:request]
label = s:unix:Linux:3.x
sig = *:64:0:*:mss*20,10:mss,sok,ts,nop,ws:df,id+:0

[tcp:response]
label = s:unix:Linux:3.x
sig = *:64:0:*:mss*20,10:mss,sok,ts,nop,ws:df,id+:0
"#;

            match P0fDatabase::parse(example_content) {
                Ok(database) => {
                    db = Some(database);
                    println!("✅ 使用示例数据库\n");
                }
                Err(e) => {
                    println!("❌ 解析示例数据库失败: {}\n", e);
                    return;
                }
            }
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
            println!("  操作系统: {:?}", sig.os_type);
            println!("  TTL: {}", sig.ttl);
            println!("  窗口大小: {}", sig.window_size);
            println!("  MSS: {:?}", sig.mss);
            println!("  Window Scale: {:?}", sig.window_scale);
            println!("  置信度: {:.2}", sig.confidence);
            println!("  样本数: {}", sig.sample_count);
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
            println!("  操作系统: {:?}", sig.os_type);
            println!("  TTL: {}", sig.ttl);
            println!("  窗口大小: {}", sig.window_size);
            println!("  MSS: {:?}", sig.mss);
            println!("  Window Scale: {:?}", sig.window_scale);
            println!("  置信度: {:.2}", sig.confidence);
            println!("  样本数: {}", sig.sample_count);
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
    }
}
