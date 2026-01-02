//! JA4+ fingerprint系列implement
//!
//! including JA4 (TLS), JA4H (HTTP), JA4T (TCP) 等algorithm的抽象 and Calculate逻辑。
//! reference自 FoxIO  JA4+ 规范。

use serde::{Deserialize, Serialize};

/// JA4 TLS clientfingerprint
/// format: t_p_c_e_s_k (例如: t13d1516h2_8daaf6152771_000a)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct JA4 {
    /// transferprotocol (t=tcp, q=quic)
    pub transport: char,
    /// TLS version
    pub version: String,
    /// whether有 SNI (d=domain, i=ip)
    pub destination: char,
    /// cipher suitecount
    pub cipher_count: usize,
    /// extensioncount
    pub extension_count: usize,
    /// first ALPN
    pub alpn: String,
    /// cipher suitehash (front 12-bit)
    pub cipher_hash: String,
    /// extensionhash (front 12-bit)
    pub extension_hash: String,
    /// signaturealgorithmhash (front 4-bit)
    pub signature_hash: String,
}

impl JA4 {
    /// Calculate JA4 fingerprint
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

        // filter并sortcipher suite (GREASE removed)
        let mut filtered_ciphers: Vec<u16> = ciphers
            .iter()
            .filter(|&&c| !crate::grease::is_grease_value(c))
            .cloned()
            .collect();
        filtered_ciphers.sort();

        let c_count = filtered_ciphers.len().min(99);

        // filter并sortextension (GREASE removed)
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

        // Calculatehash
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

    /// convert tostandard JA4 string
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

/// JA4H HTTP fingerprint
/// format: [Method][Version][Cookie][Referer][HeaderCount][HeaderOrderHash][HeaderValueHash]
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
    /// Calculate JA4H fingerprint
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

        // 简化版 Header Hash
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

/// JA4T TCP fingerprint
/// format: [WindowSize]_[TCP_Options]_[MSS]_[TTL]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct JA4T {
    pub window_size: u16,
    pub options: String,
    pub mss: u16,
    pub ttl: u8,
}

impl JA4T {
    /// Generate JA4T string
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

/// JA4S TLS serverfingerprint（JA4 风格）
/// 
///  and JA3S 类似，butuse SHA256 而非 MD5
/// format: t_v_c_e (例如: t13d_1301_0000)
/// 
/// ## Examples
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
    /// transferprotocol (t=tcp, q=quic)
    pub transport: char,
    /// TLS version
    pub version: String,
    /// select's cipher suitescount（通常为 1）
    pub cipher_count: usize,
    /// extensioncount
    pub extension_count: usize,
    /// first ALPN（ if 有）
    pub alpn: String,
    /// select's cipher suites（十六进制）
    pub cipher: u16,
    /// extensionhash (SHA256 front 12-bit)
    pub extension_hash: String,
}

impl JA4S {
    /// Generate JA4S serverfingerprint
    ///
    /// # Parameters
    /// - `transport`: transferprotocol ('t' for TCP, 'q' for QUIC)
    /// - `version`: TLS version ("1.0", "1.1", "1.2", "1.3")
    /// - `cipher`: serverselect's cipher suites
    /// - `extensions`: serverreturn's extensionslist
    /// - `alpn`: serverselect ALPN（optional）
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

        // filter GREASE value
        let filtered_extensions: Vec<u16> = extensions
            .iter()
            .filter(|&&e| !crate::grease::is_grease_value(e))
            .cloned()
            .collect();

        let e_count = filtered_extensions.len().min(99);

        // ALPN process
        let alpn_id = match alpn {
            Some(a) if a.len() >= 2 => &a[0..2],
            Some(a) => a,
            None => "00",
        };

        // Calculateextensionhash (SHA256)
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
            cipher_count: 1, // server只selectancipher suite
            extension_count: e_count,
            alpn: alpn_id.to_string(),
            cipher,
            extension_hash,
        }
    }

    /// convert tostandard JA4S fingerprintstring
    /// format: t{version}{cipher_count:02}{extension_count:02}{alpn}_{cipher:04x}_{extension_hash}
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

/// fingerprint一致性报告
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ConsistencyReport {
    /// 总体score (0-100)
    pub score: u8,
    /// 发现的不一致项
    pub discrepancies: Vec<String>,
    /// whether疑似机器人
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
        // testincluding GREASE value的situation
        let ja4s = JA4S::generate('t', "1.3", 0x1302, &[0x0a0a, 0, 10], Some("http/1.1"));
        
        // GREASE value 0x0a0a should被filter
        assert!(ja4s.extension_count < 3);
    }

    #[test]
    fn test_ja4s_fingerprint_string() {
        let ja4s = JA4S::generate('t', "1.3", 0x1301, &[0, 10, 11], Some("h2"));
        let fingerprint = ja4s.fingerprint_string();
        
        // Validateformat
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
        // testextensionsort（pairhash的影响）
        let ja4s1 = JA4S::generate('t', "1.3", 0x1301, &[0, 10, 11], None);
        let ja4s2 = JA4S::generate('t', "1.3", 0x1301, &[11, 10, 0], None);
        
        // sortbackshouldproducesame的hash
        assert_eq!(ja4s1.extension_hash, ja4s2.extension_hash);
    }
}

/// JA4L - 轻量levelfingerprint（Light Version）
///
/// 简化版 JA4，适 for 资source受限environment
/// - use更快的hashalgorithm
/// - decreaseCalculate复杂度
/// - 更小的inside存占用
///
/// format: t{version}{cipher_count:02}{extension_count:02}_{cipher_sample}_{ext_sample}
///
/// ## Examples
/// ```
/// use fingerprint_core::ja4::JA4L;
///
/// let ja4l = JA4L::generate(
///     't',
///     "1.3",
///     true,
///     &[0x1301, 0x1302, 0x1303],
///     &[0, 10, 11, 13],
/// );
/// assert!(!ja4l.fingerprint_string().is_empty());
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct JA4L {
    /// transferprotocol (t=tcp, q=quic)
    pub transport: char,
    
    /// TLS version
    pub version: String,
    
    /// whether有 SNI (d=domain, i=ip)
    pub destination: char,
    
    /// cipher suitecount
    pub cipher_count: usize,
    
    /// extensioncount
    pub extension_count: usize,
    
    /// cipher suite采样（front3个，十六进制）
    pub cipher_sample: String,
    
    /// extension采样（front3个，十六进制）
    pub extension_sample: String,
}

impl JA4L {
    /// Generate JA4L 轻量levelfingerprint
    ///
    /// # Parameters
    /// - `transport`: transferprotocol ('t' for TCP, 'q' for QUIC)
    /// - `version`: TLS version ("1.0", "1.1", "1.2", "1.3")
    /// - `has_sni`: whetherincluding SNI extension
    /// - `ciphers`: cipher suitelist
    /// - `extensions`: extensionlist
    pub fn generate(
        transport: char,
        version: &str,
        has_sni: bool,
        ciphers: &[u16],
        extensions: &[u16],
    ) -> Self {
        let v = match version {
            "1.3" => "13",
            "1.2" => "12",
            "1.1" => "11",
            "1.0" => "10",
            _ => "00",
        };
        
        let d = if has_sni { 'd' } else { 'i' };

        // filter GREASE value
        let filtered_ciphers: Vec<u16> = ciphers
            .iter()
            .filter(|&&c| !crate::grease::is_grease_value(c))
            .cloned()
            .collect();

        let filtered_extensions: Vec<u16> = extensions
            .iter()
            .filter(|&&e| !crate::grease::is_grease_value(e))
            .cloned()
            .collect();

        let cipher_count = filtered_ciphers.len().min(99);
        let extension_count = filtered_extensions.len().min(99);

        // 采样front3个cipher suite（轻量levelmethod）
        let cipher_sample = filtered_ciphers
            .iter()
            .take(3)
            .map(|c| format!("{:04x}", c))
            .collect::<Vec<_>>()
            .join("");

        // 采样front3个extension（轻量levelmethod）
        let extension_sample = filtered_extensions
            .iter()
            .take(3)
            .map(|e| format!("{:04x}", e))
            .collect::<Vec<_>>()
            .join("");

        Self {
            transport,
            version: v.to_string(),
            destination: d,
            cipher_count,
            extension_count,
            cipher_sample: if cipher_sample.is_empty() {
                "000000000000".to_string()
            } else {
                cipher_sample
            },
            extension_sample: if extension_sample.is_empty() {
                "000000000000".to_string()
            } else {
                extension_sample
            },
        }
    }

    /// convert tostandard JA4L fingerprintstring
    /// format: t{version}{destination}{cipher_count:02}{extension_count:02}_{cipher_sample}_{extension_sample}
    pub fn fingerprint_string(&self) -> String {
        format!(
            "{}{}{}{:02}{:02}_{}_{}",
            self.transport,
            self.version,
            self.destination,
            self.cipher_count,
            self.extension_count,
            self.cipher_sample,
            self.extension_sample
        )
    }

    ///  from complete JA4 fingerprintGenerate轻量levelversion
    pub fn from_ja4(ja4: &JA4) -> Self {
        Self {
            transport: ja4.transport,
            version: ja4.version.clone(),
            destination: ja4.destination,
            cipher_count: ja4.cipher_count,
            extension_count: ja4.extension_count,
            //  from hash中Extract采样（简化）
            cipher_sample: ja4.cipher_hash[0..12.min(ja4.cipher_hash.len())].to_string(),
            extension_sample: ja4.extension_hash[0..12.min(ja4.extension_hash.len())].to_string(),
        }
    }

    /// 估算fingerprint的Calculate成本（相pairvalue）
    /// returnvalue：1-10，1 为最轻量，10 为最重
    pub fn computational_cost() -> u8 {
        2 // JA4L 是轻量level的，成本评分为 2/10
    }

    /// 估算inside存占用（bytes）
    pub fn memory_footprint(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.version.capacity()
            + self.cipher_sample.capacity()
            + self.extension_sample.capacity()
    }
}

impl std::fmt::Display for JA4L {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.fingerprint_string())
    }
}

#[cfg(test)]
mod ja4l_tests {
    use super::*;

    #[test]
    fn test_ja4l_generation() {
        let ja4l = JA4L::generate(
            't',
            "1.3",
            true,
            &[0x1301, 0x1302, 0x1303, 0x1304],
            &[0, 10, 11, 13, 16],
        );

        assert_eq!(ja4l.transport, 't');
        assert_eq!(ja4l.version, "13");
        assert_eq!(ja4l.destination, 'd');
        assert_eq!(ja4l.cipher_count, 4);
        assert_eq!(ja4l.extension_count, 5);
        assert!(!ja4l.cipher_sample.is_empty());
        assert!(!ja4l.extension_sample.is_empty());
    }

    #[test]
    fn test_ja4l_fingerprint_string() {
        let ja4l = JA4L::generate('t', "1.2", true, &[0xc02f, 0xc030], &[0, 10]);

        let fp = ja4l.fingerprint_string();
        assert!(fp.starts_with("t12d"));
        assert!(fp.contains('_'));
    }

    #[test]
    fn test_ja4l_empty_ciphers() {
        let ja4l = JA4L::generate('t', "1.3", false, &[], &[0, 10]);

        assert_eq!(ja4l.cipher_count, 0);
        assert_eq!(ja4l.cipher_sample, "000000000000");
    }

    #[test]
    fn test_ja4l_display() {
        let ja4l = JA4L::generate('t', "1.3", true, &[0x1301], &[0]);

        let displayed = format!("{}", ja4l);
        assert_eq!(displayed, ja4l.fingerprint_string());
    }

    #[test]
    fn test_ja4l_computational_cost() {
        let cost = JA4L::computational_cost();
        assert!(cost <= 3); // should是轻量level的
    }

    #[test]
    fn test_ja4l_memory_footprint() {
        let ja4l = JA4L::generate('t', "1.3", true, &[0x1301], &[0]);
        let footprint = ja4l.memory_footprint();
        
        // inside存占用should很小
        assert!(footprint < 200); // 少于 200 bytes
    }
}
