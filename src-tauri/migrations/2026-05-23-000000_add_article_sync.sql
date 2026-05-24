-- add updated_at column
ALTER TABLE articles
ADD COLUMN updated_at TEXT NOT NULL DEFAULT '1970-01-01 00:00:10';

-- add is_deleted column
ALTER TABLE articles
ADD COLUMN is_deleted INTEGER DEFAULT 0;
