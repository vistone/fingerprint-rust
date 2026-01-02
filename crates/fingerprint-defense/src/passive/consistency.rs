//! fingerprint一致性Check器
//!
//! 交叉Validate TCP、TLS  and HTTP layer的count据，检测欺骗行为 and 异常机器人。

use fingerprint_core::fingerprint::FingerprintType;
use fingerprint_core::ja4::ConsistencyReport;
use fingerprint_core::system::NetworkFlow;

/// 一致性analysis引擎
pub struct ConsistencyAnalyzer;

impl Default for ConsistencyAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl ConsistencyAnalyzer {
    pub fn new() -> Self {
        Self
    }
}

impl ConsistencyAnalyzer {
    /// analysistraffic中的多layer一致性
    pub fn analyze_flow(&self, flow: &NetworkFlow) -> ConsistencyReport {
        let mut report = ConsistencyReport::new();

        let tls_fingerprints = flow.get_fingerprints_by_type(FingerprintType::Tls);
        let http_fingerprints = flow.get_fingerprints_by_type(FingerprintType::Http);
        let tcp_fingerprints = flow.get_fingerprints_by_type(FingerprintType::Tcp);

        // 1. Validate TCP  and HTTP (OS level一致性)
        if let (Some(tcp), Some(http)) = (tcp_fingerprints.first(), http_fingerprints.first()) {
            let tcp_os = tcp.to_string().to_lowercase();
            let ua = http.to_string().to_lowercase();

            if ua.contains("windows") && !tcp_os.contains("windows") && tcp_os.contains("linux") {
                report.add_discrepancy(
                    "TCP stack识别为 Linux，but HTTP User-Agent 声称是 Windows".to_string(),
                    50,
                );
            }

            if ua.contains("iphone")
                && !tcp_os.contains("apple")
                && !tcp_os.contains("ios")
                && tcp_os.contains("linux")
            {
                report.add_discrepancy(
                    "User-Agent 为 iPhone，but TCP trait更接近 Linux (may是 Android  or  爬虫库)"
                        .to_string(),
                    30,
                );
            }

            // Check TTL  and OS whethermatch
            if ua.contains("windows") && tcp_os.contains("linux") {
                // mayuse了proxy or fingerprint混淆不完全
            }
        }

        // 2. Validate TLS  and HTTP (browserlevel一致性)
        if let (Some(tls), Some(http)) = (tls_fingerprints.first(), http_fingerprints.first()) {
            let tls_info = tls.to_string().to_lowercase();
            let ua = http.to_string().to_lowercase();

            // Check Chrome trait
            if ua.contains("chrome") {
                // If是现代 Chrome (120+), mustsupport TLS 1.3
                if (ua.contains("chrome/1") || ua.contains("chrome/12") || ua.contains("chrome/13"))
                    && !tls_info.contains("version: some(0x0304)")
                {
                    report.add_discrepancy(
                        "现代 Chrome (120+) mustuse TLS 1.3，检测 to protocol降level".to_string(),
                        50,
                    );
                }

                // Check ALPN 冲突
                if ua.contains("h2") && !tls_info.contains("h2") && tls_info.contains("alpn") {
                    report.add_discrepancy(
                        "HTTP/2 request来自not in TLS handshake中协商 h2 的connection".to_string(),
                        60,
                    );
                }
            }
        }

        // 3. Validateprotocol降level异常
        if flow.context.protocol == fingerprint_core::system::ProtocolType::Http
            && (flow.context.target_port == Some(443))
        {
            report.add_discrepancy(
                " in 443 port检测 to 明文 HTTP traffic (may是强制protocol降level攻击)".to_string(),
                50,
            );
        }

        // 4. JA4+ 系列交叉Validate (更深layer的fingerprint一致性)
        if let (Some(tls), Some(http)) = (tls_fingerprints.first(), http_fingerprints.first()) {
            if let (Some(ja4), Some(ja4h)) =
                (tls.metadata().get("ja4"), http.metadata().get("ja4h"))
            {
                //  if  JA4 显示是现代 Chrome (t13d...), but JA4H 显示是 HTTP/1.1 (..11..) 且没有 Cookie (..n..)
                // 这是an常见的爬虫trait
                if ja4.starts_with("t13") && ja4h.contains("11n") {
                    report.add_discrepancy(
                        format!("检测 to 现代 TLS trait (JA4: {})，but HTTP 行为表现为传统无 Cookie request (JA4H: {})", ja4, ja4h),
                        20,
                    );
                }

                // Check ALPN 一致性
                if ja4.contains("h2") && ja4h.contains("11") {
                    // TLS 协商了 h2，but实际send了 HTTP/1.1
                    report.add_discrepancy(
                        "TLS handshake协商了 h2，but实际requestuse了 HTTP/1.1".to_string(),
                        30,
                    );
                }
            }
        }

        report
    }
}
