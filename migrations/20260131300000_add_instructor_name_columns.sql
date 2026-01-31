-- Add structured first/last name columns to instructors.
-- Populated by Rust-side backfill (parse_banner_name) since we need
-- HTML entity decoding and suffix extraction that SQL can't handle well.
ALTER TABLE instructors ADD COLUMN first_name VARCHAR;
ALTER TABLE instructors ADD COLUMN last_name VARCHAR;
