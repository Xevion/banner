-- Performance optimization indexes

-- Index for term-based queries (most common access pattern)
CREATE INDEX IF NOT EXISTS idx_courses_term_code ON courses(term_code);

-- Index for subject-based filtering
CREATE INDEX IF NOT EXISTS idx_courses_subject ON courses(subject);

-- Composite index for subject + term queries
CREATE INDEX IF NOT EXISTS idx_courses_subject_term ON courses(subject, term_code);

-- Index for course number lookups
CREATE INDEX IF NOT EXISTS idx_courses_course_number ON courses(course_number);

-- Index for last scraped timestamp (useful for finding stale data)
CREATE INDEX IF NOT EXISTS idx_courses_last_scraped ON courses(last_scraped_at);

-- Index for course metrics time-series queries
-- BRIN index is optimal for time-series data
CREATE INDEX IF NOT EXISTS idx_course_metrics_timestamp ON course_metrics USING BRIN(timestamp);

-- B-tree index for specific course metric lookups
CREATE INDEX IF NOT EXISTS idx_course_metrics_course_timestamp
    ON course_metrics(course_id, timestamp DESC);

-- Partial index for pending scrape jobs (only unlocked jobs)
CREATE INDEX IF NOT EXISTS idx_scrape_jobs_pending
    ON scrape_jobs(execute_at ASC)
    WHERE locked_at IS NULL;

-- Index for high-priority job processing
CREATE INDEX IF NOT EXISTS idx_scrape_jobs_priority_pending
    ON scrape_jobs(priority DESC, execute_at ASC)
    WHERE locked_at IS NULL;

-- Index for retry tracking
CREATE INDEX IF NOT EXISTS idx_scrape_jobs_retry_count
    ON scrape_jobs(retry_count)
    WHERE retry_count > 0 AND locked_at IS NULL;

-- Analyze tables to update statistics
ANALYZE courses;
ANALYZE course_metrics;
ANALYZE course_audits;
ANALYZE scrape_jobs;
