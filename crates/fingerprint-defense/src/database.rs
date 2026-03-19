//! Implementation of a fingerprint database
//!
//! Provides persistent storage and querying capabilities for network flow fingerprints.

use fingerprint_core::system::NetworkFlow;
use rusqlite::{params, Connection, Result as SqliteResult};
use serde_json;
use std::path::Path;

struct Migration {
    version: i64,
    name: &'static str,
    sql: &'static str,
}

const MIGRATIONS: &[Migration] = &[
    Migration {
        version: 1,
        name: "create_flows",
        sql: include_str!("../migrations/001_create_flows.sql"),
    },
    Migration {
        version: 2,
        name: "create_fingerprints",
        sql: include_str!("../migrations/002_create_fingerprints.sql"),
    },
    Migration {
        version: 3,
        name: "create_candidate_fingerprints",
        sql: include_str!("../migrations/003_create_candidate_fingerprints.sql"),
    },
];

/// Stores a fingerprint record
pub struct FingerprintDatabase {
    conn: Connection,
}

impl FingerprintDatabase {
    /// open or Createdatabase
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let conn = Connection::open(path).map_err(|e| e.to_string())?;
        Self::open_with_connection(conn)
    }

    /// Create an in-memory database instance.
    pub fn new_in_memory() -> Result<Self, String> {
        let conn = Connection::open_in_memory().map_err(|e| e.to_string())?;
        Self::open_with_connection(conn)
    }

    fn open_with_connection(conn: Connection) -> Result<Self, String> {
        let db = Self { conn };
        db.configure_connection().map_err(|e| e.to_string())?;
        db.run_migrations().map_err(|e| e.to_string())?;
        Ok(db)
    }

    fn configure_connection(&self) -> SqliteResult<()> {
        self.conn.execute_batch("PRAGMA foreign_keys = ON;")
    }

    fn ensure_migration_table(&self) -> SqliteResult<()> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS schema_migrations (
                version INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                applied_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;
        Ok(())
    }

    fn schema_version_internal(&self) -> SqliteResult<i64> {
        self.ensure_migration_table()?;
        self.conn.query_row(
            "SELECT COALESCE(MAX(version), 0) FROM schema_migrations",
            [],
            |row| row.get(0),
        )
    }

    fn run_migrations(&self) -> SqliteResult<()> {
        self.ensure_migration_table()?;

        for migration in MIGRATIONS
            .iter()
            .filter(|migration| migration.version > self.schema_version_internal().unwrap_or(0))
        {
            let tx = self.conn.unchecked_transaction()?;
            tx.execute_batch(migration.sql)?;
            tx.execute(
                "INSERT OR IGNORE INTO schema_migrations (version, name) VALUES (?1, ?2)",
                params![migration.version, migration.name],
            )?;
            tx.commit()?;
        }

        Ok(())
    }

    /// Return the currently applied schema version.
    pub fn current_schema_version(&self) -> Result<i64, String> {
        self.schema_version_internal().map_err(|e| e.to_string())
    }

    /// Return the list of applied migration versions in ascending order.
    pub fn applied_migrations(&self) -> Result<Vec<i64>, String> {
        self.ensure_migration_table().map_err(|e| e.to_string())?;
        let mut stmt = self
            .conn
            .prepare("SELECT version FROM schema_migrations ORDER BY version ASC")
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], |row| row.get(0))
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())
    }

    /// Store a complete traffic flow record
    pub fn store_flow(&self, flow: &NetworkFlow, score: u8, bot: bool) -> Result<(), String> {
        self.conn.execute(
 "INSERT OR REPLACE INTO flows (id, source_ip, target_ip, protocol, timestamp, consistency_score, bot_detected)
 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
 params![
 flow.flow_id(),
 flow.context.source_ip.to_string(),
 flow.context.target_ip.to_string(),
 format!("{:?}", flow.context.protocol),
 flow.context.timestamp.to_rfc3339(),
 score,
 bot,
 ],
 ).map_err(|e| e.to_string())?;

        // storeeachlayerlevelfingerprint
        for fp in flow.fingerprints() {
            let fp_type = format!("{:?}", fp.fingerprint_type());
            let fp_id = fp.id();
            let ja4_plus = fp
                .metadata()
                .get("ja4")
                .or_else(|| fp.metadata().get("ja4h"))
                .or_else(|| fp.metadata().get("ja4t"))
                .unwrap_or_default();

            let metadata_json = serde_json::to_string(fp.metadata()).unwrap_or_default();

            self.conn
                .execute(
                    "INSERT INTO fingerprints (flow_id, fp_type, fp_id, ja4_plus, metadata_json)
 VALUES (?1, ?2, ?3, ?4, ?5)",
                    params![flow.flow_id(), fp_type, fp_id, ja4_plus, metadata_json],
                )
                .map_err(|e| e.to_string())?;
        }

        Ok(())
    }

    /// Get database statistics information
    pub fn get_stats(&self) -> Result<String, String> {
        let flow_count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM flows", [], |r| r.get(0))
            .map_err(|e| e.to_string())?;
        let bot_count: i64 = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM flows WHERE bot_detected = 1",
                [],
                |r| r.get(0),
            )
            .map_err(|e| e.to_string())?;

        Ok(format!(
            "Total Flows: {}, Bots Detected: {}",
            flow_count, bot_count
        ))
    }

    /// Store a candidate fingerprint (for stable unknown fingerprints pending review)
    pub fn store_candidate_fingerprint(
        &self,
        fingerprint_type: &str,
        fingerprint_id: &str,
        observation_count: u32,
        stability_score: f64,
        notes: Option<&str>,
    ) -> Result<i64, String> {
        let now = chrono::Utc::now().to_rfc3339();

        let id = self.conn.query_row(
            "INSERT INTO candidate_fingerprints 
             (fingerprint_type, fingerprint_id, observation_count, stability_score, first_seen, last_seen, notes)
             VALUES (?1, ?2, ?3, ?4, ?5, ?5, ?6)
             RETURNING id",
            params![fingerprint_type, fingerprint_id, observation_count, stability_score, now, notes],
            |row| row.get(0)
        ).map_err(|e| e.to_string())?;

        log::info!(
            "[Database] Stored candidate fingerprint - ID: {}, Type: {}, Count: {}, Score: {:.2}",
            id,
            fingerprint_type,
            observation_count,
            stability_score
        );

        Ok(id)
    }

    /// Get list of candidate fingerprints pending review
    pub fn get_pending_candidates(
        &self,
        limit: Option<u32>,
    ) -> Result<Vec<CandidateFingerprint>, String> {
        let limit_clause = limit.map(|l| format!("LIMIT {}", l)).unwrap_or_default();
        let sql = format!(
            "SELECT id, fingerprint_type, fingerprint_id, observation_count, 
                          stability_score, first_seen, last_seen, status, notes 
                          FROM candidate_fingerprints 
                          WHERE status = 'pending' 
                          ORDER BY first_seen DESC {}",
            limit_clause
        );

        let mut stmt = self.conn.prepare(&sql).map_err(|e| e.to_string())?;
        let candidates = stmt
            .query_map([], |row| {
                Ok(CandidateFingerprint {
                    id: row.get(0)?,
                    fingerprint_type: row.get(1)?,
                    fingerprint_id: row.get(2)?,
                    observation_count: row.get(3)?,
                    stability_score: row.get(4)?,
                    first_seen: row.get(5)?,
                    last_seen: row.get(6)?,
                    status: row.get(7)?,
                    notes: row.get(8)?,
                })
            })
            .map_err(|e| e.to_string())?;

        candidates
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())
    }

    /// Update candidate fingerprint status
    pub fn update_candidate_status(
        &self,
        id: i64,
        status: &str,
        notes: Option<&str>,
    ) -> Result<(), String> {
        let rows_affected = self
            .conn
            .execute(
                "UPDATE candidate_fingerprints SET status = ?1, notes = ?2 WHERE id = ?3",
                params![status, notes, id],
            )
            .map_err(|e| e.to_string())?;

        if rows_affected == 0 {
            return Err(format!("No candidate fingerprint found with id: {}", id));
        }

        log::info!(
            "[Database] Updated candidate fingerprint {} status to {}",
            id,
            status
        );
        Ok(())
    }

    /// Get candidate fingerprint statistics information
    pub fn get_candidate_stats(&self) -> Result<CandidateStats, String> {
        let pending_count: i64 = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM candidate_fingerprints WHERE status = 'pending'",
                [],
                |r| r.get(0),
            )
            .map_err(|e| e.to_string())?;

        let approved_count: i64 = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM candidate_fingerprints WHERE status = 'approved'",
                [],
                |r| r.get(0),
            )
            .map_err(|e| e.to_string())?;

        let rejected_count: i64 = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM candidate_fingerprints WHERE status = 'rejected'",
                [],
                |r| r.get(0),
            )
            .map_err(|e| e.to_string())?;

        Ok(CandidateStats {
            pending: pending_count as u32,
            approved: approved_count as u32,
            rejected: rejected_count as u32,
        })
    }
}

/// Candidate fingerprint data structure
#[derive(Debug, Clone)]
pub struct CandidateFingerprint {
    pub id: i64,
    pub fingerprint_type: String,
    pub fingerprint_id: String,
    pub observation_count: u32,
    pub stability_score: f64,
    pub first_seen: String,
    pub last_seen: String,
    pub status: String,
    pub notes: Option<String>,
}

/// Candidate fingerprint statistics information
#[derive(Debug, Clone)]
pub struct CandidateStats {
    pub pending: u32,
    pub approved: u32,
    pub rejected: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn applies_embedded_migrations_for_in_memory_database() {
        let db = FingerprintDatabase::new_in_memory().expect("open in-memory db");
        assert_eq!(db.current_schema_version().unwrap(), 3);
        assert_eq!(db.applied_migrations().unwrap(), vec![1, 2, 3]);
    }

    #[test]
    fn reopening_database_keeps_single_migration_history() {
        let temp_dir = tempdir().expect("create temp dir");
        let db_path = temp_dir.path().join("fingerprints.db");

        let first = FingerprintDatabase::open(&db_path).expect("open db");
        assert_eq!(first.current_schema_version().unwrap(), 3);
        drop(first);

        let reopened = FingerprintDatabase::open(&db_path).expect("reopen db");
        assert_eq!(reopened.applied_migrations().unwrap(), vec![1, 2, 3]);

        let migration_count: i64 = reopened
            .conn
            .query_row("SELECT COUNT(*) FROM schema_migrations", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(migration_count, 3);
    }
}
