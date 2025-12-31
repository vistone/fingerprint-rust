//! 指纹自学习模块
//!
//! 自动从观察到的流量中学习和更新指纹签名。

use crate::database::FingerprintDatabase;
use crate::passive::PassiveAnalysisResult;
use dashmap::DashMap;
use std::sync::Arc;

use fingerprint_core::fingerprint::Fingerprint;

/// 自学习分析器
pub struct SelfLearningAnalyzer {
    #[allow(dead_code)] // 将来会用于存储学习到的指纹
    db: Arc<FingerprintDatabase>,
    /// 未知指纹的观察计数器 (fp_id -> count)
    observations: DashMap<String, u64>,
    /// 学习阈值（观察多少次后转入数据库）
    learning_threshold: u64,
}

impl SelfLearningAnalyzer {
    /// 创建新的学习分析器
    pub fn new(db: Arc<FingerprintDatabase>) -> Self {
        Self {
            db,
            observations: DashMap::new(),
            learning_threshold: 10,
        }
    }

    /// 处理分析结果并学习
    pub fn process_result(&self, result: &PassiveAnalysisResult) {
        // 分别处理各个层级的指纹
        if let Some(tls) = &result.tls {
            // TLS 目前直接观察 ID (JA4)
            self.observe(tls.id(), "tls");
        }

        if let Some(http) = &result.http {
            if http.signature.is_none() {
                self.observe(http.id(), "http");
            }
        }

        if let Some(tcp) = &result.tcp {
            if tcp.signature.is_none() {
                self.observe(tcp.id(), "tcp");
            }
        }
    }

    /// 观察到一个指纹
    fn observe(&self, fp_id: String, fp_type: &str) {
        if fp_id == "unknown" || fp_id.is_empty() {
            return;
        }

        let key = format!("{}:{}", fp_type, fp_id);

        // 防护点：限制观察列表的大小，防止内存撑爆 (DoS 防护)
        const MAX_OBSERVATIONS: usize = 10000;
        if self.observations.len() >= MAX_OBSERVATIONS && !self.observations.contains_key(&key) {
            // 如果达到上限且是新 key，则忽略
            return;
        }

        let mut count = self.observations.entry(key.clone()).or_insert(0);
        *count += 1;

        if *count >= self.learning_threshold {
            // 达到阈值，可以在数据库中建立初步条目
            // TODO: 提取特征并存储为待核准的签名
            println!("[Learner] Detected stable unknown fingerprint: {}", key);
        }
    }

    /// 设置学习阈值
    pub fn set_threshold(&mut self, threshold: u64) {
        self.learning_threshold = threshold;
    }
}
