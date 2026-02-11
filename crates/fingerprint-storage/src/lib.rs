#![allow(clippy::all, dead_code, unused_variables, unused_parens)]

//! # fingerprint-storage
//!
//! 存储特征识别模块
//!
//! 提供 LocalStorage/SessionStorage/IndexedDB 指纹识别能力

use std::collections::HashMap;

/// 存储指纹
#[derive(Debug, Clone)]
pub struct StorageFingerprint {
    /// LocalStorage 键值对
    pub localstorage: HashMap<String, String>,
    /// SessionStorage 键值对
    pub sessionstorage: HashMap<String, String>,
    /// IndexedDB 数据库列表
    pub indexeddb_databases: Vec<String>,
    /// Cookie 列表
    pub cookies: Vec<CookieInfo>,
    /// 存储可用性检查
    pub storage_available: StorageAvailability,
    /// 存储指纹哈希
    pub storage_hash: String,
}

/// Cookie 信息
#[derive(Debug, Clone, PartialEq)]
pub struct CookieInfo {
    pub name: String,
    pub domain: String,
    pub path: String,
    pub secure: bool,
    pub http_only: bool,
}

/// 存储可用性
#[derive(Debug, Clone)]
pub struct StorageAvailability {
    pub localstorage_available: bool,
    pub sessionstorage_available: bool,
    pub indexeddb_available: bool,
    pub cookies_available: bool,
}

/// 存储错误类型
#[derive(Debug)]
pub enum StorageError {
    /// 无效数据
    InvalidData,
    /// 分析失败
    AnalysisFailed(String),
    /// 其他错误
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

/// 存储分析器
pub struct StorageAnalyzer;

impl StorageAnalyzer {
    /// 分析存储数据
    pub fn analyze(
        localstorage: &HashMap<String, String>,
        sessionstorage: &HashMap<String, String>,
        indexeddb_dbs: &[&str],
        cookies: &[(&str, &str, &str)],
    ) -> Result<StorageFingerprint, StorageError> {
        // 检查存储可用性
        let storage_available = StorageAvailability {
            localstorage_available: !localstorage.is_empty(),
            sessionstorage_available: !sessionstorage.is_empty(),
            indexeddb_available: !indexeddb_dbs.is_empty(),
            cookies_available: !cookies.is_empty(),
        };

        // 转换 IndexedDB 列表
        let indexeddb_databases: Vec<String> =
            indexeddb_dbs.iter().map(|s| s.to_string()).collect();

        // 转换 Cookie
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

        // 生成存储哈希
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

    /// 生成存储哈希
    fn generate_storage_hash(
        localstorage: &HashMap<String, String>,
        sessionstorage: &HashMap<String, String>,
        indexeddb_dbs: &[String],
        cookies: &[CookieInfo],
    ) -> String {
        let mut hash_input = String::new();

        // 添加 LocalStorage 数据
        for (k, v) in localstorage.iter() {
            hash_input.push_str(&format!("{}:{};", k, v));
        }

        // 添加 SessionStorage 数据
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

    /// 检测存储更改
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

/// 存储更改信息
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
