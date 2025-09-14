pub mod subject;

use crate::banner::BannerApi;
use crate::data::models::TargetType;
use crate::error::Result;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::fmt;

/// Errors that can occur during job parsing
#[derive(Debug)]
pub enum JobParseError {
    InvalidJson(serde_json::Error),
    UnsupportedTargetType(TargetType),
}

impl fmt::Display for JobParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JobParseError::InvalidJson(e) => write!(f, "Invalid JSON in job payload: {}", e),
            JobParseError::UnsupportedTargetType(t) => {
                write!(f, "Unsupported target type: {:?}", t)
            }
        }
    }
}

impl std::error::Error for JobParseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            JobParseError::InvalidJson(e) => Some(e),
            _ => None,
        }
    }
}

/// Errors that can occur during job processing
#[derive(Debug)]
pub enum JobError {
    Recoverable(anyhow::Error),   // API failures, network issues
    Unrecoverable(anyhow::Error), // Parse errors, corrupted data
}

impl fmt::Display for JobError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JobError::Recoverable(e) => write!(f, "Recoverable error: {}", e),
            JobError::Unrecoverable(e) => write!(f, "Unrecoverable error: {}", e),
        }
    }
}

impl std::error::Error for JobError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            JobError::Recoverable(e) => e.source(),
            JobError::Unrecoverable(e) => e.source(),
        }
    }
}

/// Common trait interface for all job types
#[async_trait::async_trait]
pub trait Job: Send + Sync {
    /// The target type this job handles
    #[allow(dead_code)]
    fn target_type(&self) -> TargetType;

    /// Process the job with the given API client and database pool
    async fn process(&self, banner_api: &BannerApi, db_pool: &PgPool) -> Result<()>;

    /// Get a human-readable description of the job
    fn description(&self) -> String;
}

/// Main job enum that dispatches to specific job implementations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum JobType {
    Subject(subject::SubjectJob),
}

impl JobType {
    /// Create a job from the target type and payload
    pub fn from_target_type_and_payload(
        target_type: TargetType,
        payload: serde_json::Value,
    ) -> Result<Self, JobParseError> {
        match target_type {
            TargetType::Subject => {
                let subject_job: subject::SubjectJob =
                    serde_json::from_value(payload).map_err(JobParseError::InvalidJson)?;
                Ok(JobType::Subject(subject_job))
            }
            _ => Err(JobParseError::UnsupportedTargetType(target_type)),
        }
    }

    /// Convert to a Job trait object
    pub fn boxed(self) -> Box<dyn Job> {
        match self {
            JobType::Subject(job) => Box::new(job),
        }
    }
}
