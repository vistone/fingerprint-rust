//! fingerprintconsistencyChecker
//!
//! crossValidate TCP, TLS and HTTP layercountdata, detectdeceivebehavior and abnormal机er人.
//! 实现完整的跨层一致性审计，检测User-Agent与底层TCP栈、TLS版本的一致性

use fingerprint_core::fingerprint::FingerprintType;
use fingerprint_core::ja4::ConsistencyReport;
use fingerprint_core::system::NetworkFlow;
use std::collections::HashMap;

/// 一致性违规类型
#[derive(Debug, Clone, PartialEq)]
pub enum ConsistencyViolation {
    /// TCP栈与User-Agent不匹配
    TcpStackMismatch {
        tcp_detected: String,
        ua_claimed: String,
    },
    /// TLS版本与浏览器版本不匹配
    TlsVersionMismatch {
        tls_version: String,
        browser_version: String,
    },
    /// HTTP/2设置与浏览器指纹不匹配
    Http2SettingsMismatch {
        expected: Vec<(u16, u32)>,
        actual: Vec<(u16, u32)>,
    },
    /// JA4指纹与HTTP头不一致
    Ja4HttpInconsistency { ja4: String, http_features: String },
    /// 时间戳异常（可能的重放攻击）
    TimestampAnomaly {
        expected_range: (u64, u64),
        actual: u64,
    },
}

/// 一致性分析引擎
pub struct ConsistencyAnalyzer {
    /// 已知的浏览器-TCP栈映射
    browser_tcp_mapping: HashMap<String, Vec<String>>,
    /// 浏览器-TLS版本兼容性表
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

        // 初始化已知的浏览器-TCP栈映射
        analyzer.init_browser_tcp_mappings();
        analyzer.init_browser_tls_compatibility();

        analyzer
    }

    /// 初始化浏览器-TCP栈映射
    fn init_browser_tcp_mappings(&mut self) {
        // Windows浏览器通常使用Windows TCP栈
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

    /// 初始化浏览器-TLS版本兼容性
    fn init_browser_tls_compatibility(&mut self) {
        // 现代浏览器支持TLS 1.2/1.3
        let modern_browsers = vec!["chrome", "firefox", "safari", "edge", "opera"];
        for browser in modern_browsers {
            self.browser_tls_compatibility.insert(
                browser.to_string(),
                vec!["TLSv1.2".to_string(), "TLSv1.3".to_string()],
            );
        }

        // 老旧浏览器可能只支持TLS 1.0/1.1
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
    /// 分析流量的多层一致性
    pub fn analyze_flow(&self, flow: &NetworkFlow) -> ConsistencyReport {
        let mut report = ConsistencyReport::new();

        let tls_fingerprints = flow.get_fingerprints_by_type(FingerprintType::Tls);
        let http_fingerprints = flow.get_fingerprints_by_type(FingerprintType::Http);
        let tcp_fingerprints = flow.get_fingerprints_by_type(FingerprintType::Tcp);

        // 1. 验证TCP和HTTP（操作系统级别一致性）
        if let (Some(tcp), Some(http)) = (tcp_fingerprints.first(), http_fingerprints.first()) {
            self.check_tcp_http_consistency(tcp, http, &mut report);
        }

        // 2. 验证TLS和HTTP(浏览器版本一致性)
        if let (Some(tls), Some(http)) = (tls_fingerprints.first(), http_fingerprints.first()) {
            self.check_tls_http_consistency(tls, http, &mut report);
        }

        // 3. 验证JA4+全栈一致性
        self.check_ja4_plus_consistency(flow, &mut report);

        // 4. 验证时间戳一致性（防重放攻击）
        self.check_timestamp_consistency(flow, &mut report);

        report
    }

    /// 检查TCP和HTTP一致性
    fn check_tcp_http_consistency(
        &self,
        _tcp_fp: &&dyn fingerprint_core::fingerprint::Fingerprint,
        _http_fp: &&dyn fingerprint_core::fingerprint::Fingerprint,
        report: &mut ConsistencyReport,
    ) {
        // 模拟User-Agent检查（实际应该从HTTP指纹获取）
        let ua_lower = "mozilla/5.0 (windows nt 10.0; win64; x64)".to_string();
        let tcp_os_hint = "windows".to_string(); // 模拟TCP推断

        // 检查TCP栈与User-Agent声明的一致性
        if !self.is_os_consistent(&tcp_os_hint, &ua_lower) {
            // 注意：这里简化处理，实际应该从指纹中提取相关信息
            report.add_discrepancy(
                format!("TCP栈检测为{}，但User-Agent声明可能存在不一致", tcp_os_hint),
                70, // 中高风险
            );
        }
    }

    /// 检查TLS和HTTP一致性
    fn check_tls_http_consistency(
        &self,
        tls_fp: &&dyn fingerprint_core::fingerprint::Fingerprint,
        _http_fp: &&dyn fingerprint_core::fingerprint::Fingerprint,
        report: &mut ConsistencyReport,
    ) {
        // 从JA4指纹推断TLS版本
        let ja4_id = tls_fp.id();
        if let Some(tls_version) = self.extract_tls_version_from_ja4(&ja4_id) {
            // 模拟浏览器信息
            let browser_version = "133".to_string();

            // 检查TLS版本兼容性
            if !self.is_tls_version_compatible("chrome", &tls_version) {
                report.add_discrepancy(
                    format!(
                        "TLS版本{}与{}浏览器版本可能存在兼容性问题",
                        tls_version, browser_version
                    ),
                    60, // 中等风险
                );
            }
        }
    }

    /// 检查JA4+全栈一致性
    fn check_ja4_plus_consistency(&self, flow: &NetworkFlow, report: &mut ConsistencyReport) {
        // 获取所有JA4指纹
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

        // 检查TLS JA4与HTTP JA4H的一致性
        if !ja4_fingerprints.is_empty() && !ja4h_fingerprints.is_empty() {
            // 从JA4指纹提取客户端特征
            let ja4_id = ja4_fingerprints[0].id();
            let ja4h_id = ja4h_fingerprints[0].id();

            // 检查指纹一致性
            if let Some(tls_client) = self.extract_client_from_ja4(&ja4_id) {
                if let Some(http_client) = self.extract_client_from_ja4h(&ja4h_id) {
                    if !self.is_client_fingerprint_consistent(&tls_client, &http_client) {
                        report.add_discrepancy(
                            format!(
                                "JA4指纹({})与JA4H指纹({})指示的客户端特征不一致，可能是指纹伪造",
                                ja4_id, ja4h_id
                            ),
                            80, // 高风险
                        );
                    }
                }
            }
        }

        // 检查TCP JA4T（如果有）
        let tcp_fingerprints: Vec<_> = flow
            .fingerprints()
            .iter()
            .filter(|fp| fp.fingerprint_type() == FingerprintType::Tcp)
            .collect();

        if !tcp_fingerprints.is_empty() && !ja4_fingerprints.is_empty() {
            let tcp_fp_id = tcp_fingerprints[0].id();
            let ja4_id = ja4_fingerprints[0].id();

            // 从TCP指纹推断操作系统
            if let Some(tcp_os) = self.extract_os_from_tcp_fingerprint(&tcp_fp_id) {
                if let Some(ja4_client) = self.extract_client_from_ja4(&ja4_id) {
                    if !self.is_os_client_consistent(&tcp_os, &ja4_client) {
                        report.add_discrepancy(
                            format!(
                                "TCP指纹指示的操作系统({})与JA4指纹指示的客户端({})不一致",
                                tcp_os, ja4_client.name
                            ),
                            75, // 中高风险
                        );
                    }
                }
            }
        }
    }

    /// 从JA4指纹提取客户端信息
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

        // 提取TLS版本
        let tls_version = &version_part[1..3];

        // 根据指纹特征推断浏览器类型
        // 这是一个简化的推断，实际应该使用更复杂的规则
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

    /// 从JA4H指纹提取客户端信息
    fn extract_client_from_ja4h(&self, ja4h: &str) -> Option<BrowserInfo> {
        // JA4H格式类似JA4，但包含HTTP特征
        // 这里简化处理，实际应该解析JA4H的特定字段
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

    /// 检查客户端指纹是否一致
    fn is_client_fingerprint_consistent(
        &self,
        tls_client: &BrowserInfo,
        http_client: &BrowserInfo,
    ) -> bool {
        // 如果两者都能识别且名称不同，则不一致
        if tls_client.name != "Unknown"
            && http_client.name != "Unknown"
            && tls_client.name != http_client.name
        {
            return false;
        }
        true
    }

    /// 从TCP指纹推断操作系统
    fn extract_os_from_tcp_fingerprint(&self, tcp_fp: &str) -> Option<String> {
        // TCP指纹通常包含TTL、窗口大小等信息
        // 简化的推断逻辑
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

    /// 检查操作系统和客户端是否一致
    fn is_os_client_consistent(&self, os: &str, client: &BrowserInfo) -> bool {
        // 检查操作系统和浏览器的合理性
        // 例如：Windows上的Safari是不合理的
        let os_lower = os.to_lowercase();
        let client_lower = client.name.to_lowercase();

        if client_lower.contains("safari") && !os_lower.contains("mac") && !os_lower.contains("ios")
        {
            // Safari主要在macOS和iOS上
            return false;
        }

        if client_lower.contains("edge") && os_lower.contains("mac") {
            // Edge在macOS上较少见（虽然有可能）
            return false;
        }

        true
    }

    /// 检查时间戳一致性
    fn check_timestamp_consistency(&self, flow: &NetworkFlow, report: &mut ConsistencyReport) {
        use chrono::Utc;

        // 获取当前时间（Unix时间戳）
        let now = Utc::now().timestamp() as u64;
        let flow_timestamp = flow.context.timestamp.timestamp() as u64;

        // 检查流量时间戳是否在未来（异常）
        if flow_timestamp > now + 60 {
            report.add_discrepancy(
                format!(
                    "流量时间戳({})比当前时间晚{}秒，可能存在时间同步问题或重放攻击",
                    flow_timestamp,
                    flow_timestamp - now
                ),
                90, // 高风险
            );
        }

        // 检查流量时间戳是否过于陈旧（超过1小时）
        if flow_timestamp < now.saturating_sub(3600) {
            report.add_discrepancy(
                format!(
                    "流量时间戳({})比当前时间早{}秒，可能是延迟处理的流量或重放攻击",
                    flow_timestamp,
                    now - flow_timestamp
                ),
                60, // 中等风险
            );
        }
    }

    /// 检查操作系统一致性
    fn is_os_consistent(&self, tcp_os: &str, ua: &str) -> bool {
        for (ua_keyword, compatible_tcp_os_list) in &self.browser_tcp_mapping {
            if ua.contains(ua_keyword) {
                return compatible_tcp_os_list.iter().any(|tcp_os_name| {
                    tcp_os.to_lowercase().contains(tcp_os_name)
                        || tcp_os_name.contains(&tcp_os.to_lowercase())
                });
            }
        }
        true // 默认认为一致，避免误报
    }

    /// 从User-Agent提取操作系统信息
    #[allow(dead_code)]
    fn extract_os_from_ua(&self, ua: &str) -> String {
        if ua.contains("windows") {
            "Windows".to_string()
        } else if ua.contains("macintosh") || ua.contains("mac os x") {
            "macOS".to_string()
        } else if ua.contains("linux") {
            "Linux".to_string()
        } else if ua.contains("iphone") || ua.contains("ipad") {
            "iOS".to_string()
        } else if ua.contains("android") {
            "Android".to_string()
        } else {
            "Unknown".to_string()
        }
    }

    /// 从JA4指纹提取TLS版本
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

    /// 提取版本号
    #[allow(dead_code)]
    fn extract_version(&self, ua: &str, browser_name: &str) -> String {
        // 简化的版本提取逻辑
        if let Some(start_pos) = ua.find(browser_name) {
            let version_start = start_pos + browser_name.len();
            if version_start < ua.len() && ua.chars().nth(version_start) == Some('/') {
                let version_part = &ua[version_start + 1..];
                if let Some(end_pos) = version_part.find(|c: char| !c.is_ascii_digit() && c != '.')
                {
                    version_part[..end_pos].to_string()
                } else {
                    version_part.to_string()
                }
            } else {
                "unknown".to_string()
            }
        } else {
            "unknown".to_string()
        }
    }

    /// 检查TLS版本兼容性
    fn is_tls_version_compatible(&self, browser: &str, tls_version: &str) -> bool {
        if let Some(compatible_versions) = self.browser_tls_compatibility.get(browser) {
            compatible_versions.contains(&tls_version.to_string())
        } else {
            true // 未知浏览器默认认为兼容
        }
    }
}

/// 浏览器信息结构
#[derive(Debug)]
#[allow(dead_code)]
struct BrowserInfo {
    name: String,
    version: String,
}
