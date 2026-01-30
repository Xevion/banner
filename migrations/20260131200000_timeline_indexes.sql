-- Indexes for the timeline aggregation endpoint.
-- The query buckets course_metrics by 15-minute intervals, joins to courses
-- for subject, and aggregates enrollment. These indexes support efficient
-- time-range scans and the join.

-- Primary access pattern: scan course_metrics by timestamp range
CREATE INDEX IF NOT EXISTS idx_course_metrics_timestamp
    ON course_metrics (timestamp);

-- Composite index for the DISTINCT ON (bucket, course_id) ordered by timestamp DESC
-- to efficiently pick the latest metric per course per bucket.
CREATE INDEX IF NOT EXISTS idx_course_metrics_course_timestamp
    ON course_metrics (course_id, timestamp DESC);
