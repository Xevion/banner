//! Course scraping functionality for the Banner API.

use crate::banner::{api::BannerApi, models::*, query::SearchQuery};
use anyhow::{Context, Result};
use redis::AsyncCommands;
use std::sync::Arc;
use std::time::Duration;
use tokio::time;
use tracing::{debug, error, info, warn};

/// Priority majors that should be scraped more frequently
const PRIORITY_MAJORS: &[&str] = &["CS", "CPE", "MAT", "EE", "IS"];

/// Maximum number of courses to fetch per page
const MAX_PAGE_SIZE: i32 = 500;

/// Course scraper for Banner API
pub struct CourseScraper {
    api: Arc<BannerApi>,
    redis_client: redis::Client,
}

impl CourseScraper {
    /// Creates a new course scraper
    pub fn new(api: Arc<BannerApi>, redis_url: &str) -> Result<Self> {
        let redis_client =
            redis::Client::open(redis_url).context("Failed to create Redis client")?;

        Ok(Self { api, redis_client })
    }

    /// Scrapes all courses and stores them in Redis
    pub async fn scrape_all(&self, term: &str) -> Result<()> {
        // Get all subjects
        let subjects = self
            .api
            .get_subjects("", term, 1, 100)
            .await
            .context("Failed to get subjects for scraping")?;

        if subjects.is_empty() {
            return Err(anyhow::anyhow!("No subjects found for term {}", term));
        }

        // Categorize subjects
        let (priority_subjects, other_subjects): (Vec<_>, Vec<_>) = subjects
            .into_iter()
            .partition(|subject| PRIORITY_MAJORS.contains(&subject.code.as_str()));

        // Get expired subjects that need scraping
        let mut expired_subjects = Vec::new();
        expired_subjects.extend(self.get_expired_subjects(&priority_subjects, term).await?);
        expired_subjects.extend(self.get_expired_subjects(&other_subjects, term).await?);

        if expired_subjects.is_empty() {
            info!("no expired subjects found, skipping scrape");
            return Ok(());
        }

        info!(
            "scraping {} subjects for term {}",
            expired_subjects.len(),
            term
        );

        // Scrape each expired subject
        for subject in expired_subjects {
            if let Err(e) = self.scrape_subject(&subject.code, term).await {
                error!("failed to scrape subject {}: {}", subject.code, e);
            }

            // Rate limiting between subjects
            time::sleep(Duration::from_secs(2)).await;
        }

        Ok(())
    }

    /// Gets subjects that have expired and need to be scraped
    async fn get_expired_subjects(&self, subjects: &[Pair], term: &str) -> Result<Vec<Pair>> {
        let mut conn = self
            .redis_client
            .get_multiplexed_async_connection()
            .await
            .context("Failed to get Redis connection")?;

        let mut expired = Vec::new();

        for subject in subjects {
            let key = format!("scraped:{}:{}", subject.code, term);
            let scraped: Option<String> = conn
                .get(&key)
                .await
                .context("Failed to check scrape status in Redis")?;

            // If not scraped or marked as expired (empty/0), add to list
            if scraped.is_none() || scraped.as_deref() == Some("0") {
                expired.push(subject.clone());
            }
        }

        Ok(expired)
    }

    /// Scrapes all courses for a specific subject
    pub async fn scrape_subject(&self, subject: &str, term: &str) -> Result<()> {
        let mut offset = 0;
        let mut total_courses = 0;

        loop {
            let query = SearchQuery::new()
                .subject(subject)
                .offset(offset)
                .max_results(MAX_PAGE_SIZE * 2);

            // Ensure session term is selected before searching
            self.api.select_term(term).await?;

            let result = self
                .api
                .search(term, &query, "subjectDescription", false)
                .await
                .with_context(|| {
                    format!(
                        "Failed to search for subject {} at offset {}",
                        subject, offset
                    )
                })?;

            if !result.success {
                return Err(anyhow::anyhow!(
                    "Search marked unsuccessful for subject {}",
                    subject
                ));
            }

            let course_count = result.data.as_ref().map(|v| v.len() as i32).unwrap_or(0);
            total_courses += course_count;

            debug!(
                "retrieved {} courses for subject {} at offset {}",
                course_count, subject, offset
            );

            // Store each course in Redis
            for course in result.data.unwrap_or_default() {
                if let Err(e) = self.store_course(&course).await {
                    error!(
                        "failed to store course {}: {}",
                        course.course_reference_number, e
                    );
                }
            }

            // Check if we got a full page and should continue
            if course_count >= MAX_PAGE_SIZE {
                if course_count > MAX_PAGE_SIZE {
                    warn!(
                        "course count {} exceeds max page size {}",
                        course_count, MAX_PAGE_SIZE
                    );
                }

                offset += MAX_PAGE_SIZE;
                debug!(
                    "continuing to next page for subject {} at offset {}",
                    subject, offset
                );

                // Rate limiting between pages
                time::sleep(Duration::from_secs(3)).await;
                continue;
            }

            break;
        }

        info!(
            "scraped {} total courses for subject {}",
            total_courses, subject
        );

        // Mark subject as scraped with expiry
        self.mark_subject_scraped(subject, term, total_courses)
            .await?;

        Ok(())
    }

    /// Stores a course in Redis
    async fn store_course(&self, course: &Course) -> Result<()> {
        let mut conn = self
            .redis_client
            .get_multiplexed_async_connection()
            .await
            .context("Failed to get Redis connection")?;

        let key = format!("class:{}", course.course_reference_number);
        let serialized = serde_json::to_string(course).context("Failed to serialize course")?;

        let _: () = conn
            .set(&key, serialized)
            .await
            .context("Failed to store course in Redis")?;

        Ok(())
    }

    /// Marks a subject as scraped with appropriate expiry time
    async fn mark_subject_scraped(
        &self,
        subject: &str,
        term: &str,
        course_count: i32,
    ) -> Result<()> {
        let mut conn = self
            .redis_client
            .get_multiplexed_async_connection()
            .await
            .context("Failed to get Redis connection")?;

        let key = format!("scraped:{}:{}", subject, term);
        let expiry = self.calculate_expiry(subject, course_count);

        let value = if course_count == 0 { -1 } else { course_count };

        let _: () = conn
            .set_ex(&key, value, expiry.as_secs() as u64)
            .await
            .context("Failed to mark subject as scraped")?;

        debug!(
            "marked subject {} as scraped with {} courses, expiry: {:?}",
            subject, course_count, expiry
        );

        Ok(())
    }

    /// Calculates expiry time for a scraped subject based on various factors
    fn calculate_expiry(&self, subject: &str, course_count: i32) -> Duration {
        // Base calculation: 1 hour per 100 courses
        let mut base_expiry = Duration::from_secs(3600 * (course_count as u64 / 100).max(1));

        // Special handling for subjects with few courses
        if course_count < 50 {
            // Linear interpolation: 1 course = 12 hours, 49 courses = 1 hour
            let hours = 12.0 - ((course_count as f64 - 1.0) / 48.0) * 11.0;
            base_expiry = Duration::from_secs((hours * 3600.0) as u64);
        }

        // Priority subjects get shorter expiry (more frequent updates)
        if PRIORITY_MAJORS.contains(&subject) {
            base_expiry = base_expiry / 3;
        }

        // Add random variance (±15%)
        let variance = (base_expiry.as_secs() as f64 * 0.15) as u64;
        let random_offset = (rand::random::<f64>() - 0.5) * 2.0 * variance as f64;

        let final_expiry = if random_offset > 0.0 {
            base_expiry + Duration::from_secs(random_offset as u64)
        } else {
            base_expiry.saturating_sub(Duration::from_secs((-random_offset) as u64))
        };

        // Ensure minimum of 1 hour
        final_expiry.max(Duration::from_secs(3600))
    }

    /// Gets a course from Redis cache
    pub async fn get_course(&self, crn: &str) -> Result<Option<Course>> {
        let mut conn = self
            .redis_client
            .get_multiplexed_async_connection()
            .await
            .context("Failed to get Redis connection")?;

        let key = format!("class:{}", crn);
        let serialized: Option<String> = conn
            .get(&key)
            .await
            .context("Failed to get course from Redis")?;

        match serialized {
            Some(data) => {
                let course: Course = serde_json::from_str(&data)
                    .context("Failed to deserialize course from Redis")?;
                Ok(Some(course))
            }
            None => Ok(None),
        }
    }
}
