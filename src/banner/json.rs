//! JSON parsing utilities for the Banner API client.

use anyhow::Result;

/// Attempt to parse JSON and, on failure, include a contextual snippet of the
/// line where the error occurred. This prevents dumping huge JSON bodies to logs.
pub fn parse_json_with_context<T: serde::de::DeserializeOwned>(body: &str) -> Result<T> {
    match serde_json::from_str::<T>(body) {
        Ok(value) => Ok(value),
        Err(err) => {
            let (line, column) = (err.line(), err.column());
            let snippet = build_error_snippet(body, line, column, 80);
            Err(anyhow::anyhow!(
                "{err} at line {line}, column {column}\nSnippet:\n{snippet}",
            ))
        }
    }
}

fn build_error_snippet(body: &str, line: usize, column: usize, context_len: usize) -> String {
    let target_line = body.lines().nth(line.saturating_sub(1)).unwrap_or("");
    if target_line.is_empty() {
        return "(empty line)".to_string();
    }

    // column is 1-based, convert to 0-based for slicing
    let error_idx = column.saturating_sub(1);

    let half_len = context_len / 2;
    let start = error_idx.saturating_sub(half_len);
    let end = (error_idx + half_len).min(target_line.len());

    let slice = &target_line[start..end];
    let indicator_pos = error_idx - start;

    let indicator = " ".repeat(indicator_pos) + "^";

    format!("...{slice}...\n   {indicator}")
}
