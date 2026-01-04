//! Fingerprint Database
//!
//! Centralized database for storing and querying fingerprints and associated metadata.
//! Supports multiple fingerprint types (JA3, JA4, HASSH, JARM, etc.) and enables
//! efficient lookup, matching, and threat intelligence integration.
//!
//! ## Features
//!
//! - **Multi-fingerprint support**: Store JA3, JA3S, JA4, JA4S, HASSH, JARM fingerprints
//! - **Metadata storage**: Browser/server type, OS, threat indicators, timestamps
//! - **Efficient lookup**: O(1) fingerprint lookups using HashMap
//! - **Threat intelligence**: Tag fingerprints with threat levels and categories
//! - **Statistical data**: Track frequency, first/last seen timestamps
//! - **Import/Export**: JSON serialization for sharing threat intelligence
//!
//! ## Usage
//!
//! ```rust
//! use fingerprint_core::database::{FingerprintDatabase, FingerprintEntry, FingerprintType, ThreatLevel};
//!
//! // Create database
//! let mut db = FingerprintDatabase::new();
//!
//! // Add fingerprint
//! let entry = FingerprintEntry::new(
//! FingerprintType::JA3,
//! "d8321312332df7ff".to_string(),
//! Some("Chrome 119".to_string()),
//! ThreatLevel::Safe,
//! );
//! db.add(entry);
//!
//! // Query fingerprint
//! if let Some(entry) = db.get_ja3("d8321312332df7ff") {
//! println!("Found: {} - Threat level: {}", entry.client_info.unwrap(), entry.threat_level);
//! }
//! ```

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// Fingerprint type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FingerprintType {
    /// JA3 (TLS Client fingerprint - MD5)
    JA3,
    /// JA3S (TLS Server fingerprint - MD5)
    JA3S,
    /// JA4 (TLS Client fingerprint - SHA256)
    JA4,
    /// JA4S (TLS Server fingerprint - SHA256)
    JA4S,
    /// JA4H (HTTP fingerprint)
    JA4H,
    /// JA4L (Lightweight fingerprint)
    JA4L,
    /// JA4SSH (SSH fingerprint - JA4 style)
    JA4SSH,
    /// JA4T (TCP fingerprint)
    JA4T,
    /// HASSH (SSH Client fingerprint - MD5)
    HASSH,
    /// HASSH Server (SSH Server fingerprint - MD5)
    HASSSHServer,
    /// JARM (Active TLS Server fingerprint)
    JARM,
    /// p0f (TCP/IP passive fingerprint)
    P0f,
}

impl fmt::Display for FingerprintType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FingerprintType::JA3 => write!(f, "JA3"),
            FingerprintType::JA3S => write!(f, "JA3S"),
            FingerprintType::JA4 => write!(f, "JA4"),
            FingerprintType::JA4S => write!(f, "JA4S"),
            FingerprintType::JA4H => write!(f, "JA4H"),
            FingerprintType::JA4L => write!(f, "JA4L"),
            FingerprintType::JA4SSH => write!(f, "JA4SSH"),
            FingerprintType::JA4T => write!(f, "JA4T"),
            FingerprintType::HASSH => write!(f, "HASSH"),
            FingerprintType::HASSSHServer => write!(f, "HASSH-Server"),
            FingerprintType::JARM => write!(f, "JARM"),
            FingerprintType::P0f => write!(f, "p0f"),
        }
    }
}

/// Threat level classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ThreatLevel {
    /// Safe/benign fingerprint
    Safe,
    /// Suspicious activity
    Suspicious,
    /// Known malicious fingerprint
    Malicious,
    /// Unknown/unclassified
    Unknown,
}

impl fmt::Display for ThreatLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ThreatLevel::Safe => write!(f, "Safe"),
            ThreatLevel::Suspicious => write!(f, "Suspicious"),
            ThreatLevel::Malicious => write!(f, "Malicious"),
            ThreatLevel::Unknown => write!(f, "Unknown"),
        }
    }
}

/// Fingerprint database entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FingerprintEntry {
    /// Type of fingerprint
    pub fingerprint_type: FingerprintType,
    /// Fingerprint value (hash string)
    pub fingerprint: String,
    /// Client/Server information (browser, OS, server type, etc.)
    pub client_info: Option<String>,
    /// Threat level classification
    pub threat_level: ThreatLevel,
    /// Optional threat description/notes
    pub threat_description: Option<String>,
    /// Tags for categorization
    pub tags: Vec<String>,
    /// First time this fingerprint was seen
    pub first_seen: DateTime<Utc>,
    /// Last time this fingerprint was seen
    pub last_seen: DateTime<Utc>,
    /// Number of times this fingerprint has been observed
    pub observation_count: u64,
}

impl FingerprintEntry {
    /// Create a new fingerprint entry
    pub fn new(
        fingerprint_type: FingerprintType,
        fingerprint: String,
        client_info: Option<String>,
        threat_level: ThreatLevel,
    ) -> Self {
        let now = Utc::now();
        Self {
            fingerprint_type,
            fingerprint,
            client_info,
            threat_level,
            threat_description: None,
            tags: Vec::new(),
            first_seen: now,
            last_seen: now,
            observation_count: 1,
        }
    }

    /// Add a tag to this entry
    pub fn add_tag(&mut self, tag: String) {
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
        }
    }

    /// Update observation (increment count and update last_seen)
    pub fn record_observation(&mut self) {
        self.observation_count += 1;
        self.last_seen = Utc::now();
    }

    /// Set threat description
    pub fn set_threat_description(&mut self, description: String) {
        self.threat_description = Some(description);
    }

    /// Check if entry is a threat
    pub fn is_threat(&self) -> bool {
        matches!(
            self.threat_level,
            ThreatLevel::Suspicious | ThreatLevel::Malicious
        )
    }
}

/// Fingerprint database
///
/// Stores and manages fingerprints with efficient lookup capabilities.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FingerprintDatabase {
    /// JA3 fingerprints (TLS Client - MD5)
    ja3: HashMap<String, FingerprintEntry>,
    /// JA3S fingerprints (TLS Server - MD5)
    ja3s: HashMap<String, FingerprintEntry>,
    /// JA4 fingerprints (TLS Client - SHA256)
    ja4: HashMap<String, FingerprintEntry>,
    /// JA4S fingerprints (TLS Server - SHA256)
    ja4s: HashMap<String, FingerprintEntry>,
    /// JA4H fingerprints (HTTP)
    ja4h: HashMap<String, FingerprintEntry>,
    /// JA4L fingerprints (Lightweight)
    ja4l: HashMap<String, FingerprintEntry>,
    /// JA4SSH fingerprints (SSH - JA4 style)
    ja4ssh: HashMap<String, FingerprintEntry>,
    /// JA4T fingerprints (TCP)
    ja4t: HashMap<String, FingerprintEntry>,
    /// HASSH fingerprints (SSH Client - MD5)
    hassh: HashMap<String, FingerprintEntry>,
    /// HASSH Server fingerprints (SSH Server - MD5)
    hassh_server: HashMap<String, FingerprintEntry>,
    /// JARM fingerprints (Active TLS Server)
    jarm: HashMap<String, FingerprintEntry>,
    /// p0f fingerprints (TCP/IP passive)
    p0f: HashMap<String, FingerprintEntry>,
}

impl FingerprintDatabase {
    /// Create a new empty fingerprint database
    pub fn new() -> Self {
        Self {
            ja3: HashMap::new(),
            ja3s: HashMap::new(),
            ja4: HashMap::new(),
            ja4s: HashMap::new(),
            ja4h: HashMap::new(),
            ja4l: HashMap::new(),
            ja4ssh: HashMap::new(),
            ja4t: HashMap::new(),
            hassh: HashMap::new(),
            hassh_server: HashMap::new(),
            jarm: HashMap::new(),
            p0f: HashMap::new(),
        }
    }

    /// Add a fingerprint entry to the database
    pub fn add(&mut self, entry: FingerprintEntry) {
        let map = self.get_map_mut(entry.fingerprint_type);

        // If entry already exists, update observation count
        if let Some(existing) = map.get_mut(&entry.fingerprint) {
            existing.record_observation();
        } else {
            map.insert(entry.fingerprint.clone(), entry);
        }
    }

    /// Get a fingerprint entry by type and fingerprint value
    pub fn get(
        &self,
        fingerprint_type: FingerprintType,
        fingerprint: &str,
    ) -> Option<&FingerprintEntry> {
        let map = self.get_map(fingerprint_type);
        map.get(fingerprint)
    }

    /// Get a mutable reference to a fingerprint entry
    pub fn get_mut(
        &mut self,
        fingerprint_type: FingerprintType,
        fingerprint: &str,
    ) -> Option<&mut FingerprintEntry> {
        let map = self.get_map_mut(fingerprint_type);
        map.get_mut(fingerprint)
    }

    /// Remove a fingerprint entry
    pub fn remove(
        &mut self,
        fingerprint_type: FingerprintType,
        fingerprint: &str,
    ) -> Option<FingerprintEntry> {
        let map = self.get_map_mut(fingerprint_type);
        map.remove(fingerprint)
    }

    /// Get JA3 fingerprint entry
    pub fn get_ja3(&self, fingerprint: &str) -> Option<&FingerprintEntry> {
        self.ja3.get(fingerprint)
    }

    /// Get JA4 fingerprint entry
    pub fn get_ja4(&self, fingerprint: &str) -> Option<&FingerprintEntry> {
        self.ja4.get(fingerprint)
    }

    /// Get HASSH fingerprint entry
    pub fn get_hassh(&self, fingerprint: &str) -> Option<&FingerprintEntry> {
        self.hassh.get(fingerprint)
    }

    /// Get JARM fingerprint entry
    pub fn get_jarm(&self, fingerprint: &str) -> Option<&FingerprintEntry> {
        self.jarm.get(fingerprint)
    }

    /// Get all entries for a specific fingerprint type
    pub fn get_all(&self, fingerprint_type: FingerprintType) -> Vec<&FingerprintEntry> {
        let map = self.get_map(fingerprint_type);
        map.values().collect()
    }

    /// Get all threat entries (Suspicious or Malicious)
    pub fn get_threats(&self) -> Vec<&FingerprintEntry> {
        let mut threats = Vec::new();

        for map in self.all_maps() {
            for entry in map.values() {
                if entry.is_threat() {
                    threats.push(entry);
                }
            }
        }

        threats
    }

    /// Get entries with specific tags
    pub fn find_by_tag(&self, tag: &str) -> Vec<&FingerprintEntry> {
        let mut results = Vec::new();

        for map in self.all_maps() {
            for entry in map.values() {
                if entry.tags.contains(&tag.to_string()) {
                    results.push(entry);
                }
            }
        }

        results
    }

    /// Get total number of entries
    pub fn total_entries(&self) -> usize {
        self.ja3.len()
            + self.ja3s.len()
            + self.ja4.len()
            + self.ja4s.len()
            + self.ja4h.len()
            + self.ja4l.len()
            + self.ja4ssh.len()
            + self.ja4t.len()
            + self.hassh.len()
            + self.hassh_server.len()
            + self.jarm.len()
            + self.p0f.len()
    }

    /// Get statistics about the database
    pub fn stats(&self) -> DatabaseStats {
        DatabaseStats {
            total_entries: self.total_entries(),
            ja3_count: self.ja3.len(),
            ja4_count: self.ja4.len(),
            hassh_count: self.hassh.len(),
            jarm_count: self.jarm.len(),
            threat_count: self.get_threats().len(),
        }
    }

    /// Export database to JSON
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Import database from JSON
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Get the appropriate map for a fingerprint type
    fn get_map(&self, fingerprint_type: FingerprintType) -> &HashMap<String, FingerprintEntry> {
        match fingerprint_type {
            FingerprintType::JA3 => &self.ja3,
            FingerprintType::JA3S => &self.ja3s,
            FingerprintType::JA4 => &self.ja4,
            FingerprintType::JA4S => &self.ja4s,
            FingerprintType::JA4H => &self.ja4h,
            FingerprintType::JA4L => &self.ja4l,
            FingerprintType::JA4SSH => &self.ja4ssh,
            FingerprintType::JA4T => &self.ja4t,
            FingerprintType::HASSH => &self.hassh,
            FingerprintType::HASSSHServer => &self.hassh_server,
            FingerprintType::JARM => &self.jarm,
            FingerprintType::P0f => &self.p0f,
        }
    }

    /// Get mutable reference to the appropriate map
    fn get_map_mut(
        &mut self,
        fingerprint_type: FingerprintType,
    ) -> &mut HashMap<String, FingerprintEntry> {
        match fingerprint_type {
            FingerprintType::JA3 => &mut self.ja3,
            FingerprintType::JA3S => &mut self.ja3s,
            FingerprintType::JA4 => &mut self.ja4,
            FingerprintType::JA4S => &mut self.ja4s,
            FingerprintType::JA4H => &mut self.ja4h,
            FingerprintType::JA4L => &mut self.ja4l,
            FingerprintType::JA4SSH => &mut self.ja4ssh,
            FingerprintType::JA4T => &mut self.ja4t,
            FingerprintType::HASSH => &mut self.hassh,
            FingerprintType::HASSSHServer => &mut self.hassh_server,
            FingerprintType::JARM => &mut self.jarm,
            FingerprintType::P0f => &mut self.p0f,
        }
    }

    /// Get all maps as a vector
    fn all_maps(&self) -> Vec<&HashMap<String, FingerprintEntry>> {
        vec![
            &self.ja3,
            &self.ja3s,
            &self.ja4,
            &self.ja4s,
            &self.ja4h,
            &self.ja4l,
            &self.ja4ssh,
            &self.ja4t,
            &self.hassh,
            &self.hassh_server,
            &self.jarm,
            &self.p0f,
        ]
    }
}

impl Default for FingerprintDatabase {
    fn default() -> Self {
        Self::new()
    }
}

/// Database statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseStats {
    pub total_entries: usize,
    pub ja3_count: usize,
    pub ja4_count: usize,
    pub hassh_count: usize,
    pub jarm_count: usize,
    pub threat_count: usize,
}

impl fmt::Display for DatabaseStats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Database Stats: {} total entries (JA3: {}, JA4: {}, HASSH: {}, JARM: {}, Threats: {})",
            self.total_entries,
            self.ja3_count,
            self.ja4_count,
            self.hassh_count,
            self.jarm_count,
            self.threat_count
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fingerprint_entry_creation() {
        let entry = FingerprintEntry::new(
            FingerprintType::JA3,
            "d8321312332df7ff".to_string(),
            Some("Chrome 119".to_string()),
            ThreatLevel::Safe,
        );

        assert_eq!(entry.fingerprint_type, FingerprintType::JA3);
        assert_eq!(entry.fingerprint, "d8321312332df7ff");
        assert_eq!(entry.client_info, Some("Chrome 119".to_string()));
        assert_eq!(entry.threat_level, ThreatLevel::Safe);
        assert_eq!(entry.observation_count, 1);
        assert!(!entry.is_threat());
    }

    #[test]
    fn test_fingerprint_entry_tags() {
        let mut entry = FingerprintEntry::new(
            FingerprintType::JA4,
            "t13d1516h2".to_string(),
            Some("Firefox 120".to_string()),
            ThreatLevel::Safe,
        );

        entry.add_tag("browser".to_string());
        entry.add_tag("desktop".to_string());
        entry.add_tag("browser".to_string()); // Duplicate

        assert_eq!(entry.tags.len(), 2);
        assert!(entry.tags.contains(&"browser".to_string()));
        assert!(entry.tags.contains(&"desktop".to_string()));
    }

    #[test]
    fn test_fingerprint_entry_threat() {
        let mut entry = FingerprintEntry::new(
            FingerprintType::JARM,
            "27d40d40d29d40d1dc42d43d00041d".to_string(),
            Some("Botnet C2".to_string()),
            ThreatLevel::Malicious,
        );

        assert!(entry.is_threat());
        entry.set_threat_description("Known botnet command & control server".to_string());
        assert_eq!(
            entry.threat_description,
            Some("Known botnet command & control server".to_string())
        );
    }

    #[test]
    fn test_fingerprint_entry_observation() {
        let mut entry = FingerprintEntry::new(
            FingerprintType::JA3,
            "abc123".to_string(),
            None,
            ThreatLevel::Unknown,
        );

        assert_eq!(entry.observation_count, 1);
        let first_seen = entry.first_seen;

        std::thread::sleep(std::time::Duration::from_millis(10));
        entry.record_observation();

        assert_eq!(entry.observation_count, 2);
        assert!(entry.last_seen > first_seen);
    }

    #[test]
    fn test_database_creation() {
        let db = FingerprintDatabase::new();
        assert_eq!(db.total_entries(), 0);
    }

    #[test]
    fn test_database_add_and_get() {
        let mut db = FingerprintDatabase::new();

        let entry = FingerprintEntry::new(
            FingerprintType::JA3,
            "test_fingerprint".to_string(),
            Some("Test Client".to_string()),
            ThreatLevel::Safe,
        );

        db.add(entry);

        assert_eq!(db.total_entries(), 1);
        let retrieved = db.get_ja3("test_fingerprint");
        assert!(retrieved.is_some());
        assert_eq!(
            retrieved.unwrap().client_info,
            Some("Test Client".to_string())
        );
    }

    #[test]
    fn test_database_duplicate_adds() {
        let mut db = FingerprintDatabase::new();

        let entry1 = FingerprintEntry::new(
            FingerprintType::JA4,
            "duplicate_fp".to_string(),
            Some("Client 1".to_string()),
            ThreatLevel::Safe,
        );

        let entry2 = FingerprintEntry::new(
            FingerprintType::JA4,
            "duplicate_fp".to_string(),
            Some("Client 2".to_string()),
            ThreatLevel::Safe,
        );

        db.add(entry1);
        db.add(entry2);

        assert_eq!(db.total_entries(), 1); // Should not create duplicate
        let retrieved = db.get_ja4("duplicate_fp").unwrap();
        assert_eq!(retrieved.observation_count, 2); // Should increment count
    }

    #[test]
    fn test_database_remove() {
        let mut db = FingerprintDatabase::new();

        let entry = FingerprintEntry::new(
            FingerprintType::HASSH,
            "to_remove".to_string(),
            None,
            ThreatLevel::Unknown,
        );

        db.add(entry);
        assert_eq!(db.total_entries(), 1);

        let removed = db.remove(FingerprintType::HASSH, "to_remove");
        assert!(removed.is_some());
        assert_eq!(db.total_entries(), 0);
    }

    #[test]
    fn test_database_get_threats() {
        let mut db = FingerprintDatabase::new();

        db.add(FingerprintEntry::new(
            FingerprintType::JA3,
            "safe1".to_string(),
            None,
            ThreatLevel::Safe,
        ));

        db.add(FingerprintEntry::new(
            FingerprintType::JA3,
            "suspicious1".to_string(),
            None,
            ThreatLevel::Suspicious,
        ));

        db.add(FingerprintEntry::new(
            FingerprintType::JARM,
            "malicious1".to_string(),
            None,
            ThreatLevel::Malicious,
        ));

        let threats = db.get_threats();
        assert_eq!(threats.len(), 2);
    }

    #[test]
    fn test_database_find_by_tag() {
        let mut db = FingerprintDatabase::new();

        let mut entry1 = FingerprintEntry::new(
            FingerprintType::JA3,
            "fp1".to_string(),
            None,
            ThreatLevel::Safe,
        );
        entry1.add_tag("browser".to_string());
        db.add(entry1);

        let mut entry2 = FingerprintEntry::new(
            FingerprintType::JA4,
            "fp2".to_string(),
            None,
            ThreatLevel::Safe,
        );
        entry2.add_tag("browser".to_string());
        entry2.add_tag("mobile".to_string());
        db.add(entry2);

        let mut entry3 = FingerprintEntry::new(
            FingerprintType::HASSH,
            "fp3".to_string(),
            None,
            ThreatLevel::Safe,
        );
        entry3.add_tag("ssh".to_string());
        db.add(entry3);

        let browser_entries = db.find_by_tag("browser");
        assert_eq!(browser_entries.len(), 2);

        let mobile_entries = db.find_by_tag("mobile");
        assert_eq!(mobile_entries.len(), 1);
    }

    #[test]
    fn test_database_stats() {
        let mut db = FingerprintDatabase::new();

        db.add(FingerprintEntry::new(
            FingerprintType::JA3,
            "ja3_1".to_string(),
            None,
            ThreatLevel::Safe,
        ));

        db.add(FingerprintEntry::new(
            FingerprintType::JA4,
            "ja4_1".to_string(),
            None,
            ThreatLevel::Safe,
        ));

        db.add(FingerprintEntry::new(
            FingerprintType::JA4,
            "ja4_2".to_string(),
            None,
            ThreatLevel::Malicious,
        ));

        let stats = db.stats();
        assert_eq!(stats.total_entries, 3);
        assert_eq!(stats.ja3_count, 1);
        assert_eq!(stats.ja4_count, 2);
        assert_eq!(stats.threat_count, 1);
    }

    #[test]
    fn test_database_json_serialization() {
        let mut db = FingerprintDatabase::new();

        db.add(FingerprintEntry::new(
            FingerprintType::JA3,
            "test123".to_string(),
            Some("Test".to_string()),
            ThreatLevel::Safe,
        ));

        let json = db.to_json();
        assert!(json.is_ok());

        let json_str = json.unwrap();
        let restored = FingerprintDatabase::from_json(&json_str);
        assert!(restored.is_ok());

        let restored_db = restored.unwrap();
        assert_eq!(restored_db.total_entries(), 1);
        assert!(restored_db.get_ja3("test123").is_some());
    }

    #[test]
    fn test_fingerprint_type_display() {
        assert_eq!(format!("{}", FingerprintType::JA3), "JA3");
        assert_eq!(format!("{}", FingerprintType::JA4S), "JA4S");
        assert_eq!(format!("{}", FingerprintType::JARM), "JARM");
    }

    #[test]
    fn test_threat_level_display() {
        assert_eq!(format!("{}", ThreatLevel::Safe), "Safe");
        assert_eq!(format!("{}", ThreatLevel::Malicious), "Malicious");
    }
}
