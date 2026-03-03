//! JA4+ fingerprint series implementation
//!
//! Including JA4 (TLS), JA4H (HTTP), JA4T (TCP), JA4TS (TCP Server) etc.
//! Algorithm abstractions and calculation logic.
//! Reference: FoxIO JA4+ specification.

use serde::{Deserialize, Serialize};

/// Safe string slicing function that prevents panics
/// Returns slice[start..min(end, slice.len())] or entire slice if start is out of bounds
fn safe_slice(s: &str, start: usize, end: usize) -> &str {
    let len = s.len();
    if start >= len {
        return s;
    }
    &s[start..end.min(len)]
}

/// JA4 TLS clientfingerprint
/// format: t_p_c_e_s_k (for example: t13d1516h2_8daaf6152771_000a)
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
            cipher_hash: safe_slice(&c_hash_full, 0, 12).to_string(),
            extension_hash: safe_slice(&e_hash_full, 0, 12).to_string(),
            signature_hash: safe_slice(&s_hash_full, 0, 4).to_string(),
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
/// format: \[Method\]\[Version\]\[Cookie\]\[Referer\]\[HeaderCount\]\[HeaderOrderHash\]\[HeaderValueHash\]
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

        // simplify版 Header Hash
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
/// format: \[WindowSize\]_\[TCP_Options\]_\[MSS\]_\[TTL\]
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

/// JA4S TLS serverfingerprint (JA4 style)
///
/// and JA3S similar, butuse SHA256 rather than MD5
/// format: t_v_c_e (for example: t13d_1301_0000)
///
/// ## Examples
/// ```
/// use fingerprint_core::ja4::JA4S;
///
/// let ja4s = JA4S::generate(
/// 't', // transport (TCP)
/// "1.3", // TLS version
/// 0x1301, // selected cipher
/// &[0, 10, 11], // extensions
/// None, // ALPN
/// );
/// assert!(!ja4s.fingerprint_string().is_empty());
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct JA4S {
    /// transferprotocol (t=tcp, q=quic)
    pub transport: char,
    /// TLS version
    pub version: String,
    /// select's cipher suitescount (usually as 1)
    pub cipher_count: usize,
    /// extensioncount
    pub extension_count: usize,
    /// first ALPN ( if 有)
    pub alpn: String,
    /// select's cipher suites (hexadecimal)
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
    /// - `alpn`: serverselect ALPN (optional)
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
        let mut filtered_extensions: Vec<u16> = extensions
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

        // Calculate extension hash (SHA256)
        use sha2::{Digest, Sha256};
        filtered_extensions.sort_unstable();

        let ext_string = filtered_extensions
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
            cipher_count: 1, // server只select ancipher suite
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

/// fingerprintconsistencyreport
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ConsistencyReport {
    /// overallscore (0-100)
    pub score: u8,
    /// discover不consistentitem
    pub discrepancies: Vec<String>,
    /// whethersuspected machineer人
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

/// TLS Extension Order Fingerprint
///
/// The ordering of extensions in TLS ClientHello is a strong fingerprinting signal.
/// Different browsers and TLS libraries have distinct extension orderings that
/// remain consistent across versions. This is particularly useful for:
/// - Distinguishing browsers even when cipher suites/curves are identical
/// - Detecting TLS client impersonation (e.g., bot pretending to be Chrome)
/// - Complementing JA3/JA4 fingerprints with ordering information
///
/// ## Known Browser Extension Patterns
/// - Chrome: SNI(0), extended_master_secret(23), renegotiation_info(65281), supported_groups(10)...
/// - Firefox: SNI(0), extended_master_secret(23), renegotiation_info(65281), supported_groups(10)...
///   (similar start but diverges after ~5 extensions)
/// - Safari: SNI(0), supported_groups(10), ec_point_formats(11)...
///
/// ## Examples
/// ```
/// use fingerprint_core::ja4::TlsExtensionOrderFingerprint;
///
/// let extensions = vec![0, 23, 65281, 10, 11, 35, 16, 5, 13, 18, 51, 45, 43, 27, 21];
/// let fp = TlsExtensionOrderFingerprint::generate(&extensions);
/// assert!(!fp.fingerprint_string().is_empty());
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TlsExtensionOrderFingerprint {
    /// Original extension order (GREASE filtered)
    pub extension_order: Vec<u16>,
    /// SHA256 hash of the extension order (first 16 hex chars)
    pub order_hash: String,
    /// Number of extensions
    pub count: usize,
}

impl TlsExtensionOrderFingerprint {
    /// Generate extension order fingerprint from a list of TLS extension IDs
    /// in the order they appear in the ClientHello
    pub fn generate(extensions: &[u16]) -> Self {
        use sha2::{Digest, Sha256};

        // Filter GREASE values but preserve ordering
        let filtered: Vec<u16> = extensions
            .iter()
            .filter(|&&e| !crate::grease::is_grease_value(e))
            .cloned()
            .collect();

        let count = filtered.len();

        // Create ordered string representation
        let order_string = filtered
            .iter()
            .map(|e| format!("{:04x}", e))
            .collect::<Vec<_>>()
            .join("-");

        let mut hasher = Sha256::new();
        hasher.update(order_string.as_bytes());
        let hash_result = hasher.finalize();
        let hash_hex = format!("{:x}", hash_result);
        let order_hash = hash_hex[..16.min(hash_hex.len())].to_string();

        Self {
            extension_order: filtered,
            order_hash,
            count,
        }
    }

    /// Convert to fingerprint string
    /// format: {count:02}_{order_hash}
    pub fn fingerprint_string(&self) -> String {
        format!("{:02}_{}", self.count, self.order_hash)
    }

    /// Calculate similarity with another extension order (0.0 - 1.0)
    /// Uses longest common subsequence ratio for ordering comparison
    pub fn similarity(&self, other: &TlsExtensionOrderFingerprint) -> f64 {
        if self.extension_order.is_empty() || other.extension_order.is_empty() {
            return 0.0;
        }

        // LCS-based ordering similarity
        let lcs_len = longest_common_subsequence_len(&self.extension_order, &other.extension_order);
        let max_len = self.extension_order.len().max(other.extension_order.len());

        lcs_len as f64 / max_len as f64
    }
}

/// Calculate length of longest common subsequence
fn longest_common_subsequence_len(a: &[u16], b: &[u16]) -> usize {
    let m = a.len();
    let n = b.len();
    let mut dp = vec![vec![0usize; n + 1]; m + 1];

    for i in 1..=m {
        for j in 1..=n {
            if a[i - 1] == b[j - 1] {
                dp[i][j] = dp[i - 1][j - 1] + 1;
            } else {
                dp[i][j] = dp[i - 1][j].max(dp[i][j - 1]);
            }
        }
    }

    dp[m][n]
}

impl std::fmt::Display for TlsExtensionOrderFingerprint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.fingerprint_string())
    }
}

#[cfg(test)]
mod extension_order_tests {
    use super::*;

    #[test]
    fn test_extension_order_generation() {
        let extensions = vec![0, 23, 65281, 10, 11, 35, 16, 5, 13, 18, 51, 45, 43, 27, 21];
        let fp = TlsExtensionOrderFingerprint::generate(&extensions);
        assert_eq!(fp.count, 15);
        assert_eq!(fp.order_hash.len(), 16);
        assert_eq!(fp.extension_order, extensions);
    }

    #[test]
    fn test_extension_order_grease_filtered() {
        let extensions = vec![0x0a0a, 0, 23, 0x1a1a, 65281, 10];
        let fp = TlsExtensionOrderFingerprint::generate(&extensions);
        assert_eq!(fp.count, 4);
        assert_eq!(fp.extension_order, vec![0, 23, 65281, 10]);
    }

    #[test]
    fn test_extension_order_preserves_ordering() {
        let ext1 = vec![0, 23, 10, 11];
        let ext2 = vec![0, 10, 23, 11]; // Different order

        let fp1 = TlsExtensionOrderFingerprint::generate(&ext1);
        let fp2 = TlsExtensionOrderFingerprint::generate(&ext2);

        // Different orderings should produce different hashes
        assert_ne!(fp1.order_hash, fp2.order_hash);
    }

    #[test]
    fn test_extension_order_similarity_identical() {
        let extensions = vec![0, 23, 65281, 10, 11];
        let fp1 = TlsExtensionOrderFingerprint::generate(&extensions);
        let fp2 = TlsExtensionOrderFingerprint::generate(&extensions);
        assert!((fp1.similarity(&fp2) - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_extension_order_similarity_different() {
        let ext1 = vec![0, 23, 65281, 10, 11, 35, 16];
        let ext2 = vec![0, 10, 11, 16, 35, 23, 65281];

        let fp1 = TlsExtensionOrderFingerprint::generate(&ext1);
        let fp2 = TlsExtensionOrderFingerprint::generate(&ext2);

        let sim = fp1.similarity(&fp2);
        assert!(sim > 0.0 && sim < 1.0);
    }

    #[test]
    fn test_extension_order_display() {
        let extensions = vec![0, 23, 10];
        let fp = TlsExtensionOrderFingerprint::generate(&extensions);
        let displayed = format!("{}", fp);
        assert_eq!(displayed, fp.fingerprint_string());
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
        // testincluding GREASE valuesituation
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
        // testextensionsort (pairhashimpact)
        let ja4s1 = JA4S::generate('t', "1.3", 0x1301, &[0, 10, 11], None);
        let ja4s2 = JA4S::generate('t', "1.3", 0x1301, &[11, 10, 0], None);

        // sortbackshouldproducesamehash
        assert_eq!(ja4s1.extension_hash, ja4s2.extension_hash);
    }
}

/// JA4L - lightweightlevelfingerprint (Light Version)
///
/// simplify版 JA4, suitable for 资source受limitenvironment
/// - usemorefasthashalgorithm
/// - decreaseCalculatecomplexdegree
/// - moresmallinsidesaveusage
///
/// format: t{version}{cipher_count:02}{extension_count:02}_{cipher_sample}_{ext_sample}
///
/// ## Examples
/// ```
/// use fingerprint_core::ja4::JA4L;
///
/// let ja4l = JA4L::generate(
/// 't',
/// "1.3",
/// true,
/// &[0x1301, 0x1302, 0x1303],
/// &[0, 10, 11, 13],
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

    /// cipher suitesampling (front3, hexadecimal)
    pub cipher_sample: String,

    /// extensionsampling (front3, hexadecimal)
    pub extension_sample: String,
}

impl JA4L {
    /// Generate JA4L lightweightlevelfingerprint
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

        // samplingfront3cipher suite (lightweightlevelmethod)
        let cipher_sample = filtered_ciphers
            .iter()
            .take(3)
            .map(|c| format!("{:04x}", c))
            .collect::<Vec<_>>()
            .join("");

        // samplingfront3extension (lightweightlevelmethod)
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

    /// from complete JA4 fingerprintGeneratelightweightlevelversion
    pub fn from_ja4(ja4: &JA4) -> Self {
        Self {
            transport: ja4.transport,
            version: ja4.version.clone(),
            destination: ja4.destination,
            cipher_count: ja4.cipher_count,
            extension_count: ja4.extension_count,
            // from hash in Extractsampling (simplify)
            cipher_sample: ja4.cipher_hash[0..12.min(ja4.cipher_hash.len())].to_string(),
            extension_sample: ja4.extension_hash[0..12.min(ja4.extension_hash.len())].to_string(),
        }
    }

    /// estimatefingerprintCalculatebecomethis (mutualpairvalue)
    /// returnvalue：1-10, 1 as mostlightweight, 10 as most重
    pub fn computational_cost() -> u8 {
        2 // JA4L is lightweightlevelof，成本score as 2/10
    }

    /// estimateinsidesaveusage (bytes)
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

/// JA4X - X.509 Certificate Fingerprinting
///
/// Provides fingerprinting for X.509 certificates based on certificate attributes
/// format: {sig_algo}_{key_algo}_{key_size}_{ext_count}_{ext_hash}
///
/// ## Examples
/// ```
/// use fingerprint_core::ja4::JA4X;
///
/// let ja4x = JA4X::generate(
///     "sha256_rsa",
///     "rsa",
///     2048,
///     &["subjectAltName", "keyUsage", "basicConstraints"],
/// );
/// assert!(!ja4x.fingerprint_string().is_empty());
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct JA4X {
    /// Signature algorithm (e.g., "sha256_rsa", "sha256_ecdsa")
    pub signature_algorithm: String,

    /// Public key algorithm (e.g., "rsa", "ecdsa", "ed25519")
    pub key_algorithm: String,

    /// Key size in bits
    pub key_size: u16,

    /// Number of X.509 extensions
    pub extension_count: usize,

    /// Hash of sorted extension OIDs
    pub extension_hash: String,
}

impl JA4X {
    /// Generate JA4X certificate fingerprint
    ///
    /// # Parameters
    /// - `signature_algorithm`: Certificate signature algorithm
    /// - `key_algorithm`: Public key algorithm
    /// - `key_size`: Public key size in bits
    /// - `extensions`: List of X.509 extension names/OIDs
    pub fn generate(
        signature_algorithm: &str,
        key_algorithm: &str,
        key_size: u16,
        extensions: &[&str],
    ) -> Self {
        use sha2::{Digest, Sha256};

        let ext_count = extensions.len().min(99);

        // Sort and hash extensions
        let mut sorted_ext: Vec<&str> = extensions.to_vec();
        sorted_ext.sort_unstable();

        let ext_string = sorted_ext.join(",");
        let mut hasher = Sha256::new();
        hasher.update(ext_string.as_bytes());
        let hash_result = hasher.finalize();
        let hash_hex = format!("{:x}", hash_result);
        let extension_hash = hash_hex[0..12].to_string();

        Self {
            signature_algorithm: signature_algorithm.to_string(),
            key_algorithm: key_algorithm.to_string(),
            key_size,
            extension_count: ext_count,
            extension_hash,
        }
    }

    /// Convert to standard JA4X fingerprint string
    /// format: {sig_algo}_{key_algo}_{key_size}_{ext_count:02}_{ext_hash}
    pub fn fingerprint_string(&self) -> String {
        format!(
            "{}_{}_{}_{:02}_{}",
            self.signature_algorithm,
            self.key_algorithm,
            self.key_size,
            self.extension_count,
            self.extension_hash
        )
    }
}

impl std::fmt::Display for JA4X {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.fingerprint_string())
    }
}

/// JA4SSH - SSH Protocol Fingerprinting
///
/// Fingerprints SSH client/server based on protocol specifics
/// format: {version}_{kex_count:02}_{cipher_count:02}_{mac_count:02}_{comp_count:02}_{kex_hash}_{cipher_hash}
///
/// ## Examples
/// ```
/// use fingerprint_core::ja4::JA4SSH;
///
/// let ja4ssh = JA4SSH::generate(
///     "2.0",
///     &["diffie-hellman-group14-sha256", "ecdh-sha2-nistp256"],
///     &["aes128-ctr", "aes256-ctr"],
///     &["hmac-sha2-256", "hmac-sha2-512"],
///     &["none", "zlib"],
/// );
/// assert!(!ja4ssh.fingerprint_string().is_empty());
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct JA4SSH {
    /// SSH protocol version
    pub version: String,

    /// Number of key exchange algorithms
    pub kex_count: usize,

    /// Number of encryption algorithms
    pub cipher_count: usize,

    /// Number of MAC algorithms
    pub mac_count: usize,

    /// Number of compression algorithms
    pub compression_count: usize,

    /// Hash of key exchange algorithms
    pub kex_hash: String,

    /// Hash of cipher algorithms
    pub cipher_hash: String,

    /// Hash of MAC algorithms
    pub mac_hash: String,
}

impl JA4SSH {
    /// Generate JA4SSH fingerprint
    ///
    /// # Parameters
    /// - `version`: SSH protocol version (e.g., "2.0")
    /// - `kex_algorithms`: Key exchange algorithms
    /// - `ciphers`: Encryption algorithms
    /// - `macs`: MAC algorithms
    /// - `compressions`: Compression algorithms
    pub fn generate(
        version: &str,
        kex_algorithms: &[&str],
        ciphers: &[&str],
        macs: &[&str],
        compressions: &[&str],
    ) -> Self {
        use sha2::{Digest, Sha256};

        // Calculate hash for each algorithm list
        let hash_list = |items: &[&str]| -> String {
            let sorted_items = items.join(",");
            let mut hasher = Sha256::new();
            hasher.update(sorted_items.as_bytes());
            let hash_result = hasher.finalize();
            let hash_hex = format!("{:x}", hash_result);
            hash_hex[0..8].to_string()
        };

        Self {
            version: version.to_string(),
            kex_count: kex_algorithms.len().min(99),
            cipher_count: ciphers.len().min(99),
            mac_count: macs.len().min(99),
            compression_count: compressions.len().min(99),
            kex_hash: hash_list(kex_algorithms),
            cipher_hash: hash_list(ciphers),
            mac_hash: hash_list(macs),
        }
    }

    /// Convert to standard JA4SSH fingerprint string
    /// format: {version}_{kex_count:02}_{cipher_count:02}_{mac_count:02}_{comp_count:02}_{kex_hash}_{cipher_hash}_{mac_hash}
    pub fn fingerprint_string(&self) -> String {
        format!(
            "{}_{:02}_{:02}_{:02}_{:02}_{}_{}_{}",
            self.version,
            self.kex_count,
            self.cipher_count,
            self.mac_count,
            self.compression_count,
            self.kex_hash,
            self.cipher_hash,
            self.mac_hash
        )
    }
}

impl std::fmt::Display for JA4SSH {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.fingerprint_string())
    }
}

#[cfg(test)]
mod ja4x_tests {
    use super::*;

    #[test]
    fn test_ja4x_generation() {
        let ja4x = JA4X::generate(
            "sha256_rsa",
            "rsa",
            2048,
            &["subjectAltName", "keyUsage", "basicConstraints"],
        );

        assert_eq!(ja4x.signature_algorithm, "sha256_rsa");
        assert_eq!(ja4x.key_algorithm, "rsa");
        assert_eq!(ja4x.key_size, 2048);
        assert_eq!(ja4x.extension_count, 3);
        assert_eq!(ja4x.extension_hash.len(), 12);
    }

    #[test]
    fn test_ja4x_fingerprint_string() {
        let ja4x = JA4X::generate("sha256_ecdsa", "ecdsa", 256, &["keyUsage"]);

        let fp = ja4x.fingerprint_string();
        assert!(fp.contains("sha256_ecdsa"));
        assert!(fp.contains("ecdsa"));
        assert!(fp.contains("256"));
    }

    #[test]
    fn test_ja4x_extension_sorting() {
        // Test that extension order doesn't affect hash
        let ja4x1 = JA4X::generate("sha256_rsa", "rsa", 2048, &["a", "b", "c"]);
        let ja4x2 = JA4X::generate("sha256_rsa", "rsa", 2048, &["c", "b", "a"]);

        assert_eq!(ja4x1.extension_hash, ja4x2.extension_hash);
    }

    #[test]
    fn test_ja4x_display() {
        let ja4x = JA4X::generate("sha256_rsa", "rsa", 4096, &["keyUsage"]);

        let displayed = format!("{}", ja4x);
        assert_eq!(displayed, ja4x.fingerprint_string());
    }
}

#[cfg(test)]
mod ja4ssh_tests {
    use super::*;

    #[test]
    fn test_ja4ssh_generation() {
        let ja4ssh = JA4SSH::generate(
            "2.0",
            &["diffie-hellman-group14-sha256", "ecdh-sha2-nistp256"],
            &["aes128-ctr", "aes256-ctr"],
            &["hmac-sha2-256", "hmac-sha2-512"],
            &["none"],
        );

        assert_eq!(ja4ssh.version, "2.0");
        assert_eq!(ja4ssh.kex_count, 2);
        assert_eq!(ja4ssh.cipher_count, 2);
        assert_eq!(ja4ssh.mac_count, 2);
        assert_eq!(ja4ssh.compression_count, 1);
        assert_eq!(ja4ssh.kex_hash.len(), 8);
        assert_eq!(ja4ssh.cipher_hash.len(), 8);
        assert_eq!(ja4ssh.mac_hash.len(), 8);
    }

    #[test]
    fn test_ja4ssh_fingerprint_string() {
        let ja4ssh = JA4SSH::generate(
            "2.0",
            &["diffie-hellman-group14-sha256"],
            &["aes128-ctr"],
            &["hmac-sha2-256"],
            &["none"],
        );

        let fp = ja4ssh.fingerprint_string();
        assert!(fp.starts_with("2.0_"));
        assert!(fp.contains('_'));
    }

    #[test]
    fn test_ja4ssh_display() {
        let ja4ssh = JA4SSH::generate(
            "2.0",
            &["ecdh-sha2-nistp256"],
            &["aes256-ctr"],
            &["hmac-sha2-512"],
            &["none"],
        );

        let displayed = format!("{}", ja4ssh);
        assert_eq!(displayed, ja4ssh.fingerprint_string());
    }

    #[test]
    fn test_ja4ssh_empty_algorithms() {
        let ja4ssh = JA4SSH::generate("2.0", &[], &[], &[], &[]);

        assert_eq!(ja4ssh.kex_count, 0);
        assert_eq!(ja4ssh.cipher_count, 0);
        assert_eq!(ja4ssh.mac_count, 0);
        assert_eq!(ja4ssh.compression_count, 0);
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
        assert!(cost <= 3); // should is lightweightlevelof
    }

    #[test]
    fn test_ja4l_memory_footprint() {
        let ja4l = JA4L::generate('t', "1.3", true, &[0x1301], &[0]);
        let footprint = ja4l.memory_footprint();

        // insidesaveusageshouldverysmall
        assert!(footprint < 200); // 少于 200 bytes
    }
}

/// JA4TS - TCP Server Response Fingerprint
///
/// Captures the server's SYN-ACK TCP characteristics for server-side fingerprinting.
/// Complementary to JA4T (client-side TCP fingerprint).
/// format: \[WindowSize\]_\[TCP_Options\]_\[MSS\]_\[TTL\]_\[WScale\]
///
/// ## Key Differences from JA4T (Client)
/// - Captures server SYN-ACK response parameters
/// - Includes window scaling factor (WScale) for server identification
/// - Server TCP stacks have distinct default configurations
///
/// ## OS Identification via JA4TS
/// - Linux: TTL=64, WScale=7, MSS=1460
/// - Windows: TTL=128, WScale=8, MSS=1460
/// - macOS/iOS: TTL=64, WScale=6, MSS=1460
/// - FreeBSD: TTL=64, WScale=6, MSS=1460
///
/// ## Examples
/// ```
/// use fingerprint_core::ja4::JA4TS;
///
/// let ja4ts = JA4TS::generate(65535, "mss,nop,ws,nop,nop,ts", 1460, 64, 7);
/// assert!(!ja4ts.fingerprint_string().is_empty());
///
/// let os = ja4ts.infer_os();
/// assert!(!os.is_empty());
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct JA4TS {
    /// Server SYN-ACK window size
    pub window_size: u16,
    /// TCP options in SYN-ACK (ordered, comma-separated)
    pub options: String,
    /// Maximum Segment Size from server
    pub mss: u16,
    /// Time-To-Live (initial TTL before routing)
    pub ttl: u8,
    /// Window scaling factor (from TCP WScale option)
    pub wscale: u8,
}

impl JA4TS {
    /// Generate JA4TS server TCP fingerprint from SYN-ACK parameters
    pub fn generate(window_size: u16, options: &str, mss: u16, ttl: u8, wscale: u8) -> Self {
        Self {
            window_size,
            options: options.to_string(),
            mss,
            ttl,
            wscale,
        }
    }

    /// Convert to standard JA4TS fingerprint string
    /// format: {window_size}_{options}_{mss}_{ttl}_{wscale}
    pub fn fingerprint_string(&self) -> String {
        format!(
            "{}_{}_{}_{}_{}",
            self.window_size, self.options, self.mss, self.ttl, self.wscale
        )
    }

    /// Normalize TTL to likely initial value (nearest power-of-2 boundary)
    /// Routers decrement TTL, so we estimate the original value
    pub fn normalized_ttl(&self) -> u8 {
        match self.ttl {
            0..=32 => 32,
            33..=64 => 64,
            65..=128 => 128,
            _ => 255,
        }
    }

    /// Infer server operating system based on TCP stack characteristics
    pub fn infer_os(&self) -> String {
        let norm_ttl = self.normalized_ttl();
        match (norm_ttl, self.wscale) {
            (128, 8) => "Windows".to_string(),
            (128, _) => "Windows (variant)".to_string(),
            (64, 7) => "Linux".to_string(),
            (64, 6) => "macOS/FreeBSD".to_string(),
            (64, _) => "Unix-like".to_string(),
            (255, _) => "Solaris/AIX".to_string(),
            _ => "Unknown".to_string(),
        }
    }

    /// Calculate similarity between two JA4TS fingerprints (0.0 - 1.0)
    pub fn similarity(&self, other: &JA4TS) -> f64 {
        let mut score = 0.0;
        let total = 5.0;

        if self.normalized_ttl() == other.normalized_ttl() {
            score += 1.0;
        }
        if self.wscale == other.wscale {
            score += 1.0;
        }
        if self.mss == other.mss {
            score += 1.0;
        }
        if self.options == other.options {
            score += 1.0;
        }
        if self.window_size == other.window_size {
            score += 1.0;
        }

        score / total
    }
}

impl std::fmt::Display for JA4TS {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.fingerprint_string())
    }
}

#[cfg(test)]
mod ja4ts_tests {
    use super::*;

    #[test]
    fn test_ja4ts_generation() {
        let ja4ts = JA4TS::generate(65535, "mss,nop,ws,nop,nop,ts", 1460, 64, 7);
        assert_eq!(ja4ts.window_size, 65535);
        assert_eq!(ja4ts.mss, 1460);
        assert_eq!(ja4ts.ttl, 64);
        assert_eq!(ja4ts.wscale, 7);
    }

    #[test]
    fn test_ja4ts_fingerprint_string() {
        let ja4ts = JA4TS::generate(65535, "mss,nop,ws", 1460, 64, 7);
        let fp = ja4ts.fingerprint_string();
        assert_eq!(fp, "65535_mss,nop,ws_1460_64_7");
    }

    #[test]
    fn test_ja4ts_display() {
        let ja4ts = JA4TS::generate(65535, "mss,nop,ws", 1460, 128, 8);
        assert_eq!(format!("{}", ja4ts), ja4ts.fingerprint_string());
    }

    #[test]
    fn test_ja4ts_normalized_ttl() {
        assert_eq!(JA4TS::generate(0, "", 0, 64, 0).normalized_ttl(), 64);
        assert_eq!(JA4TS::generate(0, "", 0, 128, 0).normalized_ttl(), 128);
        assert_eq!(JA4TS::generate(0, "", 0, 60, 0).normalized_ttl(), 64);
        assert_eq!(JA4TS::generate(0, "", 0, 120, 0).normalized_ttl(), 128);
        assert_eq!(JA4TS::generate(0, "", 0, 250, 0).normalized_ttl(), 255);
    }

    #[test]
    fn test_ja4ts_infer_os_linux() {
        let ja4ts = JA4TS::generate(65535, "mss,nop,ws,nop,nop,ts", 1460, 64, 7);
        assert_eq!(ja4ts.infer_os(), "Linux");
    }

    #[test]
    fn test_ja4ts_infer_os_windows() {
        let ja4ts = JA4TS::generate(65535, "mss,nop,ws,nop,nop,ts", 1460, 128, 8);
        assert_eq!(ja4ts.infer_os(), "Windows");
    }

    #[test]
    fn test_ja4ts_infer_os_macos() {
        let ja4ts = JA4TS::generate(65535, "mss,nop,ws,nop,nop,ts", 1460, 64, 6);
        assert_eq!(ja4ts.infer_os(), "macOS/FreeBSD");
    }

    #[test]
    fn test_ja4ts_similarity() {
        let ja4ts1 = JA4TS::generate(65535, "mss,nop,ws", 1460, 64, 7);
        let ja4ts2 = JA4TS::generate(65535, "mss,nop,ws", 1460, 64, 7);
        assert!((ja4ts1.similarity(&ja4ts2) - 1.0).abs() < f64::EPSILON);

        let ja4ts3 = JA4TS::generate(65535, "mss,nop,ws", 1460, 128, 8);
        assert!(ja4ts1.similarity(&ja4ts3) < 1.0);
    }
}
