use std::collections::HashMap;

/// Storage analyzer for detecting fingerprinting attempts
/// 存储分析器用于检测指纹识别尝试
pub struct StorageAnalyzer {
    /// localStorage访问统计
    local_storage_access: HashMap<String, u32>,
    /// sessionStorage访问统计
    session_storage_access: HashMap<String, u32>,
    /// cookie访问模式
    cookie_patterns: Vec<String>,
}

impl StorageAnalyzer {
    /// Create a new storage analyzer
    /// 创建新的存储分析器
    pub fn new() -> Self {
        Self {
            local_storage_access: HashMap::new(),
            session_storage_access: HashMap::new(),
            cookie_patterns: Vec::new(),
        }
    }

    /// Detect localStorage fingerprinting attempts
    /// 检测localStorage指纹识别尝试
    pub fn detect_local_storage_fingerprinting(&self) -> bool {
        // 检测异常的localStorage访问模式
        // 如果同一键被频繁访问，可能是指纹识别行为
        self.local_storage_access.values().any(|&count| count > 10) // 阈值：同一键访问超过10次
    }

    /// Detect sessionStorage fingerprinting attempts
    /// 检测sessionStorage指纹识别尝试
    pub fn detect_session_storage_fingerprinting(&self) -> bool {
        // 检测异常的sessionStorage访问模式
        self.session_storage_access.values().any(|&count| count > 5) // 阈值：同一键访问超过5次
    }

    /// Analyze cookie-based tracking
    /// 分析基于cookie的跟踪
    pub fn analyze_cookie_tracking(&self, cookies: &[&str]) -> Vec<String> {
        let mut suspicious_cookies = Vec::new();

        for cookie in cookies {
            // 检测可疑的cookie名称模式
            let cookie_lower = cookie.to_lowercase();
            if cookie_lower.contains("fingerprint")
                || cookie_lower.contains("track")
                || cookie_lower.contains("identify")
            {
                suspicious_cookies.push(cookie.to_string());
            }

            // 检测超长的cookie值（可能包含指纹数据）
            if cookie.len() > 1000 {
                suspicious_cookies.push(format!("{} (too long)", cookie));
            }
        }

        suspicious_cookies
    }

    /// Record localStorage access for analysis
    /// 记录localStorage访问用于分析
    pub fn record_local_storage_access(&mut self, key: &str) {
        *self
            .local_storage_access
            .entry(key.to_string())
            .or_insert(0) += 1;
    }

    /// Record sessionStorage access for analysis
    /// 记录sessionStorage访问用于分析
    pub fn record_session_storage_access(&mut self, key: &str) {
        *self
            .session_storage_access
            .entry(key.to_string())
            .or_insert(0) += 1;
    }

    /// Record cookie access patterns
    /// 记录cookie访问模式
    pub fn record_cookie_pattern(&mut self, pattern: &str) {
        self.cookie_patterns.push(pattern.to_string());
    }

    /// Get storage access statistics
    /// 获取存储访问统计信息
    pub fn get_statistics(&self) -> StorageStatistics {
        StorageStatistics {
            local_storage_keys: self.local_storage_access.len(),
            session_storage_keys: self.session_storage_access.len(),
            total_local_accesses: self.local_storage_access.values().sum(),
            total_session_accesses: self.session_storage_access.values().sum(),
            suspicious_cookie_count: self.cookie_patterns.len(),
        }
    }
}

/// Storage access statistics
/// 存储访问统计信息
#[derive(Debug, Clone)]
pub struct StorageStatistics {
    pub local_storage_keys: usize,
    pub session_storage_keys: usize,
    pub total_local_accesses: u32,
    pub total_session_accesses: u32,
    pub suspicious_cookie_count: usize,
}

impl Default for StorageAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_local_storage_detection() {
        let mut analyzer = StorageAnalyzer::new();

        // 正常访问不应该触发检测
        analyzer.record_local_storage_access("normal_key");
        analyzer.record_local_storage_access("normal_key");
        assert!(!analyzer.detect_local_storage_fingerprinting());

        // 频繁访问应该触发检测
        for _ in 0..15 {
            analyzer.record_local_storage_access("fingerprint_key");
        }
        assert!(analyzer.detect_local_storage_fingerprinting());
    }

    #[test]
    fn test_session_storage_detection() {
        let mut analyzer = StorageAnalyzer::new();

        // 正常访问不应该触发检测
        analyzer.record_session_storage_access("session_key");
        assert!(!analyzer.detect_session_storage_fingerprinting());

        // 频繁访问应该触发检测
        for _ in 0..10 {
            analyzer.record_session_storage_access("tracking_key");
        }
        assert!(analyzer.detect_session_storage_fingerprinting());
    }

    #[test]
    fn test_cookie_analysis() {
        let analyzer = StorageAnalyzer::new();

        let long_cookie = "x".repeat(1500); // 创建超长cookie
        let cookies = vec![
            "normal_cookie=value",
            "fingerprint_data=suspicious_value",
            "tracking_id=12345",
            &long_cookie, // 超长cookie
        ];

        let suspicious = analyzer.analyze_cookie_tracking(&cookies);
        assert!(!suspicious.is_empty());
        assert!(suspicious.iter().any(|c| c.contains("fingerprint")));
        assert!(suspicious.iter().any(|c| c.contains("too long")));
    }
}
