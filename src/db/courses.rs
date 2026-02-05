//! Course database operations with automatic event emission.

use crate::banner::Course;
use crate::data::batch::batch_upsert_courses as batch_upsert_impl;
use crate::data::models::UpsertCounts;
use crate::db::DbContext;
use crate::error::Result;
use crate::events::{AuditLogEvent, DomainEvent};

/// Course operations.
pub struct CourseOps<'a> {
    ctx: &'a DbContext,
}

impl<'a> CourseOps<'a> {
    pub(crate) fn new(ctx: &'a DbContext) -> Self {
        Self { ctx }
    }

    /// Batch upsert courses and emit audit log events.
    ///
    /// This wraps the existing `batch_upsert_courses` function but handles
    /// event emission automatically.
    pub async fn batch_upsert(&self, courses: &[Course]) -> Result<UpsertCounts> {
        let (counts, audit_entries) = batch_upsert_impl(courses, self.ctx.pool()).await?;

        if !audit_entries.is_empty() {
            self.ctx
                .events()
                .publish(DomainEvent::AuditLog(AuditLogEvent {
                    entries: audit_entries,
                }));
        }

        Ok(counts)
    }
}
