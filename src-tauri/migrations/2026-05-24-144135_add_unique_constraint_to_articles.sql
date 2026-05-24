-- Drop non-latest rows for each unique URL
DELETE FROM articles
WHERE id NOT IN (
    SELECT id
    FROM (
        SELECT id, ROW_NUMBER() OVER (PARTITION BY url ORDER BY created_at DESC) as rn
        FROM articles
    ) t
    WHERE t.rn = 1
);

-- Add unique constraint
CREATE UNIQUE INDEX IF NOT EXISTS idx_articles_url ON articles(url);
