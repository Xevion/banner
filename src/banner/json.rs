//! JSON parsing utilities for the Banner API client.

use anyhow::Result;
use serde_json::{self, Value};

/// Attempt to parse JSON and, on failure, include a contextual snippet of the
/// line where the error occurred.
///
/// In debug builds, this provides detailed context including the full JSON object
/// containing the error and type mismatch information. In release builds, it shows
/// a minimal snippet to prevent dumping huge JSON bodies to production logs.
pub fn parse_json_with_context<T: serde::de::DeserializeOwned>(body: &str) -> Result<T> {
    let jd = &mut serde_json::Deserializer::from_str(body);
    match serde_path_to_error::deserialize(jd) {
        Ok(value) => Ok(value),
        Err(err) => {
            let inner_err = err.inner();
            let (line, column) = (inner_err.line(), inner_err.column());
            let path = err.path().to_string();

            let msg = inner_err.to_string();
            let loc = format!(" at line {line} column {column}");
            let msg_without_loc = msg.strip_suffix(&loc).unwrap_or(&msg).to_string();

            // Build error message differently for debug vs release builds
            let final_err = if cfg!(debug_assertions) {
                // Debug mode: provide detailed context
                let type_info = parse_type_mismatch(&msg_without_loc);
                let context = extract_json_object_at_path(body, err.path(), line, column);

                let mut err_msg = String::new();
                if !path.is_empty() && path != "." {
                    err_msg.push_str(&format!("for path '{}'\n", path));
                }
                err_msg.push_str(&format!(
                    "({}) at line {} column {}\n\n",
                    type_info, line, column
                ));
                err_msg.push_str(&context);

                err_msg
            } else {
                // Release mode: minimal snippet to keep logs concise
                let snippet = build_error_snippet(body, line, column, 20);

                let mut err_msg = String::new();
                if !path.is_empty() && path != "." {
                    err_msg.push_str(&format!("for path '{}' ", path));
                }
                err_msg.push_str(&format!(
                    "({}) at line {} column {}",
                    msg_without_loc, line, column
                ));
                err_msg.push_str(&format!("\n{}", snippet));

                err_msg
            };

            Err(anyhow::anyhow!(final_err))
        }
    }
}

/// Extract type mismatch information from a serde error message.
///
/// Parses error messages like "invalid type: null, expected a string" to extract
/// the expected and actual types for clearer error reporting.
///
/// Returns a formatted string like "(expected a string, got null)" or the original
/// message if parsing fails.
fn parse_type_mismatch(error_msg: &str) -> String {
    // Try to parse "invalid type: X, expected Y" format
    if let Some(invalid_start) = error_msg.find("invalid type: ") {
        let after_prefix = &error_msg[invalid_start + "invalid type: ".len()..];

        if let Some(comma_pos) = after_prefix.find(", expected ") {
            let actual_type = &after_prefix[..comma_pos];
            let expected_part = &after_prefix[comma_pos + ", expected ".len()..];

            // Clean up expected part (remove " at line X column Y" if present)
            let expected_type = expected_part
                .split(" at line ")
                .next()
                .unwrap_or(expected_part)
                .trim();

            return format!("expected {}, got {}", expected_type, actual_type);
        }
    }

    // Try to parse "expected X at line Y" format
    if error_msg.starts_with("expected ")
        && let Some(expected_part) = error_msg.split(" at line ").next()
    {
        return expected_part.to_string();
    }

    // Fallback: return original message without location info
    error_msg.to_string()
}

/// Extract and pretty-print the JSON object/array containing the parse error.
///
/// This function navigates to the error location using the serde path and extracts
/// the parent object or array to provide better context for debugging.
///
/// # Arguments
/// * `body` - The raw JSON string
/// * `path` - The serde path to the error (e.g., "data[0].faculty[0].displayName")
/// * `line` - Line number of the error (for fallback)
/// * `column` - Column number of the error (for fallback)
///
/// # Returns
/// A formatted string containing the JSON object with the error, or a fallback snippet
fn extract_json_object_at_path(
    body: &str,
    path: &serde_path_to_error::Path,
    line: usize,
    column: usize,
) -> String {
    // Try to parse the entire JSON structure
    let root_value: Value = match serde_json::from_str(body) {
        Ok(v) => v,
        Err(_) => {
            // If we can't parse the JSON at all, fall back to line snippet
            return build_error_snippet(body, line, column, 20);
        }
    };

    // Navigate to the error location using the path
    let path_str = path.to_string();
    let segments = parse_path_segments(&path_str);

    let (context_value, context_name) = navigate_to_context(&root_value, &segments);

    // Pretty-print the context value with limited depth to avoid huge output
    match serde_json::to_string_pretty(&context_value) {
        Ok(pretty) => {
            // Limit output to ~50 lines to prevent log spam
            let lines: Vec<&str> = pretty.lines().collect();
            let truncated = if lines.len() > 50 {
                let mut result = lines[..47].join("\n");
                result.push_str("\n  ... (truncated, ");
                result.push_str(&(lines.len() - 47).to_string());
                result.push_str(" more lines)");
                result
            } else {
                pretty
            };

            format!("{} at '{}':\n{}", context_name, path_str, truncated)
        }
        Err(_) => {
            // Fallback to simple snippet if pretty-print fails
            build_error_snippet(body, line, column, 20)
        }
    }
}

/// Parse a JSON path string into segments for navigation.
///
/// Converts paths like "data[0].faculty[1].displayName" into a sequence of
/// object keys and array indices.
fn parse_path_segments(path: &str) -> Vec<PathSegment> {
    let mut segments = Vec::new();
    let mut current = String::new();
    let mut in_bracket = false;

    for ch in path.chars() {
        match ch {
            '.' if !in_bracket => {
                if !current.is_empty() {
                    segments.push(PathSegment::Key(current.clone()));
                    current.clear();
                }
            }
            '[' => {
                if !current.is_empty() {
                    segments.push(PathSegment::Key(current.clone()));
                    current.clear();
                }
                in_bracket = true;
            }
            ']' => {
                if in_bracket && !current.is_empty() {
                    if let Ok(index) = current.parse::<usize>() {
                        segments.push(PathSegment::Index(index));
                    }
                    current.clear();
                }
                in_bracket = false;
            }
            _ => current.push(ch),
        }
    }

    if !current.is_empty() {
        segments.push(PathSegment::Key(current));
    }

    segments
}

/// Represents a segment in a JSON path (either an object key or array index).
#[derive(Debug)]
enum PathSegment {
    Key(String),
    Index(usize),
}

/// Navigate through a JSON value using path segments and return the appropriate context.
///
/// This function walks the JSON structure and returns the parent object/array that
/// contains the error, providing meaningful context for debugging.
///
/// # Returns
/// A tuple of (context_value, description) where context_value is the JSON to display
/// and description is a human-readable name for what we're showing.
fn navigate_to_context<'a>(
    mut current: &'a Value,
    segments: &[PathSegment],
) -> (&'a Value, &'static str) {
    // If path is empty or just root, return the whole value
    if segments.is_empty() {
        return (current, "Root object");
    }

    // Try to navigate to the parent of the error location
    // We want to show the containing object/array, not just the failing field
    let parent_depth = segments.len().saturating_sub(1);

    for (i, segment) in segments.iter().enumerate() {
        // Stop one level before the end to show the parent context
        if i >= parent_depth {
            break;
        }

        match segment {
            PathSegment::Key(key) => {
                if let Some(next) = current.get(key) {
                    current = next;
                } else {
                    // Can't navigate further, return what we have
                    return (current, "Partial context (navigation stopped)");
                }
            }
            PathSegment::Index(idx) => {
                if let Some(next) = current.get(idx) {
                    current = next;
                } else {
                    return (current, "Partial context (index out of bounds)");
                }
            }
        }
    }

    (current, "Object containing error")
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[test]
    fn test_parse_type_mismatch_invalid_type() {
        let msg = "invalid type: null, expected a string at line 45 column 29";
        let result = parse_type_mismatch(msg);
        assert_eq!(result, "expected a string, got null");
    }

    #[test]
    fn test_parse_type_mismatch_expected() {
        let msg = "expected value at line 1 column 1";
        let result = parse_type_mismatch(msg);
        assert_eq!(result, "expected value");
    }

    #[test]
    fn test_parse_path_segments_simple() {
        let segments = parse_path_segments("data.name");
        assert_eq!(segments.len(), 2);
        match &segments[0] {
            PathSegment::Key(k) => assert_eq!(k, "data"),
            _ => panic!("Expected Key segment"),
        }
    }

    #[test]
    fn test_parse_path_segments_with_array() {
        let segments = parse_path_segments("data[0].faculty[1].displayName");
        assert_eq!(segments.len(), 5);
        match &segments[0] {
            PathSegment::Key(k) => assert_eq!(k, "data"),
            _ => panic!("Expected Key segment"),
        }
        match &segments[1] {
            PathSegment::Index(i) => assert_eq!(*i, 0),
            _ => panic!("Expected Index segment"),
        }
    }

    #[test]
    fn test_parse_json_with_context_null_value() {
        #[derive(Debug, Deserialize)]
        struct TestStruct {
            name: String,
        }

        let json = r#"{"name": null}"#;
        let result: Result<TestStruct> = parse_json_with_context(json);

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();

        // Should contain path info
        assert!(err_msg.contains("name"));

        // In debug mode, should contain detailed context
        if cfg!(debug_assertions) {
            assert!(err_msg.contains("expected"));
        }
    }

    #[test]
    fn test_navigate_to_context() {
        let json = r#"{"data": [{"faculty": [{"name": "John"}]}]}"#;
        let value: Value = serde_json::from_str(json).unwrap();

        let segments = parse_path_segments("data[0].faculty[0].name");
        let (context, _) = navigate_to_context(&value, &segments);

        // Should return the faculty[0] object (parent of 'name')
        assert!(context.is_object());
        assert!(context.get("name").is_some());
    }

    #[test]
    fn test_realistic_banner_error() {
        #[derive(Debug, Deserialize)]
        struct Course {
            #[allow(dead_code)]
            #[serde(rename = "courseTitle")]
            course_title: String,
            faculty: Vec<Faculty>,
        }

        #[derive(Debug, Deserialize)]
        struct Faculty {
            #[serde(rename = "displayName")]
            display_name: String,
            #[allow(dead_code)]
            email: String,
        }

        #[derive(Debug, Deserialize)]
        struct SearchResult {
            data: Vec<Course>,
        }

        // Simulate Banner API response with null faculty displayName
        // This mimics the actual error from SPN subject scrape
        let json = r#"{
            "data": [
                {
                    "courseTitle": "Spanish Conversation",
                    "faculty": [
                        {
                            "displayName": null,
                            "email": "instructor@utsa.edu"
                        }
                    ]
                }
            ]
        }"#;

        let result: Result<SearchResult> = parse_json_with_context(json);
        assert!(result.is_err());

        let err_msg = result.unwrap_err().to_string();
        println!("\n=== Error output in debug mode ===\n{}\n", err_msg);

        // Verify error contains key information
        assert!(err_msg.contains("data[0].faculty[0].displayName"));

        // In debug mode, should show detailed context
        if cfg!(debug_assertions) {
            // Should show type mismatch info
            assert!(err_msg.contains("expected") && err_msg.contains("got"));
            // Should show surrounding JSON context with the faculty object
            assert!(err_msg.contains("email"));
        }
    }
}
