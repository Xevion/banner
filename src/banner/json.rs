//! JSON parsing utilities for the Banner API client.

use anyhow::Result;
use serde_json;

/// Attempt to parse JSON and, on failure, include a contextual snippet of the
/// line where the error occurred. This prevents dumping huge JSON bodies to logs.
pub fn parse_json_with_context<T: serde::de::DeserializeOwned>(body: &str) -> Result<T> {
    let jd = &mut serde_json::Deserializer::from_str(body);
    match serde_path_to_error::deserialize(jd) {
        Ok(value) => Ok(value),
        Err(err) => {
            let inner_err = err.inner();
            let (line, column) = (inner_err.line(), inner_err.column());
            let snippet = build_error_snippet(body, line, column, 20);
            let path = err.path().to_string();

            let msg = inner_err.to_string();
            let loc = format!(" at line {line} column {column}");
            let msg_without_loc = msg.strip_suffix(&loc).unwrap_or(&msg).to_string();

            let mut final_err = String::new();
            if !path.is_empty() && path != "." {
                final_err.push_str(&format!("for path '{}' ", path));
            }
            final_err.push_str(&format!(
                "({msg_without_loc}) at line {line} column {column}"
            ));
            final_err.push_str(&format!("\n{snippet}"));

            Err(anyhow::anyhow!(final_err))
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
