-- Collapse instructors from per-banner-id rows to per-person rows (deduped by lowercased email).
-- All existing RMP matches are deliberately dropped; the new auto-matcher will re-score from scratch.

-- 1. Create the new instructors table (1 row per person, keyed by email)
CREATE TABLE instructors_new (
    id SERIAL PRIMARY KEY,
    display_name VARCHAR NOT NULL,
    email VARCHAR NOT NULL,
    rmp_professor_id INTEGER UNIQUE REFERENCES rmp_professors(legacy_id),
    rmp_match_status VARCHAR NOT NULL DEFAULT 'unmatched',
    CONSTRAINT instructors_email_unique UNIQUE (email)
);

-- 2. Populate from existing data, deduplicating by lowercased email.
--    For each email, pick the display_name from the row with the highest banner_id
--    (deterministic tiebreaker). All rmp fields start fresh (NULL / 'unmatched').
INSERT INTO instructors_new (display_name, email)
SELECT DISTINCT ON (LOWER(email))
    display_name,
    LOWER(email)
FROM instructors
ORDER BY LOWER(email), banner_id DESC;

-- 3. Create the new course_instructors table with integer FK and banner_id column
CREATE TABLE course_instructors_new (
    course_id INTEGER NOT NULL REFERENCES courses(id) ON DELETE CASCADE,
    instructor_id INTEGER NOT NULL REFERENCES instructors_new(id) ON DELETE CASCADE,
    banner_id VARCHAR NOT NULL,
    is_primary BOOLEAN NOT NULL DEFAULT false,
    PRIMARY KEY (course_id, instructor_id)
);

-- 4. Populate from old data, mapping old banner_id → new instructor id via lowercased email.
--    Use DISTINCT ON to handle cases where multiple old banner_ids for the same person
--    taught the same course (would cause duplicate (course_id, instructor_id) pairs).
INSERT INTO course_instructors_new (course_id, instructor_id, banner_id, is_primary)
SELECT DISTINCT ON (ci.course_id, inew.id)
    ci.course_id,
    inew.id,
    ci.instructor_id,  -- old banner_id
    ci.is_primary
FROM course_instructors ci
JOIN instructors iold ON iold.banner_id = ci.instructor_id
JOIN instructors_new inew ON inew.email = LOWER(iold.email)
ORDER BY ci.course_id, inew.id, ci.is_primary DESC;

-- 5. Drop old tables (course_instructors first due to FK dependency)
DROP TABLE course_instructors;
DROP TABLE instructors;

-- 6. Rename new tables into place
ALTER TABLE instructors_new RENAME TO instructors;
ALTER TABLE course_instructors_new RENAME TO course_instructors;

-- 7. Rename constraints to match the final table names
ALTER TABLE instructors RENAME CONSTRAINT instructors_new_pkey TO instructors_pkey;
ALTER TABLE instructors RENAME CONSTRAINT instructors_new_rmp_professor_id_key TO instructors_rmp_professor_id_key;
ALTER TABLE course_instructors RENAME CONSTRAINT course_instructors_new_pkey TO course_instructors_pkey;

-- 8. Recreate indexes
CREATE INDEX idx_course_instructors_instructor ON course_instructors (instructor_id);
CREATE INDEX idx_instructors_rmp_status ON instructors (rmp_match_status);
CREATE INDEX idx_instructors_email ON instructors (email);

-- 9. Create rmp_match_candidates table
CREATE TABLE rmp_match_candidates (
    id SERIAL PRIMARY KEY,
    instructor_id INTEGER NOT NULL REFERENCES instructors(id) ON DELETE CASCADE,
    rmp_legacy_id INTEGER NOT NULL REFERENCES rmp_professors(legacy_id),
    score REAL NOT NULL,
    score_breakdown JSONB NOT NULL DEFAULT '{}',
    status VARCHAR NOT NULL DEFAULT 'pending',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    resolved_at TIMESTAMPTZ,
    resolved_by BIGINT REFERENCES users(discord_id),
    CONSTRAINT uq_candidate_pair UNIQUE (instructor_id, rmp_legacy_id)
);

CREATE INDEX idx_match_candidates_instructor ON rmp_match_candidates (instructor_id);
CREATE INDEX idx_match_candidates_status ON rmp_match_candidates (status);
