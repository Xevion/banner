-- Add retry tracking columns to scrape_jobs table
ALTER TABLE scrape_jobs ADD COLUMN retry_count INTEGER NOT NULL DEFAULT 0 CHECK (retry_count >= 0);
ALTER TABLE scrape_jobs ADD COLUMN max_retries INTEGER NOT NULL DEFAULT 5 CHECK (max_retries >= 0);
