#![allow(clippy::all, dead_code, unused_variables, unused_parens)]

//! # fingerprint-storage
//!
// ! storefeaturesrecognitionmodule
//!
// ! provide LocalStorage/SessionStorage/IndexedDB fingerprintrecognitioncapabilities

use std::collections::HashMap;

// / storefingerprint
#[derive(Debug, Clone)]
pub struct StorageFingerprint {
    // / LocalStorage 键值对
    pub localstorage: HashMap<String, String>,
    // / SessionStorage 键值对
    pub sessionstorage: HashMap<String, String>,
    // / IndexedDB datalibrarylist
    pub indexeddb_databases: Vec<String>,
    // / Cookie list
    pub cookies: Vec<CookieInfo>,
    // / storeavailabilitycheck
    pub storage_available: StorageAvailability,
    // / storefingerprinthash
    pub storage_hash: String,
}

// / Cookie info
#[derive(Debug, Clone, PartialEq)]
pub struct CookieInfo {
    pub name: String,
    pub domain: String,
    pub path: String,
    pub secure: bool,
    pub http_only: bool,
}

// / storeavailability
#[derive(Debug, Clone)]
pub struct StorageAvailability {
    pub localstorage_available: bool,
    pub sessionstorage_available: bool,
    pub indexeddb_available: bool,
    pub cookies_available: bool,
}

// / storeerrortype
#[derive(Debug)]
pub enum StorageError {
    // / invaliddata
    InvalidData,
    // / analyzefailure
    AnalysisFailed(String),
    // / othererror
    Other(String),
}

impl std::fmt::Display for StorageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StorageError::InvalidData => write!(f, "Invalid storage data"),
            StorageError::AnalysisFailed(msg) => write!(f, "Analysis failed: {}", msg),
            StorageError::Other(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl std::error::Error for StorageError {}

// / storeanalyzer
pub struct StorageAnalyzer;

impl StorageAnalyzer {
    // / analyzestoredata
    pub fn analyze(
        localstorage: &HashMap<String, String>,
        sessionstorage: &HashMap<String, String>,
        indexeddb_dbs: &[&str],
        cookies: &[(&str, &str, &str)],
    ) -> Result<StorageFingerprint, StorageError> {
        // checkstoreavailability
        let storage_available = StorageAvailability {
            localstorage_available: !localstorage.is_empty(),
            sessionstorage_available: !sessionstorage.is_empty(),
            indexeddb_available: !indexeddb_dbs.is_empty(),
            cookies_available: !cookies.is_empty(),
        };

        // convert IndexedDB list
        let indexeddb_databases: Vec<String> =
            indexeddb_dbs.iter().map(|s| s.to_string()).collect();

        // convert Cookie
        let cookie_list: Vec<CookieInfo> = cookies
            .iter()
            .map(|(name, domain, path)| CookieInfo {
                name: name.to_string(),
                domain: domain.to_string(),
                path: path.to_string(),
                secure: domain.ends_with(".secure"),
                http_only: path.contains("httponly"),
            })
            .collect();

        // generatestorehash
        let storage_hash = Self::generate_storage_hash(
            localstorage,
            sessionstorage,
            &indexeddb_databases,
            &cookie_list,
        );

        Ok(StorageFingerprint {
            localstorage: localstorage.clone(),
            sessionstorage: sessionstorage.clone(),
            indexeddb_databases,
            cookies: cookie_list,
            storage_available,
            storage_hash,
        })
    }

    // / generatestorehash
    fn generate_storage_hash(
        localstorage: &HashMap<String, String>,
        sessionstorage: &HashMap<String, String>,
        indexeddb_dbs: &[String],
        cookies: &[CookieInfo],
    ) -> String {
        let mut hash_input = String::new();

        // 添加 LocalStorage data
        for (k, v) in localstorage.iter() {
            hash_input.push_str(&format!("{}:{};", k, v));
        }

        // 添加 SessionStorage data
        for (k, v) in sessionstorage.iter() {
            hash_input.push_str(&format!("{}:{};", k, v));
        }

        // 添加 IndexedDB
        for db in indexeddb_dbs {
            hash_input.push_str(&format!("{};", db));
        }

        // 添加 Cookie
        for cookie in cookies {
            hash_input.push_str(&format!("{}:{};", cookie.name, cookie.domain));
        }

        let hash_value = hash_input
            .chars()
            .fold(0u64, |acc, c| acc.wrapping_mul(31).wrapping_add(c as u64));
        format!("{:x}", hash_value)
    }

    // / detectstore更改
    pub fn detect_changes(
        before: &StorageFingerprint,
        after: &StorageFingerprint,
    ) -> StorageChanges {
        StorageChanges {
            localstorage_changed: before.localstorage != after.localstorage,
            sessionstorage_changed: before.sessionstorage != after.sessionstorage,
            indexeddb_changed: before.indexeddb_databases != after.indexeddb_databases,
            cookies_changed: before.cookies != after.cookies,
            hash_changed: before.storage_hash != after.storage_hash,
        }
    }
}

// / store更改info
#[derive(Debug, Clone)]
pub struct StorageChanges {
    pub localstorage_changed: bool,
    pub sessionstorage_changed: bool,
    pub indexeddb_changed: bool,
    pub cookies_changed: bool,
    pub hash_changed: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storage_analysis() {
        let mut ls = HashMap::new();
        ls.insert("key1".to_string(), "value1".to_string());

        let ss = HashMap::new();
        let dbs = vec!["mydb"];
        let cookies = vec![("session_id", "example.com", "/")];

        let result = StorageAnalyzer::analyze(&ls, &ss, &dbs, &cookies);
        assert!(result.is_ok());
        let fp = result.unwrap();
        assert_eq!(fp.localstorage.len(), 1);
        assert_eq!(fp.indexeddb_databases.len(), 1);
    }

    #[test]
    fn test_storage_changes_detection() {
        let mut ls1 = HashMap::new();
        ls1.insert("key".to_string(), "value".to_string());
        let fp1 = StorageAnalyzer::analyze(&ls1, &HashMap::new(), &[], &[]).unwrap();

        let mut ls2 = HashMap::new();
        ls2.insert("key".to_string(), "new_value".to_string());
        let fp2 = StorageAnalyzer::analyze(&ls2, &HashMap::new(), &[], &[]).unwrap();

        let changes = StorageAnalyzer::detect_changes(&fp1, &fp2);
        assert!(changes.localstorage_changed);
        assert!(changes.hash_changed);
    }
}
