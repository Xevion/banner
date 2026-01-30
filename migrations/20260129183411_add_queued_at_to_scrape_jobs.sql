-- Add queued_at column to track when a job last entered the "ready to pick up" state.
-- For fresh jobs this equals execute_at; for retried jobs it is updated to NOW().
ALTER TABLE scrape_jobs
    ADD COLUMN queued_at TIMESTAMPTZ NOT NULL DEFAULT NOW();

-- Backfill existing rows: set queued_at = execute_at (best approximation)
UPDATE scrape_jobs SET queued_at = execute_at;
