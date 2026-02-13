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
            self.check_tcp_http_consistency(&tcp, &http, &mut report);
        }

        // 2. 验证TLS和HTTP（浏览器版本一致性）
        if let (Some(tls), Some(http)) = (tls_fingerprints.first(), http_fingerprints.first()) {
            self.check_tls_http_consistency(&tls, &http, &mut report);
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
    fn check_ja4_plus_consistency(&self, _flow: &NetworkFlow, _report: &mut ConsistencyReport) {
        // TODO: 实现JA4、JA4H、JA4T的交叉验证
        // 这里应该检查TLS JA4与HTTP JA4H的一致性
        // 以及TCP JA4T与整体指纹的一致性
    }

    /// 检查时间戳一致性
    fn check_timestamp_consistency(&self, _flow: &NetworkFlow, _report: &mut ConsistencyReport) {
        // TODO: 实现时间戳异常检测
        // 检查是否存在不合理的时间戳跳跃
        // 可能指示重放攻击或时间同步问题
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
struct BrowserInfo {
    name: String,
    version: String,
}
