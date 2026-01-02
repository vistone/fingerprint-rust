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

/// JA4S TLS 服务器指纹（JA4 风格）
/// 
/// 与 JA3S 类似，但使用 SHA256 而非 MD5
/// 格式: t_v_c_e (例如: t13d_1301_0000)
/// 
/// ## 示例
/// ```
/// use fingerprint_core::ja4::JA4S;
///
/// let ja4s = JA4S::generate(
///     't',           // transport (TCP)
///     "1.3",         // TLS version
///     0x1301,        // selected cipher
///     &[0, 10, 11],  // extensions
///     None,          // ALPN
/// );
/// assert!(!ja4s.fingerprint_string().is_empty());
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct JA4S {
    /// 传输协议 (t=tcp, q=quic)
    pub transport: char,
    /// TLS 版本
    pub version: String,
    /// 选择的密码套件数量（通常为 1）
    pub cipher_count: usize,
    /// 扩展数量
    pub extension_count: usize,
    /// 第一个 ALPN（如果有）
    pub alpn: String,
    /// 选择的密码套件（十六进制）
    pub cipher: u16,
    /// 扩展哈希 (SHA256 前 12 位)
    pub extension_hash: String,
}

impl JA4S {
    /// 生成 JA4S 服务器指纹
    ///
    /// # 参数
    /// - `transport`: 传输协议 ('t' for TCP, 'q' for QUIC)
    /// - `version`: TLS 版本 ("1.0", "1.1", "1.2", "1.3")
    /// - `cipher`: 服务器选择的密码套件
    /// - `extensions`: 服务器返回的扩展列表
    /// - `alpn`: 服务器选择的 ALPN（可选）
    pub fn generate(
        transport: char,
        version: &str,
        cipher: u16,
        extensions: &[u16],
        alpn: Option<&str>,
    ) -> Self {
        let v = match version {
            "1.3" => "13",
            "1.2" => "12",
            "1.1" => "11",
            "1.0" => "10",
            _ => "00",
        };

        // 过滤 GREASE 值
        let filtered_extensions: Vec<u16> = extensions
            .iter()
            .filter(|&&e| !crate::grease::is_grease_value(e))
            .cloned()
            .collect();

        let e_count = filtered_extensions.len().min(99);

        // ALPN 处理
        let alpn_id = match alpn {
            Some(a) if a.len() >= 2 => &a[0..2],
            Some(a) => a,
            None => "00",
        };

        // 计算扩展哈希 (SHA256)
        use sha2::{Digest, Sha256};
        let mut sorted_extensions = filtered_extensions.clone();
        sorted_extensions.sort_unstable();

        let ext_string = sorted_extensions
            .iter()
            .map(|e| format!("{:04x}", e))
            .collect::<Vec<String>>()
            .join(",");

        let mut hasher = Sha256::new();
        hasher.update(ext_string.as_bytes());
        let hash_result = hasher.finalize();
        let hash_hex = format!("{:x}", hash_result);
        let extension_hash = hash_hex[0..12].to_string();

        Self {
            transport,
            version: v.to_string(),
            cipher_count: 1, // 服务器只选择一个密码套件
            extension_count: e_count,
            alpn: alpn_id.to_string(),
            cipher,
            extension_hash,
        }
    }

    /// 转换为标准的 JA4S 指纹字符串
    /// 格式: t{version}{cipher_count:02}{extension_count:02}{alpn}_{cipher:04x}_{extension_hash}
    pub fn fingerprint_string(&self) -> String {
        format!(
            "{}{}{:02}{:02}{}_{:04x}_{}",
            self.transport,
            self.version,
            self.cipher_count,
            self.extension_count,
            self.alpn,
            self.cipher,
            self.extension_hash
        )
    }
}

impl std::fmt::Display for JA4S {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.fingerprint_string())
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

#[cfg(test)]
mod ja4s_tests {
    use super::*;

    #[test]
    fn test_ja4s_generation() {
        let ja4s = JA4S::generate('t', "1.3", 0x1301, &[0, 10, 11, 13], Some("h2"));
        
        assert_eq!(ja4s.transport, 't');
        assert_eq!(ja4s.version, "13");
        assert_eq!(ja4s.cipher, 0x1301);
        assert_eq!(ja4s.cipher_count, 1);
        assert_eq!(ja4s.alpn, "h2");
        assert!(!ja4s.extension_hash.is_empty());
        assert_eq!(ja4s.extension_hash.len(), 12);
    }

    #[test]
    fn test_ja4s_tls12() {
        let ja4s = JA4S::generate('t', "1.2", 0xc02f, &[0, 10], None);
        
        assert_eq!(ja4s.version, "12");
        assert_eq!(ja4s.cipher, 0xc02f);
        assert_eq!(ja4s.alpn, "00");
    }

    #[test]
    fn test_ja4s_with_grease() {
        // 测试包含 GREASE 值的情况
        let ja4s = JA4S::generate('t', "1.3", 0x1302, &[0x0a0a, 0, 10], Some("http/1.1"));
        
        // GREASE 值 0x0a0a 应该被过滤
        assert!(ja4s.extension_count < 3);
    }

    #[test]
    fn test_ja4s_fingerprint_string() {
        let ja4s = JA4S::generate('t', "1.3", 0x1301, &[0, 10, 11], Some("h2"));
        let fingerprint = ja4s.fingerprint_string();
        
        // 验证格式
        assert!(fingerprint.starts_with("t13"));
        assert!(fingerprint.contains("h2"));
        assert!(fingerprint.contains("1301"));
    }

    #[test]
    fn test_ja4s_display() {
        let ja4s = JA4S::generate('t', "1.2", 0xc030, &[0], Some("h2"));
        let displayed = format!("{}", ja4s);
        
        assert!(!displayed.is_empty());
        assert_eq!(displayed, ja4s.fingerprint_string());
    }

    #[test]
    fn test_ja4s_quic_transport() {
        let ja4s = JA4S::generate('q', "1.3", 0x1301, &[0, 10], Some("h3"));
        
        assert_eq!(ja4s.transport, 'q');
        assert_eq!(ja4s.alpn, "h3");
    }

    #[test]
    fn test_ja4s_extension_sorting() {
        // 测试扩展排序（对哈希的影响）
        let ja4s1 = JA4S::generate('t', "1.3", 0x1301, &[0, 10, 11], None);
        let ja4s2 = JA4S::generate('t', "1.3", 0x1301, &[11, 10, 0], None);
        
        // 排序后应该产生相同的哈希
        assert_eq!(ja4s1.extension_hash, ja4s2.extension_hash);
    }
}
