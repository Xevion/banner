-- ============================================================
-- Expand courses table with rich Banner API fields
-- ============================================================

-- Section identifiers
ALTER TABLE courses ADD COLUMN sequence_number VARCHAR;
ALTER TABLE courses ADD COLUMN part_of_term VARCHAR;

-- Schedule & delivery (store codes, descriptions come from reference_data)
ALTER TABLE courses ADD COLUMN instructional_method VARCHAR;
ALTER TABLE courses ADD COLUMN campus VARCHAR;

-- Credit hours
ALTER TABLE courses ADD COLUMN credit_hours INTEGER;
ALTER TABLE courses ADD COLUMN credit_hour_low INTEGER;
ALTER TABLE courses ADD COLUMN credit_hour_high INTEGER;

-- Cross-listing
ALTER TABLE courses ADD COLUMN cross_list VARCHAR;
ALTER TABLE courses ADD COLUMN cross_list_capacity INTEGER;
ALTER TABLE courses ADD COLUMN cross_list_count INTEGER;

-- Section linking
ALTER TABLE courses ADD COLUMN link_identifier VARCHAR;
ALTER TABLE courses ADD COLUMN is_section_linked BOOLEAN;

-- JSONB columns for 1-to-many data
ALTER TABLE courses ADD COLUMN meeting_times JSONB NOT NULL DEFAULT '[]'::jsonb;
ALTER TABLE courses ADD COLUMN attributes JSONB NOT NULL DEFAULT '[]'::jsonb;

-- ============================================================
-- Full-text search support
-- ============================================================

-- Generated tsvector for word-based search on title
ALTER TABLE courses ADD COLUMN title_search tsvector
    GENERATED ALWAYS AS (to_tsvector('simple', coalesce(title, ''))) STORED;

CREATE INDEX idx_courses_title_search ON courses USING GIN (title_search);

-- Trigram index for substring/ILIKE search on title
CREATE EXTENSION IF NOT EXISTS pg_trgm;
CREATE INDEX idx_courses_title_trgm ON courses USING GIN (title gin_trgm_ops);

-- ============================================================
-- New filter indexes
-- ============================================================

CREATE INDEX idx_courses_instructional_method ON courses(instructional_method);
CREATE INDEX idx_courses_campus ON courses(campus);

-- Composite for "open CS courses in Fall 2024" pattern
CREATE INDEX idx_courses_term_subject_avail ON courses(term_code, subject, max_enrollment, enrollment);

-- ============================================================
-- Instructors table (normalized, deduplicated)
-- ============================================================

CREATE TABLE instructors (
    banner_id VARCHAR PRIMARY KEY,
    display_name VARCHAR NOT NULL,
    email VARCHAR
);

CREATE TABLE course_instructors (
    course_id INTEGER NOT NULL REFERENCES courses(id) ON DELETE CASCADE,
    instructor_id VARCHAR NOT NULL REFERENCES instructors(banner_id) ON DELETE CASCADE,
    is_primary BOOLEAN NOT NULL DEFAULT false,
    PRIMARY KEY (course_id, instructor_id)
);

CREATE INDEX idx_course_instructors_instructor ON course_instructors(instructor_id);

-- ============================================================
-- Reference data table (all code→description lookups)
-- ============================================================

CREATE TABLE reference_data (
    category VARCHAR NOT NULL,
    code VARCHAR NOT NULL,
    description VARCHAR NOT NULL,
    PRIMARY KEY (category, code)
);
