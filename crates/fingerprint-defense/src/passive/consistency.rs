//! fingerprintconsistencyChecker
//!
// ! crossValidate TCP, TLS and HTTP layercountdata, detectdeceivebehavior and abnormal机er人.
// ! implementation完整of跨层consistency审计，detectUser-Agent与底层TCPstack、TLSversionofconsistency

use fingerprint_core::fingerprint::FingerprintType;
use fingerprint_core::ja4::ConsistencyReport;
use fingerprint_core::system::NetworkFlow;
use std::collections::HashMap;

// / consistency违规type
#[derive(Debug, Clone, PartialEq)]
pub enum ConsistencyViolation {
    // / TCPstack与User-Agent不匹配
    TcpStackMismatch {
        tcp_detected: String,
        ua_claimed: String,
    },
    // / TLSversion与浏览器version不匹配
    TlsVersionMismatch {
        tls_version: String,
        browser_version: String,
    },
    // / HTTP/2set与浏览器fingerprint不匹配
    Http2SettingsMismatch {
        expected: Vec<(u16, u32)>,
        actual: Vec<(u16, u32)>,
    },
    // / JA4fingerprint与HTTP头不一致
    Ja4HttpInconsistency {
        ja4: String,
        http_features: String,
    },
    // / time戳exception（可能of重放攻击）
    TimestampAnomaly {
        expected_range: (u64, u64),
        actual: u64,
    },
}

// / consistencyanalyze引擎
pub struct ConsistencyAnalyzer {
    // / 已知of浏览器-TCPstackmap
    browser_tcp_mapping: HashMap<String, Vec<String>>,
    // / 浏览器-TLSversioncompatibility表
    browser_tls_compatibility: HashMap<String, Vec<String>>,
}

impl Default for ConsistencyAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl ConsistencyAnalyzer {
    pub fn new() -> Self {
        let mut analyzer = Self {
            browser_tcp_mapping: HashMap::new(),
            browser_tls_compatibility: HashMap::new(),
        };

        // initialization已知of浏览器-TCPstackmap
        analyzer.init_browser_tcp_mappings();
        analyzer.init_browser_tls_compatibility();

        analyzer
    }

    // / initialization浏览器-TCPstackmap
    fn init_browser_tcp_mappings(&mut self) {
        // Windows浏览器通常useWindows TCPstack
        self.browser_tcp_mapping.insert(
            "windows".to_string(),
            vec!["windows".to_string(), "winnt".to_string()],
        );

        // macOS浏览器
        self.browser_tcp_mapping.insert(
            "macintosh".to_string(),
            vec![
                "darwin".to_string(),
                "macos".to_string(),
                "apple".to_string(),
            ],
        );

        // Linux浏览器
        self.browser_tcp_mapping.insert(
            "linux".to_string(),
            vec!["linux".to_string(), "unix".to_string()],
        );

        // iOS浏览器
        self.browser_tcp_mapping.insert(
            "iphone".to_string(),
            vec!["ios".to_string(), "darwin".to_string(), "apple".to_string()],
        );
        self.browser_tcp_mapping.insert(
            "ipad".to_string(),
            vec!["ios".to_string(), "darwin".to_string(), "apple".to_string()],
        );

        // Android浏览器
        self.browser_tcp_mapping.insert(
            "android".to_string(),
            vec!["linux".to_string(), "android".to_string()],
        );
    }

    // / initialization浏览器-TLSversioncompatibility
    fn init_browser_tls_compatibility(&mut self) {
        // 现代浏览器supportTLS 1.2/1.3
        let modern_browsers = vec!["chrome", "firefox", "safari", "edge", "opera"];
        for browser in modern_browsers {
            self.browser_tls_compatibility.insert(
                browser.to_string(),
                vec!["TLSv1.2".to_string(), "TLSv1.3".to_string()],
            );
        }

        // 老旧浏览器可能只supportTLS 1.0/1.1
        self.browser_tls_compatibility.insert(
            "ie".to_string(),
            vec![
                "TLSv1.0".to_string(),
                "TLSv1.1".to_string(),
                "TLSv1.2".to_string(),
            ],
        );
    }
}

impl ConsistencyAnalyzer {
    // / analyze流量of多层consistency
    pub fn analyze_flow(&self, flow: &NetworkFlow) -> ConsistencyReport {
        let mut report = ConsistencyReport::new();

        let tls_fingerprints = flow.get_fingerprints_by_type(FingerprintType::Tls);
        let http_fingerprints = flow.get_fingerprints_by_type(FingerprintType::Http);
        let tcp_fingerprints = flow.get_fingerprints_by_type(FingerprintType::Tcp);

        // 1. validateTCPandHTTP（operating system级别consistency）
        if let (Some(tcp), Some(http)) = (tcp_fingerprints.first(), http_fingerprints.first()) {
            self.check_tcp_http_consistency(tcp, http, &mut report);
        }

        // 2. validateTLSandHTTP(浏览器versionconsistency)
        if let (Some(tls), Some(http)) = (tls_fingerprints.first(), http_fingerprints.first()) {
            self.check_tls_http_consistency(tls, http, &mut report);
        }

        // 3. validateJA4+全stackconsistency
        self.check_ja4_plus_consistency(flow, &mut report);

        // 4. validatetime戳consistency（防重放攻击）
        self.check_timestamp_consistency(flow, &mut report);

        report
    }

    // / checkTCPandHTTPconsistency
    fn check_tcp_http_consistency(
        &self,
        tcp_fp: &&dyn fingerprint_core::fingerprint::Fingerprint,
        http_fp: &&dyn fingerprint_core::fingerprint::Fingerprint,
        report: &mut ConsistencyReport,
    ) {
        let ua = http_fp.metadata().get("user_agent").or_else(|| {
            let id = http_fp.id();
            if id != "unknown" {
                Some(id)
            } else {
                None
            }
        });

        let tcp_os_hint = tcp_fp
            .metadata()
            .get("os")
            .or_else(|| self.extract_os_from_tcp_fingerprint(&tcp_fp.id()));

        let (ua_lower, tcp_os_hint) = match (ua, tcp_os_hint) {
            (Some(ua_value), Some(os_value)) => (ua_value.to_lowercase(), os_value),
            _ => return,
        };

        // checkTCPstack与User-Agent声明ofconsistency
        if !self.is_os_consistent(&tcp_os_hint, &ua_lower) {
            // 注意：这里simplifyprocess，实际应该从fingerprint中extract相关info
            report.add_discrepancy(
                format!("TCP栈检测为{}，但User-Agent声明可能存在不一致", tcp_os_hint),
                70, // 中高risk
            );
        }
    }

    // / checkTLSandHTTPconsistency
    fn check_tls_http_consistency(
        &self,
        tls_fp: &&dyn fingerprint_core::fingerprint::Fingerprint,
        http_fp: &&dyn fingerprint_core::fingerprint::Fingerprint,
        report: &mut ConsistencyReport,
    ) {
        let tls_version = tls_fp
            .metadata()
            .get("tls_version")
            .and_then(|hex| self.parse_tls_version_hex(&hex))
            .or_else(|| self.extract_tls_version_from_ja4(&tls_fp.id()));

        let ua = http_fp.metadata().get("user_agent").or_else(|| {
            let id = http_fp.id();
            if id != "unknown" {
                Some(id)
            } else {
                None
            }
        });

        let browser_name = http_fp.metadata().get("browser").or_else(|| {
            ua.as_ref()
                .and_then(|value| self.infer_browser_from_ua(value))
        });

        let (tls_version, browser_name) = match (tls_version, browser_name) {
            (Some(version), Some(browser)) => (version, browser),
            _ => return,
        };

        let browser_key = browser_name.to_lowercase();
        if !self.is_tls_version_compatible(&browser_key, &tls_version) {
            report.add_discrepancy(
                format!(
                    "TLS版本{}与浏览器({})可能存在兼容性问题",
                    tls_version, browser_name
                ),
                60, // 中等risk
            );
        }
    }

    // / checkJA4+全stackconsistency
    fn check_ja4_plus_consistency(&self, flow: &NetworkFlow, report: &mut ConsistencyReport) {
        // getallJA4fingerprint
        let ja4_fingerprints: Vec<_> = flow
            .fingerprints()
            .iter()
            .filter(|fp| fp.fingerprint_type() == FingerprintType::Tls)
            .collect();

        let ja4h_fingerprints: Vec<_> = flow
            .fingerprints()
            .iter()
            .filter(|fp| fp.fingerprint_type() == FingerprintType::Http)
            .collect();

        // checkTLS JA4与HTTP JA4Hofconsistency
        if !ja4_fingerprints.is_empty() && !ja4h_fingerprints.is_empty() {
            // 从JA4fingerprintextractclientfeatures
            let ja4_id = ja4_fingerprints[0].id();
            let ja4h_id = ja4h_fingerprints[0].id();

            // checkfingerprintconsistency
            if let Some(tls_client) = self.extract_client_from_ja4(&ja4_id) {
                if let Some(http_client) = self.extract_client_from_ja4h(&ja4h_id) {
                    if !self.is_client_fingerprint_consistent(&tls_client, &http_client) {
                        report.add_discrepancy(
                            format!(
                                "JA4指纹({})与JA4H指纹({})指示的客户端特征不一致，可能是指纹伪造",
                                ja4_id, ja4h_id
                            ),
                            80, // 高risk
                        );
                    }
                }
            }
        }

        // checkTCP JA4T（如果有）
        let tcp_fingerprints: Vec<_> = flow
            .fingerprints()
            .iter()
            .filter(|fp| fp.fingerprint_type() == FingerprintType::Tcp)
            .collect();

        if !tcp_fingerprints.is_empty() && !ja4_fingerprints.is_empty() {
            let tcp_fp_id = tcp_fingerprints[0].id();
            let ja4_id = ja4_fingerprints[0].id();

            // 从TCPfingerprint推断operating system
            if let Some(tcp_os) = self.extract_os_from_tcp_fingerprint(&tcp_fp_id) {
                if let Some(ja4_client) = self.extract_client_from_ja4(&ja4_id) {
                    if !self.is_os_client_consistent(&tcp_os, &ja4_client) {
                        report.add_discrepancy(
                            format!(
                                "TCP指纹指示的操作系统({})与JA4指纹指示的客户端({})不一致",
                                tcp_os, ja4_client.name
                            ),
                            75, // 中高risk
                        );
                    }
                }
            }
        }
    }

    // / 从JA4fingerprintextractclientinfo
    fn extract_client_from_ja4(&self, ja4: &str) -> Option<BrowserInfo> {
        // JA4格式: t13d1516h2_8daaf6152771_000a
        let parts: Vec<&str> = ja4.split('_').collect();
        if parts.is_empty() {
            return None;
        }

        let version_part = parts[0];
        if version_part.len() < 3 {
            return None;
        }

        // extractTLSversion
        let tls_version = &version_part[1..3];

        // 根据fingerprintfeatures推断浏览器type
        // 这是一个simplifyof推断，实际应该use更复杂of规则
        let browser = if version_part.contains("d") {
            // d = domain (SNI present), 通常是现代浏览器
            BrowserInfo {
                name: "Modern Browser".to_string(),
                version: format!("TLS {}", tls_version),
            }
        } else {
            BrowserInfo {
                name: "Unknown".to_string(),
                version: tls_version.to_string(),
            }
        };

        Some(browser)
    }

    // / 从JA4Hfingerprintextractclientinfo
    fn extract_client_from_ja4h(&self, ja4h: &str) -> Option<BrowserInfo> {
        // JA4H格式类似JA4，但includeHTTPfeatures
        // 这里simplifyprocess，实际应该parseJA4Hof特定字段
        let parts: Vec<&str> = ja4h.split('_').collect();
        if parts.is_empty() {
            return None;
        }

        // 从JA4H推断浏览器
        let first_part = parts[0];
        let browser = if first_part.contains("ge") || first_part.contains("ch") {
            BrowserInfo {
                name: "Chrome".to_string(),
                version: "unknown".to_string(),
            }
        } else if first_part.contains("ff") || first_part.contains("fx") {
            BrowserInfo {
                name: "Firefox".to_string(),
                version: "unknown".to_string(),
            }
        } else {
            BrowserInfo {
                name: "Unknown".to_string(),
                version: "unknown".to_string(),
            }
        };

        Some(browser)
    }

    // / checkclientfingerprint是否一致
    fn is_client_fingerprint_consistent(
        &self,
        tls_client: &BrowserInfo,
        http_client: &BrowserInfo,
    ) -> bool {
        // 如果两者都能recognition且name不同，则不一致
        if tls_client.name != "Unknown"
            && http_client.name != "Unknown"
            && tls_client.name != http_client.name
        {
            return false;
        }
        true
    }

    // / 从TCPfingerprint推断operating system
    fn extract_os_from_tcp_fingerprint(&self, tcp_fp: &str) -> Option<String> {
        // TCPfingerprint通常includeTTL、窗口size等info
        // simplifyof推断逻辑
        if tcp_fp.contains("ttl=64") {
            Some("Linux/macOS".to_string())
        } else if tcp_fp.contains("ttl=128") {
            Some("Windows".to_string())
        } else if tcp_fp.contains("ttl=255") {
            Some("BSD/Unix".to_string())
        } else {
            None
        }
    }

    // / checkoperating systemandclient是否一致
    fn is_os_client_consistent(&self, os: &str, client: &BrowserInfo) -> bool {
        // checkoperating systemand浏览器of合理性
        // 例如：Windows上ofSafari是不合理of
        let os_lower = os.to_lowercase();
        let client_lower = client.name.to_lowercase();

        if client_lower.contains("safari") && !os_lower.contains("mac") && !os_lower.contains("ios")
        {
            // Safari主要在macOSandiOS上
            return false;
        }

        if client_lower.contains("edge") && os_lower.contains("mac") {
            // Edge在macOS上较少见（虽然有可能）
            return false;
        }

        true
    }

    // / checktime戳consistency
    fn check_timestamp_consistency(&self, flow: &NetworkFlow, report: &mut ConsistencyReport) {
        use chrono::Utc;

        // get当前time（Unixtime戳）
        let now = Utc::now().timestamp() as u64;
        let flow_timestamp = flow.context.timestamp.timestamp() as u64;

        // check流量time戳是否在未来（exception）
        if flow_timestamp > now + 60 {
            report.add_discrepancy(
                format!(
                    "流量时间戳({})比当前时间晚{}秒，可能存在时间同步问题或重放攻击",
                    flow_timestamp,
                    flow_timestamp - now
                ),
                90, // 高risk
            );
        }

        // check流量time戳是否过于陈旧（超过1小时）
        if flow_timestamp < now.saturating_sub(3600) {
            report.add_discrepancy(
                format!(
                    "流量时间戳({})比当前时间早{}秒，可能是延迟处理的流量或重放攻击",
                    flow_timestamp,
                    now - flow_timestamp
                ),
                60, // 中等risk
            );
        }
    }

    // / checkoperating systemconsistency
    fn is_os_consistent(&self, tcp_os: &str, ua: &str) -> bool {
        for (ua_keyword, compatible_tcp_os_list) in &self.browser_tcp_mapping {
            if ua.contains(ua_keyword) {
                return compatible_tcp_os_list.iter().any(|tcp_os_name| {
                    tcp_os.to_lowercase().contains(tcp_os_name)
                        || tcp_os_name.contains(&tcp_os.to_lowercase())
                });
            }
        }
        true // default认to一致，避免误报
    }

    // / 从JA4fingerprintextractTLSversion
    fn extract_tls_version_from_ja4(&self, ja4: &str) -> Option<String> {
        // JA4格式: <TLSVersion>_<Ciphers>_<Extensions>_<Curves>_<SigAlgs>
        if let Some(version_part) = ja4.split('_').next() {
            match version_part {
                "13" => Some("TLSv1.3".to_string()),
                "12" => Some("TLSv1.2".to_string()),
                "11" => Some("TLSv1.1".to_string()),
                "10" => Some("TLSv1.0".to_string()),
                _ => None,
            }
        } else {
            None
        }
    }

    fn parse_tls_version_hex(&self, hex: &str) -> Option<String> {
        let normalized = hex.trim_start_matches("0x");
        let value = u16::from_str_radix(normalized, 16).ok()?;
        match value {
            0x0304 => Some("TLSv1.3".to_string()),
            0x0303 => Some("TLSv1.2".to_string()),
            0x0302 => Some("TLSv1.1".to_string()),
            0x0301 => Some("TLSv1.0".to_string()),
            _ => None,
        }
    }

    fn infer_browser_from_ua(&self, user_agent: &str) -> Option<String> {
        let ua_lower = user_agent.to_lowercase();

        if ua_lower.contains("edg/") {
            Some("Edge".to_string())
        } else if ua_lower.contains("opr/") || ua_lower.contains("opera") {
            Some("Opera".to_string())
        } else if ua_lower.contains("chrome")
            && !ua_lower.contains("chromium")
            && !ua_lower.contains("edg/")
        {
            Some("Chrome".to_string())
        } else if ua_lower.contains("firefox") {
            Some("Firefox".to_string())
        } else if ua_lower.contains("safari") && !ua_lower.contains("chrome") {
            Some("Safari".to_string())
        } else {
            None
        }
    }

    // / checkTLSversioncompatibility
    fn is_tls_version_compatible(&self, browser: &str, tls_version: &str) -> bool {
        if let Some(compatible_versions) = self.browser_tls_compatibility.get(browser) {
            compatible_versions.contains(&tls_version.to_string())
        } else {
            true // unknown浏览器default认to兼容
        }
    }
}

// / 浏览器infostructure
#[derive(Debug)]
struct BrowserInfo {
    name: String,
    #[allow(dead_code)]
    version: String,
}
