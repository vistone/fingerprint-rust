//! JA4+ 指纹系列实现
//!
//! 包含 JA4 (TLS), JA4H (HTTP), JA4T (TCP) 等算法的抽象和计算逻辑。
//! 参考自 FoxIO 的 JA4+ 规范。

use serde::{Deserialize, Serialize};

/// JA4 TLS 客户端指纹
/// 格式: t_p_c_e_s_k (例如: t13d1516h2_8daaf6152771_000a)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct JA4 {
    /// 传输协议 (t=tcp, q=quic)
    pub transport: char,
    /// TLS 版本
    pub version: String,
    /// 是否有 SNI (d=domain, i=ip)
    pub destination: char,
    /// 密码套件数量
    pub cipher_count: usize,
    /// 扩展数量
    pub extension_count: usize,
    /// 第一个 ALPN
    pub alpn: String,
    /// 密码套件哈希 (前 12 位)
    pub cipher_hash: String,
    /// 扩展哈希 (前 12 位)
    pub extension_hash: String,
    /// 签名算法哈希 (前 4 位)
    pub signature_hash: String,
}

impl JA4 {
    /// 计算 JA4 指纹
    pub fn generate(
        transport: char,
        version: &str,
        has_sni: bool,
        ciphers: &[u16],
        extensions: &[u16],
        alpn: Option<&str>,
        signature_algorithms: &[u16],
    ) -> Self {
        let v = match version {
            "1.3" => "13",
            "1.2" => "12",
            "1.1" => "11",
            "1.0" => "10",
            _ => "00",
        };
        let d = if has_sni { 'd' } else { 'i' };

        // 过滤并排序密码套件 (GREASE removed)
        let mut filtered_ciphers: Vec<u16> = ciphers
            .iter()
            .filter(|&&c| !crate::grease::is_grease_value(c))
            .cloned()
            .collect();
        filtered_ciphers.sort();

        let c_count = filtered_ciphers.len().min(99);

        // 过滤并排序扩展 (GREASE removed)
        let mut filtered_extensions: Vec<u16> = extensions
            .iter()
            .filter(|&&e| !crate::grease::is_grease_value(e))
            .cloned()
            .collect();
        filtered_extensions.sort();

        let e_count = filtered_extensions.len().min(99);

        let first_alpn = alpn.unwrap_or("00");
        let alpn_id = if first_alpn.len() >= 2 {
            &first_alpn[0..2]
        } else {
            first_alpn
        };

        // 计算哈希
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut c_hasher = DefaultHasher::new();
        for c in &filtered_ciphers {
            c.hash(&mut c_hasher);
        }
        let c_hash_full = format!("{:012x}", c_hasher.finish());

        let mut e_hasher = DefaultHasher::new();
        for e in &filtered_extensions {
            e.hash(&mut e_hasher);
        }
        let e_hash_full = format!("{:012x}", e_hasher.finish());

        let mut s_hasher = DefaultHasher::new();
        for s in signature_algorithms {
            s.hash(&mut s_hasher);
        }
        let s_hash_full = format!("{:04x}", s_hasher.finish());

        Self {
            transport,
            version: v.to_string(),
            destination: d,
            cipher_count: c_count,
            extension_count: e_count,
            alpn: alpn_id.to_string(),
            cipher_hash: c_hash_full[0..12].to_string(),
            extension_hash: e_hash_full[0..12].to_string(),
            signature_hash: s_hash_full[0..4].to_string(),
        }
    }

    /// 转换为标准的 JA4 字符串
    pub fn to_fingerprint_string(&self) -> String {
        format!(
            "{}{}{}{:02}{:02}{}_{}_{}_{}",
            self.transport,
            self.version,
            self.destination,
            self.cipher_count,
            self.extension_count,
            self.alpn,
            self.cipher_hash,
            self.extension_hash,
            self.signature_hash
        )
    }
}

impl std::fmt::Display for JA4 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}{}{:02}{:02}{}_{}_{}_{}",
            self.transport,
            self.version,
            self.destination,
            self.cipher_count,
            self.extension_count,
            self.alpn,
            self.cipher_hash,
            self.extension_hash,
            self.signature_hash
        )
    }
}

/// JA4H HTTP 指纹
/// 格式: [Method][Version][Cookie][Referer][HeaderCount][HeaderOrderHash][HeaderValueHash]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct JA4H {
    pub method: String,
    pub version: String,
    pub has_cookie: bool,
    pub has_referer: bool,
    pub header_count: usize,
    pub header_order_hash: String,
    pub header_value_hash: String,
}

impl JA4H {
    /// 计算 JA4H 指纹
    pub fn generate(
        method: &str,
        version: &str,
        has_cookie: bool,
        has_referer: bool,
        headers: &[(&str, &str)],
    ) -> String {
        let m_raw = method.to_lowercase();
        let m = if m_raw.len() >= 2 {
            &m_raw[0..2]
        } else {
            &m_raw
        };
        let v = if version.contains("1.1") {
            "11"
        } else if version.contains("2") {
            "20"
        } else {
            "10"
        };
        let c = if has_cookie { "c" } else { "n" };
        let r = if has_referer { "r" } else { "n" };
        let count = format!("{:02}", headers.len().min(99));

        // 简化版的 Header Hash
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        for (k, _) in headers {
            k.to_lowercase().hash(&mut hasher);
        }
        let hash = format!("{:012x}", hasher.finish());

        format!("{}{}{}{}{}_{}", m, v, c, r, count, &hash[0..12])
    }
}

/// JA4T TCP 指纹
/// 格式: [WindowSize]_[TCP_Options]_[MSS]_[TTL]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct JA4T {
    pub window_size: u16,
    pub options: String,
    pub mss: u16,
    pub ttl: u8,
}

impl JA4T {
    /// 生成 JA4T 字符串
    pub fn generate(window_size: u16, options: &str, mss: u16, ttl: u8) -> String {
        format!("{}_{}_{}_{}", window_size, options, mss, ttl)
    }
}

impl std::fmt::Display for JA4T {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}_{}_{}_{}",
            self.window_size, self.options, self.mss, self.ttl
        )
    }
}

/// 指纹一致性报告
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ConsistencyReport {
    /// 总体得分 (0-100)
    pub score: u8,
    /// 发现的不一致项
    pub discrepancies: Vec<String>,
    /// 是否疑似机器人
    pub bot_detected: bool,
}

impl Default for ConsistencyReport {
    fn default() -> Self {
        Self {
            score: 100,
            discrepancies: Vec::new(),
            bot_detected: false,
        }
    }
}

impl ConsistencyReport {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_discrepancy(&mut self, msg: String, impact: u8) {
        self.discrepancies.push(msg);
        self.score = self.score.saturating_sub(impact);
        if self.score < 60 {
            self.bot_detected = true;
        }
    }
}
