-- Drop all old tables
DROP TABLE IF EXISTS scrape_jobs;
DROP TABLE IF EXISTS course_metrics;
DROP TABLE IF EXISTS course_audits;
DROP TABLE IF EXISTS courses;

-- Enums for scrape_jobs
CREATE TYPE scrape_priority AS ENUM ('Low', 'Medium', 'High', 'Critical');
CREATE TYPE target_type AS ENUM ('Subject', 'CourseRange', 'CrnList', 'SingleCrn');

-- Main course data table
CREATE TABLE courses (
    id SERIAL PRIMARY KEY,
    crn VARCHAR NOT NULL,
    subject VARCHAR NOT NULL,
    course_number VARCHAR NOT NULL,
    title VARCHAR NOT NULL,
    term_code VARCHAR NOT NULL,
    enrollment INTEGER NOT NULL,
    max_enrollment INTEGER NOT NULL,
    wait_count INTEGER NOT NULL,
    wait_capacity INTEGER NOT NULL,
    last_scraped_at TIMESTAMPTZ NOT NULL,
    UNIQUE(crn, term_code)
);

-- Time-series data for course enrollment
CREATE TABLE course_metrics (
    id SERIAL PRIMARY KEY,
    course_id INTEGER NOT NULL REFERENCES courses(id) ON DELETE CASCADE,
    timestamp TIMESTAMPTZ NOT NULL,
    enrollment INTEGER NOT NULL,
    wait_count INTEGER NOT NULL,
    seats_available INTEGER NOT NULL
);

-- Audit trail for changes to course data
CREATE TABLE course_audits (
    id SERIAL PRIMARY KEY,
    course_id INTEGER NOT NULL REFERENCES courses(id) ON DELETE CASCADE,
    timestamp TIMESTAMPTZ NOT NULL,
    field_changed VARCHAR NOT NULL,
    old_value TEXT NOT NULL,
    new_value TEXT NOT NULL
);

-- Job queue for the scraper
CREATE TABLE scrape_jobs (
    id SERIAL PRIMARY KEY,
    target_type target_type NOT NULL,
    target_payload JSONB NOT NULL,
    priority scrape_priority NOT NULL,
    execute_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    locked_at TIMESTAMPTZ
);
