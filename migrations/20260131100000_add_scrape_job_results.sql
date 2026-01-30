-- Scrape job results log: one row per completed (or failed) job for effectiveness tracking.
CREATE TABLE scrape_job_results (
    id              BIGSERIAL PRIMARY KEY,
    target_type     target_type NOT NULL,
    payload         JSONB NOT NULL,
    priority        scrape_priority NOT NULL,

    -- Timing
    queued_at       TIMESTAMPTZ NOT NULL,
    started_at      TIMESTAMPTZ NOT NULL,
    completed_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    duration_ms     INT NOT NULL,

    -- Outcome
    success         BOOLEAN NOT NULL,
    error_message   TEXT,
    retry_count     INT NOT NULL DEFAULT 0,

    -- Effectiveness (NULL when success = false)
    courses_fetched     INT,
    courses_changed     INT,
    courses_unchanged   INT,
    audits_generated    INT,
    metrics_generated   INT
);

CREATE INDEX idx_scrape_job_results_target_time
    ON scrape_job_results (target_type, completed_at);

CREATE INDEX idx_scrape_job_results_completed
    ON scrape_job_results (completed_at);
