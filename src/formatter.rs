//! Custom tracing formatter

use serde::Serialize;
use serde_json::{Map, Value};
use std::fmt;
use time::macros::format_description;
use time::{OffsetDateTime, format_description::FormatItem};
use tracing::field::{Field, Visit};
use tracing::{Event, Level, Subscriber};
use tracing_subscriber::fmt::format::Writer;
use tracing_subscriber::fmt::{FmtContext, FormatEvent, FormatFields, FormattedFields};
use tracing_subscriber::registry::LookupSpan;

/// Cached format description for timestamps
/// Uses 3 subsecond digits on Emscripten, 5 otherwise for better performance
#[cfg(target_os = "emscripten")]
const TIMESTAMP_FORMAT: &[FormatItem<'static>] =
    format_description!("[hour]:[minute]:[second].[subsecond digits:3]");

#[cfg(not(target_os = "emscripten"))]
const TIMESTAMP_FORMAT: &[FormatItem<'static>] =
    format_description!("[hour]:[minute]:[second].[subsecond digits:5]");

/// A custom formatter with enhanced timestamp formatting
///
/// Re-implementation of the Full formatter with improved timestamp display.
pub struct CustomPrettyFormatter;

/// A custom JSON formatter that flattens fields to root level
///
/// Outputs logs in the format: { "message": "...", "level": "...", "customAttribute": "..." }
pub struct CustomJsonFormatter;

impl<S, N> FormatEvent<S, N> for CustomPrettyFormatter
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        ctx: &FmtContext<'_, S, N>,
        mut writer: Writer<'_>,
        event: &Event<'_>,
    ) -> fmt::Result {
        let meta = event.metadata();

        // 1) Timestamp (dimmed when ANSI)
        let now = OffsetDateTime::now_utc();
        let formatted_time = now.format(&TIMESTAMP_FORMAT).map_err(|e| {
            eprintln!("Failed to format timestamp: {}", e);
            fmt::Error
        })?;
        write_dimmed(&mut writer, formatted_time)?;
        writer.write_char(' ')?;

        // 2) Colored 5-char level like Full
        write_colored_level(&mut writer, meta.level())?;
        writer.write_char(' ')?;

        // 3) Span scope chain (bold names, fields in braces, dimmed ':')
        if let Some(scope) = ctx.event_scope() {
            let mut saw_any = false;
            for span in scope.from_root() {
                write_bold(&mut writer, span.metadata().name())?;
                saw_any = true;
                let ext = span.extensions();
                if let Some(fields) = &ext.get::<FormattedFields<N>>() {
                    if !fields.is_empty() {
                        write_bold(&mut writer, "{")?;
                        write!(writer, "{}", fields)?;
                        write_bold(&mut writer, "}")?;
                    }
                }
                if writer.has_ansi_escapes() {
                    write!(writer, "\x1b[2m:\x1b[0m")?;
                } else {
                    writer.write_char(':')?;
                }
            }
            if saw_any {
                writer.write_char(' ')?;
            }
        }

        // 4) Target (dimmed), then a space
        if writer.has_ansi_escapes() {
            write!(writer, "\x1b[2m{}\x1b[0m\x1b[2m:\x1b[0m ", meta.target())?;
        } else {
            write!(writer, "{}: ", meta.target())?;
        }

        // 5) Event fields
        ctx.format_fields(writer.by_ref(), event)?;

        // 6) Newline
        writeln!(writer)
    }
}

impl<S, N> FormatEvent<S, N> for CustomJsonFormatter
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        _ctx: &FmtContext<'_, S, N>,
        mut writer: Writer<'_>,
        event: &Event<'_>,
    ) -> fmt::Result {
        let meta = event.metadata();

        #[derive(Serialize)]
        struct EventFields {
            message: String,
            level: String,
            target: String,
            #[serde(flatten)]
            fields: Map<String, Value>,
        }

        let (message, fields) = {
            let mut message: Option<String> = None;
            let mut fields: Map<String, Value> = Map::new();

            struct FieldVisitor<'a> {
                message: &'a mut Option<String>,
                fields: &'a mut Map<String, Value>,
            }

            impl<'a> Visit for FieldVisitor<'a> {
                fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
                    let key = field.name();
                    if key == "message" {
                        *self.message = Some(format!("{:?}", value));
                    } else {
                        // Use typed methods for better performance
                        self.fields
                            .insert(key.to_string(), Value::String(format!("{:?}", value)));
                    }
                }

                fn record_str(&mut self, field: &Field, value: &str) {
                    let key = field.name();
                    if key == "message" {
                        *self.message = Some(value.to_string());
                    } else {
                        self.fields
                            .insert(key.to_string(), Value::String(value.to_string()));
                    }
                }

                fn record_i64(&mut self, field: &Field, value: i64) {
                    let key = field.name();
                    if key != "message" {
                        self.fields.insert(
                            key.to_string(),
                            Value::Number(serde_json::Number::from(value)),
                        );
                    }
                }

                fn record_u64(&mut self, field: &Field, value: u64) {
                    let key = field.name();
                    if key != "message" {
                        self.fields.insert(
                            key.to_string(),
                            Value::Number(serde_json::Number::from(value)),
                        );
                    }
                }

                fn record_bool(&mut self, field: &Field, value: bool) {
                    let key = field.name();
                    if key != "message" {
                        self.fields.insert(key.to_string(), Value::Bool(value));
                    }
                }
            }

            let mut visitor = FieldVisitor {
                message: &mut message,
                fields: &mut fields,
            };
            event.record(&mut visitor);

            (message, fields)
        };

        let json = EventFields {
            message: message.unwrap_or_default(),
            level: meta.level().to_string(),
            target: meta.target().to_string(),
            fields,
        };

        writeln!(
            writer,
            "{}",
            serde_json::to_string(&json).unwrap_or_else(|_| "{}".to_string())
        )
    }
}

/// Write the verbosity level with the same coloring/alignment as the Full formatter.
fn write_colored_level(writer: &mut Writer<'_>, level: &Level) -> fmt::Result {
    if writer.has_ansi_escapes() {
        // Basic ANSI color sequences; reset with \x1b[0m
        let (color, text) = match *level {
            Level::TRACE => ("\x1b[35m", "TRACE"), // purple
            Level::DEBUG => ("\x1b[34m", "DEBUG"), // blue
            Level::INFO => ("\x1b[32m", " INFO"),  // green, note leading space
            Level::WARN => ("\x1b[33m", " WARN"),  // yellow, note leading space
            Level::ERROR => ("\x1b[31m", "ERROR"), // red
        };
        write!(writer, "{}{}\x1b[0m", color, text)
    } else {
        // Right-pad to width 5 like Full's non-ANSI mode
        match *level {
            Level::TRACE => write!(writer, "{:>5}", "TRACE"),
            Level::DEBUG => write!(writer, "{:>5}", "DEBUG"),
            Level::INFO => write!(writer, "{:>5}", " INFO"),
            Level::WARN => write!(writer, "{:>5}", " WARN"),
            Level::ERROR => write!(writer, "{:>5}", "ERROR"),
        }
    }
}

fn write_dimmed(writer: &mut Writer<'_>, s: impl fmt::Display) -> fmt::Result {
    if writer.has_ansi_escapes() {
        write!(writer, "\x1b[2m{}\x1b[0m", s)
    } else {
        write!(writer, "{}", s)
    }
}

fn write_bold(writer: &mut Writer<'_>, s: impl fmt::Display) -> fmt::Result {
    if writer.has_ansi_escapes() {
        write!(writer, "\x1b[1m{}\x1b[0m", s)
    } else {
        write!(writer, "{}", s)
    }
}
