#![allow(clippy::all, dead_code, unused_variables, unused_parens)]

//! # fingerprint-webrtc
//!
// ! WebRTC leakprotectionmodule
//!
// ! provide WebRTC IP leakprotectionandfingerprintrecognitioncapabilities

use std::collections::HashSet;
use std::net::IpAddr;

// / WebRTC fingerprint
#[derive(Debug, Clone)]
pub struct WebRTCFingerprint {
    // / local IP candidatesaddress
    pub local_candidates: Vec<String>,
    // / remote IP address
    pub remote_ip: Option<String>,
    // / connectstate
    pub connection_state: ConnectionState,
    // / mDNS candidateshide
    pub mdns_hidden: bool,
    // / candidatesfilterstatistics
    pub candidate_stats: CandidateStats,
}

// / connectstate
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    // / new
    New,
    // / connect中
    Connecting,
    // / 已connect
    Connected,
    // / completed
    Completed,
    // / disconnectconnect
    Disconnected,
    // / failure
    Failed,
    // / closed
    Closed,
}

// / candidatesstatisticsinfo
#[derive(Debug, Clone)]
pub struct CandidateStats {
    // / hostcandidates
    pub host_candidates: u32,
    // / srflx candidates
    pub srflx_candidates: u32,
    // / prflx candidates
    pub prflx_candidates: u32,
    // / relay candidates
    pub relay_candidates: u32,
}

// / WebRTC errortype
#[derive(Debug)]
pub enum WebRTCError {
    // / invalid IP
    InvalidIP,
    // / analyzefailure
    AnalysisFailed(String),
    // / othererror
    Other(String),
}

impl std::fmt::Display for WebRTCError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WebRTCError::InvalidIP => write!(f, "Invalid IP address"),
            WebRTCError::AnalysisFailed(msg) => write!(f, "Analysis failed: {}", msg),
            WebRTCError::Other(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl std::error::Error for WebRTCError {}

// / WebRTC analyzer
pub struct WebRTCAnalyzer;

impl WebRTCAnalyzer {
    // / analyze WebRTC candidates
    pub fn analyze(candidates: &[&str]) -> Result<WebRTCFingerprint, WebRTCError> {
        let mut stats = CandidateStats {
            host_candidates: 0,
            srflx_candidates: 0,
            prflx_candidates: 0,
            relay_candidates: 0,
        };

        let mut local_candidates = Vec::new();
        let mut remote_ip = None;

        for candidate in candidates {
            if candidate.contains("host") {
                stats.host_candidates += 1;
                if let Some(ip) = Self::extract_ip(candidate) {
                    local_candidates.push(ip);
                }
            } else if candidate.contains("srflx") {
                stats.srflx_candidates += 1;
                if let Some(ip) = Self::extract_ip(candidate) {
                    remote_ip = Some(ip);
                }
            } else if candidate.contains("prflx") {
                stats.prflx_candidates += 1;
            } else if candidate.contains("relay") {
                stats.relay_candidates += 1;
            }
        }

        Ok(WebRTCFingerprint {
            local_candidates,
            remote_ip,
            connection_state: ConnectionState::New,
            mdns_hidden: false,
            candidate_stats: stats,
        })
    }

    // / extract IP address
    fn extract_ip(candidate: &str) -> Option<String> {
        // 简单of IP extract逻辑
        let parts: Vec<&str> = candidate.split_whitespace().collect();
        for part in parts {
            if Self::is_valid_ip(part) {
                return Some(part.to_string());
            }
        }
        None
    }

    // / validate IP address
    fn is_valid_ip(s: &str) -> bool {
        s.parse::<IpAddr>().is_ok()
    }
}

// / WebRTC protector
pub struct WebRTCProtection;

impl WebRTCProtection {
    // / hide mDNS candidates
    pub fn hide_mdns_candidates(candidates: &[&str]) -> Vec<String> {
        candidates
            .iter()
            .filter(|c| !c.contains(".local"))
            .map(|s| s.to_string())
            .collect()
    }

    // / forge IP address
    pub fn spoof_ip(fingerprint: &WebRTCFingerprint, fake_ip: &str) -> WebRTCFingerprint {
        let mut spoofed = fingerprint.clone();
        spoofed.remote_ip = Some(fake_ip.to_string());
        spoofed
    }

    // / detect WebRTC leak
    pub fn detect_leaks(candidates: &[&str]) -> WebRTCLeakReport {
        let private_ips = HashSet::from(["10.0.0.0", "172.16.0.0", "192.168.0.0", "127.0.0.1"]);

        let mut leaked_ips = Vec::new();

        for candidate in candidates {
            if let Some(ip) = WebRTCAnalyzer::extract_ip(candidate) {
                for private in &private_ips {
                    if ip.starts_with(private.split('.').next().unwrap_or("")) {
                        leaked_ips.push(ip);
                        break;
                    }
                }
            }
        }

        WebRTCLeakReport {
            has_leaks: !leaked_ips.is_empty(),
            leaked_ips,
        }
    }
}

// / WebRTC leakreport
#[derive(Debug, Clone)]
pub struct WebRTCLeakReport {
    pub has_leaks: bool,
    pub leaked_ips: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_webrtc_analysis() {
        let candidates = vec!["candidate:1 1 udp 192.168.1.1 50000"];
        let result = WebRTCAnalyzer::analyze(&candidates);
        assert!(result.is_ok());
    }

    #[test]
    fn test_mdns_hiding() {
        // mDNSaddressending with..local（(RFC 6762)
        let candidates = vec![
            "candidate:1 1 udp 192.168.1.1",
            "candidate:2 1 udp device.local", // 真正ofmDNS格式
        ];
        let filtered = WebRTCProtection::hide_mdns_candidates(&candidates);
        assert_eq!(filtered.len(), 1); // 只有第一个非mDNScandidates保留
        assert_eq!(filtered[0], "candidate:1 1 udp 192.168.1.1");
    }
}
