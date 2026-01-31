//! WebSocket event types and handler for real-time scrape job updates.

use axum::{
    extract::{
        State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    response::IntoResponse,
};
use futures::{SinkExt, StreamExt};
use serde::Serialize;
use sqlx::PgPool;
use tokio::sync::broadcast;
use tracing::debug;
use ts_rs::TS;

use crate::data::models::{ScrapeJob, ScrapeJobStatus};
use crate::state::AppState;
use crate::web::extractors::AdminUser;

/// A serializable DTO for `ScrapeJob` with computed `status`.
#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ScrapeJobDto {
    pub id: i32,
    pub target_type: String,
    pub target_payload: serde_json::Value,
    pub priority: String,
    pub execute_at: String,
    pub created_at: String,
    pub locked_at: Option<String>,
    pub retry_count: i32,
    pub max_retries: i32,
    pub queued_at: String,
    pub status: ScrapeJobStatus,
}

impl From<&ScrapeJob> for ScrapeJobDto {
    fn from(job: &ScrapeJob) -> Self {
        Self {
            id: job.id,
            target_type: format!("{:?}", job.target_type),
            target_payload: job.target_payload.clone(),
            priority: format!("{:?}", job.priority),
            execute_at: job.execute_at.to_rfc3339(),
            created_at: job.created_at.to_rfc3339(),
            locked_at: job.locked_at.map(|t| t.to_rfc3339()),
            retry_count: job.retry_count,
            max_retries: job.max_retries,
            queued_at: job.queued_at.to_rfc3339(),
            status: job.status(),
        }
    }
}

/// Events broadcast when scrape job state changes.
#[derive(Debug, Clone, Serialize, TS)]
#[serde(tag = "type", rename_all = "camelCase")]
#[ts(export)]
pub enum ScrapeJobEvent {
    Init {
        jobs: Vec<ScrapeJobDto>,
    },
    JobCreated {
        job: ScrapeJobDto,
    },
    JobLocked {
        id: i32,
        #[serde(rename = "lockedAt")]
        locked_at: String,
        status: ScrapeJobStatus,
    },
    JobCompleted {
        id: i32,
    },
    JobRetried {
        id: i32,
        #[serde(rename = "retryCount")]
        retry_count: i32,
        #[serde(rename = "queuedAt")]
        queued_at: String,
        status: ScrapeJobStatus,
    },
    JobExhausted {
        id: i32,
    },
    JobDeleted {
        id: i32,
    },
}

/// Fetch current scrape jobs from the DB and build an `Init` event.
async fn build_init_event(db_pool: &PgPool) -> Result<ScrapeJobEvent, sqlx::Error> {
    let rows = sqlx::query_as::<_, ScrapeJob>(
        "SELECT * FROM scrape_jobs ORDER BY priority DESC, execute_at ASC LIMIT 100",
    )
    .fetch_all(db_pool)
    .await?;

    let jobs = rows.iter().map(ScrapeJobDto::from).collect();
    Ok(ScrapeJobEvent::Init { jobs })
}

/// WebSocket endpoint for real-time scrape job updates.
///
/// Auth is checked via `AdminUser` before the upgrade occurs — if rejected,
/// a 401/403 is returned and the upgrade never happens.
pub async fn scrape_jobs_ws(
    ws: WebSocketUpgrade,
    AdminUser(_user): AdminUser,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_scrape_jobs_ws(socket, state))
}

/// Serialize an event and send it over the WebSocket sink.
/// Returns `true` if the message was sent, `false` if the client disconnected.
async fn send_event(
    sink: &mut futures::stream::SplitSink<WebSocket, Message>,
    event: &ScrapeJobEvent,
) -> bool {
    let Ok(json) = serde_json::to_string(event) else {
        return true; // serialization failed, but connection is still alive
    };
    sink.send(Message::Text(json.into())).await.is_ok()
}

async fn handle_scrape_jobs_ws(socket: WebSocket, state: AppState) {
    debug!("scrape-jobs WebSocket connected");

    let (mut sink, mut stream) = socket.split();

    // Send initial state
    let init_event = match build_init_event(&state.db_pool).await {
        Ok(event) => event,
        Err(e) => {
            debug!(error = %e, "failed to build init event, closing WebSocket");
            return;
        }
    };
    if !send_event(&mut sink, &init_event).await {
        debug!("client disconnected during init send");
        return;
    }

    // Subscribe to broadcast events
    let mut rx = state.scrape_job_events();

    loop {
        tokio::select! {
            result = rx.recv() => {
                match result {
                    Ok(ref event) => {
                        if !send_event(&mut sink, event).await {
                            debug!("client disconnected during event send");
                            break;
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(n)) => {
                        debug!(missed = n, "broadcast lagged, resyncing");
                        match build_init_event(&state.db_pool).await {
                            Ok(ref event) => {
                                if !send_event(&mut sink, event).await {
                                    debug!("client disconnected during resync send");
                                    break;
                                }
                            }
                            Err(e) => {
                                debug!(error = %e, "failed to build resync init event");
                            }
                        }
                    }
                    Err(broadcast::error::RecvError::Closed) => {
                        debug!("broadcast channel closed");
                        break;
                    }
                }
            }
            msg = stream.next() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&text)
                            && parsed.get("type").and_then(|t| t.as_str()) == Some("resync")
                        {
                            debug!("client requested resync");
                            match build_init_event(&state.db_pool).await {
                                Ok(ref event) => {
                                    if !send_event(&mut sink, event).await {
                                        debug!("client disconnected during resync send");
                                        break;
                                    }
                                }
                                Err(e) => {
                                    debug!(error = %e, "failed to build resync init event");
                                }
                            }
                        }
                    }
                    Some(Ok(Message::Close(_))) | None => {
                        debug!("client disconnected");
                        break;
                    }
                    _ => {}
                }
            }
        }
    }

    debug!("scrape-jobs WebSocket disconnected");
}
