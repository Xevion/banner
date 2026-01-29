-- RMP professor data (bulk synced from RateMyProfessors)
CREATE TABLE rmp_professors (
    legacy_id INTEGER PRIMARY KEY,
    graphql_id VARCHAR NOT NULL,
    first_name VARCHAR NOT NULL,
    last_name VARCHAR NOT NULL,
    department VARCHAR,
    avg_rating REAL,
    avg_difficulty REAL,
    num_ratings INTEGER NOT NULL DEFAULT 0,
    would_take_again_pct REAL,
    last_synced_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Link Banner instructors to RMP professors
ALTER TABLE instructors ADD COLUMN rmp_legacy_id INTEGER REFERENCES rmp_professors(legacy_id);
ALTER TABLE instructors ADD COLUMN rmp_match_status VARCHAR NOT NULL DEFAULT 'pending';
