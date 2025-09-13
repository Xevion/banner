use super::Job;
use crate::banner::{BannerApi, Course, SearchQuery, Term};
use crate::data::models::TargetType;
use crate::error::Result;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tracing::{debug, info, trace};

/// Job implementation for scraping subject data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubjectJob {
    pub subject: String,
}

impl SubjectJob {
    pub fn new(subject: String) -> Self {
        Self { subject }
    }
}

#[async_trait::async_trait]
impl Job for SubjectJob {
    fn target_type(&self) -> TargetType {
        TargetType::Subject
    }

    async fn process(&self, banner_api: &BannerApi, db_pool: &PgPool) -> Result<()> {
        let subject_code = &self.subject;
        debug!(subject = subject_code, "Processing subject job");

        // Get the current term
        let term = Term::get_current().inner().to_string();
        let query = SearchQuery::new().subject(subject_code).max_results(500);

        let search_result = banner_api
            .search(&term, &query, "subjectDescription", false)
            .await?;

        if let Some(courses_from_api) = search_result.data {
            info!(
                subject = subject_code,
                count = courses_from_api.len(),
                "Found courses"
            );
            for course in courses_from_api {
                self.upsert_course(&course, db_pool).await?;
            }
        }

        debug!(subject = subject_code, "Subject job completed");
        Ok(())
    }

    fn description(&self) -> String {
        format!("Scrape subject: {}", self.subject)
    }
}

impl SubjectJob {
    async fn upsert_course(&self, course: &Course, db_pool: &PgPool) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO courses (crn, subject, course_number, title, term_code, enrollment, max_enrollment, wait_count, wait_capacity, last_scraped_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            ON CONFLICT (crn, term_code) DO UPDATE SET
                subject = EXCLUDED.subject,
                course_number = EXCLUDED.course_number,
                title = EXCLUDED.title,
                enrollment = EXCLUDED.enrollment,
                max_enrollment = EXCLUDED.max_enrollment,
                wait_count = EXCLUDED.wait_count,
                wait_capacity = EXCLUDED.wait_capacity,
                last_scraped_at = EXCLUDED.last_scraped_at
            "#,
        )
        .bind(&course.course_reference_number)
        .bind(&course.subject)
        .bind(&course.course_number)
        .bind(&course.course_title)
        .bind(&course.term)
        .bind(course.enrollment)
        .bind(course.maximum_enrollment)
        .bind(course.wait_count)
        .bind(course.wait_capacity)
        .bind(chrono::Utc::now())
        .execute(db_pool)
        .await
        .map(|result| {
            trace!(result = ?result, "Course upserted");
        })
        .map_err(|e| anyhow::anyhow!("Failed to upsert course: {e}"))
    }
}
