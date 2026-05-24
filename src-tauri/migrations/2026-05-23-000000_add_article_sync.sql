-- add updated_at column
ALTER TABLE articles
ADD COLUMN updated_at TEXT NOT NULL DEFAULT (datetime('now'));

-- add is_deleted column
ALTER TABLE articles
ADD COLUMN is_deleted INTEGER DEFAULT 0;
