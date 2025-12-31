//! TCP 被动指纹识别
//!
//! 实现 p0f 风格的 TCP 指纹识别。

use crate::passive::packet::{Packet, TcpHeader};
use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// TCP 签名（简化版，用于匹配）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TcpSignature {
    pub id: String,
    pub ttl: u8,
    pub window_size: u16,
    pub mss: Option<u16>,
    pub window_scale: Option<u8>,
    pub os_type: Option<String>,
    pub confidence: f64,
    pub sample_count: u64,
}

/// TCP 分析器
pub struct TcpAnalyzer {
    /// 签名数据库
    signatures: HashMap<String, TcpSignature>,
}

/// TCP 指纹
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TcpFingerprint {
    /// 匹配的签名
    pub signature: Option<TcpSignature>,

    /// 相似度
    pub similarity: f64,

    /// 检测到的操作系统
    pub os: Option<String>,

    /// 原始特征
    pub features: TcpFeatures,

    /// 指纹元数据
    pub metadata: fingerprint_core::metadata::FingerprintMetadata,
}

impl fingerprint_core::fingerprint::Fingerprint for TcpFingerprint {
    fn fingerprint_type(&self) -> fingerprint_core::fingerprint::FingerprintType {
        fingerprint_core::fingerprint::FingerprintType::Tcp
    }

    fn id(&self) -> String {
        self.signature
            .as_ref()
            .map(|s| s.id.clone())
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
        if other.fingerprint_type() != fingerprint_core::fingerprint::FingerprintType::Tcp {
            return false;
        }
        self.id() == other.id()
    }

    fn to_string(&self) -> String {
        format!(
            "TCP Fingerprint (ID: {}, Similarity: {:.2})",
            self.id(),
            self.similarity
        )
    }
}

/// TCP 特征
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TcpFeatures {
    /// TTL
    pub ttl: u8,

    /// 初始 TTL（推断）
    pub initial_ttl: u8,

    /// 窗口大小
    pub window: u16,

    /// MSS
    pub mss: Option<u16>,

    /// Window Scale
    pub window_scale: Option<u8>,

    /// TCP 选项字符串
    pub options_str: String,

    /// IP 标志
    pub ip_flags: u8,
}

impl TcpAnalyzer {
    /// 创建新的 TCP 分析器
    pub fn new() -> Result<Self, String> {
        let mut analyzer = Self {
            signatures: HashMap::new(),
        };

        // 加载默认签名
        analyzer.load_default_signatures()?;

        Ok(analyzer)
    }

    /// 从 p0f 数据库加载签名
    pub fn load_from_p0f<P: AsRef<std::path::Path>>(&mut self, path: P) -> Result<(), String> {
        use crate::passive::p0f::P0fDatabase;

        let db = P0fDatabase::load_from_file(path)
            .map_err(|e| format!("Failed to load p0f database: {}", e))?;

        // 加载所有 TCP 请求签名
        for sig in db.get_all_tcp_request() {
            self.signatures.insert(sig.id.clone(), sig.clone());
        }

        Ok(())
    }

    /// 加载默认签名
    fn load_default_signatures(&mut self) -> Result<(), String> {
        // 添加一些基础签名作为示例
        // 这些是常见的操作系统签名

        // Linux 示例签名
        let linux_sig = TcpSignature {
            id: "linux-generic".to_string(),
            ttl: 64,
            window_size: 0,
            mss: Some(1460),
            window_scale: Some(7),
            os_type: Some("Linux".to_string()),
            confidence: 0.8,
            sample_count: 1000,
        };
        self.signatures.insert(linux_sig.id.clone(), linux_sig);

        // Windows 10 示例签名
        let win10_sig = TcpSignature {
            id: "windows-10".to_string(),
            ttl: 128,
            window_size: 64240,
            mss: Some(1460),
            window_scale: Some(8),
            os_type: Some("Windows".to_string()),
            confidence: 0.85,
            sample_count: 1000,
        };
        self.signatures.insert(win10_sig.id.clone(), win10_sig);

        // macOS 示例签名
        let macos_sig = TcpSignature {
            id: "macos-generic".to_string(),
            ttl: 64,
            window_size: 65535,
            mss: Some(1460),
            window_scale: Some(6),
            os_type: Some("macOS".to_string()),
            confidence: 0.8,
            sample_count: 1000,
        };
        self.signatures.insert(macos_sig.id.clone(), macos_sig);

        Ok(())
    }

    /// 分析 TCP 数据包
    pub fn analyze(&self, packet: &Packet) -> Option<TcpFingerprint> {
        let tcp_header = packet.tcp_header.as_ref()?;

        // 提取 TCP 特征
        let features = self.extract_features(packet, tcp_header);

        // 匹配签名
        let (signature, similarity) = self.match_signature(&features);

        let mut metadata = fingerprint_core::metadata::FingerprintMetadata::new();

        // 计算 JA4T
        let ja4t = fingerprint_core::ja4::JA4T::generate(
            features.window,
            &features.options_str,
            features.mss.unwrap_or(0),
            features.ttl,
        );
        metadata.add_tag(format!("ja4t:{}", ja4t));

        Some(TcpFingerprint {
            signature: signature.clone(),
            similarity,
            os: signature.as_ref().and_then(|s| s.os_type.clone()),
            features,
            metadata,
        })
    }

    /// 提取 TCP 特征
    fn extract_features(&self, packet: &Packet, tcp_header: &TcpHeader) -> TcpFeatures {
        // 推断初始 TTL
        let initial_ttl = self.infer_initial_ttl(packet.ttl);

        // 提取 MSS
        let mss = self.extract_mss(&tcp_header.options);

        // 提取 Window Scale
        let window_scale = self.extract_window_scale(&tcp_header.options);

        // 生成选项字符串
        let options_str = self.build_options_string(&tcp_header.options);

        TcpFeatures {
            ttl: packet.ttl,
            initial_ttl,
            window: tcp_header.window,
            mss,
            window_scale,
            options_str,
            ip_flags: packet.ip_flags,
        }
    }

    /// 推断初始 TTL
    fn infer_initial_ttl(&self, ttl: u8) -> u8 {
        // 根据观察到的 TTL 推断初始 TTL
        // 常见的初始 TTL 值：32, 64, 128, 255
        if ttl <= 32 {
            32
        } else if ttl <= 64 {
            64
        } else if ttl <= 128 {
            128
        } else {
            255
        }
    }

    /// 提取 MSS
    fn extract_mss(&self, options: &[crate::passive::packet::TcpOption]) -> Option<u16> {
        for opt in options {
            if opt.kind == 2 && opt.data.len() >= 2 {
                // MSS option
                return Some(u16::from_be_bytes([opt.data[0], opt.data[1]]));
            }
        }
        None
    }

    /// 提取 Window Scale
    fn extract_window_scale(&self, options: &[crate::passive::packet::TcpOption]) -> Option<u8> {
        for opt in options {
            if opt.kind == 3 && !opt.data.is_empty() {
                // Window Scale option
                return Some(opt.data[0]);
            }
        }
        None
    }

    /// 构建选项字符串
    fn build_options_string(&self, options: &[crate::passive::packet::TcpOption]) -> String {
        let mut parts = Vec::new();

        for opt in options {
            match opt.kind {
                0 => break,                          // End of options
                1 => parts.push("nop".to_string()),  // NOP
                2 => parts.push("mss".to_string()),  // MSS
                3 => parts.push("ws".to_string()),   // Window Scale
                4 => parts.push("sack".to_string()), // SACK permitted
                8 => parts.push("ts".to_string()),   // Timestamp
                _ => parts.push(format!("opt{}", opt.kind)),
            }
        }

        parts.join(",")
    }

    /// 匹配签名
    fn match_signature(&self, features: &TcpFeatures) -> (Option<TcpSignature>, f64) {
        let mut best_match: Option<(&TcpSignature, f64)> = None;

        for sig in self.signatures.values() {
            let similarity = self.calculate_similarity(features, sig);

            if let Some((_, best_sim)) = best_match {
                if similarity > best_sim {
                    best_match = Some((sig, similarity));
                }
            } else {
                best_match = Some((sig, similarity));
            }
        }

        if let Some((sig, sim)) = best_match {
            if sim > 0.6 {
                // 相似度阈值
                (Some(sig.clone()), sim)
            } else {
                (None, sim)
            }
        } else {
            (None, 0.0)
        }
    }

    /// 计算相似度
    fn calculate_similarity(&self, features: &TcpFeatures, signature: &TcpSignature) -> f64 {
        let mut score = 0.0;
        let mut total = 0.0;

        // TTL 匹配
        if signature.ttl > 0 {
            total += 1.0;
            let ttl_diff = (features.initial_ttl as i16 - signature.ttl as i16).abs();
            if ttl_diff == 0 {
                score += 1.0;
            } else if ttl_diff <= 1 {
                score += 0.8;
            } else {
                score += 0.5;
            }
        }

        // Window Size 匹配（简化）
        if signature.window_size > 0 {
            total += 1.0;
            let window_diff = (features.window as i32 - signature.window_size as i32).abs();
            if window_diff < 100 {
                score += 1.0;
            } else if window_diff < 1000 {
                score += 0.7;
            } else {
                score += 0.3;
            }
        }

        // MSS 匹配
        if let Some(sig_mss) = signature.mss {
            total += 1.0;
            if let Some(feat_mss) = features.mss {
                if sig_mss == feat_mss {
                    score += 1.0;
                } else {
                    let mss_diff = (sig_mss as i16 - feat_mss as i16).abs();
                    if mss_diff < 10 {
                        score += 0.8;
                    } else {
                        score += 0.4;
                    }
                }
            }
        }

        // Window Scale 匹配
        if let Some(sig_ws) = signature.window_scale {
            total += 1.0;
            if let Some(feat_ws) = features.window_scale {
                if sig_ws == feat_ws {
                    score += 1.0;
                } else {
                    score += 0.5;
                }
            }
        }

        if total > 0.0 {
            score / total
        } else {
            0.0
        }
    }
}
