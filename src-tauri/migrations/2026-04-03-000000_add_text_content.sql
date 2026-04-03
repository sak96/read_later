-- 1. Add the column
ALTER TABLE articles
ADD COLUMN text_content TEXT DEFAULT '';

-- 2. Strip HTML safely
WITH RECURSIVE strip_html(id, text, depth) AS (
    -- base case
    SELECT rowid, body, 0
    FROM articles
    UNION ALL
    -- recursive step
    SELECT id,
           substr(text, 1, instr(text, '<') - 1) ||
           substr(text, instr(text, '>') + 1),
           depth + 1
    FROM strip_html
    WHERE text LIKE '%<%>%'
)
UPDATE articles
SET text_content = (
    SELECT text
    FROM strip_html s
    WHERE s.id = articles.rowid
    ORDER BY depth DESC
    LIMIT 1
);
