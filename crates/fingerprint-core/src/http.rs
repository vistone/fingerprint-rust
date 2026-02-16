//! HTTP fingerprintcoretype
//!
//! define HTTP fingerprintcorecountdatastruct.

use crate::fingerprint::{Fingerprint, FingerprintType};
use crate::metadata::FingerprintMetadata;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

/// HTTP fingerprint
#[derive(Debug, Clone)]
pub struct HttpFingerprint {
    /// fingerprint ID (based on User-Agent and Headers hash)
    pub id: String,

    /// User-Agent
    pub user_agent: String,

    /// HTTP header
    pub headers: HashMap<String, String>,

    /// HTTP/2 settings ( if 有)
    pub http2_settings: Option<Http2Settings>,

    /// metadata
    pub metadata: FingerprintMetadata,
}

/// HTTP/2 settings
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Http2Settings {
    /// Header Table Size
    pub header_table_size: u32,

    /// Enable Push
    pub enable_push: bool,

    /// Max Concurrent Streams
    pub max_concurrent_streams: u32,

    /// Initial Window Size
    pub initial_window_size: u32,

    /// Max Frame Size
    pub max_frame_size: u32,

    /// Max Header List Size
    pub max_header_list_size: u32,
}

impl HttpFingerprint {
    /// Create a new HTTP fingerprint
    pub fn new(user_agent: String, headers: HashMap<String, String>) -> Self {
        let id = Self::calculate_id(&user_agent, &headers);
        Self {
            id,
            user_agent,
            headers,
            http2_settings: None,
            metadata: FingerprintMetadata::new(),
        }
    }

    /// Calculatefingerprint ID
    fn calculate_id(user_agent: &str, headers: &HashMap<String, String>) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(user_agent.as_bytes());

        // pair headers performsortbackhash
        let mut header_vec: Vec<_> = headers.iter().collect();
        header_vec.sort_by_key(|(k, _)| *k);
        for (k, v) in header_vec {
            hasher.update(k.as_bytes());
            hasher.update(v.as_bytes());
        }

        format!("{:x}", hasher.finalize())
    }

    /// settings HTTP/2 settings
    pub fn with_http2_settings(mut self, settings: Http2Settings) -> Self {
        self.http2_settings = Some(settings);
        // reCalculate ID (including HTTP/2 settings)
        self.id = Self::calculate_id(&self.user_agent, &self.headers);
        self
    }
}

impl Fingerprint for HttpFingerprint {
    fn fingerprint_type(&self) -> FingerprintType {
        FingerprintType::Http
    }

    fn id(&self) -> String {
        self.id.clone()
    }

    fn metadata(&self) -> &FingerprintMetadata {
        &self.metadata
    }

    fn metadata_mut(&mut self) -> &mut FingerprintMetadata {
        &mut self.metadata
    }

    fn hash(&self) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        let mut hasher = DefaultHasher::new();
        self.user_agent.hash(&mut hasher);

        // pair headers performsortbackhash
        let mut header_vec: Vec<_> = self.headers.iter().collect();
        header_vec.sort_by_key(|(k, _)| *k);
        for (k, v) in header_vec {
            k.hash(&mut hasher);
            v.hash(&mut hasher);
        }

        if let Some(ref settings) = self.http2_settings {
            settings.header_table_size.hash(&mut hasher);
            settings.enable_push.hash(&mut hasher);
            settings.max_concurrent_streams.hash(&mut hasher);
            settings.initial_window_size.hash(&mut hasher);
            settings.max_frame_size.hash(&mut hasher);
            settings.max_header_list_size.hash(&mut hasher);
        }

        hasher.finish()
    }

    fn similar_to(&self, other: &dyn Fingerprint) -> bool {
        if other.fingerprint_type() != FingerprintType::Http {
            return false;
        }

        // tryconvert to HttpFingerprint
        // Note: hereneedtypeConvert, butdue to trait limit, wecan onlycomparehashvalue
        self.hash() == other.hash()
    }

    fn to_string(&self) -> String {
        format!("HttpFingerprint(id={}, ua={})", self.id, self.user_agent)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_http_fingerprint_new() {
        let mut headers = HashMap::new();
        headers.insert("Accept".to_string(), "text/html".to_string());
        let fp = HttpFingerprint::new("Mozilla/5.0".to_string(), headers);
        assert!(!fp.id.is_empty());
        assert_eq!(fp.user_agent, "Mozilla/5.0");
    }

    #[test]
    fn test_http_fingerprint_hash() {
        let mut headers1 = HashMap::new();
        headers1.insert("Accept".to_string(), "text/html".to_string());
        let fp1 = HttpFingerprint::new("Mozilla/5.0".to_string(), headers1);

        let mut headers2 = HashMap::new();
        headers2.insert("Accept".to_string(), "text/html".to_string());
        let fp2 = HttpFingerprint::new("Mozilla/5.0".to_string(), headers2);

        // sameinputshouldproducesameofhash
        assert_eq!(fp1.hash(), fp2.hash());
    }
}
