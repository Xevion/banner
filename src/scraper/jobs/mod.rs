pub mod subject;

use crate::data::models::TargetType;
use crate::error::Result;
use crate::{banner::BannerApi, scraper::jobs::subject::SubjectJob};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

/// Common trait interface for all job types
#[async_trait::async_trait]
pub trait Job: Send + Sync {
    /// The target type this job handles
    fn target_type(&self) -> TargetType;

    /// Process the job with the given API client and database pool
    async fn process(&self, banner_api: &BannerApi, db_pool: &PgPool) -> Result<()>;

    /// Get a human-readable description of the job
    fn description(&self) -> String;
}

/// Main job enum that dispatches to specific job implementations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum JobType {
    Subject(SubjectJob),
}

impl JobType {
    /// Create a job from the target type and payload
    pub fn from_target_type_and_payload(
        target_type: TargetType,
        payload: serde_json::Value,
    ) -> Result<Self> {
        match target_type {
            TargetType::Subject => {
                let subject_payload: SubjectJob = serde_json::from_value(payload)?;
                Ok(JobType::Subject(subject_payload))
            }
            _ => Err(anyhow::anyhow!(
                "Unsupported target type: {:?}",
                target_type
            )),
        }
    }

    /// Convert to a Job trait object
    pub fn as_job(self) -> Box<dyn Job> {
        match self {
            JobType::Subject(payload) => Box::new(payload),
        }
    }
}
