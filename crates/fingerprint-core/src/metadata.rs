//! fingerprintmetadata
//!
//! definefingerprintmetadata, includebrowsertype, operating system, confidence etc.info.

use crate::types::{BrowserType, OperatingSystem};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// fingerprintmetadata
///
/// includingallfingerprinttypesharedmetadatainfo
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FingerprintMetadata {
    /// browsertype
    pub browser_type: Option<BrowserType>,

    /// operating systemtype
    pub os_type: Option<OperatingSystem>,

    /// confidence (0.0 - 1.0)
    pub confidence: f64,

    /// samplecount
    pub sample_count: u64,

    /// 首timediscover when between
    pub first_seen: DateTime<Utc>,

    /// finallydiscover when between
    pub last_seen: DateTime<Utc>,

    /// tag
    pub tags: Vec<String>,

    /// 备note
    pub notes: Option<String>,
}

impl FingerprintMetadata {
    /// Create a newmetadata
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

    /// Createbringbrowser and operating systemmetadata
    pub fn with_browser_os(
        browser_type: Option<BrowserType>,
        os_type: Option<OperatingSystem>,
    ) -> Self {
        let mut metadata = Self::new();
        metadata.browser_type = browser_type;
        metadata.os_type = os_type;
        metadata
    }

    /// Updatesample (increasesamplecount, Updatefinallydiscover when between)
    pub fn update_sample(&mut self) {
        self.sample_count += 1;
        self.last_seen = Utc::now();
    }

    /// Updateconfidence
    pub fn update_confidence(&mut self, confidence: f64) {
        self.confidence = confidence.clamp(0.0, 1.0);
    }

    /// Addtag
    pub fn add_tag(&mut self, tag: String) {
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
        }
    }

    /// removetag
    pub fn remove_tag(&mut self, tag: &str) {
        self.tags.retain(|t| t != tag);
    }

    /// Checkwhetherincludingtag
    pub fn has_tag(&self, tag: &str) -> bool {
        self.tags.contains(&tag.to_string())
    }

    /// settingscustomproperty (through tags store, format as "key:value")
    pub fn set(&mut self, key: &str, value: &str) {
        let tag = format!("{}:{}", key, value);
        // 先removeold same name key
        self.tags.retain(|t| !t.starts_with(&format!("{}:", key)));
        self.add_tag(tag);
    }

    /// Getcustomproperty ( from tags in find, format as "key:value")
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
