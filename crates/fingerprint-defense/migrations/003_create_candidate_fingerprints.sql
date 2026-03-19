CREATE TABLE IF NOT EXISTS candidate_fingerprints (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    fingerprint_type TEXT NOT NULL,
    fingerprint_id TEXT NOT NULL,
    observation_count INTEGER NOT NULL,
    stability_score REAL NOT NULL,
    first_seen DATETIME DEFAULT CURRENT_TIMESTAMP,
    last_seen DATETIME DEFAULT CURRENT_TIMESTAMP,
    status TEXT DEFAULT 'pending',
    notes TEXT
);

CREATE INDEX IF NOT EXISTS idx_candidate_status
ON candidate_fingerprints(status);

CREATE INDEX IF NOT EXISTS idx_candidate_type_id
ON candidate_fingerprints(fingerprint_type, fingerprint_id);
