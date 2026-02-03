-- Terms table for multi-term scraping support
-- See docs/plans/2026-02-03-terms-table-design.md for design details

CREATE TABLE terms (
    code VARCHAR(6) PRIMARY KEY,        -- e.g., "202510"
    description TEXT NOT NULL,          -- e.g., "Fall 2024" (from Banner)
    year SMALLINT NOT NULL,             -- e.g., 2024 (parsed from code)
    season VARCHAR(10) NOT NULL,        -- 'Fall', 'Spring', 'Summer'
    
    -- Scrape control (NO DEFAULT - must be explicit on insert)
    scrape_enabled BOOLEAN NOT NULL,
    
    -- Metadata from Banner
    is_archived BOOLEAN NOT NULL DEFAULT false,  -- "View Only" terms
    
    -- Timestamps
    discovered_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_scraped_at TIMESTAMPTZ,
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Partial index for efficient "get enabled terms" query
CREATE INDEX idx_terms_scrape_enabled ON terms(scrape_enabled) WHERE scrape_enabled = true;

-- Index for ordering by term code (common for dropdowns, newest first)
CREATE INDEX idx_terms_code_desc ON terms(code DESC);

-- CHECK constraints for data integrity
ALTER TABLE terms ADD CONSTRAINT chk_terms_code_format 
    CHECK (code ~ '^[0-9]{4}(10|20|30)$');
    
ALTER TABLE terms ADD CONSTRAINT chk_terms_season_valid 
    CHECK (season IN ('Fall', 'Spring', 'Summer'));

ALTER TABLE terms ADD CONSTRAINT chk_terms_year_range 
    CHECK (year >= 2007 AND year <= 2100);
