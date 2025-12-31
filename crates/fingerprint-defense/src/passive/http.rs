//! HTTP 被动指纹识别
//!
//! 实现 HTTP 请求/响应的被动指纹识别。

use crate::passive::packet::Packet;
use std::collections::HashMap;

/// HTTP 分析器
pub struct HttpAnalyzer {
    /// 签名数据库
    #[allow(dead_code)]
    signatures: HashMap<String, HttpSignature>,
}

use serde::{Deserialize, Serialize};

/// HTTP 指纹
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpFingerprint {
    /// 匹配的签名
    pub signature: Option<HttpSignature>,

    /// 相似度
    pub similarity: f64,

    /// User-Agent
    pub user_agent: Option<String>,

    /// 检测到的浏览器
    pub browser: Option<String>,

    /// HTTP/2 特有特征
    pub h2_settings: Option<Vec<(u16, u32)>>,
    pub h2_window_update: Option<u32>,

    /// 指纹元数据
    pub metadata: fingerprint_core::metadata::FingerprintMetadata,
}

impl fingerprint_core::fingerprint::Fingerprint for HttpFingerprint {
    fn fingerprint_type(&self) -> fingerprint_core::fingerprint::FingerprintType {
        fingerprint_core::fingerprint::FingerprintType::Http
    }

    fn id(&self) -> String {
        self.user_agent
            .clone()
            .unwrap_or_else(|| "unknown".to_string())
    }

    fn metadata(&self) -> &fingerprint_core::metadata::FingerprintMetadata {
        &self.metadata
    }

    fn metadata_mut(&mut self) -> &mut fingerprint_core::metadata::FingerprintMetadata {
        &mut self.metadata
    }

    fn hash(&self) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        self.id().hash(&mut hasher);
        hasher.finish()
    }

    fn similar_to(&self, other: &dyn fingerprint_core::fingerprint::Fingerprint) -> bool {
        if other.fingerprint_type() != fingerprint_core::fingerprint::FingerprintType::Http {
            return false;
        }
        self.id() == other.id()
    }

    fn to_string(&self) -> String {
        format!("HTTP Fingerprint (UA: {:?})", self.user_agent)
    }
}

/// HTTP 签名（用于匹配）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpSignature {
    pub id: String,
    pub user_agent: String,
    pub headers: HashMap<String, String>,
    pub browser_type: Option<String>,
    pub confidence: f64,
    pub sample_count: u64,
}

/// HTTP 签名（用于 p0f 数据库）
#[derive(Debug, Clone)]
pub struct P0fHttpSignature {
    pub id: String,
    pub label: String,
    pub user_agent_pattern: Option<String>,
    pub headers: Vec<String>,
}

impl HttpAnalyzer {
    /// 创建新的 HTTP 分析器
    pub fn new() -> Result<Self, String> {
        Ok(Self {
            signatures: HashMap::new(),
        })
    }

    /// 分析 HTTP 数据包
    pub fn analyze(&self, packet: &Packet) -> Option<HttpFingerprint> {
        // 尝试解析 HTTP 请求
        if let Some(request) = self.parse_http_request(&packet.payload) {
            let user_agent = request.headers.get("user-agent").cloned();
            let browser = user_agent.as_ref().and_then(|ua| self.detect_browser(ua));

            // 匹配签名
            let (signature, similarity) = self.match_signature(&request);

            let mut metadata = fingerprint_core::metadata::FingerprintMetadata::new();

            // 计算 JA4H
            let header_tuples: Vec<(&str, &str)> = request
                .headers
                .iter()
                .map(|(k, v)| (k.as_str(), v.as_str()))
                .collect();
            let ja4h_string = fingerprint_core::ja4::JA4H::generate(
                &request.method,
                &request.version,
                request.cookie_count > 0,
                request.has_referer,
                &header_tuples,
            );
            metadata.add_tag(format!("ja4h:{}", ja4h_string));

            return Some(HttpFingerprint {
                signature: signature.clone(),
                similarity,
                user_agent,
                browser,
                h2_settings: request.h2_settings,
                h2_window_update: request.h2_window_update,
                metadata,
            });
        }

        None
    }

    /// 解析 HTTP 请求
    fn parse_http_request(&self, data: &[u8]) -> Option<HttpRequest> {
        // 简单的 HTTP 请求解析
        let text = String::from_utf8_lossy(data);
        let lines: Vec<&str> = text.lines().collect();

        if lines.is_empty() {
            return None;
        }

        let request_line = lines[0];
        let parts: Vec<&str> = request_line.split_whitespace().collect();
        if parts.len() < 3 {
            return None;
        }

        let method = parts[0].to_string();
        let path = parts[1].to_string();
        let version = parts[2].to_string();

        if !["GET", "POST", "PUT", "DELETE", "HEAD", "OPTIONS", "PATCH"].contains(&method.as_str())
        {
            // 检查是否是 H2 Connection Preface
            // PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n (24 bytes)
            if method == "PRI" && path == "*" && version.contains("HTTP/2") {
                let preface_end = 24;
                if data.len() > preface_end {
                    return self.parse_h2_request(&data[preface_end..]);
                }
            }
            return None;
        }

        let mut headers = HashMap::new();
        let mut header_names = Vec::new();
        let mut cookie_count = 0;
        let mut has_referer = false;

        for line in lines.iter().skip(1) {
            if line.is_empty() {
                break;
            }
            if let Some((key, value)) = line.split_once(':') {
                let raw_key = key.trim();
                let lower_key = raw_key.to_lowercase();
                let value = value.trim().to_string();

                header_names.push(raw_key.to_string());
                headers.insert(lower_key.clone(), value);

                if lower_key == "cookie" {
                    cookie_count += 1;
                }
                if lower_key == "referer" {
                    has_referer = true;
                }
            }
        }

        Some(HttpRequest {
            method,
            path,
            version,
            headers,
            header_names,
            cookie_count,
            has_referer,
            h2_settings: None,
            h2_window_update: None,
        })
    }

    /// 解析 H2 请求帧
    fn parse_h2_request(&self, data: &[u8]) -> Option<HttpRequest> {
        let mut offset = 0;
        let mut h2_settings = Vec::new();
        let mut h2_window_update = None;

        while offset + 9 <= data.len() {
            let length = ((data[offset] as u32) << 16)
                | ((data[offset + 1] as u32) << 8)
                | (data[offset + 2] as u32);
            let frame_type = data[offset + 3];
            let _flags = data[offset + 4];
            let stream_id = u32::from_be_bytes([
                data[offset + 5],
                data[offset + 6],
                data[offset + 7],
                data[offset + 8],
            ]) & 0x7FFFFFFF;

            offset += 9;
            let payload_end = offset + length as usize;
            if payload_end > data.len() {
                break;
            }

            match frame_type {
                0x04 => {
                    // SETTINGS
                    let mut s_offset = offset;
                    while s_offset + 6 <= payload_end {
                        let id = u16::from_be_bytes([data[s_offset], data[s_offset + 1]]);
                        let value = u32::from_be_bytes([
                            data[s_offset + 2],
                            data[s_offset + 3],
                            data[s_offset + 4],
                            data[s_offset + 5],
                        ]);
                        h2_settings.push((id, value));
                        s_offset += 6;
                    }
                }
                0x08 => {
                    // WINDOW_UPDATE
                    if length == 4 && offset + 4 <= payload_end {
                        let increment = u32::from_be_bytes([
                            data[offset],
                            data[offset + 1],
                            data[offset + 2],
                            data[offset + 3],
                        ]) & 0x7FFFFFFF;
                        if stream_id == 0 {
                            h2_window_update = Some(increment);
                        }
                    }
                }
                _ => {}
            }
            offset = payload_end;
        }

        if h2_settings.is_empty() && h2_window_update.is_none() {
            return None;
        }

        Some(HttpRequest {
            method: "PRI".to_string(),
            path: "*".to_string(),
            version: "HTTP/2".to_string(),
            headers: HashMap::new(),
            header_names: Vec::new(),
            cookie_count: 0,
            has_referer: false,
            h2_settings: Some(h2_settings),
            h2_window_update,
        })
    }

    /// 检测浏览器
    fn detect_browser(&self, user_agent: &str) -> Option<String> {
        let ua_lower = user_agent.to_lowercase();

        if ua_lower.contains("chrome") && !ua_lower.contains("edg") {
            Some("Chrome".to_string())
        } else if ua_lower.contains("firefox") {
            Some("Firefox".to_string())
        } else if ua_lower.contains("safari") && !ua_lower.contains("chrome") {
            Some("Safari".to_string())
        } else if ua_lower.contains("edg") {
            Some("Edge".to_string())
        } else if ua_lower.contains("opera") {
            Some("Opera".to_string())
        } else {
            None
        }
    }

    /// 匹配签名
    fn match_signature(&self, _request: &HttpRequest) -> (Option<HttpSignature>, f64) {
        // TODO: 实现签名匹配
        (None, 0.0)
    }
}

/// HTTP 请求
#[derive(Debug, Clone)]
struct HttpRequest {
    method: String,
    #[allow(dead_code)]
    path: String,
    version: String,
    headers: HashMap<String, String>,
    #[allow(dead_code)]
    header_names: Vec<String>,
    cookie_count: usize,
    has_referer: bool,
    h2_settings: Option<Vec<(u16, u32)>>,
    h2_window_update: Option<u32>,
}
