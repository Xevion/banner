-- Index Optimization Follow-up Migration

-- Reason: Redundant with composite index idx_courses_subject_term
DROP INDEX IF EXISTS idx_courses_subject;

-- Remove: idx_scrape_jobs_retry_count
DROP INDEX IF EXISTS idx_scrape_jobs_retry_count;

-- Purpose: Optimize the scheduler's frequent query (runs every 60 seconds)
CREATE INDEX IF NOT EXISTS idx_scrape_jobs_scheduler_lookup
    ON scrape_jobs(target_type, target_payload)
    WHERE locked_at IS NULL;

-- Note: We use (target_type, target_payload) instead of including locked_at
-- in the index columns because:
-- 1. The WHERE clause filters locked_at IS NULL (partial index optimization)
-- 2. target_payload is JSONB and already large; keeping it as an indexed column
--    allows PostgreSQL to use index-only scans for the SELECT target_payload query
-- 3. This design minimizes index size while maximizing query performance


-- Purpose: Enable efficient audit trail queries by course
CREATE INDEX IF NOT EXISTS idx_course_audits_course_timestamp
    ON course_audits(course_id, timestamp DESC);

-- Purpose: Enable queries like "Show all changes in the last 24 hours"
CREATE INDEX IF NOT EXISTS idx_course_audits_timestamp
    ON course_audits(timestamp DESC);


-- The BRIN index on course_metrics(timestamp) assumes data is inserted in
-- chronological order. BRIN indexes are only effective when data is physically
-- ordered on disk. If you perform:
-- - Backfills of historical data
-- - Out-of-order inserts
-- - Frequent UPDATEs that move rows
--
-- Then the BRIN index effectiveness will degrade. Monitor with:
--   SELECT * FROM brin_page_items(get_raw_page('idx_course_metrics_timestamp', 1));
--
-- If you see poor selectivity, consider:
-- 1. REINDEX to rebuild after bulk loads
-- 2. Switch to B-tree if inserts are not time-ordered
-- 3. Use CLUSTER to physically reorder the table (requires downtime)

COMMENT ON INDEX idx_course_metrics_timestamp IS
    'BRIN index - requires chronologically ordered inserts for efficiency. Monitor selectivity.';

-- Update statistics for query planner
ANALYZE courses;
ANALYZE course_metrics;
ANALYZE course_audits;
ANALYZE scrape_jobs;
