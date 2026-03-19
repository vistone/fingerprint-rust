CREATE TABLE IF NOT EXISTS flows (
    id TEXT PRIMARY KEY,
    source_ip TEXT,
    target_ip TEXT,
    protocol TEXT,
    timestamp DATETIME,
    consistency_score INTEGER,
    bot_detected BOOLEAN
);
