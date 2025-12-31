//! 指纹元数据
//!
//! 定义指纹的元数据，包括浏览器类型、操作系统、置信度等信息.

use crate::types::{BrowserType, OperatingSystem};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// 指纹元数据
///
/// 包含所有指纹类型共用的元数据信息
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FingerprintMetadata {
    /// 浏览器类型
    pub browser_type: Option<BrowserType>,

    /// 操作系统类型
    pub os_type: Option<OperatingSystem>,

    /// 置信度 (0.0 - 1.0)
    pub confidence: f64,

    /// 样本数量
    pub sample_count: u64,

    /// 首次发现时间
    pub first_seen: DateTime<Utc>,

    /// 最后发现时间
    pub last_seen: DateTime<Utc>,

    /// 标签
    pub tags: Vec<String>,

    /// 备注
    pub notes: Option<String>,
}

impl FingerprintMetadata {
    /// 创建新的元数据
    pub fn new() -> Self {
        let now = Utc::now();
        Self {
            browser_type: None,
            os_type: None,
            confidence: 0.5,
            sample_count: 1,
            first_seen: now,
            last_seen: now,
            tags: Vec::new(),
            notes: None,
        }
    }

    /// 创建带浏览器和操作系统的元数据
    pub fn with_browser_os(
        browser_type: Option<BrowserType>,
        os_type: Option<OperatingSystem>,
    ) -> Self {
        let mut metadata = Self::new();
        metadata.browser_type = browser_type;
        metadata.os_type = os_type;
        metadata
    }

    /// 更新样本（增加样本数，更新最后发现时间）
    pub fn update_sample(&mut self) {
        self.sample_count += 1;
        self.last_seen = Utc::now();
    }

    /// 更新置信度
    pub fn update_confidence(&mut self, confidence: f64) {
        self.confidence = confidence.clamp(0.0, 1.0);
    }

    /// 添加标签
    pub fn add_tag(&mut self, tag: String) {
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
        }
    }

    /// 移除标签
    pub fn remove_tag(&mut self, tag: &str) {
        self.tags.retain(|t| t != tag);
    }

    /// 检查是否包含标签
    pub fn has_tag(&self, tag: &str) -> bool {
        self.tags.contains(&tag.to_string())
    }

    /// 设置自定义属性（通过 tags 存储，格式为 "key:value"）
    pub fn set(&mut self, key: &str, value: &str) {
        let tag = format!("{}:{}", key, value);
        // 先移除旧的同名 key
        self.tags.retain(|t| !t.starts_with(&format!("{}:", key)));
        self.add_tag(tag);
    }

    /// 获取自定义属性（从 tags 中查找，格式为 "key:value"）
    pub fn get(&self, key: &str) -> Option<String> {
        let prefix = format!("{}:", key);
        self.tags
            .iter()
            .find(|t| t.starts_with(&prefix))
            .and_then(|t| t.strip_prefix(&prefix).map(|s| s.to_string()))
    }
}

impl Default for FingerprintMetadata {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{BrowserType, OperatingSystem};

    #[test]
    fn test_metadata_new() {
        let metadata = FingerprintMetadata::new();
        assert_eq!(metadata.confidence, 0.5);
        assert_eq!(metadata.sample_count, 1);
        assert!(metadata.browser_type.is_none());
        assert!(metadata.os_type.is_none());
    }

    #[test]
    fn test_metadata_with_browser_os() {
        let metadata = FingerprintMetadata::with_browser_os(
            Some(BrowserType::Chrome),
            Some(OperatingSystem::Windows10),
        );
        assert_eq!(metadata.browser_type, Some(BrowserType::Chrome));
        assert_eq!(metadata.os_type, Some(OperatingSystem::Windows10));
    }

    #[test]
    fn test_metadata_update_sample() {
        let mut metadata = FingerprintMetadata::new();
        let initial_count = metadata.sample_count;
        metadata.update_sample();
        assert_eq!(metadata.sample_count, initial_count + 1);
    }

    #[test]
    fn test_metadata_tags() {
        let mut metadata = FingerprintMetadata::new();
        metadata.add_tag("test".to_string());
        assert!(metadata.has_tag("test"));

        metadata.remove_tag("test");
        assert!(!metadata.has_tag("test"));
    }
}
