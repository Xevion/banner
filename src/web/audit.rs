//! Audit log DTOs shared by HTTP and stream handlers.

use serde::Serialize;
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct AuditLogEntry {
    pub id: i32,
    pub course_id: i32,
    pub timestamp: String,
    pub field_changed: String,
    pub old_value: String,
    pub new_value: String,
    pub subject: Option<String>,
    pub course_number: Option<String>,
    pub crn: Option<String>,
    pub course_title: Option<String>,
    pub term_code: Option<String>,
}

#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct AuditLogResponse {
    pub entries: Vec<AuditLogEntry>,
}
