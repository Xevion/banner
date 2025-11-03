use super::Job;
use crate::banner::{BannerApi, SearchQuery, Term};
use crate::data::batch::batch_upsert_courses;
use crate::data::models::TargetType;
use crate::error::Result;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tracing::{debug, info};

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

    #[tracing::instrument(skip(self, banner_api, db_pool), fields(subject = %self.subject))]
    async fn process(&self, banner_api: &BannerApi, db_pool: &PgPool) -> Result<()> {
        let subject_code = &self.subject;

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
            batch_upsert_courses(&courses_from_api, db_pool).await?;
        }

        debug!(subject = subject_code, "Subject job completed");
        Ok(())
    }

    fn description(&self) -> String {
        format!("Scrape subject: {}", self.subject)
    }
}
