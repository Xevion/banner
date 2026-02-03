-- Relax year constraint to allow historical terms from Banner
-- Some terms date back to 2000 or earlier

ALTER TABLE terms DROP CONSTRAINT chk_terms_year_range;
ALTER TABLE terms ADD CONSTRAINT chk_terms_year_range CHECK (year >= 1990 AND year <= 2100);
