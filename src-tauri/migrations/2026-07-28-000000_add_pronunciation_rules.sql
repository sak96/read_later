CREATE TABLE IF NOT EXISTS pronunciation_rules (
    match_pattern TEXT PRIMARY KEY,
    replacement TEXT NOT NULL DEFAULT ''
);
