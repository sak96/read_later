-- add update_at column
ALTER TABLE articles
ADD COLUMN update_at TEXT NOT NULL DEFAULT (datetime('now'));

-- add update_at column
ALTER TABLE articles
ADD COLUMN is_deleted INTEGER DEFAULT 0;
