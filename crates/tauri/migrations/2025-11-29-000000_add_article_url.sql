-- 1. Create a new table with the 'url' column NOT NULL
CREATE TABLE IF NOT EXISTS articles_temp (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL,
    body TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    url TEXT NOT NULL
);


-- 2. Copy data from old table to new table, setting url to 'https://www.example.com/' for existing rows
INSERT INTO articles_temp (id, title, body, created_at, url)
SELECT id, title, body, created_at, 'https://www.example.com/' as url
FROM articles;

-- 3. Drop the old table
DROP TABLE articles;

-- 4. Rename new table to original name
ALTER TABLE articles_temp RENAME TO articles;
