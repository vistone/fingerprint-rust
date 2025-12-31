//! 指纹一致性检查器
//!
//! 交叉验证 TCP、TLS 和 HTTP 层的数据，检测欺骗行为和异常机器人。

use fingerprint_core::fingerprint::FingerprintType;
use fingerprint_core::ja4::ConsistencyReport;
use fingerprint_core::system::NetworkFlow;

/// 一致性分析引擎
pub struct ConsistencyAnalyzer;

impl ConsistencyAnalyzer {
    pub fn new() -> Self {
        Self
    }
}

impl ConsistencyAnalyzer {
    /// 分析流量中的多层一致性
    pub fn analyze_flow(&self, flow: &NetworkFlow) -> ConsistencyReport {
        let mut report = ConsistencyReport::new();

        let tls_fingerprints = flow.get_fingerprints_by_type(FingerprintType::Tls);
        let http_fingerprints = flow.get_fingerprints_by_type(FingerprintType::Http);
        let tcp_fingerprints = flow.get_fingerprints_by_type(FingerprintType::Tcp);

        // 1. 验证 TCP 与 HTTP (OS 级别一致性)
        if let (Some(tcp), Some(http)) = (tcp_fingerprints.first(), http_fingerprints.first()) {
            let tcp_os = tcp.to_string().to_lowercase();
            let ua = http.to_string().to_lowercase();

            if ua.contains("windows") && !tcp_os.contains("windows") && tcp_os.contains("linux") {
                report.add_discrepancy(
                    "TCP 栈识别为 Linux，但 HTTP User-Agent 声称是 Windows".to_string(),
                    50,
                );
            }

            if ua.contains("iphone")
                && !tcp_os.contains("apple")
                && !tcp_os.contains("ios")
                && tcp_os.contains("linux")
            {
                report.add_discrepancy(
                    "User-Agent 为 iPhone，但 TCP 特征更接近 Linux (可能是 Android 或 爬虫库)"
                        .to_string(),
                    30,
                );
            }

            // 检查 TTL 与 OS 是否匹配
            if ua.contains("windows") && tcp_os.contains("linux") {
                // 可能使用了代理或指纹混淆不完全
            }
        }

        // 2. 验证 TLS 与 HTTP (浏览器级别一致性)
        if let (Some(tls), Some(http)) = (tls_fingerprints.first(), http_fingerprints.first()) {
            let tls_info = tls.to_string().to_lowercase();
            let ua = http.to_string().to_lowercase();

            // 检查 Chrome 特征
            if ua.contains("chrome") {
                // 如果是现代 Chrome (120+)，必须支持 TLS 1.3
                if (ua.contains("chrome/1") || ua.contains("chrome/12") || ua.contains("chrome/13"))
                    && !tls_info.contains("version: some(0x0304)")
                {
                    report.add_discrepancy(
                        "现代 Chrome (120+) 必须使用 TLS 1.3，检测到协议降级".to_string(),
                        50,
                    );
                }

                // 检查 ALPN 冲突
                if ua.contains("h2") && !tls_info.contains("h2") && tls_info.contains("alpn") {
                    report.add_discrepancy(
                        "HTTP/2 请求来自未在 TLS 握手中协商 h2 的连接".to_string(),
                        60,
                    );
                }
            }
        }

        // 3. 验证协议降级异常
        if flow.context.protocol == fingerprint_core::system::ProtocolType::Http
            && (flow.context.target_port == Some(443))
        {
            report.add_discrepancy(
                "在 443 端口检测到明文 HTTP 流量 (可能是强制协议降级攻击)".to_string(),
                50,
            );
        }

        // 4. JA4+ 系列交叉验证 (更深层的指纹一致性)
        if let (Some(tls), Some(http)) = (tls_fingerprints.first(), http_fingerprints.first()) {
            if let (Some(ja4), Some(ja4h)) =
                (tls.metadata().get("ja4"), http.metadata().get("ja4h"))
            {
                // 如果 JA4 显示是现代 Chrome (t13d...), 但 JA4H 显示是 HTTP/1.1 (..11..) 且没有 Cookie (..n..)
                // 这是一个常见的爬虫特征
                if ja4.starts_with("t13") && ja4h.contains("11n") {
                    report.add_discrepancy(
                        format!("检测到现代 TLS 特征 (JA4: {})，但 HTTP 行为表现为传统无 Cookie 请求 (JA4H: {})", ja4, ja4h),
                        20,
                    );
                }

                // 检查 ALPN 一致性
                if ja4.contains("h2") && ja4h.contains("11") {
                    // TLS 协商了 h2，但实际发送了 HTTP/1.1
                    report.add_discrepancy(
                        "TLS 握手协商了 h2，但实际请求使用了 HTTP/1.1".to_string(),
                        30,
                    );
                }
            }
        }

        report
    }
}
