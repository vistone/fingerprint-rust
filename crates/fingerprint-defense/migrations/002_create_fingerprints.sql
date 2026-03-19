CREATE TABLE IF NOT EXISTS fingerprints (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    flow_id TEXT,
    fp_type TEXT,
    fp_id TEXT,
    ja4_plus TEXT,
    metadata_json TEXT,
    FOREIGN KEY(flow_id) REFERENCES flows(id)
);
