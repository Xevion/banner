use serde::{Deserialize, Serialize};

use super::courses::Course;

/// Search result wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResult {
    pub success: bool,
    pub total_count: i32,
    pub page_offset: i32,
    pub page_max_size: i32,
    pub path_mode: String,
    pub search_results_config: Vec<SearchResultConfig>,
    pub data: Vec<Course>,
}

/// Search result configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResultConfig {
    pub config: String,
    pub display: String,
}
