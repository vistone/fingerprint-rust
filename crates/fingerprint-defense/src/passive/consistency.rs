//! fingerprintconsistencyChecker
//!
//! 交叉Validate TCP、TLS and HTTP layercountdata，detect欺骗behavior and abnormal机er人。

use fingerprint_core::fingerprint::FingerprintType;
use fingerprint_core::ja4::ConsistencyReport;
use fingerprint_core::system::NetworkFlow;

/// consistencyanalysisengine
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
 /// analysistrafficin多layerconsistency
 pub fn analyze_flow(&self, flow: &NetworkFlow) -> ConsistencyReport {
 let mut report = ConsistencyReport::new();

 let tls_fingerprints = flow.get_fingerprints_by_type(FingerprintType::Tls);
 let http_fingerprints = flow.get_fingerprints_by_type(FingerprintType::Http);
 let tcp_fingerprints = flow.get_fingerprints_by_type(FingerprintType::Tcp);

 // 1. Validate TCP and HTTP (OS levelconsistency)
 if let (Some(tcp), Some(http)) = (tcp_fingerprints.first(), http_fingerprints.first()) {
 let tcp_os = tcp.to_string().to_lowercase();
 let ua = http.to_string().to_lowercase();

 if ua.contains("windows") && !tcp_os.contains("windows") && tcp_os.contains("linux") {
 report.add_discrepancy(
 "TCP stackidentify as Linux，but HTTP User-Agent 声称 is Windows".to_string(),
 50,
 );
 }

 if ua.contains("iphone")
 && !tcp_os.contains("apple")
 && !tcp_os.contains("ios")
 && tcp_os.contains("linux")
 {
 report.add_discrepancy(
 "User-Agent as iPhone，but TCP trait更close to Linux (may is Android or 爬虫库)"
.to_string(),
 30,
 );
 }

 // Check TTL and OS whethermatch
 if ua.contains("windows") && tcp_os.contains("linux") {
 // mayuse了proxy or fingerprint混淆不completely
 }
 }

 // 2. Validate TLS and HTTP (browserlevelconsistency)
 if let (Some(tls), Some(http)) = (tls_fingerprints.first(), http_fingerprints.first()) {
 let tls_info = tls.to_string().to_lowercase();
 let ua = http.to_string().to_lowercase();

 // Check Chrome trait
 if ua.contains("chrome") {
 // If is modern Chrome (120+), mustsupport TLS 1.3
 if (ua.contains("chrome/1") || ua.contains("chrome/12") || ua.contains("chrome/13"))
 && !tls_info.contains("version: some(0x0304)")
 {
 report.add_discrepancy(
 "modern Chrome (120+) mustuse TLS 1.3，detect to protocol降level".to_string(),
 50,
 );
 }

 // Check ALPN conflict
 if ua.contains("h2") && !tls_info.contains("h2") && tls_info.contains("alpn") {
 report.add_discrepancy(
 "HTTP/2 requestfromnot in TLS handshake in negotiate h2 connection".to_string(),
 60,
 );
 }
 }
 }

 // 3. Validateprotocol降levelabnormal
 if flow.context.protocol == fingerprint_core::system::ProtocolType::Http
 && (flow.context.target_port == Some(443))
 {
 report.add_discrepancy(
 " in 443 portdetect to 明文 HTTP traffic (may is 强制protocol降levelattack)".to_string(),
 50,
 );
 }

 // 4. JA4+ series交叉Validate (更深layerfingerprintconsistency)
 if let (Some(tls), Some(http)) = (tls_fingerprints.first(), http_fingerprints.first()) {
 if let (Some(ja4), Some(ja4h)) =
 (tls.metadata().get("ja4"), http.metadata().get("ja4h"))
 {
 // if JA4 display is modern Chrome (t13d...), but JA4H display is HTTP/1.1 (..11..) 且no Cookie (..n..)
 // this isancommon爬虫trait
 if ja4.starts_with("t13") && ja4h.contains("11n") {
 report.add_discrepancy(
 format!("detect to modern TLS trait (JA4: {})，but HTTP behavior表现 as 传统无 Cookie request (JA4H: {})", ja4, ja4h),
 20,
 );
 }

 // Check ALPN consistency
 if ja4.contains("h2") && ja4h.contains("11") {
 // TLS negotiate了 h2，butactualsend了 HTTP/1.1
 report.add_discrepancy(
 "TLS handshakenegotiate了 h2，butactualrequestuse了 HTTP/1.1".to_string(),
 30,
 );
 }
 }
 }

 report
 }
}
