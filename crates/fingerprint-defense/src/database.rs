//! fingerprintdatabaseimplement
//!
//! providefingerprintpersistent化store and queryFeatures.

use fingerprint_core::system::NetworkFlow;
use rusqlite::{params, Connection, Result as SqliteResult};
use serde_json;
use std::path::Path;

/// storefingerprintpair象
pub struct FingerprintDatabase {
    conn: Connection,
}

impl FingerprintDatabase {
    /// open or Createdatabase
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let conn = Connection::open(path).map_err(|e| e.to_string())?;
        let db = Self { conn };
        db.init().map_err(|e| e.to_string())?;
        Ok(db)
    }

    /// Initializetablestruct
    fn init(&self) -> SqliteResult<()> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS flows (
 id TEXT PRIMARY KEY,
 source_ip TEXT,
 target_ip TEXT,
 protocol TEXT,
 timestamp DATETIME,
 consistency_score INTEGER,
 bot_detected BOOLEAN
 )",
            [],
        )?;

        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS fingerprints (
 id INTEGER PRIMARY KEY AUTOINCREMENT,
 flow_id TEXT,
 fp_type TEXT,
 fp_id TEXT,
 ja4_plus TEXT,
 metadata_json TEXT,
 FOREIGN KEY(flow_id) REFERENCES flows(id)
 )",
            [],
        )?;

        Ok(())
    }

    /// storecompletetrafficrecord
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

    /// Getstatisticsinfo
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
}
