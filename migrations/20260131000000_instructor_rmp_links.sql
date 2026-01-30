-- Multi-RMP profile support: allow many RMP profiles per instructor.
-- Each RMP profile still links to at most one instructor (rmp_legacy_id UNIQUE).

-- 1. Create junction table
CREATE TABLE instructor_rmp_links (
    id SERIAL PRIMARY KEY,
    instructor_id INTEGER NOT NULL REFERENCES instructors(id) ON DELETE CASCADE,
    rmp_legacy_id INTEGER NOT NULL UNIQUE REFERENCES rmp_professors(legacy_id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by BIGINT REFERENCES users(discord_id),
    source VARCHAR NOT NULL DEFAULT 'manual'  -- 'auto' | 'manual'
);

CREATE INDEX idx_instructor_rmp_links_instructor ON instructor_rmp_links (instructor_id);

-- 2. Migrate existing matches
INSERT INTO instructor_rmp_links (instructor_id, rmp_legacy_id, source)
SELECT id, rmp_professor_id,
       CASE rmp_match_status WHEN 'auto' THEN 'auto' ELSE 'manual' END
FROM instructors
WHERE rmp_professor_id IS NOT NULL;

-- 3. Drop old column (and its unique constraint)
ALTER TABLE instructors DROP COLUMN rmp_professor_id;
